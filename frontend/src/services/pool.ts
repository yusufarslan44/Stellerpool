import { contract } from '@stellar/stellar-sdk'
import { config, poolContractId } from '@/lib/stellar'
import type { MemberStatus, PoolInfo, PoolStatus, RoundInfo, TxResult } from '@/types/pool'

/**
 * RotatingPool kontratı için istemci katmanı. Tüm çağrılar gerçek Soroban RPC'ye gider,
 * sahte veri yoktur.
 *
 * NOT: Kontrat henüz deploy edilmedi. Fonksiyon adları docs/plan.md bölüm 5'ten alındı;
 * parametre ve alan adları (snake_case) VARSAYIMDIR. Kontrat yazılınca
 * `stellar contract bindings typescript` çıktısıyla doğrulanacak; uyuşmazlık varsa
 * yalnızca bu dosya (ve types/pool.ts) değişir.
 */

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
      sponsor: string
      token: string
      contribution_amount: bigint
      member_limit: number
      round_duration: number
      verifiers: string[]
      approval_threshold: number
    },
    number
  >
  fund_guarantee: Method<{ pool_id: number; sponsor: string; amount: bigint }, null>
  join_pool: Method<{ pool_id: number; member: string }, null>
  start_pool: Method<{ pool_id: number; creator: string; recipient_order: string[] }, null>
  deposit: Method<{ pool_id: number; member: string }, null>
  propose_purchase: Method<
    { pool_id: number; member: string; seller: string; doc_hash: Uint8Array },
    null
  >
  approve_purchase: Method<{ pool_id: number; round: number; verifier: string }, null>
  execute_round: Method<{ pool_id: number }, null>
  mark_overdue: Method<{ pool_id: number }, null>
  top_up: Method<{ pool_id: number; sponsor: string; member: string }, null>
  abort_pool: Method<{ pool_id: number }, null>
  claim_refund: Method<{ pool_id: number; member: string }, null>
  claim_sponsor_remainder: Method<{ pool_id: number; sponsor: string }, null>
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
  'fund_guarantee',
  'join_pool',
  'start_pool',
  'deposit',
  'propose_purchase',
  'approve_purchase',
  'execute_round',
  'mark_overdue',
  'top_up',
  'abort_pool',
  'claim_refund',
  'claim_sponsor_remainder',
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

async function getClient(signer?: Signer): Promise<PoolClient> {
  if (!poolContractId) throw new ContractNotConfiguredError()
  const client = await contract.Client.from({
    contractId: poolContractId,
    rpcUrl: config.rpcUrl,
    networkPassphrase: config.passphrase,
    publicKey: signer?.address,
    signTransaction: signer?.signTransaction,
  })
  const methods = client as unknown as Record<string, unknown>
  const missing = EXPECTED_METHODS.filter((m) => typeof methods[m] !== 'function')
  if (missing.length > 0) throw new ContractInterfaceError(missing)
  return client as PoolClient
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

const POOL_STATUSES: PoolStatus[] = ['Filling', 'Active', 'Paused', 'Completed', 'Aborted']

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
    sponsor: String(r.sponsor),
    token: String(r.token),
    contributionAmount: toBigInt(r.contribution_amount),
    memberLimit: toNumber(r.member_limit),
    members: toStringList(r.members),
    recipientOrder: toStringList(r.recipient_order),
    currentRound: toNumber(r.current_round),
    roundDuration: toNumber(r.round_duration),
    requiredGuarantee: toBigInt(r.required_guarantee),
    guaranteeDeposited: toBigInt(r.guarantee_deposited),
    verifiers: toStringList(r.verifiers),
    approvalThreshold: toNumber(r.approval_threshold),
    status,
  }
}

function mapRound(raw: unknown): RoundInfo {
  const r = record(raw, 'tur')
  return {
    round: toNumber(r.round),
    recipient: String(r.recipient),
    startedAt: toNumber(r.started_at),
    deadline: toNumber(r.deadline),
    paid: toStringList(r.paid),
    sponsorCovered: toStringList(r.sponsor_covered),
    pot: toBigInt(r.pot),
    seller: toOptionalString(r.seller),
    docHash: toHexOrNull(r.doc_hash),
    approvals: toStringList(r.approvals),
    abortAfter: toNumber(r.abort_after),
  }
}

function mapMember(address: string, raw: unknown): MemberStatus {
  const r = record(raw, 'üye')
  return {
    address,
    contributed: toBigInt(r.contributed),
    refundable: toBigInt(r.refundable),
    received: Boolean(r.received),
  }
}

// --- Okuma (imza gerekmez, simülasyon) ---------------------------------------------------

export async function getPool(poolId: number): Promise<PoolInfo> {
  const c = await getClient()
  const tx = await c.get_pool({ pool_id: poolId })
  return mapPool(poolId, tx.result)
}

export async function getRound(poolId: number, round: number): Promise<RoundInfo> {
  const c = await getClient()
  const tx = await c.get_round({ pool_id: poolId, round })
  return mapRound(tx.result)
}

export async function getMemberStatus(poolId: number, member: string): Promise<MemberStatus> {
  const c = await getClient()
  const tx = await c.get_member_status({ pool_id: poolId, member })
  return mapMember(member, tx.result)
}

// --- Yazma (cüzdan imzası gerekir) -------------------------------------------------------

export async function createPool(
  signer: Signer,
  params: {
    sponsor: string
    token: string
    contributionAmount: bigint
    memberLimit: number
    roundDuration: number
    verifiers: string[]
    approvalThreshold: number
  },
): Promise<TxResult & { poolId: number | null }> {
  const c = await getClient(signer)
  const tx = await c.create_pool({
    creator: signer.address,
    sponsor: params.sponsor,
    token: params.token,
    contribution_amount: params.contributionAmount,
    member_limit: params.memberLimit,
    round_duration: params.roundDuration,
    verifiers: params.verifiers,
    approval_threshold: params.approvalThreshold,
  })
  const sent = (await tx.signAndSend()) as SentTx
  return { hash: hashOf(sent), poolId: sent.result === undefined ? null : Number(sent.result) }
}

/** Sponsor güvencesini kontrata kilitler. Yeterli güvence olmadan havuz başlamaz. */
export async function fundGuarantee(signer: Signer, poolId: number, amount: bigint) {
  const c = await getClient(signer)
  return send(await c.fund_guarantee({ pool_id: poolId, sponsor: signer.address, amount }))
}

export async function joinPool(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.join_pool({ pool_id: poolId, member: signer.address }))
}

export async function startPool(signer: Signer, poolId: number, recipientOrder: string[]) {
  const c = await getClient(signer)
  return send(
    await c.start_pool({ pool_id: poolId, creator: signer.address, recipient_order: recipientOrder }),
  )
}

export async function deposit(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.deposit({ pool_id: poolId, member: signer.address }))
}

/** Sıradaki üye satıcı adresini ve zincir dışı alım belgesinin özetini kaydeder. */
export async function proposePurchase(
  signer: Signer,
  poolId: number,
  seller: string,
  docHash: Uint8Array,
) {
  const c = await getClient(signer)
  return send(
    await c.propose_purchase({
      pool_id: poolId,
      member: signer.address,
      seller,
      doc_hash: docHash,
    }),
  )
}

/** Doğrulayıcı alım kaydını onaylar. Creator'ın tek başına onayı yeterli değildir. */
export async function approvePurchase(signer: Signer, poolId: number, round: number) {
  const c = await getClient(signer)
  return send(await c.approve_purchase({ pool_id: poolId, round, verifier: signer.address }))
}

/** Tur tutarı tam ve alım onaylıysa tutarı satıcıya gönderir. Herkes çağırabilir. */
export async function executeRound(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.execute_round({ pool_id: poolId }))
}

/** Süre dolduysa turu durdurur (Paused). Herkes çağırabilir. */
export async function markOverdue(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.mark_overdue({ pool_id: poolId }))
}

/** Sponsor, eksik üyenin katkısını ilave fonla tamamlar; katkı sponsor adına kaydedilir. */
export async function topUp(signer: Signer, poolId: number, member: string) {
  const c = await getClient(signer)
  return send(await c.top_up({ pool_id: poolId, sponsor: signer.address, member }))
}

/** Bekleme süresi sonunda güvenli devam yoksa iptal eder. Herkes çağırabilir. */
export async function abortPool(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.abort_pool({ pool_id: poolId }))
}

export async function claimRefund(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.claim_refund({ pool_id: poolId, member: signer.address }))
}

export async function claimSponsorRemainder(signer: Signer, poolId: number) {
  const c = await getClient(signer)
  return send(await c.claim_sponsor_remainder({ pool_id: poolId, sponsor: signer.address }))
}
