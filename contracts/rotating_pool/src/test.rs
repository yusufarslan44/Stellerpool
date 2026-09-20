extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, BytesN, Env, Vec,
};

use crate::{
    storage, ContractError, OrderMode, PoolStatus, RotatingPoolContract,
    RotatingPoolContractClient, RoundPhase, CONTRACT_VERSION, MAX_MEMBERS,
};

const NOW: u64 = 1_700_000_000;
const ROUND_DURATION: u64 = 180;
const GRACE_DURATION: u64 = 120;
const PURCHASE_DURATION: u64 = 150;
const SETUP_WINDOW: u64 = 3_600;

fn register_contract(env: &Env) -> Address {
    env.register(RotatingPoolContract, ())
}

fn register_token(env: &Env, holder: &Address, balance: i128) -> Address {
    let issuer = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    token::StellarAssetClient::new(env, &token_address).mint(holder, &balance);
    token_address
}

fn create_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    token: &Address,
    demo_seller: &Address,
    contribution_amount: i128,
    member_limit: u32,
) -> u64 {
    create_pool_with_mode(
        env,
        contract_id,
        creator,
        token,
        demo_seller,
        contribution_amount,
        member_limit,
        OrderMode::Fixed,
    )
}

fn create_pool_with_mode(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    token: &Address,
    demo_seller: &Address,
    contribution_amount: i128,
    member_limit: u32,
    order_mode: OrderMode,
) -> u64 {
    let setup_deadline = env.ledger().timestamp() + SETUP_WINDOW;
    RotatingPoolContractClient::new(env, contract_id).create_pool(
        creator,
        token,
        &contribution_amount,
        &member_limit,
        &order_mode,
        &0,
        &ROUND_DURATION,
        &GRACE_DURATION,
        &PURCHASE_DURATION,
        &setup_deadline,
        demo_seller,
    )
}

/// Joins and funds members, has each approve the automatically recorded terms,
/// then starts the pool.
fn prepare_active_pool_n(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    token_address: &Address,
    demo_seller: &Address,
    member_count: u32,
    order_mode: OrderMode,
) -> (u64, Vec<Address>) {
    let pool_id = create_pool_with_mode(
        env,
        contract_id,
        creator,
        token_address,
        demo_seller,
        10,
        member_count,
        order_mode,
    );
    let members = join_members(env, contract_id, pool_id, member_count);
    let token_admin = token::StellarAssetClient::new(env, token_address);
    for member in members.iter() {
        token_admin.mint(&member, &(10 * (member_count as i128) * 4));
    }
    let client = RotatingPoolContractClient::new(env, contract_id);
    let version = client.get_pool(&pool_id).terms_version;
    for member in members.iter() {
        client.approve_terms(&member, &pool_id, &version);
    }
    client.start_pool(&pool_id);
    (pool_id, members)
}

/// Every member in `members` (in order) deposits into the pool's current round.
fn deposit_all(env: &Env, contract_id: &Address, pool_id: u64, members: &Vec<Address>) {
    let client = RotatingPoolContractClient::new(env, contract_id);
    for member in members.iter() {
        client.deposit(&member, &pool_id);
    }
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

fn approve_current_terms(
    env: &Env,
    contract_id: &Address,
    pool_id: u64,
    members: &Vec<Address>,
) -> u32 {
    let client = RotatingPoolContractClient::new(env, contract_id);
    let version = client.get_pool(&pool_id).terms_version;
    for member in members.iter() {
        client.approve_terms(&member, &pool_id, &version);
    }
    version
}

fn prepare_active_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    token_address: &Address,
    demo_seller: &Address,
) -> (u64, Vec<Address>) {
    let pool_id = create_pool(env, contract_id, creator, token_address, demo_seller, 10, 2);
    let members = join_members(env, contract_id, pool_id, 2);
    let token_admin = token::StellarAssetClient::new(env, token_address);
    for member in members.iter() {
        token_admin.mint(&member, &50);
    }
    let client = RotatingPoolContractClient::new(env, contract_id);
    approve_current_terms(env, contract_id, pool_id, &members);
    client.start_pool(&pool_id);
    (pool_id, members)
}

fn prepare_awaiting_purchase_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    token_address: &Address,
    demo_seller: &Address,
) -> (u64, Vec<Address>) {
    let (pool_id, members) =
        prepare_active_pool(env, contract_id, creator, token_address, demo_seller);
    let client = RotatingPoolContractClient::new(env, contract_id);
    for member in members.iter() {
        client.deposit(&member, &pool_id);
    }
    (pool_id, members)
}

fn propose_current_purchase(
    env: &Env,
    contract_id: &Address,
    pool_id: u64,
    recipient: &Address,
    demo_seller: &Address,
    token_address: &Address,
    digest_marker: u8,
) {
    let client = RotatingPoolContractClient::new(env, contract_id);
    let pool = client.get_pool(&pool_id);
    let amount = pool.contribution_amount * (pool.member_limit as i128) + pool.down_payment;
    client.propose_purchase(
        recipient,
        &pool_id,
        demo_seller,
        token_address,
        &amount,
        &BytesN::from_array(env, &[digest_marker; 32]),
    );
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
fn create_pool_stores_fields_and_validates_parameters() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = Address::generate(&env);
    let contract_id = register_contract(&env);

    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        10,
        4,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool = client.get_pool(&pool_id);

    assert_eq!(pool.id, 1);
    assert_eq!(pool.creator, creator);
    assert_eq!(pool.token, token_address);
    assert_eq!(pool.contribution_amount, 10);
    assert_eq!(pool.member_limit, 4);
    assert_eq!(pool.down_payment, 0);
    assert_eq!(pool.members.len(), 0);
    assert_eq!(pool.round_duration, ROUND_DURATION);
    assert_eq!(pool.grace_duration, GRACE_DURATION);
    assert_eq!(pool.purchase_duration, PURCHASE_DURATION);
    assert_eq!(pool.setup_deadline, NOW + SETUP_WINDOW);
    assert_eq!(pool.demo_seller, demo_seller);
    assert_eq!(pool.status, PoolStatus::Filling);
    assert_eq!(pool.created_at, NOW);
    assert_eq!(pool.started_at, None);
    assert_eq!(client.next_pool_id(), 2);

    let setup_deadline = NOW + SETUP_WINDOW;
    assert_eq!(
        client.try_create_pool(
            &creator,
            &token_address,
            &0,
            &4,
            &OrderMode::Fixed,
            &0,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &setup_deadline,
            &demo_seller,
        ),
        Err(Ok(ContractError::InvalidContributionAmount))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &token_address,
            &10,
            &(MAX_MEMBERS + 1),
            &OrderMode::Fixed,
            &0,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &setup_deadline,
            &demo_seller,
        ),
        Err(Ok(ContractError::InvalidMemberLimit))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &token_address,
            &10,
            &4,
            &OrderMode::Fixed,
            &0,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &NOW,
            &demo_seller,
        ),
        Err(Ok(ContractError::InvalidSetupDeadline))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &token_address,
            &10,
            &4,
            &OrderMode::Fixed,
            &0,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &setup_deadline,
            &creator,
        ),
        Err(Ok(ContractError::SellerCannotBeParticipant))
    );
}

#[test]
fn members_join_without_locking_funds_and_seller_is_excluded() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let member = Address::generate(&env);
    let token_address = register_token(&env, &member, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    client.join_pool(&member, &pool_id);

    assert_eq!(token_client.balance(&member), 100);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(client.get_pool(&pool_id).members.len(), 1);
    let status = client.get_member_status(&pool_id, &member);
    assert!(!status.received);
    assert_eq!(status.refundable, 0);
    assert_eq!(
        client.try_join_pool(&member, &pool_id),
        Err(Ok(ContractError::AlreadyMember))
    );
    assert_eq!(
        client.try_join_pool(&demo_seller, &pool_id),
        Err(Ok(ContractError::SellerCannotBeParticipant))
    );
}

#[test]
fn full_pool_sets_fixed_join_order_and_requires_member_approval() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_address = Address::generate(&env);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(&env, &contract_id, &creator, &token_address, &seller, 10, 2);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    client.join_pool(&first, &pool_id);
    assert_eq!(client.get_pool(&pool_id).terms_version, 0);
    client.join_pool(&second, &pool_id);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.terms_version, 1);
    assert_eq!(
        pool.recipient_order,
        Vec::from_array(&env, [first.clone(), second.clone()])
    );
    assert_eq!(pool.terms_approvals.len(), 0);
    let outsider = Address::generate(&env);
    assert_eq!(
        client.try_approve_terms(&outsider, &pool_id, &1),
        Err(Ok(ContractError::NotApprover))
    );
    assert_eq!(
        client.try_approve_terms(&first, &pool_id, &2),
        Err(Ok(ContractError::TermsVersionMismatch))
    );
    client.approve_terms(&first, &pool_id, &1);
    assert_eq!(
        client.try_approve_terms(&first, &pool_id, &1),
        Err(Ok(ContractError::AlreadyApprovedTerms))
    );
}

#[test]
fn start_pool_requires_full_membership_and_all_member_approvals() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = Address::generate(&env);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    assert_eq!(
        client.try_start_pool(&pool_id),
        Err(Ok(ContractError::PoolNotFull))
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let version = client.get_pool(&pool_id).terms_version;
    client.approve_terms(&members.get(0).unwrap(), &pool_id, &version);
    assert_eq!(
        client.try_start_pool(&pool_id),
        Err(Ok(ContractError::TermsNotFullyApproved))
    );
    client.approve_terms(&members.get(1).unwrap(), &pool_id, &version);

    // Anyone (not just the creator) may call start_pool.
    let outsider = Address::generate(&env);
    let _ = outsider;
    client.start_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.status, PoolStatus::Active);
    assert_eq!(pool.started_at, Some(NOW));
    assert_eq!(pool.current_round, 1);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.recipient, Some(members.get(0).unwrap()));
    assert_eq!(round.phase, RoundPhase::Collecting);
    assert_eq!(round.collect_deadline, NOW + ROUND_DURATION);
    assert_eq!(round.pot, 0);
}

#[test]
fn cancel_unstarted_pool_marks_aborted_without_any_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    join_members(&env, &contract_id, pool_id, 1);

    assert_eq!(
        client.try_cancel_unstarted_pool(&pool_id),
        Err(Ok(ContractError::SetupDeadlineNotReached))
    );

    env.ledger().set_timestamp(NOW + SETUP_WINDOW);
    client.cancel_unstarted_pool(&pool_id);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);
    assert_eq!(token_client.balance(&creator), 100);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(
        client.try_cancel_unstarted_pool(&pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn deposits_fund_the_round_and_transition_to_awaiting_purchase() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_active_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();

    assert_eq!(client.deposit(&first_member, &pool_id), 10);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.phase, RoundPhase::Collecting);
    assert_eq!(round.pot, 10);
    assert_eq!(
        client.get_member_status(&pool_id, &first_member).refundable,
        10
    );

    assert_eq!(client.deposit(&second_member, &pool_id), 20);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.phase, RoundPhase::AwaitingPurchase);
    assert_eq!(round.pot, 20);
    assert_eq!(round.purchase_deadline, Some(NOW + PURCHASE_DURATION));

    assert_eq!(token_client.balance(&contract_id), 20);
    // The round already moved past Collecting, so a repeated deposit is rejected by the
    // phase gate before it would even reach the already-deposited check.
    assert_eq!(
        client.try_deposit(&first_member, &pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn only_current_recipient_can_propose_a_valid_ready_purchase() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let outsider_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_awaiting_purchase_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let recipient = members.get(0).unwrap();
    let other_member = members.get(1).unwrap();
    let digest = BytesN::from_array(&env, &[7; 32]);

    assert_eq!(
        client.try_propose_purchase(
            &other_member,
            &pool_id,
            &demo_seller,
            &token_address,
            &20,
            &digest
        ),
        Err(Ok(ContractError::CurrentRecipientOnly))
    );
    assert_eq!(
        client.try_propose_purchase(
            &recipient,
            &pool_id,
            &outsider_seller,
            &token_address,
            &20,
            &digest
        ),
        Err(Ok(ContractError::InvalidSeller))
    );
    assert_eq!(
        client.try_propose_purchase(
            &recipient,
            &pool_id,
            &demo_seller,
            &token_address,
            &15,
            &digest
        ),
        Err(Ok(ContractError::InvalidPurchaseAmount))
    );
    assert_eq!(
        client.try_propose_purchase(
            &recipient,
            &pool_id,
            &demo_seller,
            &token_address,
            &20,
            &BytesN::from_array(&env, &[0; 32]),
        ),
        Err(Ok(ContractError::InvalidDocumentDigest))
    );

    client.propose_purchase(
        &recipient,
        &pool_id,
        &demo_seller,
        &token_address,
        &20,
        &digest,
    );
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.seller, Some(demo_seller.clone()));
    assert_eq!(round.doc_hash, Some(digest));

    let digest2 = BytesN::from_array(&env, &[8; 32]);
    client.propose_purchase(
        &recipient,
        &pool_id,
        &demo_seller,
        &token_address,
        &20,
        &digest2,
    );
    assert_eq!(client.get_round(&pool_id, &1).doc_hash, Some(digest2));
}

#[test]
fn purchase_executes_after_recipient_proposal_without_extra_approval() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_awaiting_purchase_pool(&env, &contract_id, &creator, &token_address, &seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    client.propose_purchase(
        &members.get(0).unwrap(),
        &pool_id,
        &seller,
        &token_address,
        &20,
        &BytesN::from_array(&env, &[9; 32]),
    );
    assert_eq!(client.execute_round(&pool_id), 20);
    assert_eq!(token_client.balance(&seller), 20);
}

#[test]
fn approved_rounds_pay_the_demo_seller_and_complete_the_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_awaiting_purchase_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();

    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &first_member,
        &demo_seller,
        &token_address,
        12,
    );
    assert_eq!(client.execute_round(&pool_id), 20);
    assert_eq!(token_client.balance(&demo_seller), 20);
    assert_eq!(client.get_round(&pool_id, &1).phase, RoundPhase::Settled);
    assert!(client.get_member_status(&pool_id, &first_member).received);
    assert_eq!(
        client.get_member_status(&pool_id, &first_member).refundable,
        0
    );
    assert_eq!(
        client
            .get_member_status(&pool_id, &second_member)
            .refundable,
        0
    );
    assert_eq!(client.get_pool(&pool_id).current_round, 2);
    assert_eq!(
        client.get_round(&pool_id, &2).recipient,
        Some(second_member.clone())
    );

    client.deposit(&first_member, &pool_id);
    client.deposit(&second_member, &pool_id);
    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &second_member,
        &demo_seller,
        &token_address,
        13,
    );
    assert_eq!(client.execute_round(&pool_id), 20);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Completed);
    assert_eq!(token_client.balance(&demo_seller), 40);
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn execute_round_requires_purchase_and_pool_scoped_solvency() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_awaiting_purchase_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::PurchaseNotFound))
    );
    client.propose_purchase(
        &members.get(0).unwrap(),
        &pool_id,
        &demo_seller,
        &token_address,
        &20,
        &BytesN::from_array(&env, &[14; 32]),
    );
    env.as_contract(&contract_id, || {
        storage::write_pool_assigned_balance(&env, pool_id, 15);
    });
    assert_eq!(token_client.balance(&contract_id), 20);
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::InsufficientPoolBalance))
    );
    assert_eq!(token_client.balance(&demo_seller), 0);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Active);
}

#[test]
fn failed_seller_transfer_rolls_back_round_and_liability_updates() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_awaiting_purchase_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &members.get(0).unwrap(),
        &demo_seller,
        &token_address,
        15,
    );
    token_client.burn(&contract_id, &20);
    assert_eq!(token_client.balance(&contract_id), 0);

    assert!(client.try_execute_round(&pool_id).is_err());

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Active);
    assert_eq!(
        client.get_round(&pool_id, &1).phase,
        RoundPhase::AwaitingPurchase
    );
    assert_eq!(client.get_pool(&pool_id).current_round, 1);
    assert_eq!(
        client
            .get_member_status(&pool_id, &members.get(0).unwrap())
            .refundable,
        10
    );
    assert!(
        !client
            .get_member_status(&pool_id, &members.get(0).unwrap())
            .received
    );
}

#[test]
fn overdue_round_enters_grace_then_aborts_for_safety_recovery() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_active_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let deadline = NOW + ROUND_DURATION;
    let grace_deadline = deadline + GRACE_DURATION;

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
    assert_eq!(client.get_round(&pool_id, &1).phase, RoundPhase::Grace);
    assert_eq!(
        client.try_abort_pool(&pool_id),
        Err(Ok(ContractError::DeadlineNotReached))
    );

    env.ledger().set_timestamp(grace_deadline);
    assert_eq!(
        client.try_cure_payment(&members.get(0).unwrap(), &pool_id),
        Err(Ok(ContractError::DeadlineReached))
    );
    client.abort_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);
    assert_eq!(
        client.try_abort_pool(&pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn missing_member_can_cure_during_grace_and_restore_progress() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_active_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let paid_member = members.get(0).unwrap();
    let missing_member = members.get(1).unwrap();
    client.deposit(&paid_member, &pool_id);

    env.ledger().set_timestamp(NOW + ROUND_DURATION);
    client.mark_overdue(&pool_id);
    env.ledger().set_timestamp(NOW + ROUND_DURATION + 1);
    assert_eq!(client.cure_payment(&missing_member, &pool_id), 20);

    assert_eq!(
        client.get_round(&pool_id, &1).phase,
        RoundPhase::AwaitingPurchase
    );
    assert_eq!(
        client.try_mark_overdue(&pool_id),
        Err(Ok(ContractError::RoundAlreadyFinalized))
    );

    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &paid_member,
        &demo_seller,
        &token_address,
        16,
    );
    assert_eq!(client.execute_round(&pool_id), 20);
}

#[test]
fn purchase_deadline_expiry_aborts_with_blocked_settlement() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, _members) =
        prepare_awaiting_purchase_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    let purchase_deadline = NOW + PURCHASE_DURATION;
    assert_eq!(
        client.try_abort_pool(&pool_id),
        Err(Ok(ContractError::DeadlineNotReached))
    );
    env.ledger().set_timestamp(purchase_deadline);
    client.abort_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);
}

/// The plan's canonical demo scenario (docs/plan.md section 11 / IMPLEMENTATION_PLAN.md
/// section 11): round 1 is fully paid and settles to the demo seller; in round 2 the
/// member who already received round 1's allocation stops paying; only the still-paying
/// member's round-2 deposit is refundable, and round 1's amounts are unrecoverable.
#[test]
fn plan_demo_scenario_round_one_settles_round_two_defaults_and_only_round_two_refunds() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_active_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap(); // round 1 recipient
    let second_member = members.get(1).unwrap(); // round 2 recipient

    // Round 1: both pay, purchase approved, seller paid, pool advances to round 2.
    client.deposit(&first_member, &pool_id);
    client.deposit(&second_member, &pool_id);
    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &first_member,
        &demo_seller,
        &token_address,
        20,
    );
    client.execute_round(&pool_id);
    assert_eq!(token_client.balance(&demo_seller), 20);
    assert!(client.get_member_status(&pool_id, &first_member).received);

    // Round 2: only second_member (this round's recipient) pays; first_member, who already
    // received round 1's allocation, stops paying entirely.
    client.deposit(&second_member, &pool_id);
    let deadline = client.get_round(&pool_id, &2).collect_deadline;
    let grace_deadline = deadline + GRACE_DURATION;
    env.ledger().set_timestamp(deadline);
    client.mark_overdue(&pool_id);
    env.ledger().set_timestamp(grace_deadline);
    client.abort_pool(&pool_id);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);
    // Only the round-2 deposit is refundable; round 1's contributions are gone for good.
    assert_eq!(
        client
            .get_member_status(&pool_id, &second_member)
            .refundable,
        10
    );
    assert_eq!(
        client.get_member_status(&pool_id, &first_member).refundable,
        0
    );
    assert_eq!(
        client.try_claim_refund(&first_member, &pool_id),
        Err(Ok(ContractError::RefundNotClaimable))
    );

    let contract_balance_before = token_client.balance(&contract_id);
    assert_eq!(client.claim_refund(&second_member, &pool_id), 10);
    assert_eq!(
        token_client.balance(&contract_id),
        contract_balance_before - 10
    );
    assert_eq!(token_client.balance(&second_member), 50 - 10 - 10 + 10); // minted 50, paid round1+round2, refunded round2
    assert_eq!(
        client.try_claim_refund(&second_member, &pool_id),
        Err(Ok(ContractError::RefundAlreadyClaimed))
    );
}

#[test]
fn claim_functions_require_stored_role_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let member_one = Address::generate(&env);
    let member_two = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address).mock_all_auths();
    token_admin.mint(&member_one, &50);
    token_admin.mint(&member_two, &50);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool_id = client.mock_all_auths().create_pool(
        &creator,
        &token_address,
        &10,
        &2,
        &OrderMode::Fixed,
        &0,
        &ROUND_DURATION,
        &GRACE_DURATION,
        &PURCHASE_DURATION,
        &(NOW + SETUP_WINDOW),
        &demo_seller,
    );
    client.mock_all_auths().join_pool(&member_one, &pool_id);
    client.mock_all_auths().join_pool(&member_two, &pool_id);
    let version = client.get_pool(&pool_id).terms_version;
    client
        .mock_all_auths()
        .approve_terms(&member_one, &pool_id, &version);
    client
        .mock_all_auths()
        .approve_terms(&member_two, &pool_id, &version);
    client.start_pool(&pool_id);
    client.mock_all_auths().deposit(&member_one, &pool_id);
    env.ledger().set_timestamp(NOW + ROUND_DURATION);
    client.mark_overdue(&pool_id);
    env.ledger()
        .set_timestamp(NOW + ROUND_DURATION + GRACE_DURATION);
    client.abort_pool(&pool_id);

    assert!(client.try_claim_refund(&member_one, &pool_id).is_err());
    assert_eq!(
        client.mock_all_auths().claim_refund(&member_one, &pool_id),
        10
    );
}

#[test]
fn same_token_pools_keep_assigned_balances_and_liabilities_isolated() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (first_pool, first_members) =
        prepare_active_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let (second_pool, second_members) =
        prepare_active_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    client.deposit(&first_members.get(0).unwrap(), &first_pool);
    client.deposit(&second_members.get(0).unwrap(), &second_pool);

    assert_eq!(client.get_round(&first_pool, &1).pot, 10);
    assert_eq!(client.get_round(&second_pool, &1).pot, 10);
    assert_eq!(
        client
            .get_member_status(&first_pool, &first_members.get(0).unwrap())
            .refundable,
        10
    );
    assert_eq!(
        client
            .get_member_status(&second_pool, &second_members.get(0).unwrap())
            .refundable,
        10
    );
    assert_eq!(token_client.balance(&contract_id), 20);
}

#[test]
fn expired_purchase_cannot_settle_before_abort() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members) =
        prepare_awaiting_purchase_pool(&env, &contract_id, &creator, &token_address, &seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    client.propose_purchase(
        &members.get(0).unwrap(),
        &pool_id,
        &seller,
        &token_address,
        &20,
        &BytesN::from_array(&env, &[7; 32]),
    );
    let deadline = client.get_round(&pool_id, &1).purchase_deadline.unwrap();
    env.ledger().set_timestamp(deadline);
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::DeadlineReached))
    );
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Active);
    client.abort_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);
}

// --- Draw mode (API v10, docs/CONTRACT_HANDOFF.md) ---------------------------------------

fn vec_contains(list: &Vec<Address>, candidate: &Address) -> bool {
    for item in list.iter() {
        if item == *candidate {
            return true;
        }
    }
    false
}

/// Runs a Draw pool to completion: every member pays, an outsider draws the recipient,
/// the recipient records a purchase, and anyone executes the round.
fn run_draw_pool_to_completion(
    env: &Env,
    contract_id: &Address,
    pool_id: u64,
    members: &Vec<Address>,
    demo_seller: &Address,
    token_address: &Address,
) -> Vec<Address> {
    let client = RotatingPoolContractClient::new(env, contract_id);
    let outsider = Address::generate(env);
    let mut winners: Vec<Address> = Vec::new(env);
    for round_index in 0..members.len() {
        let round_number = round_index + 1;
        deposit_all(env, contract_id, pool_id, members);
        assert_eq!(
            client.get_round(&pool_id, &round_number).phase,
            RoundPhase::AwaitingDraw
        );
        assert_eq!(client.get_round(&pool_id, &round_number).recipient, None);

        let winner = client.draw_recipient(&outsider, &pool_id);
        assert!(!vec_contains(&winners, &winner));
        assert_eq!(
            client.get_round(&pool_id, &round_number).phase,
            RoundPhase::AwaitingPurchase
        );
        assert_eq!(
            client.get_round(&pool_id, &round_number).recipient,
            Some(winner.clone())
        );

        propose_current_purchase(
            env,
            contract_id,
            pool_id,
            &winner,
            demo_seller,
            token_address,
            (round_number % 256) as u8,
        );
        client.execute_round(&pool_id);
        assert!(client.get_member_status(&pool_id, &winner).received);
        winners.push_back(winner);
    }
    winners
}

#[test]
fn draw_mode_full_flow_excludes_repeat_winners_with_four_members() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) = prepare_active_pool_n(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        4,
        OrderMode::Draw,
    );
    assert_eq!(
        client_order_mode(&env, &contract_id, pool_id),
        OrderMode::Draw
    );

    let winners = run_draw_pool_to_completion(
        &env,
        &contract_id,
        pool_id,
        &members,
        &demo_seller,
        &token_address,
    );

    assert_eq!(winners.len(), members.len());
    for member in members.iter() {
        assert!(vec_contains(&winners, &member));
    }
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Completed);
    assert_eq!(token_client.balance(&demo_seller), 40 * 4);
}

#[test]
fn draw_mode_full_flow_excludes_repeat_winners_with_thirty_members() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) = prepare_active_pool_n(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        MAX_MEMBERS,
        OrderMode::Draw,
    );
    assert_eq!(members.len(), MAX_MEMBERS);

    let winners = run_draw_pool_to_completion(
        &env,
        &contract_id,
        pool_id,
        &members,
        &demo_seller,
        &token_address,
    );

    // Every member wins exactly once (no repeats, full coverage) and the last round is
    // necessarily deterministic: only one member has not received yet by then.
    assert_eq!(winners.len(), MAX_MEMBERS);
    for member in members.iter() {
        assert!(vec_contains(&winners, &member));
    }
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Completed);
    assert_eq!(
        token_client.balance(&demo_seller),
        10 * (MAX_MEMBERS as i128) * (MAX_MEMBERS as i128)
    );
}

fn client_order_mode(env: &Env, contract_id: &Address, pool_id: u64) -> OrderMode {
    RotatingPoolContractClient::new(env, contract_id)
        .get_pool(&pool_id)
        .order_mode
}

#[test]
fn draw_recipient_rejected_before_round_fully_funded_and_after_first_draw() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members) = prepare_active_pool_n(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        3,
        OrderMode::Draw,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let outsider = Address::generate(&env);

    // Round not fully funded yet: still Collecting.
    client.deposit(&members.get(0).unwrap(), &pool_id);
    assert_eq!(
        client.try_draw_recipient(&outsider, &pool_id),
        Err(Ok(ContractError::RoundNotReady))
    );

    // Fully funded: draw succeeds once, then rejects a second draw in the same round.
    client.deposit(&members.get(1).unwrap(), &pool_id);
    client.deposit(&members.get(2).unwrap(), &pool_id);
    client.draw_recipient(&outsider, &pool_id);
    assert_eq!(
        client.try_draw_recipient(&outsider, &pool_id),
        Err(Ok(ContractError::RoundNotReady))
    );
}

#[test]
fn draw_recipient_rejected_on_fixed_order_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, _members) =
        prepare_active_pool(&env, &contract_id, &creator, &token_address, &demo_seller);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let outsider = Address::generate(&env);

    assert_eq!(
        client.try_draw_recipient(&outsider, &pool_id),
        Err(Ok(ContractError::NotDrawPool))
    );
}

#[test]
fn draw_pool_has_no_fixed_order_when_full() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_address = Address::generate(&env);
    let contract_id = register_contract(&env);
    let pool_id = create_pool_with_mode(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &seller,
        10,
        2,
        OrderMode::Draw,
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let pool = RotatingPoolContractClient::new(&env, &contract_id).get_pool(&pool_id);
    assert_eq!(pool.members, members);
    assert_eq!(pool.recipient_order.len(), 0);
    assert_eq!(pool.terms_version, 1);
}

#[test]
fn draw_mode_awaiting_draw_deadline_aborts_and_refunds_current_round_only() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) = prepare_active_pool_n(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        3,
        OrderMode::Draw,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    deposit_all(&env, &contract_id, pool_id, &members);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.phase, RoundPhase::AwaitingDraw);
    let purchase_deadline = round.purchase_deadline.unwrap();

    assert_eq!(
        client.try_abort_pool(&pool_id),
        Err(Ok(ContractError::DeadlineNotReached))
    );
    // No one draws before the deadline; abort_pool reclaims this round's deposits instead.
    env.ledger().set_timestamp(purchase_deadline);
    client.abort_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);

    let contract_balance_before = token_client.balance(&contract_id);
    for member in members.iter() {
        assert_eq!(client.get_member_status(&pool_id, &member).refundable, 10);
        assert_eq!(client.claim_refund(&member, &pool_id), 10);
        assert_eq!(
            client.try_claim_refund(&member, &pool_id),
            Err(Ok(ContractError::RefundAlreadyClaimed))
        );
    }
    assert_eq!(
        token_client.balance(&contract_id),
        contract_balance_before - 10 * (members.len() as i128)
    );
}

/// The plan's canonical demo scenario (docs/plan.md section 11), for Draw mode: round 1 is
/// fully paid, a recipient is drawn and settles; in round 2 that same member (who already
/// received) stops paying, so round 2 never reaches `AwaitingDraw` and only the still-paying
/// members' round-2 deposits are refundable — round 1's payout is gone for good.
#[test]
fn draw_mode_plan_demo_scenario_early_winner_defaults_next_round() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members) = prepare_active_pool_n(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        3,
        OrderMode::Draw,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let outsider = Address::generate(&env);

    // Round 1: everyone pays, a recipient is drawn and paid out.
    deposit_all(&env, &contract_id, pool_id, &members);
    let winner = client.draw_recipient(&outsider, &pool_id);
    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &winner,
        &demo_seller,
        &token_address,
        50,
    );
    client.execute_round(&pool_id);
    assert!(client.get_member_status(&pool_id, &winner).received);
    assert_eq!(client.get_pool(&pool_id).current_round, 2);

    // Round 2: the round-1 winner stops paying; the others still pay, but the round can never
    // reach AwaitingDraw without every member's deposit.
    let mut still_paying: Vec<Address> = Vec::new(&env);
    for member in members.iter() {
        if member != winner {
            client.deposit(&member, &pool_id);
            still_paying.push_back(member);
        }
    }
    assert_eq!(client.get_round(&pool_id, &2).phase, RoundPhase::Collecting);

    let deadline = client.get_round(&pool_id, &2).collect_deadline;
    let grace_deadline = deadline + GRACE_DURATION;
    env.ledger().set_timestamp(deadline);
    client.mark_overdue(&pool_id);
    env.ledger().set_timestamp(grace_deadline);
    client.abort_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);

    assert_eq!(client.get_member_status(&pool_id, &winner).refundable, 0);
    assert_eq!(
        client.try_claim_refund(&winner, &pool_id),
        Err(Ok(ContractError::RefundNotClaimable))
    );
    for member in still_paying.iter() {
        assert_eq!(client.get_member_status(&pool_id, &member).refundable, 10);
        assert_eq!(client.claim_refund(&member, &pool_id), 10);
    }
    let _ = token_client;
}

/// `Env::default()` enables invocation metering and enforces `InvocationResourceLimits::mainnet()`
/// on every top-level contract call by default (soroban-sdk 28), so any call in this test that
/// exceeded real Mainnet CPU/memory/read/write limits would already panic on its own — every
/// other test using a 30-member pool (e.g. `draw_mode_full_flow_excludes_repeat_winners_with_thirty_members`)
/// is implicitly proof of this too. This test additionally captures and prints the concrete
/// resource numbers for the five operations docs/CONTRACT_HANDOFF.md calls out, for
/// docs/IMPLEMENTATION_LOG.md. Native (non-Wasm) execution underestimates cost vs. the real Wasm
/// host, so these are a lower bound, not the authoritative figure — that comes from a live
/// `stellar contract invoke` simulation against the deployed instance.
#[test]
fn thirty_member_round_operations_stay_within_mainnet_resource_limits() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members) = prepare_active_pool_n(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        MAX_MEMBERS,
        OrderMode::Draw,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let outsider = Address::generate(&env);

    for member in members.iter() {
        client.deposit(&member, &pool_id);
    }
    let deposit_resources = env.cost_estimate().resources();
    std::println!("[30 members] last deposit: {:?}", deposit_resources);
    assert!(deposit_resources.write_entries <= 200);
    assert!(deposit_resources.disk_read_entries <= 200);

    let winner = client.draw_recipient(&outsider, &pool_id);
    let draw_resources = env.cost_estimate().resources();
    std::println!("[30 members] draw_recipient: {:?}", draw_resources);
    assert!(draw_resources.write_entries <= 200);
    assert!(draw_resources.disk_read_entries <= 200);

    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &winner,
        &demo_seller,
        &token_address,
        77,
    );

    client.execute_round(&pool_id);
    let execute_resources = env.cost_estimate().resources();
    std::println!("[30 members] execute_round: {:?}", execute_resources);
    assert!(execute_resources.write_entries <= 200);
    assert!(execute_resources.disk_read_entries <= 200);
}

// ---------------------------------------------------------------------------------------------
// v11: down payment (peşinat)
// ---------------------------------------------------------------------------------------------

fn create_pool_with_down_payment(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    token: &Address,
    demo_seller: &Address,
    contribution_amount: i128,
    member_limit: u32,
    order_mode: OrderMode,
    down_payment: i128,
) -> u64 {
    let setup_deadline = env.ledger().timestamp() + SETUP_WINDOW;
    RotatingPoolContractClient::new(env, contract_id).create_pool(
        creator,
        token,
        &contribution_amount,
        &member_limit,
        &order_mode,
        &down_payment,
        &ROUND_DURATION,
        &GRACE_DURATION,
        &PURCHASE_DURATION,
        &setup_deadline,
        demo_seller,
    )
}

/// Joins `count` members who already hold `mint_each` tokens (the down payment is pulled at join).
fn join_funded_members(
    env: &Env,
    contract_id: &Address,
    pool_id: u64,
    token_address: &Address,
    count: u32,
    mint_each: i128,
) -> Vec<Address> {
    let client = RotatingPoolContractClient::new(env, contract_id);
    let token_admin = token::StellarAssetClient::new(env, token_address);
    let mut members = Vec::new(env);
    for _ in 0..count {
        let member = Address::generate(env);
        token_admin.mint(&member, &mint_each);
        client.join_pool(&member, &pool_id);
        members.push_back(member);
    }
    members
}

fn start_with_terms(
    env: &Env,
    contract_id: &Address,
    _creator: &Address,
    pool_id: u64,
    members: &Vec<Address>,
    _order_mode: OrderMode,
) {
    let client = RotatingPoolContractClient::new(env, contract_id);
    approve_current_terms(env, contract_id, pool_id, members);
    client.start_pool(&pool_id);
}

#[test]
fn negative_down_payment_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = Address::generate(&env);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    assert_eq!(
        client.try_create_pool(
            &creator,
            &token_address,
            &10,
            &3,
            &OrderMode::Fixed,
            &-1,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &(NOW + SETUP_WINDOW),
            &demo_seller,
        ),
        Err(Ok(ContractError::InvalidDownPayment))
    );
}

#[test]
fn down_payment_is_escrowed_at_join_and_added_to_each_recipients_purchase() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 0);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let (contribution, down) = (10_i128, 5_i128);

    let pool_id = create_pool_with_down_payment(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        contribution,
        3,
        OrderMode::Fixed,
        down,
    );
    assert_eq!(client.get_pool(&pool_id).down_payment, down);
    let members = join_funded_members(&env, &contract_id, pool_id, &token_address, 3, 100);
    // Every member's down payment now sits in escrow; nobody has deposited a round yet.
    assert_eq!(token_client.balance(&contract_id), 3 * down);
    for member in members.iter() {
        assert_eq!(token_client.balance(&member), 100 - down);
    }
    start_with_terms(
        &env,
        &contract_id,
        &creator,
        pool_id,
        &members,
        OrderMode::Fixed,
    );

    for round in 1..=3_u32 {
        deposit_all(&env, &contract_id, pool_id, &members);
        let recipient = members.get(round - 1).unwrap();
        let pot = contribution * 3;

        // The purchase must include the recipient's own down payment; the bare pot is rejected.
        assert_eq!(
            client.try_propose_purchase(
                &recipient,
                &pool_id,
                &demo_seller,
                &token_address,
                &pot,
                &BytesN::from_array(&env, &[7; 32]),
            ),
            Err(Ok(ContractError::InvalidPurchaseAmount))
        );
        propose_current_purchase(
            &env,
            &contract_id,
            pool_id,
            &recipient,
            &demo_seller,
            &token_address,
            round as u8,
        );
        assert_eq!(client.execute_round(&pool_id), pot + down);
        assert_eq!(
            token_client.balance(&demo_seller),
            (round as i128) * (pot + down)
        );
    }

    // Completed: every down payment was spent on its owner's purchase, nothing is left in escrow.
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Completed);
    assert_eq!(
        token_client.balance(&demo_seller),
        3 * (contribution * 3 + down)
    );
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn down_payment_works_in_draw_mode_and_leaves_no_dust() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 0);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let (contribution, down) = (10_i128, 7_i128);

    let pool_id = create_pool_with_down_payment(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        contribution,
        4,
        OrderMode::Draw,
        down,
    );
    let members = join_funded_members(&env, &contract_id, pool_id, &token_address, 4, 200);
    start_with_terms(
        &env,
        &contract_id,
        &creator,
        pool_id,
        &members,
        OrderMode::Draw,
    );
    let winners = run_draw_pool_to_completion(
        &env,
        &contract_id,
        pool_id,
        &members,
        &demo_seller,
        &token_address,
    );

    assert_eq!(winners.len(), 4);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Completed);
    assert_eq!(
        token_client.balance(&demo_seller),
        4 * (contribution * 4 + down)
    );
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn aborted_pool_refunds_current_round_plus_unspent_down_payments_only() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 0);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let (contribution, down) = (10_i128, 5_i128);

    let pool_id = create_pool_with_down_payment(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        contribution,
        3,
        OrderMode::Fixed,
        down,
    );
    let members = join_funded_members(&env, &contract_id, pool_id, &token_address, 3, 100);
    let (a, b, c) = (
        members.get(0).unwrap(),
        members.get(1).unwrap(),
        members.get(2).unwrap(),
    );
    start_with_terms(
        &env,
        &contract_id,
        &creator,
        pool_id,
        &members,
        OrderMode::Fixed,
    );

    // Round 1 completes: A (first in order) receives, so A's down payment is spent.
    deposit_all(&env, &contract_id, pool_id, &members);
    propose_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &a,
        &demo_seller,
        &token_address,
        1,
    );
    client.execute_round(&pool_id);
    assert_eq!(client.get_member_status(&pool_id, &a).refundable, 0);
    // Before any abort, B and C would only be owed their still-unspent down payment.
    assert_eq!(client.get_member_status(&pool_id, &b).refundable, down);

    // Round 2: A and B pay, C does not -> Grace -> abort.
    client.deposit(&a, &pool_id);
    client.deposit(&b, &pool_id);
    env.ledger().set_timestamp(NOW + ROUND_DURATION);
    client.mark_overdue(&pool_id);
    env.ledger()
        .set_timestamp(NOW + ROUND_DURATION + GRACE_DURATION);
    client.abort_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);

    // A already received: only this round's contribution comes back. B: contribution + down payment.
    // C never paid this round: only the unspent down payment.
    assert_eq!(
        client.get_member_status(&pool_id, &a).refundable,
        contribution
    );
    assert_eq!(
        client.get_member_status(&pool_id, &b).refundable,
        contribution + down
    );
    assert_eq!(client.get_member_status(&pool_id, &c).refundable, down);
    let before: [i128; 3] = [
        token_client.balance(&a),
        token_client.balance(&b),
        token_client.balance(&c),
    ];
    assert_eq!(client.claim_refund(&a, &pool_id), contribution);
    assert_eq!(client.claim_refund(&b, &pool_id), contribution + down);
    assert_eq!(client.claim_refund(&c, &pool_id), down);
    assert_eq!(token_client.balance(&a), before[0] + contribution);
    assert_eq!(token_client.balance(&b), before[1] + contribution + down);
    assert_eq!(token_client.balance(&c), before[2] + down);

    // No double refunds, and the escrow is fully drained (round 1 already went to the seller).
    assert_eq!(
        client.try_claim_refund(&b, &pool_id),
        Err(Ok(ContractError::RefundAlreadyClaimed))
    );
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn cancelled_unstarted_pool_refunds_down_payments() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 0);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let down = 25_i128;

    let pool_id = create_pool_with_down_payment(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        10,
        3,
        OrderMode::Fixed,
        down,
    );
    let members = join_funded_members(&env, &contract_id, pool_id, &token_address, 2, 100);
    assert_eq!(token_client.balance(&contract_id), 2 * down);

    env.ledger().set_timestamp(NOW + SETUP_WINDOW);
    client.cancel_unstarted_pool(&pool_id);
    let outsider = Address::generate(&env);
    assert_eq!(
        client.try_claim_refund(&outsider, &pool_id),
        Err(Ok(ContractError::NotMember))
    );
    for member in members.iter() {
        assert_eq!(client.claim_refund(&member, &pool_id), down);
        assert_eq!(token_client.balance(&member), 100);
    }
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn join_without_enough_tokens_for_the_down_payment_fails() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &creator, 0);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let pool_id = create_pool_with_down_payment(
        &env,
        &contract_id,
        &creator,
        &token_address,
        &demo_seller,
        10,
        3,
        OrderMode::Fixed,
        5,
    );
    let broke = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_address).mint(&broke, &4);
    assert!(client.try_join_pool(&broke, &pool_id).is_err());
    assert_eq!(client.get_pool(&pool_id).members.len(), 0);
}
