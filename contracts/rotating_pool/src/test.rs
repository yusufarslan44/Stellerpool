use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, BytesN, Env, Vec,
};

use crate::{
    storage, ContractError, PoolStatus, RotatingPoolContract, RotatingPoolContractClient,
    RoundPhase, CONTRACT_VERSION,
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
    sponsor: &Address,
    token: &Address,
    demo_seller: &Address,
    contribution_amount: i128,
    member_limit: u32,
) -> u64 {
    let setup_deadline = env.ledger().timestamp() + SETUP_WINDOW;
    RotatingPoolContractClient::new(env, contract_id).create_pool(
        creator,
        sponsor,
        token,
        &contribution_amount,
        &member_limit,
        &ROUND_DURATION,
        &GRACE_DURATION,
        &PURCHASE_DURATION,
        &setup_deadline,
        demo_seller,
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

fn propose_and_approve_terms(
    env: &Env,
    contract_id: &Address,
    pool_id: u64,
    creator: &Address,
    sponsor: &Address,
    members: &Vec<Address>,
    verifiers: &Vec<Address>,
) -> u32 {
    let client = RotatingPoolContractClient::new(env, contract_id);
    let version = client.propose_terms(creator, &pool_id, members, verifiers);
    for member in members.iter() {
        client.approve_terms(&member, &pool_id, &version);
    }
    client.approve_terms(sponsor, &pool_id, &version);
    version
}

fn prepare_active_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    sponsor: &Address,
    token_address: &Address,
    demo_seller: &Address,
) -> (u64, Vec<Address>, Vec<Address>) {
    let pool_id = create_pool(
        env,
        contract_id,
        creator,
        sponsor,
        token_address,
        demo_seller,
        10,
        2,
    );
    let members = join_members(env, contract_id, pool_id, 2);
    let token_admin = token::StellarAssetClient::new(env, token_address);
    for member in members.iter() {
        token_admin.mint(&member, &50);
    }
    let client = RotatingPoolContractClient::new(env, contract_id);
    client.fund_guarantee(&pool_id, sponsor, &10);
    let verifiers = Vec::from_array(env, [Address::generate(env), Address::generate(env)]);
    propose_and_approve_terms(
        env,
        contract_id,
        pool_id,
        creator,
        sponsor,
        &members,
        &verifiers,
    );
    client.start_pool(&pool_id);
    (pool_id, members, verifiers)
}

fn prepare_awaiting_purchase_pool(
    env: &Env,
    contract_id: &Address,
    creator: &Address,
    sponsor: &Address,
    token_address: &Address,
    demo_seller: &Address,
) -> (u64, Vec<Address>, Vec<Address>) {
    let (pool_id, members, verifiers) = prepare_active_pool(
        env,
        contract_id,
        creator,
        sponsor,
        token_address,
        demo_seller,
    );
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
    demo_seller: &Address,
    token_address: &Address,
    verifiers: &Vec<Address>,
    digest_marker: u8,
) {
    let client = RotatingPoolContractClient::new(env, contract_id);
    let pool = client.get_pool(&pool_id);
    let amount = pool.contribution_amount * (pool.member_limit as i128);
    client.propose_purchase(
        recipient,
        &pool_id,
        demo_seller,
        token_address,
        &amount,
        &BytesN::from_array(env, &[digest_marker; 32]),
    );
    let round = client.get_round(&pool_id, &pool.current_round);
    for verifier in verifiers.iter() {
        client.approve_purchase(
            &verifier,
            &pool_id,
            &pool.current_round,
            &round.purchase_version,
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
fn create_pool_uses_sponsor_guarantee_formula_and_stores_fields() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 1000);
    let contract_id = register_contract(&env);

    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
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
    assert_eq!(pool.required_guarantee, 40);
    assert_eq!(pool.member_limit, 4);
    assert_eq!(pool.members.len(), 0);
    assert_eq!(pool.round_duration, ROUND_DURATION);
    assert_eq!(pool.grace_duration, GRACE_DURATION);
    assert_eq!(pool.purchase_duration, PURCHASE_DURATION);
    assert_eq!(pool.setup_deadline, NOW + SETUP_WINDOW);
    assert_eq!(pool.demo_seller, demo_seller);
    assert_eq!(pool.status, PoolStatus::Filling);
    assert_eq!(pool.created_at, NOW);
    assert_eq!(pool.started_at, None);
    assert_eq!(pool.guarantee_deposited, 0);
    assert_eq!(client.next_pool_id(), 2);
}

#[test]
fn create_pool_requires_creator_auth_and_valid_parameters() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = Address::generate(&env);
    let contract_id = register_contract(&env);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let setup_deadline = NOW + SETUP_WINDOW;

    assert!(client
        .try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &10,
            &4,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &setup_deadline,
            &demo_seller,
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
            &PURCHASE_DURATION,
            &setup_deadline,
            &demo_seller,
        ),
        Err(Ok(ContractError::InvalidContributionAmount))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &10,
            &13,
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
            &sponsor,
            &token_address,
            &10,
            &4,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &0,
            &setup_deadline,
            &demo_seller,
        ),
        Err(Ok(ContractError::InvalidPurchaseDuration))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &10,
            &4,
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
            &sponsor,
            &token_address,
            &10,
            &4,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &setup_deadline,
            &creator,
        ),
        Err(Ok(ContractError::SellerCannotBeParticipant))
    );
    assert_eq!(
        client.try_create_pool(
            &creator,
            &sponsor,
            &token_address,
            &i128::MAX,
            &12,
            &ROUND_DURATION,
            &GRACE_DURATION,
            &PURCHASE_DURATION,
            &setup_deadline,
            &demo_seller,
        ),
        Err(Ok(ContractError::ArithmeticOverflow))
    );
}

#[test]
fn members_join_without_locking_funds_and_seller_is_excluded() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
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
        &demo_seller,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    client.join_pool(&member, &pool_id);

    assert_eq!(token_client.balance(&member), 25);
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
fn sponsor_funds_guarantee_with_separate_pool_accounting() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let first_pool = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
        10,
        4,
    );
    let second_pool = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    assert_eq!(client.fund_guarantee(&first_pool, &sponsor, &25), 25);
    assert_eq!(client.fund_guarantee(&first_pool, &sponsor, &15), 40);
    assert_eq!(client.fund_guarantee(&second_pool, &sponsor, &10), 10);

    assert_eq!(client.get_pool(&first_pool).guarantee_deposited, 40);
    assert_eq!(client.get_pool(&second_pool).guarantee_deposited, 10);
    assert_eq!(token_client.balance(&sponsor), 50);
    assert_eq!(token_client.balance(&contract_id), 50);
    assert_eq!(
        client.try_fund_guarantee(&first_pool, &sponsor, &0),
        Err(Ok(ContractError::InvalidAmount))
    );
    assert_eq!(
        client.try_fund_guarantee(&first_pool, &creator, &5),
        Err(Ok(ContractError::SponsorOnly))
    );
}

#[test]
fn terms_require_creator_and_reset_approvals_on_change() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
        10,
        2,
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);

    let outsider = Address::generate(&env);
    assert_eq!(
        client.try_propose_terms(&outsider, &pool_id, &members, &verifiers),
        Err(Ok(ContractError::CreatorOnly))
    );

    let version = client.propose_terms(&creator, &pool_id, &members, &verifiers);
    assert_eq!(version, 1);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.approval_threshold, 2);
    assert_eq!(pool.recipient_order, members);
    assert_eq!(pool.verifiers, verifiers);
    assert_eq!(pool.terms_approvals.len(), 0);

    client.approve_terms(&members.get(0).unwrap(), &pool_id, &version);
    assert_eq!(client.get_pool(&pool_id).terms_approvals.len(), 1);
    assert_eq!(
        client.try_approve_terms(&members.get(0).unwrap(), &pool_id, &version),
        Err(Ok(ContractError::AlreadyApprovedTerms))
    );
    assert_eq!(
        client.try_approve_terms(&outsider, &pool_id, &version),
        Err(Ok(ContractError::NotApprover))
    );

    // Re-proposing (e.g. a different order) bumps the version and clears approvals.
    let reordered = Vec::from_array(&env, [members.get(1).unwrap(), members.get(0).unwrap()]);
    let version2 = client.propose_terms(&creator, &pool_id, &reordered, &verifiers);
    assert_eq!(version2, 2);
    assert_eq!(client.get_pool(&pool_id).terms_approvals.len(), 0);
    assert_eq!(
        client.try_approve_terms(&members.get(0).unwrap(), &pool_id, &version),
        Err(Ok(ContractError::TermsVersionMismatch))
    );
}

#[test]
fn start_pool_requires_full_membership_all_approvals_and_guarantee() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
        10,
        2,
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);

    assert_eq!(
        client.try_start_pool(&pool_id),
        Err(Ok(ContractError::TermsNotProposed))
    );

    let version = client.propose_terms(&creator, &pool_id, &members, &verifiers);
    client.approve_terms(&members.get(0).unwrap(), &pool_id, &version);
    assert_eq!(
        client.try_start_pool(&pool_id),
        Err(Ok(ContractError::TermsNotFullyApproved))
    );
    client.approve_terms(&members.get(1).unwrap(), &pool_id, &version);
    client.approve_terms(&sponsor, &pool_id, &version);

    assert_eq!(
        client.try_start_pool(&pool_id),
        Err(Ok(ContractError::InsufficientGuarantee))
    );
    client.fund_guarantee(&pool_id, &sponsor, &10);

    // Anyone (not just creator/sponsor) may call start_pool.
    let outsider = Address::generate(&env);
    client.start_pool(&pool_id);
    let _ = outsider;

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.status, PoolStatus::Active);
    assert_eq!(pool.started_at, Some(NOW));
    assert_eq!(pool.current_round, 1);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.recipient, members.get(0).unwrap());
    assert_eq!(round.phase, RoundPhase::Collecting);
    assert_eq!(round.collect_deadline, NOW + ROUND_DURATION);
    assert_eq!(round.pot, 0);
}

#[test]
fn cancel_unstarted_pool_refunds_sponsor_after_setup_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
        10,
        2,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    client.fund_guarantee(&pool_id, &sponsor, &10);
    join_members(&env, &contract_id, pool_id, 1);

    assert_eq!(
        client.try_cancel_unstarted_pool(&pool_id),
        Err(Ok(ContractError::SetupDeadlineNotReached))
    );

    env.ledger().set_timestamp(NOW + SETUP_WINDOW);
    assert_eq!(client.cancel_unstarted_pool(&pool_id), 10);

    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Aborted);
    assert_eq!(client.get_pool(&pool_id).guarantee_deposited, 0);
    assert_eq!(token_client.balance(&sponsor), 100);
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
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();

    assert_eq!(client.deposit(&first_member, &pool_id), 10);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.phase, RoundPhase::Collecting);
    assert_eq!(round.pot, 10);

    assert_eq!(client.deposit(&second_member, &pool_id), 20);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.phase, RoundPhase::AwaitingPurchase);
    assert_eq!(round.pot, 20);
    assert_eq!(round.purchase_deadline, Some(NOW + PURCHASE_DURATION));

    assert_eq!(
        client.get_member_status(&pool_id, &first_member).refundable,
        10
    );
    assert_eq!(client.get_pool(&pool_id).guarantee_deposited, 10);
    assert_eq!(token_client.balance(&contract_id), 30);
    // The round already moved past Collecting into AwaitingPurchase, so a repeated
    // deposit call is rejected by the phase gate before it would even reach the
    // already-deposited check.
    assert_eq!(
        client.try_deposit(&first_member, &pool_id),
        Err(Ok(ContractError::InvalidPoolStatus))
    );
}

#[test]
fn top_up_cannot_cover_the_current_recipient_and_creates_an_advance() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let recipient = members.get(0).unwrap();
    let missing_member = members.get(1).unwrap();

    assert_eq!(
        client.try_top_up(&pool_id, &sponsor, &recipient),
        Err(Ok(ContractError::TopUpNotAllowedForRecipient))
    );

    assert_eq!(client.top_up(&pool_id, &sponsor, &missing_member), 10);
    assert_eq!(client.get_sponsor_advance(&pool_id, &missing_member), 10);
    // Sponsor top-up does not create a member refund liability for the covered member.
    assert_eq!(
        client
            .get_member_status(&pool_id, &missing_member)
            .refundable,
        0
    );

    // Round still needs the recipient to self-pay before it can advance.
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.phase, RoundPhase::Collecting);
    assert_eq!(round.pot, 10);

    client.deposit(&recipient, &pool_id);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.phase, RoundPhase::AwaitingPurchase);
    assert_eq!(round.pot, 20);
}

#[test]
fn recipient_must_self_pay_and_close_old_advance_before_execution() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();

    // First round settles normally.
    client.deposit(&first_member, &pool_id);
    client.deposit(&second_member, &pool_id);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &first_member,
        &demo_seller,
        &token_address,
        &verifiers,
        1,
    );
    client.execute_round(&pool_id);

    // Second round: second_member is now the recipient but sponsor advances first_member's
    // contribution (not the recipient's), then second_member is late on their own payment.
    client.top_up(&pool_id, &sponsor, &first_member);
    assert_eq!(client.get_round(&pool_id, &2).phase, RoundPhase::Collecting);

    env.ledger().set_timestamp(NOW + ROUND_DURATION);
    client.mark_overdue(&pool_id);
    env.ledger().set_timestamp(NOW + ROUND_DURATION + 1);
    client.cure_payment(&second_member, &pool_id);

    let round = client.get_round(&pool_id, &2);
    assert_eq!(round.phase, RoundPhase::AwaitingPurchase);

    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &second_member,
        &demo_seller,
        &token_address,
        &verifiers,
        2,
    );
    assert_eq!(client.execute_round(&pool_id), 20);
    assert_eq!(client.get_pool(&pool_id).status, PoolStatus::Completed);
    assert_eq!(token_client.balance(&contract_id), 10); // only the sponsor guarantee remains
}

#[test]
fn repay_advance_clears_debt_and_can_unblock_the_round() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();

    client.deposit(&first_member, &pool_id);
    client.deposit(&second_member, &pool_id);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &first_member,
        &demo_seller,
        &token_address,
        &verifiers,
        3,
    );
    client.execute_round(&pool_id);

    // Sponsor covers the round-2 recipient's predecessor advance is not possible (recipient
    // rule), but a *non-recipient* advance from round 1 can still be outstanding when it's
    // that same member's turn later. Simulate second_member owing an advance from elsewhere
    // and repaying it while they are the current recipient.
    env.as_contract(&contract_id, || {
        storage::write_sponsor_advance(&env, pool_id, &second_member, 5);
    });
    client.deposit(&first_member, &pool_id);
    client.deposit(&second_member, &pool_id);
    // Pot is complete but recipient still owes an advance, so it must stay Collecting/Grace.
    assert_eq!(client.get_round(&pool_id, &2).phase, RoundPhase::Collecting);

    assert_eq!(
        client.try_repay_advance(&second_member, &pool_id),
        Ok(Ok(5))
    );
    assert_eq!(client.get_sponsor_advance(&pool_id, &second_member), 0);
    assert_eq!(
        client.get_round(&pool_id, &2).phase,
        RoundPhase::AwaitingPurchase
    );
    assert_eq!(token_client.balance(&sponsor), 100 - 10 + 5); // funded 10, got 5 repaid back
    assert_eq!(
        client.try_repay_advance(&second_member, &pool_id),
        Err(Ok(ContractError::NoOutstandingAdvance))
    );
}

#[test]
fn only_current_recipient_can_propose_a_valid_ready_purchase() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let outsider_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) = prepare_awaiting_purchase_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
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

    let version = client.propose_purchase(
        &recipient,
        &pool_id,
        &demo_seller,
        &token_address,
        &20,
        &digest,
    );
    assert_eq!(version, 1);
    let round = client.get_round(&pool_id, &1);
    assert_eq!(round.seller, Some(demo_seller.clone()));
    assert_eq!(round.doc_hash, Some(digest));
    assert_eq!(round.purchase_version, 1);

    // Re-proposing bumps the version and clears prior approvals.
    let digest2 = BytesN::from_array(&env, &[8; 32]);
    let version2 = client.propose_purchase(
        &recipient,
        &pool_id,
        &demo_seller,
        &token_address,
        &20,
        &digest2,
    );
    assert_eq!(version2, 2);
    assert_eq!(client.get_round(&pool_id, &1).approvals.len(), 0);
}

#[test]
fn verifiers_reach_the_two_thirds_quorum_once_each() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let outsider = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) = prepare_awaiting_purchase_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let recipient = members.get(0).unwrap();
    let first_verifier = verifiers.get(0).unwrap();
    let second_verifier = verifiers.get(1).unwrap();

    client.propose_purchase(
        &recipient,
        &pool_id,
        &demo_seller,
        &token_address,
        &20,
        &BytesN::from_array(&env, &[9; 32]),
    );

    assert_eq!(
        client.try_approve_purchase(&outsider, &pool_id, &1, &1),
        Err(Ok(ContractError::UnauthorizedVerifier))
    );
    assert_eq!(
        client.try_approve_purchase(&first_verifier, &pool_id, &1, &2),
        Err(Ok(ContractError::PurchaseVersionMismatch))
    );

    assert_eq!(
        client.approve_purchase(&first_verifier, &pool_id, &1, &1),
        1
    );
    assert_eq!(
        client.try_approve_purchase(&first_verifier, &pool_id, &1, &1),
        Err(Ok(ContractError::AlreadyApproved))
    );
    assert_eq!(
        client.approve_purchase(&second_verifier, &pool_id, &1, &1),
        2
    );
    assert_eq!(client.get_round(&pool_id, &1).approvals.len(), 2);
}

#[test]
fn approved_rounds_pay_the_demo_seller_and_complete_the_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) = prepare_awaiting_purchase_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let first_member = members.get(0).unwrap();
    let second_member = members.get(1).unwrap();

    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &first_member,
        &demo_seller,
        &token_address,
        &verifiers,
        12,
    );
    assert_eq!(client.execute_round(&pool_id), 20);
    assert_eq!(token_client.balance(&demo_seller), 20);
    assert_eq!(client.get_round(&pool_id, &1).phase, RoundPhase::Settled);
    assert!(client.get_member_status(&pool_id, &first_member).received);
    assert_eq!(client.get_pool(&pool_id).current_round, 2);
    assert_eq!(client.get_round(&pool_id, &2).recipient, second_member);

    client.deposit(&first_member, &pool_id);
    client.deposit(&second_member, &pool_id);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &second_member,
        &demo_seller,
        &token_address,
        &verifiers,
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
fn execute_round_requires_quorum_and_pool_scoped_solvency() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) = prepare_awaiting_purchase_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
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
    client.approve_purchase(&verifiers.get(0).unwrap(), &pool_id, &1, &1);
    assert_eq!(
        client.try_execute_round(&pool_id),
        Err(Ok(ContractError::PurchaseNotApproved))
    );
    client.approve_purchase(&verifiers.get(1).unwrap(), &pool_id, &1, &1);

    env.as_contract(&contract_id, || {
        storage::write_pool_assigned_balance(&env, pool_id, 15);
    });
    assert_eq!(token_client.balance(&contract_id), 30);
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
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) = prepare_awaiting_purchase_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &members.get(0).unwrap(),
        &demo_seller,
        &token_address,
        &verifiers,
        15,
    );
    token_client.burn(&contract_id, &20);
    assert_eq!(token_client.balance(&contract_id), 10);

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
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
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
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, members, verifiers) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
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

    propose_and_approve_current_purchase(
        &env,
        &contract_id,
        pool_id,
        &paid_member,
        &demo_seller,
        &token_address,
        &verifiers,
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
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let (pool_id, _members, _verifiers) = prepare_awaiting_purchase_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
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

#[test]
fn undelivered_member_reclaims_contribution_and_sponsor_claims_freed_remainder() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (pool_id, members, _) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let client = RotatingPoolContractClient::new(&env, &contract_id);
    let paid_member = members.get(0).unwrap();
    let missing_member = members.get(1).unwrap();
    let deadline = NOW + ROUND_DURATION;
    let grace_deadline = deadline + GRACE_DURATION;

    client.deposit(&paid_member, &pool_id);
    env.ledger().set_timestamp(deadline);
    client.mark_overdue(&pool_id);
    env.ledger().set_timestamp(grace_deadline);
    client.abort_pool(&pool_id);

    assert_eq!(
        client.get_member_status(&pool_id, &paid_member).refundable,
        10
    );
    assert_eq!(
        client
            .get_member_status(&pool_id, &missing_member)
            .refundable,
        0
    );
    assert_eq!(token_client.balance(&contract_id), 20);

    assert_eq!(
        client.try_claim_refund(&missing_member, &pool_id),
        Err(Ok(ContractError::RefundNotClaimable))
    );
    assert_eq!(client.claim_sponsor_remainder(&sponsor, &pool_id), 10);
    assert_eq!(token_client.balance(&sponsor), 100);
    assert_eq!(
        client.try_claim_sponsor_remainder(&sponsor, &pool_id),
        Err(Ok(ContractError::SponsorRemainderNotClaimable))
    );

    assert_eq!(client.claim_refund(&paid_member, &pool_id), 10);
    assert_eq!(token_client.balance(&paid_member), 50); // minted 50, deposited 10, refunded 10
    assert!(client.get_refund_claim(&pool_id, &paid_member));
    assert_eq!(
        client.try_claim_refund(&paid_member, &pool_id),
        Err(Ok(ContractError::RefundAlreadyClaimed))
    );
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn claim_functions_require_stored_role_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let member_one = Address::generate(&env);
    let member_two = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address).mock_all_auths();
    token_admin.mint(&sponsor, &100);
    token_admin.mint(&member_one, &50);
    token_admin.mint(&member_two, &50);
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
        &PURCHASE_DURATION,
        &(NOW + SETUP_WINDOW),
        &demo_seller,
    );
    client.mock_all_auths().join_pool(&member_one, &pool_id);
    client.mock_all_auths().join_pool(&member_two, &pool_id);
    let order = Vec::from_array(&env, [member_one.clone(), member_two.clone()]);
    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    let version = client
        .mock_all_auths()
        .propose_terms(&creator, &pool_id, &order, &verifiers);
    client
        .mock_all_auths()
        .approve_terms(&member_one, &pool_id, &version);
    client
        .mock_all_auths()
        .approve_terms(&member_two, &pool_id, &version);
    client
        .mock_all_auths()
        .approve_terms(&sponsor, &pool_id, &version);
    client
        .mock_all_auths()
        .fund_guarantee(&pool_id, &sponsor, &10);
    client.start_pool(&pool_id);
    client.mock_all_auths().deposit(&member_one, &pool_id);
    client.mock_all_auths().deposit(&member_two, &pool_id);
    env.ledger().set_timestamp(NOW + PURCHASE_DURATION);
    client.abort_pool(&pool_id);

    assert!(client.try_claim_refund(&member_one, &pool_id).is_err());
    assert_eq!(
        client.mock_all_auths().claim_refund(&member_one, &pool_id),
        10
    );
    assert!(client
        .try_claim_sponsor_remainder(&sponsor, &pool_id)
        .is_err());
    assert_eq!(
        client
            .mock_all_auths()
            .claim_sponsor_remainder(&sponsor, &pool_id),
        10
    );
}

#[test]
fn same_token_pools_keep_assigned_balances_and_liabilities_isolated() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let token_client = token::TokenClient::new(&env, &token_address);
    let contract_id = register_contract(&env);
    let (first_pool, first_members, _) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
    let (second_pool, second_members, _) = prepare_active_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
    );
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
    assert_eq!(token_client.balance(&contract_id), 40); // 2 x (10 guarantee + 10 deposit)
}

#[test]
fn verifier_policy_excludes_participants_and_enforces_bounds() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let demo_seller = Address::generate(&env);
    let token_address = register_token(&env, &sponsor, 100);
    let contract_id = register_contract(&env);
    let pool_id = create_pool(
        &env,
        &contract_id,
        &creator,
        &sponsor,
        &token_address,
        &demo_seller,
        10,
        3,
    );
    let members = join_members(&env, &contract_id, pool_id, 2);
    let client = RotatingPoolContractClient::new(&env, &contract_id);

    let one_verifier = Vec::from_array(&env, [Address::generate(&env)]);
    assert_eq!(
        client.try_propose_terms(&creator, &pool_id, &members, &one_verifier),
        Err(Ok(ContractError::InvalidVerifierSet))
    );
    let verifier = Address::generate(&env);
    let duplicate = Vec::from_array(&env, [verifier.clone(), verifier]);
    assert_eq!(
        client.try_propose_terms(&creator, &pool_id, &members, &duplicate),
        Err(Ok(ContractError::DuplicateVerifier))
    );
    let with_sponsor = Vec::from_array(&env, [sponsor.clone(), Address::generate(&env)]);
    assert_eq!(
        client.try_propose_terms(&creator, &pool_id, &members, &with_sponsor),
        Err(Ok(ContractError::VerifierCannotBeParticipant))
    );
    let with_member = Vec::from_array(&env, [members.get(0).unwrap(), Address::generate(&env)]);
    assert_eq!(
        client.try_propose_terms(&creator, &pool_id, &members, &with_member),
        Err(Ok(ContractError::VerifierCannotBeParticipant))
    );

    let verifiers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    client.propose_terms(&creator, &pool_id, &members, &verifiers);
    assert_eq!(
        client.try_join_pool(&verifiers.get(0).unwrap(), &pool_id),
        Err(Ok(ContractError::VerifierCannotBeParticipant))
    );
}
