use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, BytesN, Env, Vec,
};

use crate::{
    storage, ContractError, PoolStatus, RotatingPoolContract, RotatingPoolContractClient,
    RoundStatus, CONTRACT_VERSION,
};

const NOW: u64 = 1_700_000_000;
const ROUND_DURATION: u64 = 180;
const GRACE_DURATION: u64 = 120;

fn register_contract(env: &Env) -> Address {
    env.register(RotatingPoolContract, ())
}

fn register_token(env: &Env, sponsor: &Address, sponsor_balance: i128) -> Address {
    let issuer = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    token::StellarAssetClient::new(env, &token_address).mint(sponsor, &sponsor_balance);
    token_address
}

fn create_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    sponsor: &Address,
    token: &Address,
    contribution_amount: i128,
    member_limit: u32,
) -> u64 {
    RotatingPoolContractClient::new(env, contract_id).create_pool(
        creator,
        sponsor,
        token,
        &contribution_amount,
        &member_limit,
        &ROUND_DURATION,
        &GRACE_DURATION,
    )
}

fn join_members(env: &Env, contract_id: &Address, pool_id: u64, count: u32) -> Vec<Address> {
    let client = RotatingPoolContractClient::new(env, contract_id);
    let mut members = Vec::new(env);
    for _ in 0..count {
        let member = Address::generate(env);
        client.join_pool(&member, &pool_id);
        members.push_back(member);
    }
    members
}

fn configure_verifiers(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    pool_id: u64,
) -> Vec<Address> {
    let verifiers = Vec::from_array(env, [Address::generate(env), Address::generate(env)]);
    RotatingPoolContractClient::new(env, contract_id)
        .configure_verifiers(creator, &pool_id, &verifiers, &2);
    verifiers
}

fn prepare_active_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    sponsor: &Address,
    token_address: &Address,
) -> (u64, Vec<Address>, Vec<Address>) {
    let pool_id = create_pool(env, contract_id, creator, sponsor, token_address, 10, 2);
    let members = join_members(env, contract_id, pool_id, 2);
    let token_admin = token::StellarAssetClient::new(env, token_address);
    for member in members.iter() {
        token_admin.mint(&member, &10);
    }
    let client = RotatingPoolContractClient::new(env, contract_id);
    client.fund_guarantee(&pool_id, &10);
    let verifiers = configure_verifiers(env, contract_id, creator, pool_id);
    client.start_pool(creator, &pool_id, &members);
    (pool_id, members, verifiers)
}

fn prepare_ready_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    sponsor: &Address,
    token_address: &Address,
) -> (u64, Vec<Address>, Vec<Address>) {
    let (pool_id, members, verifiers) =
        prepare_active_pool(env, contract_id, creator, sponsor, token_address);
    let client = RotatingPoolContractClient::new(env, contract_id);
    for member in members.iter() {
        client.deposit(&member, &pool_id);
    }
    (pool_id, members, verifiers)
}

fn propose_and_approve_current_purchase(
    env: &Env,
    contract_id: &Address,
    pool_id: u64,
    recipient: &Address,
    verifiers: &Vec<Address>,
    seller: &Address,
    digest_marker: u8,
) {
    let client = RotatingPoolContractClient::new(env, contract_id);
    client.propose_purchase(
        recipient,
        &pool_id,
        seller,
        &BytesN::from_array(env, &[digest_marker; 32]),
    );
    for verifier in verifiers.iter() {
        client.approve_purchase(
            &verifier,
            &pool_id,
            &client.get_pool(&pool_id).current_round,
        );
    }
}

#[test]
fn exposes_version_and_initial_pool_id() {
    let env = Env::default();
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    assert_eq!(client.version(), CONTRACT_VERSION);
    assert_eq!(client.next_pool_id(), 1);
    assert!(!client.has_pool(&1));
}

#[test]
fn create_pool_uses_sponsor_guarantee_formula() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);

    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        4,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool = client.get_pool(&pool_id);

    assert_eq!(pool.id, 1);
    assert_eq!(pool.creator, creator);
    assert_eq!(pool.sponsor, sponsor);
    assert_eq!(pool.token, token_address);
    assert_eq!(pool.contribution_amount, 10);
    assert_eq!(pool.guarantee_required, 40);
    assert_eq!(pool.member_limit, 4);
    assert_eq!(pool.member_count, 0);
    assert_eq!(pool.round_duration_secs, ROUND_DURATION);
    assert_eq!(pool.grace_duration_secs, GRACE_DURATION);
    assert_eq!(pool.status, PoolStatus::Filling);
    assert_eq!(pool.created_at, NOW);
    assert_eq!(pool.started_at, None);
    assert_eq!(client.get_guarantee_balance(&pool_id), 0);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 0);
    assert_eq!(client.get_total_refund_liability(&pool_id), 0);
    assert_eq!(client.next_pool_id(), 2);
}

#[test]
fn create_pool_requires_creator_auth_and_valid_parameters() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = Address::generate(&env);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    assert!(client
        .try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &10,
            &4,
            &ROUND_DURATION,
            &GRACE_DURATION,
        )
        .is_err());

    env.mock_all_auths();
    assert_eq!(
        client.try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &0,
            &4,
            &ROUND_DURATION,
            &GRACE_DURATION,
        ),
        Err(Ok(ContractError::InvalidContributionAmount))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &10,
            &1,
            &ROUND_DURATION,
            &GRACE_DURATION,
        ),
        Err(Ok(ContractError::InvalidMemberLimit))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &i128::MAX,
            &20,
            &ROUND_DURATION,
            &GRACE_DURATION,
        ),
        Err(Ok(ContractError::ArithmeticOverflow))
    );
}

#[test]
fn members_join_and_leave_without_locking_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let member = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    token::StellarAssetClient::new(&env, &token_address).mint(&member, &25);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    client.join_pool(&member, &pool_id);

    assert_eq!(token_client.balance(&member), 25);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(client.get_pool(&pool_id).member_count, 1);
    let state = client.get_member(&pool_id, &member).unwrap();
    assert!(state.active);
    assert!(!state.allocation_received);
    assert_eq!(state.contributions_paid, 0);
    assert_eq!(
        client.try_join_pool(&member, &pool_id),
        Err(Ok(ContractError::AlreadyMember))
    );

    client.leave_pool(&member, &pool_id);
    assert_eq!(client.get_pool(&pool_id).member_count, 0);
    assert_eq!(client.get_member(&pool_id, &member), None);
    assert_eq!(token_client.balance(&member), 25);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn sponsor_funds_guarantee_with_separate_pool_accounting() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let first_pool = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        4,
    );
    let second_pool = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    assert_eq!(client.fund_guarantee(&first_pool, &25), 25);
    assert_eq!(client.fund_guarantee(&first_pool, &15), 40);
    assert_eq!(client.fund_guarantee(&second_pool, &10), 10);

    assert_eq!(client.get_guarantee_balance(&first_pool), 40);
    assert_eq!(client.get_guarantee_balance(&second_pool), 10);
    assert_eq!(client.get_pool_assigned_balance(&first_pool), 40);
    assert_eq!(client.get_pool_assigned_balance(&second_pool), 10);
    assert_eq!(token_client.balance(&sponsor), 50);
    assert_eq!(token_client.balance(&contract_id), 50);
    assert_eq!(
        client.try_fund_guarantee(&first_pool, &0),
        Err(Ok(ContractError::InvalidAmount))
    );
}

#[test]
fn full_funded_pool_starts_with_fixed_member_order() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        4,
    );
    let members = join_members(&env, &contract_id, pool_id, 4);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    client.fund_guarantee(&pool_id, &40);
    configure_verifiers(&env, &contract_id, &creator, pool_id);

    client.start_pool(&creator, &pool_id, &members);

    let pool = client.get_pool(&pool_id);
    let round = client.get_round(&pool_id, &0).unwrap();
    assert_eq!(pool.status, PoolStatus::Active);
    assert_eq!(pool.started_at, Some(NOW));
    assert_eq!(client.get_recipient_order(&pool_id), Some(members.clone()));
    assert_eq!(round.index, 0);
    assert_eq!(round.recipient, members.get(0).unwrap());
    assert_eq!(round.started_at, NOW);
    assert_eq!(round.deadline, NOW + ROUND_DURATION);
    assert_eq!(round.deposit_count, 0);
    assert_eq!(round.pot_amount, 0);
    assert_eq!(round.status, RoundStatus::Collecting);
    assert_eq!(
        client.try_leave_pool(&members.get(0).unwrap(), &pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn start_rejects_missing_guarantee_and_invalid_orders() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let outsider = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    configure_verifiers(&env, &contract_id, &creator, pool_id);

    assert_eq!(
        client.try_start_pool(&creator, &pool_id, &members),
        Err(Ok(ContractError::InsufficientGuarantee))
    );

    client.fund_guarantee(&pool_id, &10);
    let duplicate_order = Vec::from_array(&env, [members.get(0).unwrap(), members.get(0).unwrap()]);
    assert_eq!(
        client.try_start_pool(&creator, &pool_id, &duplicate_order),
        Err(Ok(ContractError::DuplicateRecipient))
    );

    let outsider_order = Vec::from_array(&env, [members.get(0).unwrap(), outsider]);
    assert_eq!(
        client.try_start_pool(&creator, &pool_id, &outsider_order),
        Err(Ok(ContractError::InvalidRecipientOrder))
    );
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Filling);
}

#[test]
fn membership_capacity_and_full_start_are_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let outsider = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = Address::generate(&env);
    client.join_pool(&first_member, &pool_id);
    client.fund_guarantee(&pool_id, &10);
    configure_verifiers(&env, &contract_id, &creator, pool_id);

    let incomplete_order = Vec::from_array(&env, [first_member.clone()]);
    assert_eq!(
        client.try_start_pool(&creator, &pool_id, &incomplete_order),
        Err(Ok(ContractError::PoolNotFull))
    );
    assert_eq!(
        client.try_leave_pool(&outsider, &pool_id),
        Err(Ok(ContractError::NotMember))
    );

    let second_member = Address::generate(&env);
    client.join_pool(&second_member, &pool_id);
    assert_eq!(
        client.try_join_pool(&outsider, &pool_id),
        Err(Ok(ContractError::PoolFull))
    );
}

#[test]
fn phase_two_mutations_require_the_stored_roles() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let member_one = Address::generate(&env);
    let member_two = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    token::StellarAssetClient::new(&env, &token_address)
        .mock_all_auths()
        .mint(&sponsor, &100);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool_id = client.mock_all_auths().create_pool(
        &creator,
        &sponsor,
        &token_address,
        &10,
        &2,
        &ROUND_DURATION,
        &GRACE_DURATION,
    );

    assert!(client.try_fund_guarantee(&pool_id, &10).is_err());
    client.mock_all_auths().fund_guarantee(&pool_id, &10);

    assert!(client.try_join_pool(&member_one, &pool_id).is_err());
    client.mock_all_auths().join_pool(&member_one, &pool_id);
    assert!(client.try_leave_pool(&member_one, &pool_id).is_err());

    client.mock_all_auths().join_pool(&member_two, &pool_id);
    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    assert!(client
        .try_configure_verifiers(&creator, &pool_id, &verifiers, &2)
        .is_err());
    client
        .mock_all_auths()
        .configure_verifiers(&creator, &pool_id, &verifiers, &2);
    let order = Vec::from_array(&env, [member_one, member_two]);
    assert!(client.try_start_pool(&creator, &pool_id, &order).is_err());
    client
        .mock_all_auths()
        .start_pool(&creator, &pool_id, &order);
}

#[test]
fn member_deposit_updates_round_and_liability_accounting_once() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_admin = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();
    token_admin.mint(&first_member, &20);
    token_admin.mint(&second_member, &20);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    client.fund_guarantee(&pool_id, &10);
    configure_verifiers(&env, &contract_id, &creator, pool_id);
    client.start_pool(&creator, &pool_id, &members);

    assert_eq!(client.deposit(&first_member, &pool_id), 10);
    assert_eq!(client.deposit(&second_member, &pool_id), 20);

    let round = client.get_round(&pool_id, &0).unwrap();
    assert_eq!(round.deposit_count, 2);
    assert_eq!(round.pot_amount, 20);
    assert_eq!(client.get_round_pot(&pool_id, &0), 20);
    assert!(client.has_deposit(&pool_id, &0, &first_member));
    assert!(client.has_deposit(&pool_id, &0, &second_member));
    assert_eq!(
        client
            .get_member(&pool_id, &first_member)
            .unwrap()
            .contributions_paid,
        1
    );
    assert_eq!(
        client.get_member_contribution_total(&pool_id, &first_member),
        10
    );
    assert_eq!(
        client.get_member_contribution_total(&pool_id, &second_member),
        10
    );
    assert_eq!(client.get_refund_liability(&pool_id, &first_member), 10);
    assert_eq!(client.get_refund_liability(&pool_id, &second_member), 10);
    assert_eq!(client.get_total_refund_liability(&pool_id), 20);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 30);
    assert_eq!(token_client.balance(&contract_id), 30);
    assert_eq!(token_client.balance(&first_member), 10);
    assert_eq!(
        client.try_deposit(&first_member, &pool_id),
        Err(Ok(ContractError::AlreadyDeposited))
    );
}

#[test]
fn deposit_requires_active_pool_membership_and_member_auth() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let member_one = Address::generate(&env);
    let member_two = Address::generate(&env);
    let outsider = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address).mock_all_auths();
    token_admin.mint(&sponsor, &100);
    token_admin.mint(&member_one, &20);
    token_admin.mint(&member_two, &20);
    token_admin.mint(&outsider, &20);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool_id = client.mock_all_auths().create_pool(
        &creator,
        &sponsor,
        &token_address,
        &10,
        &2,
        &ROUND_DURATION,
        &GRACE_DURATION,
    );
    client.mock_all_auths().join_pool(&member_one, &pool_id);

    assert_eq!(
        client.mock_all_auths().try_deposit(&member_one, &pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );

    client.mock_all_auths().join_pool(&member_two, &pool_id);
    client.mock_all_auths().fund_guarantee(&pool_id, &10);
    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    client
        .mock_all_auths()
        .configure_verifiers(&creator, &pool_id, &verifiers, &2);
    let order = Vec::from_array(&env, [member_one.clone(), member_two]);
    client
        .mock_all_auths()
        .start_pool(&creator, &pool_id, &order);

    assert_eq!(
        client.mock_all_auths().try_deposit(&outsider, &pool_id),
        Err(Ok(ContractError::NotMember))
    );
    assert!(client.try_deposit(&member_one, &pool_id).is_err());
    assert!(!client.has_deposit(&pool_id, &0, &member_one));
}

#[test]
fn failed_token_transfer_rolls_back_all_deposit_accounting() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let unfunded_member = members.get(0).unwrap();
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    client.fund_guarantee(&pool_id, &10);
    configure_verifiers(&env, &contract_id, &creator, pool_id);
    client.start_pool(&creator, &pool_id, &members);

    assert!(client.try_deposit(&unfunded_member, &pool_id).is_err());

    let round = client.get_round(&pool_id, &0).unwrap();
    assert_eq!(round.deposit_count, 0);
    assert_eq!(round.pot_amount, 0);
    assert_eq!(client.get_round_pot(&pool_id, &0), 0);
    assert!(!client.has_deposit(&pool_id, &0, &unfunded_member));
    assert_eq!(
        client.get_member_contribution_total(&pool_id, &unfunded_member),
        0
    );
    assert_eq!(client.get_refund_liability(&pool_id, &unfunded_member), 0);
    assert_eq!(client.get_total_refund_liability(&pool_id), 0);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 10);
}

#[test]
fn same_token_pools_keep_assigned_balances_and_liabilities_isolated() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_admin = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let first_pool = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let second_pool = create_pool(&env, &contract_id, &creator, &sponsor, &token_address, 7, 2);
    let first_members = join_members(&env, &contract_id, first_pool, 2);
    let second_members = join_members(&env, &contract_id, second_pool, 2);
    let first_depositor = first_members.get(0).unwrap();
    let second_depositor = second_members.get(0).unwrap();
    token_admin.mint(&first_depositor, &10);
    token_admin.mint(&second_depositor, &7);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    client.fund_guarantee(&first_pool, &10);
    client.fund_guarantee(&second_pool, &7);
    configure_verifiers(&env, &contract_id, &creator, first_pool);
    configure_verifiers(&env, &contract_id, &creator, second_pool);
    client.start_pool(&creator, &first_pool, &first_members);
    client.start_pool(&creator, &second_pool, &second_members);

    client.deposit(&first_depositor, &first_pool);
    client.deposit(&second_depositor, &second_pool);

    assert_eq!(client.get_pool_assigned_balance(&first_pool), 20);
    assert_eq!(client.get_pool_assigned_balance(&second_pool), 14);
    assert_eq!(client.get_total_refund_liability(&first_pool), 10);
    assert_eq!(client.get_total_refund_liability(&second_pool), 7);
    assert_eq!(client.get_round_pot(&first_pool, &0), 10);
    assert_eq!(client.get_round_pot(&second_pool, &0), 7);
    assert_eq!(token_client.balance(&contract_id), 34);
}

#[test]
fn verifier_policy_is_independent_required_and_immutable_after_start() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    let configured_pool = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let verifiers = configure_verifiers(&env, &contract_id, &creator, configured_pool);
    assert_eq!(
        client.try_join_pool(&verifiers.get(0).unwrap(), &configured_pool),
        Err(Ok(ContractError::VerifierCannotBeParticipant))
    );
    let members = join_members(&env, &contract_id, configured_pool, 2);
    client.fund_guarantee(&configured_pool, &10);
    client.start_pool(&creator, &configured_pool, &members);
    let policy = client.get_verifier_policy(&configured_pool).unwrap();
    assert_eq!(policy.verifiers, verifiers);
    assert_eq!(policy.approval_quorum, 2);
    assert_eq!(
        client.try_configure_verifiers(&creator, &configured_pool, &policy.verifiers, &2),
        Err(Ok(ContractError::InvalidPoolStatus))
    );

    let unconfigured_pool = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        10,
        2,
    );
    let unconfigured_members = join_members(&env, &contract_id, unconfigured_pool, 2);
    client.fund_guarantee(&unconfigured_pool, &10);
    assert_eq!(
        client.try_start_pool(&creator, &unconfigured_pool, &unconfigured_members),
        Err(Ok(ContractError::VerifierPolicyMissing))
    );

    let one_verifier = Vec::from_array(&env, [Address::generate(&env)]);
    assert_eq!(
        client.try_configure_verifiers(&creator, &unconfigured_pool, &one_verifier, &1),
        Err(Ok(ContractError::InvalidVerifierSet))
    );
    let verifier = Address::generate(&env);
    let duplicate = Vec::from_array(&env, [verifier.clone(), verifier]);
    assert_eq!(
        client.try_configure_verifiers(&creator, &unconfigured_pool, &duplicate, &2),
        Err(Ok(ContractError::DuplicateVerifier))
    );
    let independent = Address::generate(&env);
    let creator_policy = Vec::from_array(&env, [creator.clone(), independent.clone()]);
    assert_eq!(
        client.try_configure_verifiers(&creator, &unconfigured_pool, &creator_policy, &2),
        Err(Ok(ContractError::VerifierCannotBeParticipant))
    );
    let valid = Vec::from_array(&env, [independent, Address::generate(&env)]);
    assert_eq!(
        client.try_configure_verifiers(&creator, &unconfigured_pool, &valid, &1),
        Err(Ok(ContractError::InvalidApprovalQuorum))
    );
}

#[test]
fn only_current_recipient_can_propose_one_valid_ready_purchase() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_ready_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let recipient = members.get(0).unwrap();
    let other_member = members.get(1).unwrap();
    let seller = Address::generate(&env);
    let digest = BytesN::from_array(&env, &[7; 32]);

    assert_eq!(
        client.try_propose_purchase(&other_member, &pool_id, &seller, &digest,),
        Err(Ok(ContractError::CurrentRecipientOnly))
    );
    assert_eq!(
        client.try_propose_purchase(&recipient, &pool_id, &other_member, &digest,),
        Err(Ok(ContractError::InvalidSeller))
    );
    assert_eq!(
        client.try_propose_purchase(&recipient, &pool_id, &verifiers.get(0).unwrap(), &digest,),
        Err(Ok(ContractError::InvalidSeller))
    );
    assert_eq!(
        client.try_propose_purchase(
            &recipient,
            &pool_id,
            &seller,
            &BytesN::from_array(&env, &[0; 32]),
        ),
        Err(Ok(ContractError::InvalidDocumentDigest))
    );

    client.propose_purchase(&recipient, &pool_id, &seller, &digest);

    let purchase = client.get_purchase(&pool_id, &0).unwrap();
    assert_eq!(purchase.seller, seller);
    assert_eq!(purchase.document_digest, digest);
    assert_eq!(purchase.proposed_by, recipient);
    assert_eq!(purchase.proposed_at, NOW);
    assert_eq!(purchase.approval_count, 0);
    assert!(!purchase.approved);
    assert_eq!(
        client.try_propose_purchase(
            &members.get(0).unwrap(),
            &pool_id,
            &Address::generate(&env),
            &BytesN::from_array(&env, &[8; 32]),
        ),
        Err(Ok(ContractError::PurchaseAlreadyProposed))
    );
}

#[test]
fn allowlisted_verifiers_reach_quorum_once_each() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let outsider = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_ready_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_verifier = verifiers.get(0).unwrap();
    let second_verifier = verifiers.get(1).unwrap();

    assert_eq!(
        client.try_approve_purchase(&first_verifier, &pool_id, &0),
        Err(Ok(ContractError::PurchaseNotFound))
    );
    client.propose_purchase(
        &members.get(0).unwrap(),
        &pool_id,
        &Address::generate(&env),
        &BytesN::from_array(&env, &[9; 32]),
    );
    assert_eq!(
        client.try_approve_purchase(&outsider, &pool_id, &0),
        Err(Ok(ContractError::UnauthorizedVerifier))
    );
    assert_eq!(
        client.try_approve_purchase(&creator, &pool_id, &0),
        Err(Ok(ContractError::UnauthorizedVerifier))
    );
    assert_eq!(
        client.try_approve_purchase(&first_verifier, &pool_id, &1),
        Err(Ok(ContractError::WrongRound))
    );

    assert_eq!(client.approve_purchase(&first_verifier, &pool_id, &0), 1);
    assert!(!client.get_purchase(&pool_id, &0).unwrap().approved);
    assert!(client.has_verifier_approval(&pool_id, &0, &first_verifier));
    assert_eq!(
        client.try_approve_purchase(&first_verifier, &pool_id, &0),
        Err(Ok(ContractError::AlreadyApproved))
    );
    assert_eq!(client.approve_purchase(&second_verifier, &pool_id, &0), 2);
    let purchase = client.get_purchase(&pool_id, &0).unwrap();
    assert_eq!(purchase.approval_count, 2);
    assert!(purchase.approved);
}

#[test]
fn purchase_proposal_and_approval_require_stored_role_auth() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let member_one = Address::generate(&env);
    let member_two = Address::generate(&env);
    let verifier_one = Address::generate(&env);
    let verifier_two = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address).mock_all_auths();
    token_admin.mint(&sponsor, &100);
    token_admin.mint(&member_one, &10);
    token_admin.mint(&member_two, &10);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool_id = client.mock_all_auths().create_pool(
        &creator,
        &sponsor,
        &token_address,
        &10,
        &2,
        &ROUND_DURATION,
        &GRACE_DURATION,
    );
    client.mock_all_auths().join_pool(&member_one, &pool_id);
    client.mock_all_auths().join_pool(&member_two, &pool_id);
    client.mock_all_auths().fund_guarantee(&pool_id, &10);
    let verifiers = Vec::from_array(&env, [verifier_one.clone(), verifier_two]);
    client
        .mock_all_auths()
        .configure_verifiers(&creator, &pool_id, &verifiers, &2);
    let order = Vec::from_array(&env, [member_one.clone(), member_two.clone()]);
    client
        .mock_all_auths()
        .start_pool(&creator, &pool_id, &order);
    client.mock_all_auths().deposit(&member_one, &pool_id);
    client.mock_all_auths().deposit(&member_two, &pool_id);
    let seller = Address::generate(&env);
    let digest = BytesN::from_array(&env, &[11; 32]);

    assert!(client
        .try_propose_purchase(&member_one, &pool_id, &seller, &digest)
        .is_err());
    client
        .mock_all_auths()
        .propose_purchase(&member_one, &pool_id, &seller, &digest);
    assert!(client
        .try_approve_purchase(&verifier_one, &pool_id, &0)
        .is_err());
    assert_eq!(
        client
            .mock_all_auths()
            .approve_purchase(&verifier_one, &pool_id, &0),
        1
    );
}

#[test]
fn approved_rounds_pay_sellers_preserve_refunds_and_complete_in_order() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_admin = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_ready_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();
    let first_seller = Address::generate(&env);
    let second_seller = Address::generate(&env);

    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &first_member,
        &verifiers,
        &first_seller,
        12,
    );
    assert_eq!(client.execute_round(&pool_id), 20);

    assert_eq!(token_client.balance(&first_seller), 20);
    assert_eq!(token_client.balance(&contract_id), 10);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 10);
    assert_eq!(client.get_guarantee_balance(&pool_id), 10);
    assert_eq!(client.get_total_refund_liability(&pool_id), 10);
    assert_eq!(client.get_refund_liability(&pool_id, &first_member), 0);
    assert_eq!(client.get_refund_liability(&pool_id, &second_member), 10);
    assert!(
        client
            .get_member(&pool_id, &first_member)
            .unwrap()
            .allocation_received
    );
    assert!(
        !client
            .get_member(&pool_id, &second_member)
            .unwrap()
            .allocation_received
    );
    assert_eq!(
        client.get_round(&pool_id, &0).unwrap().status,
        RoundStatus::Settled
    );
    let second_round = client.get_round(&pool_id, &1).unwrap();
    assert_eq!(second_round.recipient, second_member);
    assert_eq!(second_round.started_at, NOW);
    assert_eq!(second_round.deadline, NOW + ROUND_DURATION);
    assert_eq!(client.get_pool(&pool_id).current_round, 1);

    token_admin.mint(&first_member, &10);
    token_admin.mint(&second_member, &10);
    client.deposit(&first_member, &pool_id);
    client.deposit(&second_member, &pool_id);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &second_member,
        &verifiers,
        &second_seller,
        13,
    );
    assert_eq!(client.execute_round(&pool_id), 20);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Completed);
    assert_eq!(
        client.get_round(&pool_id, &1).unwrap().status,
        RoundStatus::Settled
    );
    assert_eq!(token_client.balance(&second_seller), 20);
    assert_eq!(token_client.balance(&contract_id), 10);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 10);
    assert_eq!(client.get_total_refund_liability(&pool_id), 0);
    assert_eq!(client.get_refund_liability(&pool_id, &first_member), 0);
    assert_eq!(client.get_refund_liability(&pool_id, &second_member), 0);
    assert!(
        client
            .get_member(&pool_id, &second_member)
            .unwrap()
            .allocation_received
    );
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn execute_round_requires_purchase_quorum_and_pool_scoped_solvency() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_ready_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let seller = Address::generate(&env);

    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::PurchaseNotFound))
    );
    client.propose_purchase(
        &members.get(0).unwrap(),
        &pool_id,
        &seller,
        &BytesN::from_array(&env, &[14; 32]),
    );
    client.approve_purchase(&verifiers.get(0).unwrap(), &pool_id, &0);
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::PurchaseNotApproved))
    );
    client.approve_purchase(&verifiers.get(1).unwrap(), &pool_id, &0);

    env.as_contract(&contract_id, || {
        storage::write_pool_assigned_balance(&env, pool_id, 20);
    });
    assert_eq!(token_client.balance(&contract_id), 30);
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::RefundSolvencyViolation))
    );
    assert_eq!(token_client.balance(&seller), 0);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Active);
    assert_eq!(client.get_pool(&pool_id).current_round, 0);
    assert_eq!(
        client.get_round(&pool_id, &0).unwrap().status,
        RoundStatus::Collecting
    );
    assert_eq!(client.get_total_refund_liability(&pool_id), 20);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 20);
}

#[test]
fn failed_seller_transfer_rolls_back_round_and_liability_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_ready_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let seller = Address::generate(&env);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &members.get(0).unwrap(),
        &verifiers,
        &seller,
        15,
    );
    token_client.burn(&contract_id, &20);
    assert_eq!(token_client.balance(&contract_id), 10);

    assert!(client.try_execute_round(&pool_id).is_err());

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Active);
    assert_eq!(client.get_pool(&pool_id).current_round, 0);
    assert_eq!(
        client.get_round(&pool_id, &0).unwrap().status,
        RoundStatus::Collecting
    );
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 30);
    assert_eq!(client.get_total_refund_liability(&pool_id), 20);
    assert_eq!(
        client.get_refund_liability(&pool_id, &members.get(0).unwrap()),
        10
    );
    assert!(
        !client
            .get_member(&pool_id, &members.get(0).unwrap())
            .unwrap()
            .allocation_received
    );
}

#[test]
fn overdue_round_enters_grace_then_pauses_at_the_fixed_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) =
        prepare_active_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let deadline = NOW + ROUND_DURATION;
    let grace_deadline = deadline + GRACE_DURATION;

    env.ledger().set_timestamp(deadline - 1);
    assert_eq!(
        client.try_mark_overdue(&pool_id),
        Err(Ok(ContractError::DeadlineNotReached))
    );
    env.ledger().set_timestamp(deadline);
    assert_eq!(
        client.try_deposit(&members.get(0).unwrap(), &pool_id),
        Err(Ok(ContractError::DeadlineReached))
    );
    assert_eq!(client.mark_overdue(&pool_id), grace_deadline);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Grace);
    assert_eq!(
        client.get_round(&pool_id, &0).unwrap().grace_deadline,
        Some(grace_deadline)
    );
    assert_eq!(
        client.try_mark_overdue(&pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
    assert_eq!(
        client.try_pause_pool(&pool_id),
        Err(Ok(ContractError::DeadlineNotReached))
    );

    env.ledger().set_timestamp(grace_deadline);
    assert_eq!(
        client.try_cure_payment(&members.get(0).unwrap(), &pool_id),
        Err(Ok(ContractError::DeadlineReached))
    );
    client.pause_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Paused);
    assert_eq!(
        client.try_pause_pool(&pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn missing_member_can_cure_during_grace_and_restore_active_execution() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_active_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let paid_member = members.get(0).unwrap();
    let missing_member = members.get(1).unwrap();
    client.deposit(&paid_member, &pool_id);

    env.ledger().set_timestamp(NOW + ROUND_DURATION);
    client.mark_overdue(&pool_id);
    assert_eq!(
        client.try_cure_payment(&paid_member, &pool_id),
        Err(Ok(ContractError::AlreadyDeposited))
    );
    env.ledger().set_timestamp(NOW + ROUND_DURATION + 1);
    assert_eq!(client.cure_payment(&missing_member, &pool_id), 20);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Active);
    assert!(client.has_deposit(&pool_id, &0, &missing_member));
    assert_eq!(client.get_round_pot(&pool_id, &0), 20);
    assert_eq!(client.get_round_top_up(&pool_id, &0), 0);
    assert_eq!(client.get_total_refund_liability(&pool_id), 20);
    assert_eq!(
        client.try_mark_overdue(&pool_id),
        Err(Ok(ContractError::RoundAlreadyReady))
    );

    let seller = Address::generate(&env);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &paid_member,
        &verifiers,
        &seller,
        16,
    );
    assert_eq!(client.execute_round(&pool_id), 20);
}

#[test]
fn sponsor_top_up_covers_exact_shortfall_without_clearing_member_debt() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_active_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let paid_member = members.get(0).unwrap();
    let missing_member = members.get(1).unwrap();
    client.deposit(&paid_member, &pool_id);
    env.ledger().set_timestamp(NOW + ROUND_DURATION);
    client.mark_overdue(&pool_id);

    assert_eq!(
        client.try_top_up(&pool_id, &5),
        Err(Ok(ContractError::InvalidTopUpAmount))
    );
    assert_eq!(client.top_up(&pool_id, &10), 10);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Active);
    assert_eq!(client.get_round_pot(&pool_id, &0), 10);
    assert_eq!(client.get_round_top_up(&pool_id, &0), 10);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 30);
    assert_eq!(client.get_guarantee_balance(&pool_id), 10);
    assert_eq!(client.get_total_refund_liability(&pool_id), 10);
    assert!(!client.has_deposit(&pool_id, &0, &missing_member));
    assert_eq!(
        client.get_member_contribution_total(&pool_id, &missing_member),
        0
    );
    assert_eq!(token_client.balance(&sponsor), 80);

    let seller = Address::generate(&env);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &paid_member,
        &verifiers,
        &seller,
        17,
    );
    assert_eq!(client.execute_round(&pool_id), 20);
    assert_eq!(token_client.balance(&seller), 20);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 10);
    assert_eq!(client.get_total_refund_liability(&pool_id), 0);
}

#[test]
fn sponsor_top_up_requires_auth_and_rolls_back_on_failed_transfer() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let member_one = Address::generate(&env);
    let member_two = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address).mock_all_auths();
    token_admin.mint(&sponsor, &10);
    token_admin.mint(&member_one, &10);
    token_admin.mint(&member_two, &10);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool_id = client.mock_all_auths().create_pool(
        &creator,
        &sponsor,
        &token_address,
        &10,
        &2,
        &ROUND_DURATION,
        &GRACE_DURATION,
    );
    client.mock_all_auths().join_pool(&member_one, &pool_id);
    client.mock_all_auths().join_pool(&member_two, &pool_id);
    client.mock_all_auths().fund_guarantee(&pool_id, &10);
    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    client
        .mock_all_auths()
        .configure_verifiers(&creator, &pool_id, &verifiers, &2);
    let order = Vec::from_array(&env, [member_one.clone(), member_two]);
    client
        .mock_all_auths()
        .start_pool(&creator, &pool_id, &order);
    client.mock_all_auths().deposit(&member_one, &pool_id);
    env.ledger().set_timestamp(NOW + ROUND_DURATION);
    client.mock_all_auths().mark_overdue(&pool_id);

    assert!(client.try_top_up(&pool_id, &10).is_err());
    assert!(client.mock_all_auths().try_top_up(&pool_id, &10).is_err());

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Grace);
    assert_eq!(client.get_round_top_up(&pool_id, &0), 0);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 20);
    assert_eq!(client.get_total_refund_liability(&pool_id), 10);
}

fn pause_pool_after_unfunded_round(
    env: &Env,
    contract_id: &Address,
    pool_id: u64,
    deadline: u64,
    grace_deadline: u64,
) {
    let client = RotatingPoolContractClient::new(env, contract_id);
    env.ledger().set_timestamp(deadline);
    client.mark_overdue(&pool_id);
    env.ledger().set_timestamp(grace_deadline);
    client.pause_pool(&pool_id);
}

#[test]
fn abort_requires_paused_pool_and_freezes_further_mutations() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) =
        prepare_active_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let deadline = NOW + ROUND_DURATION;
    let grace_deadline = deadline + GRACE_DURATION;

    assert_eq!(
        client.try_abort_pool(&pool_id),
        Err(Ok(ContractError::AbortNotAllowed))
    );

    pause_pool_after_unfunded_round(&env, &contract_id, pool_id, deadline, grace_deadline);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Paused);

    client.abort_pool(&pool_id);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);
    assert_eq!(
        client.try_abort_pool(&pool_id),
        Err(Ok(ContractError::AbortNotAllowed))
    );
    assert_eq!(
        client.try_deposit(&members.get(0).unwrap(), &pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
    assert_eq!(
        client.try_top_up(&pool_id, &10),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn undelivered_member_reclaims_contribution_and_sponsor_claims_freed_remainder() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) =
        prepare_active_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let paid_member = members.get(0).unwrap();
    let missing_member = members.get(1).unwrap();
    let deadline = NOW + ROUND_DURATION;
    let grace_deadline = deadline + GRACE_DURATION;

    client.deposit(&paid_member, &pool_id);
    pause_pool_after_unfunded_round(&env, &contract_id, pool_id, deadline, grace_deadline);
    client.abort_pool(&pool_id);

    assert_eq!(client.get_refund_liability(&pool_id, &paid_member), 10);
    assert_eq!(client.get_refund_liability(&pool_id, &missing_member), 0);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 20);
    assert_eq!(token_client.balance(&contract_id), 20);

    assert_eq!(
        client.try_claim_refund(&missing_member, &pool_id),
        Err(Ok(ContractError::RefundNotClaimable))
    );
    assert_eq!(client.claim_sponsor_remainder(&pool_id), 10);
    assert_eq!(token_client.balance(&sponsor), 100);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 10);
    assert_eq!(
        client.try_claim_sponsor_remainder(&pool_id),
        Err(Ok(ContractError::SponsorRemainderNotClaimable))
    );

    assert_eq!(client.claim_refund(&paid_member, &pool_id), 10);
    assert_eq!(token_client.balance(&paid_member), 10);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 0);
    assert_eq!(client.get_total_refund_liability(&pool_id), 0);
    assert!(client.get_refund_claim(&pool_id, &paid_member));
    assert_eq!(
        client.try_claim_refund(&paid_member, &pool_id),
        Err(Ok(ContractError::RefundAlreadyClaimed))
    );
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn delivered_member_reclaims_only_unfinished_round_contribution() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_admin = token::StellarAssetClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) =
        prepare_ready_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();
    let seller = Address::generate(&env);

    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &first_member,
        &verifiers,
        &seller,
        21,
    );
    client.execute_round(&pool_id);
    assert!(
        client
            .get_member(&pool_id, &first_member)
            .unwrap()
            .allocation_received
    );

    token_admin.mint(&first_member, &10);
    client.deposit(&first_member, &pool_id);
    let deadline = client.get_round(&pool_id, &1).unwrap().deadline;
    let grace_deadline = deadline + GRACE_DURATION;
    pause_pool_after_unfunded_round(&env, &contract_id, pool_id, deadline, grace_deadline);
    client.abort_pool(&pool_id);

    assert_eq!(client.get_refund_liability(&pool_id, &first_member), 10);
    assert_eq!(client.get_refund_liability(&pool_id, &second_member), 10);
    assert_eq!(client.get_total_refund_liability(&pool_id), 20);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 20);

    assert_eq!(client.claim_refund(&first_member, &pool_id), 10);
    assert_eq!(client.claim_refund(&second_member, &pool_id), 10);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 0);
    assert_eq!(
        client.try_claim_sponsor_remainder(&pool_id),
        Err(Ok(ContractError::SponsorRemainderNotClaimable))
    );
}

#[test]
fn claim_functions_require_stored_role_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let member_one = Address::generate(&env);
    let member_two = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address).mock_all_auths();
    token_admin.mint(&sponsor, &100);
    token_admin.mint(&member_one, &10);
    token_admin.mint(&member_two, &10);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool_id = client.mock_all_auths().create_pool(
        &creator,
        &sponsor,
        &token_address,
        &10,
        &2,
        &ROUND_DURATION,
        &GRACE_DURATION,
    );
    client.mock_all_auths().join_pool(&member_one, &pool_id);
    client.mock_all_auths().join_pool(&member_two, &pool_id);
    client.mock_all_auths().fund_guarantee(&pool_id, &10);
    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    client
        .mock_all_auths()
        .configure_verifiers(&creator, &pool_id, &verifiers, &2);
    let order = Vec::from_array(&env, [member_one.clone(), member_two.clone()]);
    client
        .mock_all_auths()
        .start_pool(&creator, &pool_id, &order);
    client.mock_all_auths().deposit(&member_one, &pool_id);
    let deadline = NOW + ROUND_DURATION;
    let grace_deadline = deadline + GRACE_DURATION;
    env.ledger().set_timestamp(deadline);
    client.mock_all_auths().mark_overdue(&pool_id);
    env.ledger().set_timestamp(grace_deadline);
    client.mock_all_auths().pause_pool(&pool_id);
    client.mock_all_auths().abort_pool(&pool_id);

    assert!(client.try_claim_refund(&member_one, &pool_id).is_err());
    assert_eq!(
        client.mock_all_auths().claim_refund(&member_one, &pool_id),
        10
    );
    assert!(client.try_claim_sponsor_remainder(&pool_id).is_err());
    assert_eq!(
        client.mock_all_auths().claim_sponsor_remainder(&pool_id),
        10
    );
}

#[test]
fn failed_refund_transfer_rolls_back_claim_state() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) =
        prepare_active_pool(&env, &contract_id, &creator, &sponsor, &token_address);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let paid_member = members.get(0).unwrap();
    let deadline = NOW + ROUND_DURATION;
    let grace_deadline = deadline + GRACE_DURATION;

    client.deposit(&paid_member, &pool_id);
    pause_pool_after_unfunded_round(&env, &contract_id, pool_id, deadline, grace_deadline);
    client.abort_pool(&pool_id);
    token_client.burn(&contract_id, &15);
    assert_eq!(token_client.balance(&contract_id), 5);

    assert!(client.try_claim_refund(&paid_member, &pool_id).is_err());

    assert!(!client.get_refund_claim(&pool_id, &paid_member));
    assert_eq!(client.get_refund_liability(&pool_id, &paid_member), 10);
    assert_eq!(client.get_pool_assigned_balance(&pool_id), 20);
    assert_eq!(client.get_total_refund_liability(&pool_id), 10);
}
