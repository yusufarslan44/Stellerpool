// Storage helpers are kept typed and pool-scoped so contract-wide token balances
// never become the source of truth for an individual pool.
#![allow(dead_code)]

use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::types::{MemberState, Pool, PurchaseState, RoundState, VerifierPolicy};

pub const INITIAL_POOL_ID: u64 = 1;
pub const INSTANCE_TTL_THRESHOLD: u32 = 30 * 17_280;
pub const INSTANCE_TTL_EXTEND_TO: u32 = 90 * 17_280;
pub const PERSISTENT_TTL_THRESHOLD: u32 = 30 * 17_280;
pub const PERSISTENT_TTL_EXTEND_TO: u32 = 120 * 17_280;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NextPoolId,
    Pool(u64),
    PoolMembers(u64),
    Member(u64, Address),
    RecipientOrder(u64),
    VerifierPolicy(u64),
    Round(u64, u32),
    Deposit(u64, u32, Address),
    RoundPot(u64, u32),
    GuaranteeBalance(u64),
    MemberContributionTotal(u64, Address),
    PoolAssignedBalance(u64),
    RoundTopUp(u64, u32),
    RefundLiability(u64, Address),
    TotalRefundLiability(u64),
    Purchase(u64, u32),
    VerifierApproval(u64, u32, Address),
    RefundClaimed(u64, Address),
    SponsorRemainderClaimed(u64),
}

pub(crate) fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

pub(crate) fn read_next_pool_id(env: &Env) -> u64 {
    let key = DataKey::NextPoolId;
    let value = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or(INITIAL_POOL_ID);
    extend_instance_ttl(env);
    value
}

pub(crate) fn write_next_pool_id(env: &Env, next_pool_id: u64) {
    env.storage()
        .instance()
        .set(&DataKey::NextPoolId, &next_pool_id);
    extend_instance_ttl(env);
}

pub(crate) fn read_pool(env: &Env, pool_id: u64) -> Option<Pool> {
    let key = DataKey::Pool(pool_id);
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_pool(env: &Env, pool: &Pool) {
    let key = DataKey::Pool(pool.id);
    env.storage().persistent().set(&key, pool);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_members(env: &Env, pool_id: u64) -> Vec<Address> {
    let key = DataKey::PoolMembers(pool_id);
    let value = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_members(env: &Env, pool_id: u64, members: &Vec<Address>) {
    let key = DataKey::PoolMembers(pool_id);
    env.storage().persistent().set(&key, members);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_member(env: &Env, pool_id: u64, member: &Address) -> Option<MemberState> {
    let key = DataKey::Member(pool_id, member.clone());
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_member(env: &Env, pool_id: u64, member: &Address, state: &MemberState) {
    let key = DataKey::Member(pool_id, member.clone());
    env.storage().persistent().set(&key, state);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn remove_member(env: &Env, pool_id: u64, member: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Member(pool_id, member.clone()));
}

pub(crate) fn read_recipient_order(env: &Env, pool_id: u64) -> Option<Vec<Address>> {
    let key = DataKey::RecipientOrder(pool_id);
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_recipient_order(env: &Env, pool_id: u64, order: &Vec<Address>) {
    let key = DataKey::RecipientOrder(pool_id);
    env.storage().persistent().set(&key, order);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_verifier_policy(env: &Env, pool_id: u64) -> Option<VerifierPolicy> {
    let key = DataKey::VerifierPolicy(pool_id);
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_verifier_policy(env: &Env, pool_id: u64, policy: &VerifierPolicy) {
    let key = DataKey::VerifierPolicy(pool_id);
    env.storage().persistent().set(&key, policy);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_round(env: &Env, pool_id: u64, round: u32) -> Option<RoundState> {
    let key = DataKey::Round(pool_id, round);
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_round(env: &Env, pool_id: u64, state: &RoundState) {
    let key = DataKey::Round(pool_id, state.index);
    env.storage().persistent().set(&key, state);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn has_deposit(env: &Env, pool_id: u64, round: u32, member: &Address) -> bool {
    let key = DataKey::Deposit(pool_id, round, member.clone());
    let deposited = env.storage().persistent().get(&key).unwrap_or(false);
    if deposited {
        extend_persistent_ttl(env, &key);
    }
    deposited
}

pub(crate) fn write_deposit(env: &Env, pool_id: u64, round: u32, member: &Address) {
    let key = DataKey::Deposit(pool_id, round, member.clone());
    env.storage().persistent().set(&key, &true);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_round_pot(env: &Env, pool_id: u64, round: u32) -> i128 {
    let key = DataKey::RoundPot(pool_id, round);
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_round_pot(env: &Env, pool_id: u64, round: u32, amount: i128) {
    let key = DataKey::RoundPot(pool_id, round);
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_round_top_up(env: &Env, pool_id: u64, round: u32) -> i128 {
    let key = DataKey::RoundTopUp(pool_id, round);
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_round_top_up(env: &Env, pool_id: u64, round: u32, amount: i128) {
    let key = DataKey::RoundTopUp(pool_id, round);
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_guarantee_balance(env: &Env, pool_id: u64) -> i128 {
    let key = DataKey::GuaranteeBalance(pool_id);
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_guarantee_balance(env: &Env, pool_id: u64, amount: i128) {
    let key = DataKey::GuaranteeBalance(pool_id);
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_member_contribution_total(env: &Env, pool_id: u64, member: &Address) -> i128 {
    let key = DataKey::MemberContributionTotal(pool_id, member.clone());
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_member_contribution_total(
    env: &Env,
    pool_id: u64,
    member: &Address,
    amount: i128,
) {
    let key = DataKey::MemberContributionTotal(pool_id, member.clone());
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_pool_assigned_balance(env: &Env, pool_id: u64) -> i128 {
    let key = DataKey::PoolAssignedBalance(pool_id);
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_pool_assigned_balance(env: &Env, pool_id: u64, amount: i128) {
    let key = DataKey::PoolAssignedBalance(pool_id);
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_refund_liability(env: &Env, pool_id: u64, member: &Address) -> i128 {
    let key = DataKey::RefundLiability(pool_id, member.clone());
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_refund_liability(env: &Env, pool_id: u64, member: &Address, amount: i128) {
    let key = DataKey::RefundLiability(pool_id, member.clone());
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_total_refund_liability(env: &Env, pool_id: u64) -> i128 {
    let key = DataKey::TotalRefundLiability(pool_id);
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_total_refund_liability(env: &Env, pool_id: u64, amount: i128) {
    let key = DataKey::TotalRefundLiability(pool_id);
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn read_purchase(env: &Env, pool_id: u64, round: u32) -> Option<PurchaseState> {
    let key = DataKey::Purchase(pool_id, round);
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_purchase(env: &Env, pool_id: u64, round: u32, purchase: &PurchaseState) {
    let key = DataKey::Purchase(pool_id, round);
    env.storage().persistent().set(&key, purchase);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn has_verifier_approval(
    env: &Env,
    pool_id: u64,
    round: u32,
    verifier: &Address,
) -> bool {
    let key = DataKey::VerifierApproval(pool_id, round, verifier.clone());
    let approved = env.storage().persistent().get(&key).unwrap_or(false);
    if approved {
        extend_persistent_ttl(env, &key);
    }
    approved
}

pub(crate) fn write_verifier_approval(env: &Env, pool_id: u64, round: u32, verifier: &Address) {
    let key = DataKey::VerifierApproval(pool_id, round, verifier.clone());
    env.storage().persistent().set(&key, &true);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn has_refund_claimed(env: &Env, pool_id: u64, member: &Address) -> bool {
    let key = DataKey::RefundClaimed(pool_id, member.clone());
    let claimed = env.storage().persistent().get(&key).unwrap_or(false);
    if claimed {
        extend_persistent_ttl(env, &key);
    }
    claimed
}

pub(crate) fn write_refund_claimed(env: &Env, pool_id: u64, member: &Address) {
    let key = DataKey::RefundClaimed(pool_id, member.clone());
    env.storage().persistent().set(&key, &true);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn has_sponsor_remainder_claimed(env: &Env, pool_id: u64) -> bool {
    let key = DataKey::SponsorRemainderClaimed(pool_id);
    let claimed = env.storage().persistent().get(&key).unwrap_or(false);
    if claimed {
        extend_persistent_ttl(env, &key);
    }
    claimed
}

pub(crate) fn write_sponsor_remainder_claimed(env: &Env, pool_id: u64) {
    let key = DataKey::SponsorRemainderClaimed(pool_id);
    env.storage().persistent().set(&key, &true);
    extend_persistent_ttl(env, &key);
}

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}
