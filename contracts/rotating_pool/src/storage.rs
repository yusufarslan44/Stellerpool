// Storage helpers are kept typed and pool-scoped so contract-wide token balances
// never become the source of truth for an individual pool.
#![allow(dead_code)]

use soroban_sdk::{contracttype, Address, Env};

use crate::types::{MemberState, Pool, RoundState};

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
    Member(u64, Address),
    Round(u64, u32),
    Deposit(u64, u32, Address),
    PoolAssignedBalance(u64),
    TermsApproval(u64, u32, Address),
    PurchaseApproval(u64, u32, u32, Address),
    RefundClaimed(u64, Address),
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

pub(crate) fn read_round(env: &Env, pool_id: u64, round: u32) -> Option<RoundState> {
    let key = DataKey::Round(pool_id, round);
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_persistent_ttl(env, &key);
    }
    value
}

pub(crate) fn write_round(env: &Env, pool_id: u64, state: &RoundState) {
    let key = DataKey::Round(pool_id, state.round);
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

pub(crate) fn has_terms_approval(
    env: &Env,
    pool_id: u64,
    version: u32,
    approver: &Address,
) -> bool {
    let key = DataKey::TermsApproval(pool_id, version, approver.clone());
    let approved = env.storage().persistent().get(&key).unwrap_or(false);
    if approved {
        extend_persistent_ttl(env, &key);
    }
    approved
}

pub(crate) fn write_terms_approval(env: &Env, pool_id: u64, version: u32, approver: &Address) {
    let key = DataKey::TermsApproval(pool_id, version, approver.clone());
    env.storage().persistent().set(&key, &true);
    extend_persistent_ttl(env, &key);
}

pub(crate) fn has_purchase_approval(
    env: &Env,
    pool_id: u64,
    round: u32,
    version: u32,
    verifier: &Address,
) -> bool {
    let key = DataKey::PurchaseApproval(pool_id, round, version, verifier.clone());
    let approved = env.storage().persistent().get(&key).unwrap_or(false);
    if approved {
        extend_persistent_ttl(env, &key);
    }
    approved
}

pub(crate) fn write_purchase_approval(
    env: &Env,
    pool_id: u64,
    round: u32,
    version: u32,
    verifier: &Address,
) {
    let key = DataKey::PurchaseApproval(pool_id, round, version, verifier.clone());
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

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}
