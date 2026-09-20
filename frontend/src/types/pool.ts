/**
 * Kontrat modeli, docs/plan.md bölüm 5'teki sponsorsuz havuz tasarımına göre yazılmıştır
 * (Testnet'te yayındaki API v10 ile alan alan doğrulandı: canlı okuma ve simülasyon).
 * Tutarlar stroop (7 ondalık) cinsinden bigint, zamanlar unix saniyesidir.
 */

/** Üye sayısı sınırları (yayındaki kontrat API v10: 2–30). */
export const MIN_MEMBERS = 2
/** Arayüzün ve yayındaki kontratın (API v10) desteklediği en büyük grup. */
export const UI_MAX_MEMBERS = 30
/** Varsayılan üst sınır: API v10 kontratının kabul ettiği 30. */
const DEFAULT_MAX_MEMBERS = 30
const configuredMax = Number.parseInt(String(import.meta.env.VITE_MAX_MEMBERS ?? ''), 10)
/**
 * Formun izin verdiği üst sınır. Varsayılan 30'dur (API v10). Eski bir kontrata (v9, en çok 12 üye)
 * bağlanılıyorsa `VITE_MAX_MEMBERS=12` yazın; aksi halde 13–30 üyeli bir havuz zincirde reddedilir.
 */
export const MAX_MEMBERS =
  Number.isInteger(configuredMax) && configuredMax >= MIN_MEMBERS && configuredMax <= UI_MAX_MEMBERS
    ? configuredMax
    : DEFAULT_MAX_MEMBERS
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
  /** Üye başına peşinat (katılırken kontrata yatırılır, sıra gelince o üyenin alımına eklenir). 0 = yok. */
  downPayment: bigint
  /** Önerilen veya başlatılınca kilitlenen tahsisat sırası. Kura modunda boştur. */
  recipientOrder: string[]
  /** Koşul sürümü; havuz dolduğunda 1 olur. */
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
