use soroban_sdk::{contractevent, Address, BytesN};

use crate::types::AbortReason;

#[contractevent]
pub struct PoolCreated {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub creator: Address,
    #[topic]
    pub sponsor: Address,
    pub token: Address,
    pub contribution_amount: i128,
    pub guarantee_required: i128,
    pub member_limit: u32,
    pub round_duration_secs: u64,
    pub grace_duration_secs: u64,
}

#[contractevent]
pub struct GuaranteeFunded {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub sponsor: Address,
    pub amount: i128,
    pub total_guarantee: i128,
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
pub struct MemberLeft {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub member: Address,
    pub left_at: u64,
}

#[contractevent]
pub struct PoolStarted {
    #[topic]
    pub pool_id: u64,
    pub started_at: u64,
    pub first_deadline: u64,
    pub funded_guarantee: i128,
}

#[contractevent]
pub struct VerifierPolicyConfigured {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub creator: Address,
    pub verifier_count: u32,
    pub approval_quorum: u32,
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
pub struct PurchaseProposed {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub recipient: Address,
    pub seller: Address,
    pub document_digest: BytesN<32>,
}

#[contractevent]
pub struct PurchaseApproved {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub verifier: Address,
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
    pub sponsor_top_up: i128,
}

#[contractevent]
pub struct RoundOverdue {
    #[topic]
    pub pool_id: u64,
    pub round: u32,
    pub grace_deadline: u64,
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
pub struct SponsorTopUp {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub round: u32,
    #[topic]
    pub sponsor: Address,
    pub amount: i128,
}

#[contractevent]
pub struct PoolPaused {
    #[topic]
    pub pool_id: u64,
    pub round: u32,
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

#[contractevent]
pub struct SponsorRemainderClaimed {
    #[topic]
    pub pool_id: u64,
    #[topic]
    pub sponsor: Address,
    pub amount: i128,
}
