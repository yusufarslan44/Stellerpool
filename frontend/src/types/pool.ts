/**
 * Kontrat modeli, docs/plan.md'deki sponsor güvenceli havuz tasarımına göre yazılmıştır.
 * Kontrat henüz deploy edilmedi; alan adları kontrat yazılınca doğrulanacak.
 * Tutarlar stroop (7 ondalık) cinsinden bigint, zamanlar unix saniyesidir.
 */
export type PoolStatus = 'Filling' | 'Active' | 'Paused' | 'Completed' | 'Aborted'

export interface PoolInfo {
  id: number
  creator: string
  /** Sponsor güvencesini yatıran ve eksik katkıyı tamamlayabilen taraf. */
  sponsor: string
  /** Havuz asset'inin SAC kontrat adresi. */
  token: string
  contributionAmount: bigint
  memberLimit: number
  members: string[]
  /** Başlatılınca kilitlenen tahsisat sırası. Başlamadan önce boş olabilir. */
  recipientOrder: string[]
  /** 1'den başlar. */
  currentRound: number
  /** Her turun son ödeme süresi (saniye). */
  roundDuration: number
  /** Kontratın hesapladığı gerekli sponsor güvencesi. */
  requiredGuarantee: bigint
  guaranteeDeposited: bigint
  /** Alım kaydını onaylayan doğrulayıcılar ve gereken onay sayısı. */
  verifiers: string[]
  approvalThreshold: number
  status: PoolStatus
}

export interface RoundInfo {
  round: number
  recipient: string
  startedAt: number
  deadline: number
  /** Bu tur katkısını kendisi yatıranlar. */
  paid: string[]
  /** Katkısı sponsorun ilave fonuyla tamamlananlar (sponsor adına kayıtlı). */
  sponsorCovered: string[]
  /** Bu turda satıcıya gidecek toplam tutar (katkı × üye sayısı). */
  pot: bigint
  /** Alıcının önerdiği satıcı; henüz önerilmediyse null. */
  seller: string | null
  /** Zincir dışı alım belgesinin SHA-256 özeti (hex); henüz yoksa null. */
  docHash: string | null
  /** Alım kaydını onaylayan doğrulayıcılar. */
  approvals: string[]
  /** İptalin tetiklenebileceği zaman (Paused durumunda); yoksa 0. */
  abortAfter: number
}

export interface MemberStatus {
  address: string
  /** Bu üyenin toplam yatırdığı katkı. */
  contributed: bigint
  /** İptal olursa bu üyenin geri alabileceği tutar. */
  refundable: bigint
  /** Tahsisatını (kendi sırasını) aldı mı? */
  received: boolean
}

export interface TxResult {
  hash: string | null
}
