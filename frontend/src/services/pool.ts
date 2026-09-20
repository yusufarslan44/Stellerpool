import { contract } from '@stellar/stellar-sdk'
import { config, poolContractId, requireTestnetDemo } from '@/lib/stellar'
import type {
  MemberStatus,
  OrderMode,
  PoolInfo,
  PoolStatus,
  RoundInfo,
  RoundPhase,
  TxResult,
} from '@/types/pool'

/**
 * RotatingPool kontratı için istemci katmanı. Tüm çağrılar gerçek Soroban RPC'ye gider,
 * sahte veri yoktur.
 *
 * Yayımlı Testnet kontratı API v12'dir. Önceki kontratların havuzları zincirde kalır;
 * eski sürümde yeni akışın yazma işlemleri engellenir.
 */

export class LegacyContractError extends Error {
  constructor() {
    super(
      'The configured contract is the old sponsored version (API v8). The interface is written for the sponsor-free model; ' +
        'no on-chain action is possible until the contract is redeployed and VITE_ROTATING_POOL_CONTRACT_ID is updated.',
    )
    this.name = 'LegacyContractError'
  }
}

export class ContractNotConfiguredError extends Error {
  constructor() {
    super('The pool contract is not configured yet (VITE_ROTATING_POOL_CONTRACT_ID is empty).')
    this.name = 'ContractNotConfiguredError'
  }
}

type Method<A, R> = (
  args: A,
  options?: contract.MethodOptions,
) => Promise<contract.AssembledTransaction<R>>

interface PoolMethods {
  create_pool: Method<
    {
      creator: string
      token: string
      contribution_amount: bigint
      member_limit: number
      /** Yalnızca kura destekleyen kontratta vardır: 'Fixed' = sabit sıra, 'Draw' = kura (`{ tag, values }`). */
      order_mode?: { tag: OrderMode; values: void }
      round_duration: number
      grace_duration: number
      purchase_duration: number
      setup_deadline: number
      demo_seller: string
    },
    number
  >
  join_pool: Method<{ pool_id: number; member: string }, null>
  approve_terms: Method<{ pool_id: number; approver: string; version: number }, null>
  start_pool: Method<{ pool_id: number }, null>
  cancel_unstarted_pool: Method<{ pool_id: number }, null>
  deposit: Method<{ pool_id: number; member: string }, null>
  cure_payment: Method<{ pool_id: number; member: string }, null>
  propose_purchase: Method<
    {
      pool_id: number
      member: string
      seller: string
      asset: string
      amount: bigint
      doc_hash: Uint8Array
    },
    null
  >
  /** Kura modunda tüm katkılar tamamlanınca herkes çağırabilir; alıcıyı teslim almamışlar arasından seçer. */
  draw_recipient: Method<{ pool_id: number; caller: string }, string>
  execute_round: Method<{ pool_id: number }, null>
  mark_overdue: Method<{ pool_id: number }, null>
  abort_pool: Method<{ pool_id: number }, null>
  claim_refund: Method<{ pool_id: number; member: string }, null>
  next_pool_id: Method<Record<string, never>, bigint>
  get_pool: Method<{ pool_id: number }, unknown>
  get_round: Method<{ pool_id: number; round: number }, unknown>
  get_member_status: Method<{ pool_id: number; member: string }, unknown>
}

type PoolClient = contract.Client & PoolMethods

/** İşlem imzalayacak kullanıcı (cüzdan). Okuma çağrıları için gerekmez. */
export interface Signer {
  address: string
  signTransaction: (
    xdr: string,
    opts?: { networkPassphrase?: string; address?: string },
  ) => Promise<{ signedTxXdr: string; signerAddress?: string }>
}

/** Arayüzün çağırdığı kontrat fonksiyonları (docs/plan.md bölüm 5). */
const EXPECTED_METHODS = [
  'create_pool',
  'join_pool',
  'approve_terms',
  'start_pool',
  'cancel_unstarted_pool',
  'deposit',
  'cure_payment',
  'propose_purchase',
  'execute_round',
  'mark_overdue',
  'abort_pool',
  'claim_refund',
  'get_pool',
  'get_round',
  'get_member_status',
] as const

export class ContractInterfaceError extends Error {
  constructor(missing: string[]) {
    super(
      `The contract does not expose the expected functions: ${missing.join(', ')}. ` +
        'The contract address or version may be wrong; if function names changed, ' +
        'frontend/src/services/pool.ts must be updated.',
    )
    this.name = 'ContractInterfaceError'
  }
}

/** Kontratın sunduğu yetenekler (zincirdeki arayüzden okunur, sabit varsayım değildir). */
export interface ContractCapabilities {
  /** Eski sponsorlu (v8) kontrat: fund_guarantee gibi metotlar var. Arayüz bununla çalışmaz. */
  legacySponsor: boolean
  /** Kura (`draw_recipient`) destekleniyor mu? */
  supportsDraw: boolean
  /** Peşinat (`create_pool` girdisi `down_payment`) destekleniyor mu? Sürüm numarasına değil zincirdeki arayüze bakılır. */
  supportsDownPayment: boolean
  /** v12: order is finalized on the last join, with no verifier role or approval. */
  simpleTerms: boolean
}

async function buildClient(signer?: Signer): Promise<PoolClient> {
  requireTestnetDemo()
  if (!poolContractId) throw new ContractNotConfiguredError()
  return (await contract.Client.from({
    contractId: poolContractId,
    rpcUrl: config.rpcUrl,
    networkPassphrase: config.passphrase,
    publicKey: signer?.address,
    signTransaction: signer?.signTransaction,
  })) as PoolClient
}

const hasMethod = (client: unknown, name: string) =>
  typeof (client as Record<string, unknown>)[name] === 'function'

/**
 * `contract.Client` oluşturulurken zincirden okunan arayüzde bir fonksiyonun girdi adı var mı?
 * SDK 17 `getFunc` için düz bir nesne (`inputs` dizi, `name` metin) döndürür; eski XDR nesnesi
 * biçimi (`inputs()` / `name()` fonksiyon) de desteklenir.
 */
function hasInput(client: unknown, fn: string, input: string): boolean {
  try {
    const spec = (client as { spec?: { getFunc: (n: string) => unknown } }).spec
    const func = spec?.getFunc(fn) as { inputs?: unknown } | undefined
    const inputs = typeof func?.inputs === 'function' ? (func.inputs as () => unknown)() : func?.inputs
    return (
      Array.isArray(inputs) &&
      inputs.some((i: { name?: unknown }) => String(typeof i.name === 'function' ? (i.name as () => unknown)() : i.name) === input)
    )
  } catch {
    return false
  }
}

function capabilitiesOf(client: unknown): ContractCapabilities {
  return {
    legacySponsor: hasMethod(client, 'fund_guarantee') || hasMethod(client, 'top_up'),
    supportsDraw: hasMethod(client, 'draw_recipient'),
    supportsDownPayment: hasInput(client, 'create_pool', 'down_payment'),
    simpleTerms: !hasMethod(client, 'propose_terms') && !hasMethod(client, 'approve_purchase'),
  }
}

async function getClient(signer?: Signer): Promise<PoolClient> {
  const client = await buildClient(signer)
  const missing = EXPECTED_METHODS.filter((m) => !hasMethod(client, m))
  if (missing.length > 0) throw new ContractInterfaceError(missing)
  const caps = capabilitiesOf(client)
  if (caps.legacySponsor) throw new LegacyContractError()
  if (signer && !caps.simpleTerms) throw new Error('This old contract requires verifier approval. The new flow needs the v12 contract address.')
  return client
}

let capabilitiesCache: Promise<ContractCapabilities> | null = null

/** Formu kontratın gerçek yeteneklerine göre açıp kapatmak için; sonuç oturum boyunca önbelleğe alınır. */
export function getContractCapabilities(): Promise<ContractCapabilities> {
  capabilitiesCache ??= buildClient()
    .then(capabilitiesOf)
    .catch((e) => {
      capabilitiesCache = null
      throw e
    })
  return capabilitiesCache
}

interface SentTx {
  result?: unknown
  sendTransactionResponse?: { hash?: string }
  getTransactionResponse?: { txHash?: string }
}

function hashOf(sent: SentTx): string | null {
  return sent.sendTransactionResponse?.hash ?? sent.getTransactionResponse?.txHash ?? null
}

async function send(tx: contract.AssembledTransaction<unknown>): Promise<TxResult> {
  return { hash: hashOf((await tx.signAndSend()) as SentTx) }
}

// --- Kontrattan dönen değerleri arayüz tiplerine çevirir ---------------------------------

const POOL_STATUSES: PoolStatus[] = ['Filling', 'Active', 'Completed', 'Aborted']
const ROUND_PHASES: RoundPhase[] = ['Collecting', 'Grace', 'AwaitingDraw', 'AwaitingPurchase', 'Settled']

const toNumber = (v: unknown) => Number(v ?? 0)
const toBigInt = (v: unknown) => BigInt((v ?? 0) as bigint | number | string)
const toStringList = (v: unknown) => ((v as unknown[] | undefined) ?? []).map(String)
const toOptionalString = (v: unknown) => (v === undefined || v === null ? null : String(v))
/** Soroban enum'ları `{ tag: 'Active' }` veya düz string olarak gelebilir. */
const toTag = (v: unknown) => (typeof v === 'string' ? v : ((v as { tag?: string })?.tag ?? ''))

function toHexOrNull(v: unknown): string | null {
  if (v === undefined || v === null) return null
  if (typeof v === 'string') return v
  if (v instanceof Uint8Array) {
    return Array.from(v, (b) => b.toString(16).padStart(2, '0')).join('')
  }
  return null
}

/**
 * Rust `Result` döndüren kontrat fonksiyonlarında SDK sonucu `Ok { value }` olarak sarmalar
 * (Testnet kontratıyla doğrulandı). Hata varsa fırlatır, yoksa düz değeri döndürür.
 */
function unwrap(raw: unknown): unknown {
  const r = raw as { isErr?: () => boolean; unwrap?: () => unknown; unwrapErr?: () => { message?: string } } | null
  if (r && typeof r.isErr === 'function' && typeof r.unwrap === 'function') {
    if (r.isErr()) throw new Error(r.unwrapErr?.().message ?? 'The contract returned an error.')
    return r.unwrap()
  }
  return raw
}

function record(raw: unknown, what: string): Record<string, unknown> {
  if (!raw || typeof raw !== 'object') throw new Error(`The contract returned an unexpected ${what} response.`)
  return raw as Record<string, unknown>
}

function mapPool(id: number, raw: unknown): PoolInfo {
  const r = record(raw, 'havuz')
  const status = toTag(r.status) as PoolStatus
  if (!POOL_STATUSES.includes(status)) throw new Error(`Bilinmeyen havuz durumu: "${status}".`)
  return {
    id,
    creator: String(r.creator),
    token: String(r.token),
    contributionAmount: toBigInt(r.contribution_amount),
    memberLimit: toNumber(r.member_limit),
    members: toStringList(r.members),
    orderMode: toTag(r.order_mode) === 'Draw' ? 'Draw' : 'Fixed',
    downPayment: toBigInt(r.down_payment),
    recipientOrder: toStringList(r.recipient_order),
    termsVersion: toNumber(r.terms_version),
    termsApprovals: toStringList(r.terms_approvals),
    currentRound: toNumber(r.current_round),
    status,
    roundDuration: toNumber(r.round_duration),
    graceDuration: toNumber(r.grace_duration),
    purchaseDuration: toNumber(r.purchase_duration),
    setupDeadline: toNumber(r.setup_deadline),
    demoSeller: String(r.demo_seller),
  }
}

function mapRound(raw: unknown): RoundInfo {
  const r = record(raw, 'tur')
  const phase = toTag(r.phase) as RoundPhase
  if (!ROUND_PHASES.includes(phase)) throw new Error(`Bilinmeyen tur evresi: "${phase}".`)
  return {
    round: toNumber(r.round),
    phase,
    recipient: toOptionalString(r.recipient),
    startedAt: toNumber(r.started_at),
    collectDeadline: toNumber(r.collect_deadline),
    graceDeadline: toNumber(r.grace_deadline),
    purchaseDeadline: toNumber(r.purchase_deadline),
    paid: toStringList(r.paid),
    pot: toBigInt(r.pot),
    seller: toOptionalString(r.seller),
    docHash: toHexOrNull(r.doc_hash),
  }
}

function mapMember(address: string, raw: unknown): MemberStatus {
  const r = record(raw, 'member')
  return {
    address,
    refundable: toBigInt(r.refundable),
    received: Boolean(r.received),
  }
}

// --- Okuma (imza gerekmez, simülasyon) ---------------------------------------------------

export async function getPool(poolId: number): Promise<PoolInfo> {
  const c = await getClient()
  const tx = await c.get_pool({ pool_id: poolId })
  return mapPool(poolId, unwrap(tx.result))
}

/**
 * Yeni havuzlardan başlayarak en fazla `limit` havuzu okur (imza gerekmez). Kullanıcıyı planına uyan
 * açık bir havuza yönlendirmek için kullanılır; okunamayan tek bir havuz listeyi bozmaz.
 */
export async function listRecentPools(limit = 40): Promise<PoolInfo[]> {
  const c = await getClient()
  const next = Number(unwrap((await c.next_pool_id({})).result))
  if (!Number.isFinite(next) || next <= 1) return []
  const ids: number[] = []
  for (let id = next - 1; id >= 1 && ids.length < limit; id--) ids.push(id)
  const pools = await Promise.all(ids.map((id) => getPool(id).catch(() => null)))
  return pools.filter((p): p is PoolInfo => p !== null)
}

export async function getRound(poolId: number, round: number): Promise<RoundInfo> {
  const c = await getClient()
  const tx = await c.get_round({ pool_id: poolId, round })
  return mapRound(unwrap(tx.result))
}

export async function getMemberStatus(poolId: number, member: string): Promise<MemberStatus> {
  const c = await getClient()
  const status = await c.get_member_status({ pool_id: poolId, member })
  return mapMember(member, unwrap(status.result))
}

// --- Yazma (cüzdan imzası gerekir) -------------------------------------------------------

export async function createPool(
  signer: Signer,
  params: {
    token: string
    contributionAmount: bigint
    memberLimit: number
    orderMode: OrderMode
    /** Üye başına peşinat (yalnızca kontrat destekliyorsa gönderilir). */
    downPayment?: bigint
    roundDuration: number
    graceDuration: number
    purchaseDuration: number
    /** Kuruluş son tarihi, unix saniyesi. */
    setupDeadline: number
    demoSeller: string
  },
): Promise<TxResult & { poolId: number | null }> {
  const c = await getClient(signer)
  // Kura yoksa `order_mode` hiç gönderilmez; "Kura" seçili havuz sessizce sıralı kurulmasın diye reddedilir.
  const caps = capabilitiesOf(c)
  const supportsDraw = caps.supportsDraw
  if ((params.downPayment ?? 0n) > 0n && !caps.supportsDownPayment) {
    throw new Error('The configured contract does not support a down payment; set the down payment to 0.')
  }
  if (params.orderMode === 'Draw' && !supportsDraw) {
    throw new Error('The configured contract does not support a draw yet; choose fixed order.')
  }
  const tx = await c.create_pool({
    creator: signer.address,
    token: params.token,
    contribution_amount: params.contributionAmount,
    member_limit: params.memberLimit,
    ...(supportsDraw ? { order_mode: { tag: params.orderMode, values: undefined } } : {}),
    // v11+: `down_payment` zorunlu bir argümandır (0 = peşinatsız); eski kontrata hiç gönderilmez.
    ...(caps.supportsDownPayment ? { down_payment: params.downPayment ?? 0n } : {}),
    round_duration: params.roundDuration,
    grace_duration: params.graceDuration,
    purchase_duration: params.purchaseDuration,
    setup_deadline: params.setupDeadline,
    demo_seller: params.demoSeller,
  })
  const sent = (await tx.signAndSend()) as SentTx
  const id = sent.result === undefined ? Number.NaN : Number(unwrap(sent.result))
  return { hash: hashOf(sent), poolId: Number.isInteger(id) ? id : null }
}

export async function joinPool(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.join_pool({ pool_id: poolId, member: signer.address }))
}

/** Üye, geçerli koşul sürümünü cüzdanıyla onaylar. */
export async function approveTerms(signer: Signer, poolId: number, version: number) {
  const c = await getClient(signer)
  return send(await c.approve_terms({ pool_id: poolId, approver: signer.address, version }))
}

/** Koşullar tamamsa herkes havuzu başlatabilir; kurucu çevrim dışı kalsa da fon kilitli kalmaz. */
export async function startPool(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.start_pool({ pool_id: poolId }))
}

/** Kuruluş son tarihi geçip havuz başlamadıysa herkes iptal edebilir. */
export async function cancelUnstartedPool(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.cancel_unstarted_pool({ pool_id: poolId }))
}

export async function deposit(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.deposit({ pool_id: poolId, member: signer.address }))
}

/** Ek sürede üyenin kendi cüzdanından ödemesi; aynı borcu kapatır. */
export async function curePayment(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.cure_payment({ pool_id: poolId, member: signer.address }))
}

/**
 * Sıradaki üye satıcıyı, varlığı, tutarı ve belge özetini kaydeder. Demoda satıcı,
 * havuzun izinli test satıcısı olmalıdır.
 */
export async function proposePurchase(
  signer: Signer,
  params: {
    poolId: number
    seller: string
    asset: string
    amount: bigint
    docHash: Uint8Array
  },
) {
  const c = await getClient(signer)
  return send(
    await c.propose_purchase({
      pool_id: params.poolId,
      member: signer.address,
      seller: params.seller,
      asset: params.asset,
      amount: params.amount,
      doc_hash: params.docHash,
    }),
  )
}

/**
 * Kura modunda, tüm katkılar tamamlandıktan sonra alıcıyı henüz teslim almamış üyeler arasından
 * seçer. Herkes çağırabilir. Dönen değer kazanan adrestir. Rastgelelik hackathon düzeyindedir
 * (docs/CONTRACT_HANDOFF.md).
 */
export async function drawRecipient(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.draw_recipient({ pool_id: poolId, caller: signer.address }))
}

/** Koşullar tamamsa yalnızca o turun tutarını kayıtlı satıcıya gönderir. Herkes çağırabilir. */
export async function executeRound(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.execute_round({ pool_id: poolId }))
}

/** Katkı son tarihi geçtiyse turu ek süreye (Grace) alır. Herkes çağırabilir. */
export async function markOverdue(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.mark_overdue({ pool_id: poolId }))
}

/** Ek süre veya alım süresi sonunda koşullar sağlanmadıysa havuzu iptal eder. Herkes çağırabilir. */
export async function abortPool(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.abort_pool({ pool_id: poolId }))
}

export async function claimRefund(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.claim_refund({ pool_id: poolId, member: signer.address }))
}
