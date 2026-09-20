import type { OrderMode, PoolInfo } from '@/types/pool'

/** The fields of the plan the user chose that matter for pool matching. */
export interface PoolPlan {
  token: string
  demoSeller: string
  contributionAmount: bigint
  memberLimit: number
  orderMode: OrderMode
  /** The down payment held on-chain; 0n if the contract does not support a down payment. */
  downPayment: bigint
  roundDuration: number
  graceDuration: number
  purchaseDuration: number
}

/**
 * Whether a pool fits this plan and this user. No pool that the chain would reject on joining is suggested:
 * full, already started, setup period over, the user is already a member or the seller.
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

/** Matching pools, the fullest first (it starts sooner), the oldest on a tie. */
export function findMatches(pools: PoolInfo[], plan: PoolPlan, me: string | null, nowSeconds: number): PoolInfo[] {
  return pools
    .filter((p) => poolMatches(p, plan, me, nowSeconds))
    .sort((a, b) => b.members.length - a.members.length || a.id - b.id)
}

/** A pool that fits the same plan but that the user already belongs to (to route there instead of joining again). */
export function findOwnPool(pools: PoolInfo[], plan: PoolPlan, me: string | null, nowSeconds: number): PoolInfo | null {
  if (me === null) return null
  return (
    pools
      .filter((p) => p.members.includes(me) && poolMatches({ ...p, members: [] }, plan, null, nowSeconds))
      .sort((a, b) => b.id - a.id)[0] ?? null
  )
}
