/**
 * The contract model is written for the sponsor-free pool design in docs/plan.md section 5
 * (verified field by field against API v10 published on Testnet: live reads and simulation).
 * Amounts are bigint in stroops (7 decimals); times are unix seconds.
 */

/** Member count limits (published contract API v10: 2–30). */
export const MIN_MEMBERS = 2
/** The largest group supported by the interface and the published contract (API v10). */
export const UI_MAX_MEMBERS = 30
/** Default upper limit: the 30 accepted by the API v10 contract. */
const DEFAULT_MAX_MEMBERS = 30
const configuredMax = Number.parseInt(String(import.meta.env.VITE_MAX_MEMBERS ?? ''), 10)
/**
 * The upper limit the form allows. The default is 30 (API v10). If you connect to an old contract (v9, at most 12 members),
 * set `VITE_MAX_MEMBERS=12`; otherwise a pool of 13–30 members is rejected on-chain.
 */
export const MAX_MEMBERS =
  Number.isInteger(configuredMax) && configuredMax >= MIN_MEMBERS && configuredMax <= UI_MAX_MEMBERS
    ? configuredMax
    : DEFAULT_MAX_MEMBERS
/** Havuz: Filling → Active → Completed / Aborted. */
export type PoolStatus = 'Filling' | 'Active' | 'Completed' | 'Aborted'

/**
 * How the recipient is chosen: `Fixed` = the fixed order approved by the members,
 * `Draw` = a draw each round among members who have not yet received.
 */
export type OrderMode = 'Fixed' | 'Draw'

/**
 * Round: Collecting → AwaitingPurchase → Settled. If a payment is late,
 * Collecting → Grace → AwaitingPurchase. In draw mode, once all contributions are complete,
 * AwaitingDraw comes first; after the draw it moves to AwaitingPurchase.
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
  /** Ordered or draw. If the field is missing in an old (v8) contract, 'Fixed' is assumed. */
  orderMode: OrderMode
  /** Down payment per member (paid into the contract when joining; added to that member's purchase when their turn comes). 0 = none. */
  downPayment: bigint
  /** The allocation order proposed or locked when started. Empty in draw mode. */
  recipientOrder: string[]
  /** Terms version; becomes 1 when the pool fills. */
  termsVersion: number
  /** Members who approved the current terms version. */
  termsApprovals: string[]
  /** Starts from 1. */
  currentRound: number
  status: PoolStatus
  /** Durations (seconds): contribution, grace, purchase. */
  roundDuration: number
  graceDuration: number
  purchaseDuration: number
  /** Setup deadline: if it does not start, anyone can cancel. */
  setupDeadline: number
  /** The allowed test seller predefined for the demo. */
  demoSeller: string
}

export interface RoundInfo {
  round: number
  phase: RoundPhase
  /** This round's recipient. In draw mode null until the draw is held. */
  recipient: string | null
  startedAt: number
  /** Contribution deadline; after it, anyone can move the round to Grace. */
  collectDeadline: number
  /** End of the grace period (computed from the first contribution deadline); 0 = not yet. */
  graceDeadline: number
  /** Separate end time for the purchase; 0 = the purchase period has not started yet. */
  purchaseDeadline: number
  /** Those who paid their contribution from their own wallet (deposit or cure_payment). */
  paid: string[]
  /** The total going to the seller in this round (contribution × member count). */
  pot: bigint
  /** The seller recorded by the recipient; null if not yet proposed. */
  seller: string | null
  /** SHA-256 digest (hex) of the off-chain purchase document; null if none. */
  docHash: string | null
}

export interface MemberStatus {
  address: string
  /** What they can recover on cancellation: only their own contribution for the round not yet paid out. */
  refundable: bigint
  /** Have they received their allocation (their own turn)? */
  received: boolean
}

export interface TxResult {
  hash: string | null
}
