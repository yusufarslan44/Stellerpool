/**
 * Kontrat modeli, docs/plan.md bölüm 5'teki sponsorsuz havuz tasarımına göre yazılmıştır
 * (Testnet'te yayındaki API v9 ile alan alan doğrulandı; kura alanları API v10 hedefidir).
 * Tutarlar stroop (7 ondalık) cinsinden bigint,
 * zamanlar unix saniyesidir.
 */

/** Üye sayısı sınırları (yayındaki kontrat 12'ye kadar; 30 hedefi: docs/CONTRACT_HANDOFF.md). */
export const MIN_MEMBERS = 2
/** Arayüzün desteklediği en büyük grup; kontratın da bu sınıra yükseltilmesi gerekir. */
export const UI_MAX_MEMBERS = 30
/** Yayındaki kontratın (API v9) kabul ettiği üst sınır. */
const CONTRACT_MAX_MEMBERS = 12
const configuredMax = Number.parseInt(String(import.meta.env.VITE_MAX_MEMBERS ?? ''), 10)
/**
 * Formun izin verdiği üst sınır. Varsayılan, yayındaki kontratın kabul ettiği 12'dir; kontrat 30'a
 * yükseltilince `VITE_MAX_MEMBERS=30` ile açılır (docs/CONTRACT_HANDOFF.md). Aksi halde 13–30 üyeli
 * bir havuz zincirde reddedilirdi.
 */
export const MAX_MEMBERS =
  Number.isInteger(configuredMax) && configuredMax >= MIN_MEMBERS && configuredMax <= UI_MAX_MEMBERS
    ? configuredMax
    : CONTRACT_MAX_MEMBERS
/** Kontratın doğrulayıcı sınırları (API v9). */
export const MIN_VERIFIERS = 2
export const MAX_VERIFIERS = 10

/** Havuz: Filling → Active → Completed / Aborted. */
export type PoolStatus = 'Filling' | 'Active' | 'Completed' | 'Aborted'

/**
 * Alıcı nasıl belirlenir: `Fixed` = üyelerin onayladığı sabit sıra,
 * `Draw` = her tur, henüz teslim almamış üyeler arasından kura.
 */
export type OrderMode = 'Fixed' | 'Draw'

/**
 * Tur: Collecting → AwaitingPurchase → Settled. Ödeme gecikirse
 * Collecting → Grace → AwaitingPurchase. Kura modunda tüm katkılar tamamlanınca
 * önce AwaitingDraw gelir; kura çekilince AwaitingPurchase'a geçer.
 */
export type RoundPhase = 'Collecting' | 'Grace' | 'AwaitingDraw' | 'AwaitingPurchase' | 'Settled'

export interface PoolInfo {
  id: number
  creator: string
  /** Havuz asset'inin SAC kontrat adresi. */
  token: string
  contributionAmount: bigint
  memberLimit: number
  members: string[]
  /** Sıralı mı, kura mı. Eski (v8) kontratta alan yoksa 'Fixed' sayılır. */
  orderMode: OrderMode
  /** Önerilen veya başlatılınca kilitlenen tahsisat sırası. Kura modunda boştur. */
  recipientOrder: string[]
  /** Önerilen doğrulayıcılar (creator ve üyelerden farklı adresler). */
  verifiers: string[]
  /** Gereken doğrulayıcı onayı (demo için en az 2/3). */
  approvalThreshold: number
  /** Koşul sürümü; sıra veya doğrulayıcı değişince artar ve önceki onaylar silinir. 0 = önerilmedi. */
  termsVersion: number
  /** Geçerli koşul sürümünü onaylayan üyeler. */
  termsApprovals: string[]
  /** 1'den başlar. */
  currentRound: number
  status: PoolStatus
  /** Süreler (saniye): katkı, ek süre, alım. */
  roundDuration: number
  graceDuration: number
  purchaseDuration: number
  /** Kuruluş son tarihi: başlamazsa herkes iptal edebilir. */
  setupDeadline: number
  /** Demo için önceden belirlenmiş izinli test satıcısı. */
  demoSeller: string
}

export interface RoundInfo {
  round: number
  phase: RoundPhase
  /** Bu turun alıcısı. Kura modunda kura çekilene kadar null. */
  recipient: string | null
  startedAt: number
  /** Katkı son tarihi; geçince herkes turu Grace yapabilir. */
  collectDeadline: number
  /** Ek sürenin sonu (ilk katkı son tarihinden hesaplanır); 0 = henüz yok. */
  graceDeadline: number
  /** Alım için ayrı süre sonu; 0 = alım süresi henüz başlamadı. */
  purchaseDeadline: number
  /** Katkısını kendi cüzdanından yatıranlar (deposit veya cure_payment). */
  paid: string[]
  /** Bu turda satıcıya gidecek toplam tutar (katkı × üye sayısı). */
  pot: bigint
  /** Alıcının kaydettiği satıcı; henüz önerilmediyse null. */
  seller: string | null
  /** Zincir dışı alım belgesinin SHA-256 özeti (hex); yoksa null. */
  docHash: string | null
  /** Alım önerisi sürümü; her yeni öneride artar ve önceki onayları siler. */
  purchaseVersion: number
  /** Geçerli alım sürümünü onaylayan doğrulayıcılar. */
  approvals: string[]
}

export interface MemberStatus {
  address: string
  /** İptalde geri alabileceği tutar: yalnızca henüz ödenmemiş turun kendi katkısı. */
  refundable: bigint
  /** Tahsisatını (kendi sırasını) aldı mı? */
  received: boolean
}

export interface TxResult {
  hash: string | null
}
