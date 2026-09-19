/**
 * Kontrat modeli, docs/plan.md bölüm 5'teki sponsor güvenceli havuz tasarımına göre yazılmıştır
 * (durumlar, tur evreleri, koşul sürümü, sponsor avansı). Kontrat henüz deploy edilmedi; alan
 * adları kontrat yazılınca doğrulanacak. Tutarlar stroop (7 ondalık) cinsinden bigint,
 * zamanlar unix saniyesidir.
 */

/** Havuz: Filling → Active → Completed / Aborted. */
export type PoolStatus = 'Filling' | 'Active' | 'Completed' | 'Aborted'

/**
 * Tur: Collecting → AwaitingPurchase → Settled. Ödeme veya eski avans kapanışı gecikirse
 * Collecting → Grace → AwaitingPurchase.
 */
export type RoundPhase = 'Collecting' | 'Grace' | 'AwaitingPurchase' | 'Settled'

export interface PoolInfo {
  id: number
  creator: string
  /** Sponsor güvencesini yatıran ve eksik katkıyı yeni fonla tamamlayabilen taraf. */
  sponsor: string
  /** Havuz asset'inin SAC kontrat adresi. */
  token: string
  contributionAmount: bigint
  memberLimit: number
  members: string[]
  /** Önerilen veya başlatılınca kilitlenen tahsisat sırası. */
  recipientOrder: string[]
  /** Önerilen doğrulayıcılar (creator, sponsor ve üyelerden farklı adresler). */
  verifiers: string[]
  /** Gereken doğrulayıcı onayı (demo için en az 2/3). */
  approvalThreshold: number
  /** Koşul sürümü; sıra veya doğrulayıcı değişince artar ve önceki onaylar silinir. 0 = önerilmedi. */
  termsVersion: number
  /** Geçerli koşul sürümünü onaylayanlar (üyeler ve sponsor). */
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
  requiredGuarantee: bigint
  guaranteeDeposited: bigint
}

export interface RoundInfo {
  round: number
  phase: RoundPhase
  recipient: string
  startedAt: number
  /** Katkı son tarihi; geçince herkes turu Grace yapabilir. */
  collectDeadline: number
  /** Ek sürenin sonu (ilk katkı son tarihinden hesaplanır); 0 = henüz yok. */
  graceDeadline: number
  /** Alım için ayrı süre sonu; 0 = alım süresi henüz başlamadı. */
  purchaseDeadline: number
  /** Katkısını kendi cüzdanından yatıranlar (deposit veya cure_payment). */
  paid: string[]
  /** Katkısı sponsorun yeni fonuyla tamamlananlar. Üyenin kendi katkısı sayılmaz. */
  sponsorAdvanced: string[]
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
  /** İptalde geri alabileceği tutar (yalnızca bizzat yatırdığı katkılardan doğar). */
  refundable: bigint
  /** Tahsisatını (kendi sırasını) aldı mı? */
  received: boolean
  /** Sponsora geri ödenmesi gereken açık avans; sponsor avansı üyenin iade hakkı sayılmaz. */
  advanceOwed: bigint
}

export interface TxResult {
  hash: string | null
}
