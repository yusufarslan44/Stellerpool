use soroban_sdk::{contracttype, Address, BytesN, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolStatus {
    Filling,
    Active,
    Completed,
    Aborted,
}

/// How a round's recipient is chosen. `Fixed` uses the order approved in `propose_terms`;
/// `Draw` picks uniformly among members who have not received yet, once a round is fully
/// funded (see `RoundPhase::AwaitingDraw`).
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderMode {
    Fixed,
    Draw,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundPhase {
    Collecting,
    Grace,
    /// Draw-mode only: every member has paid into this round but the recipient has not been
    /// drawn yet. `purchase_deadline` already runs during this phase (see `draw_recipient`).
    AwaitingDraw,
    AwaitingPurchase,
    Settled,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbortReason {
    /// A round's collect deadline plus grace period expired while still underfunded.
    SafetyRecovery,
    /// A round became fully funded (AwaitingPurchase, or AwaitingDraw in Draw mode) but no
    /// approved purchase was executed (or no recipient was drawn) before the purchase deadline.
    BlockedSettlement,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    pub id: u64,
    pub creator: Address,
    pub token: Address,
    pub contribution_amount: i128,
    pub member_limit: u32,
    pub order_mode: OrderMode,
    /// Per-member down payment (peşinat), escrowed by `join_pool` and added to that member's own
    /// purchase when their round settles. `0` disables it (v10 behaviour).
    pub down_payment: i128,
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
    pub created_at: u64,
    pub started_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberState {
    pub joined_at: u64,
    pub received: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoundState {
    pub round: u32,
    pub phase: RoundPhase,
    /// Fixed mode: set as soon as the round starts. Draw mode: `None` until `draw_recipient`
    /// resolves it in `AwaitingDraw`.
    pub recipient: Option<Address>,
    pub started_at: u64,
    pub collect_deadline: u64,
    pub grace_deadline: Option<u64>,
    pub purchase_deadline: Option<u64>,
    pub paid: Vec<Address>,
    pub pot: i128,
    pub seller: Option<Address>,
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
