use soroban_sdk::{contracttype, Address, BytesN, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolStatus {
    Filling,
    Active,
    Completed,
    Aborted,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundPhase {
    Collecting,
    Grace,
    AwaitingPurchase,
    Settled,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbortReason {
    /// A round's collect deadline plus grace period expired while still underfunded,
    /// or the recipient still had an open sponsor advance or unpaid own contribution.
    SafetyRecovery,
    /// A round became fully funded and debt-clear (AwaitingPurchase) but no approved
    /// purchase was executed before the purchase deadline.
    BlockedSettlement,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    pub id: u64,
    pub creator: Address,
    pub sponsor: Address,
    pub token: Address,
    pub contribution_amount: i128,
    pub member_limit: u32,
    pub members: Vec<Address>,
    pub recipient_order: Vec<Address>,
    pub verifiers: Vec<Address>,
    pub approval_threshold: u32,
    pub terms_version: u32,
    pub terms_approvals: Vec<Address>,
    pub current_round: u32,
    pub status: PoolStatus,
    pub round_duration: u64,
    pub grace_duration: u64,
    pub purchase_duration: u64,
    pub setup_deadline: u64,
    pub demo_seller: Address,
    pub required_guarantee: i128,
    pub guarantee_deposited: i128,
    pub created_at: u64,
    pub started_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberState {
    pub joined_at: u64,
    pub active: bool,
    pub received: bool,
    pub contributions_paid: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoundState {
    pub round: u32,
    pub phase: RoundPhase,
    pub recipient: Address,
    pub started_at: u64,
    pub collect_deadline: u64,
    pub grace_deadline: Option<u64>,
    pub purchase_deadline: Option<u64>,
    pub paid: Vec<Address>,
    pub sponsor_advanced: Vec<Address>,
    pub pot: i128,
    pub seller: Option<Address>,
    pub asset: Option<Address>,
    pub amount: Option<i128>,
    pub doc_hash: Option<BytesN<32>>,
    pub purchase_version: u32,
    pub approvals: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberStatusView {
    pub refundable: i128,
    pub received: bool,
}
