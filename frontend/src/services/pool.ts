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
 * NOT: Testnet'te yayındaki sponsorsuz kontrat (API v9) ile fonksiyon adları, parametreler ve
 * okunan alanlar doğrulandı (17 metot + okuma, canlı veriyle). Kura (`draw_recipient`,
 * `order_mode`) ve 30 üye API v10 hedefidir (docs/CONTRACT_HANDOFF.md); bu alanlar hâlâ
 * VARSAYIMDIR ve kontrat yayınlanınca `stellar contract bindings typescript` ile doğrulanmalı.
 * Uyuşmazlık varsa yalnızca bu dosya (ve types/pool.ts) değişir.
 */

export class LegacyContractError extends Error {
  constructor() {
    super(
      'Yapılandırılan kontrat eski sponsorlu sürüm (API v8). Arayüz sponsorsuz modele göre yazıldı; ' +
        'kontrat yeniden yayınlanıp VITE_ROTATING_POOL_CONTRACT_ID güncellenene kadar zincir işlemi yapılamaz.',
    )
    this.name = 'LegacyContractError'
  }
}

export class ContractNotConfiguredError extends Error {
  constructor() {
    super('Havuz kontratı henüz yapılandırılmadı (VITE_ROTATING_POOL_CONTRACT_ID boş).')
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
  propose_terms: Method<
    { pool_id: number; creator: string; recipient_order: string[]; verifiers: string[] },
    number
  >
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
    number
  >
  approve_purchase: Method<
    { pool_id: number; round: number; verifier: string; proposal_version: number },
    null
  >
  /** Kura modunda tüm katkılar tamamlanınca herkes çağırabilir; alıcıyı teslim almamışlar arasından seçer. */
  draw_recipient: Method<{ pool_id: number; caller: string }, string>
  execute_round: Method<{ pool_id: number }, null>
  mark_overdue: Method<{ pool_id: number }, null>
  abort_pool: Method<{ pool_id: number }, null>
  claim_refund: Method<{ pool_id: number; member: string }, null>
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
  'propose_terms',
  'approve_terms',
  'start_pool',
  'cancel_unstarted_pool',
  'deposit',
  'cure_payment',
  'propose_purchase',
  'approve_purchase',
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
      `Kontrat beklenen fonksiyonları sunmuyor: ${missing.join(', ')}. ` +
        'Kontrat adresi veya sürümü yanlış olabilir; fonksiyon adları değiştiyse ' +
        'frontend/src/services/pool.ts güncellenmeli.',
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

function capabilitiesOf(client: unknown): ContractCapabilities {
  return {
    legacySponsor: hasMethod(client, 'fund_guarantee') || hasMethod(client, 'top_up'),
    supportsDraw: hasMethod(client, 'draw_recipient'),
  }
}

async function getClient(signer?: Signer): Promise<PoolClient> {
  const client = await buildClient(signer)
  const missing = EXPECTED_METHODS.filter((m) => !hasMethod(client, m))
  if (missing.length > 0) throw new ContractInterfaceError(missing)
  if (capabilitiesOf(client).legacySponsor) throw new LegacyContractError()
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
 * (canlı Testnet kontratıyla doğrulandı). Hata varsa fırlatır, yoksa düz değeri döndürür.
 */
function unwrap(raw: unknown): unknown {
  const r = raw as { isErr?: () => boolean; unwrap?: () => unknown; unwrapErr?: () => { message?: string } } | null
  if (r && typeof r.isErr === 'function' && typeof r.unwrap === 'function') {
    if (r.isErr()) throw new Error(r.unwrapErr?.().message ?? 'Kontrat hata döndürdü.')
    return r.unwrap()
  }
  return raw
}

function record(raw: unknown, what: string): Record<string, unknown> {
  if (!raw || typeof raw !== 'object') throw new Error(`Kontrat beklenmeyen bir ${what} yanıtı verdi.`)
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
    recipientOrder: toStringList(r.recipient_order),
    verifiers: toStringList(r.verifiers),
    approvalThreshold: toNumber(r.approval_threshold),
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
    purchaseVersion: toNumber(r.purchase_version),
    approvals: toStringList(r.approvals),
  }
}

function mapMember(address: string, raw: unknown): MemberStatus {
  const r = record(raw, 'üye')
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
  const supportsDraw = capabilitiesOf(c).supportsDraw
  if (params.orderMode === 'Draw' && !supportsDraw) {
    throw new Error('Yapılandırılan kontrat henüz kura desteklemiyor; sabit sıra seç.')
  }
  const tx = await c.create_pool({
    creator: signer.address,
    token: params.token,
    contribution_amount: params.contributionAmount,
    member_limit: params.memberLimit,
    ...(supportsDraw ? { order_mode: { tag: params.orderMode, values: undefined } } : {}),
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

/**
 * Kurucu sırayı ve doğrulayıcıları önerir. Her değişiklik önceki tüm onayları geçersiz kılar.
 * Kura modunda sıra boş gönderilir; yalnızca doğrulayıcılar önerilir.
 */
export async function proposeTerms(
  signer: Signer,
  poolId: number,
  recipientOrder: string[],
  verifiers: string[],
) {
  const c = await getClient(signer)
  return send(
    await c.propose_terms({
      pool_id: poolId,
      creator: signer.address,
      recipient_order: recipientOrder,
      verifiers,
    }),
  )
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
 * havuzun izinli test satıcısı olmalıdır. Her yeni öneri önceki onayları siler.
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

/** Doğrulayıcı, geçerli alım önerisi sürümünü onaylar. Kurucunun onayı tek başına yeterli değildir. */
export async function approvePurchase(
  signer: Signer,
  poolId: number,
  round: number,
  proposalVersion: number,
) {
  const c = await getClient(signer)
  return send(
    await c.approve_purchase({
      pool_id: poolId,
      round,
      verifier: signer.address,
      proposal_version: proposalVersion,
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
