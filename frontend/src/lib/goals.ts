import type { IlloName } from '@/lib/icon-data'
import { MAX_MEMBERS } from '@/types/pool'
import type { OrderMode } from '@/types/pool'

export interface Goal {
  id: string
  label: string
  icon: IlloName
  /** Örnek hedef tutar (bir turda satıcıya giden). Yalnızca başlangıç değeridir. */
  pot: string
  members: number
  /** Önerilen alıcı belirleme yöntemi; formda değiştirilebilir. */
  mode: OrderMode
  /** Önerilen taksit aralığı (formdaki takvim ön ayarı); gerçek hedefler aylık, demolar hızlı. */
  interval: 'month' | 'week' | 'day' | 'demo'
  /**
   * Önerilen peşinat oranı (% olarak, toplam bedelin). Peşinat alıcının satıcıya havuz dışında öderdiği
   * kısımdır; `pot` havuzun karşıladığı kısımdır (bedel − peşinat).
   */
  down: number
}

// Kategoriler yalnızca arayüz etiketidir; kontrat için hepsi aynıdır. Ev/araç seçenekleri
// tasarruf finansman gruplarının ölçeğini (kura, daha büyük grup) gösteren örnek değerlerdir;
// gerçek ev/araç teslimi yoktur (docs/altin-gunu-legal-boundary.md).
export const GOALS: Goal[] = [
  { id: 'home', label: 'Ev', icon: 'home', pot: '60000', members: 24, mode: 'Draw', interval: 'month', down: 20 },
  { id: 'car', label: 'Araç', icon: 'car', pot: '24000', members: 12, mode: 'Draw', interval: 'month', down: 20 },
  { id: 'work', label: 'İş yeri', icon: 'work', pot: '30000', members: 8, mode: 'Fixed', interval: 'month', down: 20 },
  { id: 'other', label: 'Diğer', icon: 'gift', pot: '6000', members: 6, mode: 'Fixed', interval: 'month', down: 0 },
  { id: 'drawdemo', label: 'Kura demo', icon: 'dice', pot: '50', members: 5, mode: 'Draw', interval: 'demo', down: 0 },
  { id: 'demo', label: 'Demo', icon: 'rocket', pot: '40', members: 4, mode: 'Fixed', interval: 'demo', down: 0 },
]

/** Örnek grup büyüklüğü, formun izin verdiği üst sınırla kısıtlanır. */
export const goalMembers = (g: Goal) => Math.min(g.members, MAX_MEMBERS)

const CENT = 100_000n // 0,01 birim (7 ondalık)

/** Tur başına katkı: hedef tutar / üye sayısı, 0,01 birime aşağı yuvarlanır (stroop cinsinden). */
export function contributionFor(potStroops: bigint, members: number): bigint | null {
  if (potStroops <= 0n || members < 1) return null
  const c = (potStroops / BigInt(members) / CENT) * CENT
  return c > 0n ? c : null
}

/** Toplam bedel: havuzun karşıladığı tutar ve peşinat oranından (% olarak) geri hesaplanır. */
export function priceFor(potStroops: bigint, downPct: number): bigint {
  const rest = BigInt(100 - Math.min(99, Math.max(0, Math.round(downPct))))
  return (potStroops * 100n) / rest
}
