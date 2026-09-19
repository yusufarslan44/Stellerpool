import type { IlloName } from '@/lib/icon-data'

export interface Goal {
  id: string
  label: string
  icon: IlloName
  /** Örnek hedef tutar (bir turda satıcıya giden). Yalnızca başlangıç değeridir. */
  pot: string
  members: number
}

// Kategoriler yalnızca arayüz etiketidir; kontrat için hepsi aynıdır.
export const GOALS: Goal[] = [
  { id: 'home', label: 'Ev', icon: 'home', pot: '60000', members: 12 },
  { id: 'car', label: 'Araç', icon: 'car', pot: '20000', members: 6 },
  { id: 'work', label: 'İş yeri', icon: 'work', pot: '30000', members: 8 },
  { id: 'other', label: 'Diğer', icon: 'gift', pot: '6000', members: 6 },
  { id: 'demo', label: 'Demo', icon: 'rocket', pot: '40', members: 4 },
]

const CENT = 100_000n // 0,01 birim (7 ondalık)

/** Tur başına katkı: hedef tutar / üye sayısı, 0,01 birime aşağı yuvarlanır (stroop cinsinden). */
export function contributionFor(potStroops: bigint, members: number): bigint | null {
  if (potStroops <= 0n || members < 1) return null
  const c = (potStroops / BigInt(members) / CENT) * CENT
  return c > 0n ? c : null
}
