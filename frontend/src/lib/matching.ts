import type { OrderMode, PoolInfo } from '@/types/pool'

/** Kullanıcının seçtiği planın havuz eşleştirmesinde önemli olan alanları. */
export interface PoolPlan {
  token: string
  demoSeller: string
  contributionAmount: bigint
  memberLimit: number
  orderMode: OrderMode
  /** Zincirde tutulan peşinat; kontrat peşinatı desteklemiyorsa 0n. */
  downPayment: bigint
  roundDuration: number
  graceDuration: number
  purchaseDuration: number
}

/**
 * Bir havuzun bu plana ve bu kullanıcıya uyup uymadığı. Katılım zincirde reddedilecek hiçbir havuz
 * önerilmez: dolu, başlamış, kuruluş süresi geçmiş, kullanıcı zaten üye ya da satıcı.
 */
export function poolMatches(pool: PoolInfo, plan: PoolPlan, me: string | null, nowSeconds: number): boolean {
  if (pool.status !== 'Filling') return false
  if (pool.members.length >= pool.memberLimit) return false
  if (pool.setupDeadline <= nowSeconds) return false
  if (pool.token !== plan.token) return false
  if (pool.demoSeller !== plan.demoSeller) return false
  if (pool.contributionAmount !== plan.contributionAmount) return false
  if (pool.memberLimit !== plan.memberLimit) return false
  if (pool.orderMode !== plan.orderMode) return false
  if (pool.downPayment !== plan.downPayment) return false
  if (pool.roundDuration !== plan.roundDuration) return false
  if (pool.graceDuration !== plan.graceDuration) return false
  if (pool.purchaseDuration !== plan.purchaseDuration) return false
  if (me !== null) {
    if (pool.members.includes(me)) return false
    if (pool.demoSeller === me) return false
  }
  return true
}

/** Uyan havuzlar, en dolu olan başta (daha çabuk başlar), eşitlikte en eski. */
export function findMatches(pools: PoolInfo[], plan: PoolPlan, me: string | null, nowSeconds: number): PoolInfo[] {
  return pools
    .filter((p) => poolMatches(p, plan, me, nowSeconds))
    .sort((a, b) => b.members.length - a.members.length || a.id - b.id)
}

/** Aynı plana uyan ama kullanıcının zaten üye olduğu bir havuz (yeniden katılma yerine oraya yönlendirmek için). */
export function findOwnPool(pools: PoolInfo[], plan: PoolPlan, me: string | null, nowSeconds: number): PoolInfo | null {
  if (me === null) return null
  return (
    pools
      .filter((p) => p.members.includes(me) && poolMatches({ ...p, members: [] }, plan, null, nowSeconds))
      .sort((a, b) => b.id - a.id)[0] ?? null
  )
}
