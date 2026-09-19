use soroban_sdk::{contractevent, Address, BytesN};

use crate::types::{AbortReason, OrderMode};

#[contractevent]
pub struct PoolCreated {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub creator: Address,
    pub token: Address,
    pub contribution_amount: i128,
    pub member_limit: u32,
    pub order_mode: OrderMode,
    pub round_duration: u64,
    pub grace_duration: u64,
    pub purchase_duration: u64,
    pub setup_deadline: u64,
    pub demo_seller: Address,
}

#[contractevent]
pub struct MemberJoined {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub member: Address,
    pub joined_at: u64,
}

#[contractevent]
pub struct TermsProposed {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub creator: Address,
    pub version: u32,
    pub verifier_count: u32,
    pub approval_threshold: u32,
}

#[contractevent]
pub struct TermsApproved {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub approver: Address,
    pub version: u32,
}

#[contractevent]
pub struct PoolStarted {
    #[topic]
    pub pool_id: u64,
    pub started_at: u64,
    pub first_deadline: u64,
}

#[contractevent]
pub struct PoolCancelled {
    #[topic]
    pub pool_id: u64,
}

#[contractevent]
pub struct ContributionDeposited {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub member: Address,
    pub amount: i128,
}

#[contractevent]
pub struct PaymentCured {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub member: Address,
    pub amount: i128,
}

#[contractevent]
pub struct PurchaseProposed {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub recipient: Address,
    pub seller: Address,
    pub asset: Address,
    pub amount: i128,
    pub doc_hash: BytesN<32>,
    pub version: u32,
}

#[contractevent]
pub struct PurchaseApproved {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub verifier: Address,
    pub version: u32,
    pub approval_count: u32,
}

#[contractevent]
pub struct RoundPaid {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub recipient: Address,
    pub seller: Address,
    pub amount: i128,
}

#[contractevent]
pub struct RoundOverdue {
    #[topic]
    pub pool_id: u64,
    pub round: u32,
    pub grace_deadline: u64,
}

#[contractevent]
pub struct RoundAwaitingPurchase {
    #[topic]
    pub pool_id: u64,
    pub round: u32,
    pub purchase_deadline: u64,
}

/// Draw-mode only: every member has paid into the round; `purchase_deadline` starts now and
/// covers both the draw and the purchase steps (see `draw_recipient`).
#[contractevent]
pub struct RoundAwaitingDraw {
    #[topic]
    pub pool_id: u64,
    pub round: u32,
    pub purchase_deadline: u64,
}

#[contractevent]
pub struct RecipientDrawn {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    pub recipient: Address,
}

#[contractevent]
pub struct PoolCompleted {
    #[topic]
    pub pool_id: u64,
    pub completed_at: u64,
}

#[contractevent]
pub struct PoolAborted {
    #[topic]
    pub pool_id: u64,
    pub round: u32,
    pub reason: AbortReason,
}

#[contractevent]
pub struct RefundClaimed {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub member: Address,
    pub amount: i128,
}
