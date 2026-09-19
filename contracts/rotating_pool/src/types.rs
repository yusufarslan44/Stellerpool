use soroban_sdk::{contracttype, Address, BytesN, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolStatus {
    Filling,
    Active,
    Grace,
    Paused,
    Completed,
    Aborted,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundStatus {
    Collecting,
    Executed,
    Settled,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbortReason {
    BlockedSettlement,
    SafetyRecovery,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    pub id: u64,
    pub creator: Address,
    pub sponsor: Address,
    pub token: Address,
    pub contribution_amount: i128,
    pub guarantee_required: i128,
    pub member_limit: u32,
    pub member_count: u32,
    pub round_duration_secs: u64,
    pub grace_duration_secs: u64,
    pub status: PoolStatus,
    pub current_round: u32,
    pub created_at: u64,
    pub started_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberState {
    pub joined_at: u64,
    pub active: bool,
    pub allocation_received: bool,
    pub contributions_paid: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoundState {
    pub index: u32,
    pub recipient: Address,
    pub started_at: u64,
    pub deadline: u64,
    pub grace_deadline: Option<u64>,
    pub deposit_count: u32,
    pub pot_amount: i128,
    pub status: RoundStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifierPolicy {
    pub verifiers: Vec<Address>,
    pub approval_quorum: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurchaseState {
    pub seller: Address,
    pub document_digest: BytesN<32>,
    pub proposed_by: Address,
    pub proposed_at: u64,
    pub approval_count: u32,
    pub approved: bool,
}
