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
  /** Alıcı belirleme yöntemi: ev/arabada kura, diğer planda katılım sırası. */
  mode: OrderMode
  /** Normal planlar aylık; hızlı demo formda ayrı seçilir. */
  interval: 'month' | 'week' | 'day' | 'demo'
  /**
   * Önerilen peşinat oranı (% olarak, toplam bedelin). Peşinat alıcının satıcıya havuz dışında öderdiği
   * kısımdır; `pot` havuzun karşıladığı kısımdır (bedel − peşinat).
   */
  down: number
}

// Categories are UI labels only; they are identical for the contract. The home/car options
// are sample values showing the scale of savings-finance groups (draw, larger groups);
// no real home/car delivery exists (docs/altin-gunu-legal-boundary.md).
export const GOALS: Goal[] = [
  { id: 'home', label: 'Home', icon: 'home', pot: '60000', members: 24, mode: 'Draw', interval: 'month', down: 20 },
  { id: 'car', label: 'Car', icon: 'car', pot: '24000', members: 12, mode: 'Draw', interval: 'month', down: 20 },
  { id: 'other', label: 'Other', icon: 'gift', pot: '6000', members: 6, mode: 'Fixed', interval: 'month', down: 0 },
]

/** Sample group size, capped by the upper limit the form allows. */
export const goalMembers = (g: Goal) => Math.min(g.members, MAX_MEMBERS)

const CENT = 100_000n // 0.01 unit (7 decimals)

/** Contribution per round: target amount / member count, rounded down to 0.01 unit (in stroops). */
export function contributionFor(potStroops: bigint, members: number): bigint | null {
  if (potStroops <= 0n || members < 1) return null
  const c = (potStroops / BigInt(members) / CENT) * CENT
  return c > 0n ? c : null
}

/** Total price: derived back from the amount the pool covers and the down-payment percentage. */
export function priceFor(potStroops: bigint, downPct: number): bigint {
  const rest = BigInt(100 - Math.min(99, Math.max(0, Math.round(downPct))))
  return (potStroops * 100n) / rest
}
