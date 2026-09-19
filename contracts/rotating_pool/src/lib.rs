#![no_std]

mod error;
mod events;
mod storage;
mod types;

pub use error::ContractError;
pub use events::*;
pub use types::{
    AbortReason, MemberState, Pool, PoolStatus, PurchaseState, RoundState, RoundStatus,
    VerifierPolicy,
};

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec};

pub const CONTRACT_VERSION: u32 = 7;
pub const MIN_MEMBERS: u32 = 2;
pub const MAX_MEMBERS: u32 = 20;
pub const MIN_VERIFIERS: u32 = 2;
pub const MAX_VERIFIERS: u32 = 10;

#[contract]
pub struct RotatingPoolContract;

#[contractimpl]
impl RotatingPoolContract {
    pub fn create_pool(
        env: Env,
        creator: Address,
        sponsor: Address,
        token: Address,
        contribution_amount: i128,
        member_limit: u32,
        round_duration_secs: u64,
        grace_duration_secs: u64,
    ) -> Result<u64, ContractError> {
        creator.require_auth();
        validate_pool_parameters(
            contribution_amount,
            member_limit,
            round_duration_secs,
            grace_duration_secs,
        )?;

        let pool_id = storage::read_next_pool_id(&env);
        let next_pool_id = pool_id
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let guarantee_required = calculate_required_guarantee(member_limit, contribution_amount)?;
        let created_at = env.ledger().timestamp();
        let pool = Pool {
            id: pool_id,
            creator: creator.clone(),
            sponsor: sponsor.clone(),
            token: token.clone(),
            contribution_amount,
            guarantee_required,
            member_limit,
            member_count: 0,
            round_duration_secs,
            grace_duration_secs,
            status: PoolStatus::Filling,
            current_round: 0,
            created_at,
            started_at: None,
        };

        storage::write_pool(&env, &pool);
        storage::write_members(&env, pool_id, &Vec::new(&env));
        storage::write_guarantee_balance(&env, pool_id, 0);
        storage::write_pool_assigned_balance(&env, pool_id, 0);
        storage::write_total_refund_liability(&env, pool_id, 0);
        storage::write_next_pool_id(&env, next_pool_id);

        PoolCreated {
            pool_id,
            creator,
            sponsor,
            token,
            contribution_amount,
            guarantee_required,
            member_limit,
            round_duration_secs,
            grace_duration_secs,
        }
        .publish(&env);

        Ok(pool_id)
    }

    pub fn fund_guarantee(env: Env, pool_id: u64, amount: i128) -> Result<i128, ContractError> {
        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        pool.sponsor.require_auth();

        let current = storage::read_guarantee_balance(&env, pool_id);
        let total_guarantee = current
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id)
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;

        token::TokenClient::new(&env, &pool.token).transfer(
            &pool.sponsor,
            &env.current_contract_address(),
            &amount,
        );
        storage::write_guarantee_balance(&env, pool_id, total_guarantee);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_balance);

        GuaranteeFunded {
            pool_id,
            sponsor: pool.sponsor,
            amount,
            total_guarantee,
        }
        .publish(&env);

        Ok(total_guarantee)
    }

    pub fn deposit(env: Env, member: Address, pool_id: u64) -> Result<i128, ContractError> {
        member.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round_index = pool.current_round;
        let round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if env.ledger().timestamp() >= round.deadline {
            return Err(ContractError::DeadlineReached);
        }
        let (round_index, amount, round_pot) = apply_member_payment(&env, &pool, &member)?;

        ContributionDeposited {
            pool_id,
            round: round_index,
            member,
            amount,
        }
        .publish(&env);

        Ok(round_pot)
    }

    pub fn configure_verifiers(
        env: Env,
        creator: Address,
        pool_id: u64,
        verifiers: Vec<Address>,
        approval_quorum: u32,
    ) -> Result<(), ContractError> {
        let pool = get_pool_or_error(&env, pool_id)?;
        if creator != pool.creator {
            return Err(ContractError::CreatorOnly);
        }
        creator.require_auth();
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        validate_verifier_policy(&env, &pool, &verifiers, approval_quorum)?;

        let verifier_count = verifiers.len();
        let policy = VerifierPolicy {
            verifiers,
            approval_quorum,
        };
        storage::write_verifier_policy(&env, pool_id, &policy);

        VerifierPolicyConfigured {
            pool_id,
            creator,
            verifier_count,
            approval_quorum,
        }
        .publish(&env);
        Ok(())
    }

    pub fn join_pool(env: Env, member: Address, pool_id: u64) -> Result<(), ContractError> {
        member.require_auth();
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        if pool.member_count >= pool.member_limit {
            return Err(ContractError::PoolFull);
        }
        if storage::read_member(&env, pool_id, &member).is_some() {
            return Err(ContractError::AlreadyMember);
        }
        if let Some(policy) = storage::read_verifier_policy(&env, pool_id) {
            if is_verifier(&policy.verifiers, &member) {
                return Err(ContractError::VerifierCannotBeParticipant);
            }
        }

        let joined_at = env.ledger().timestamp();
        let state = MemberState {
            joined_at,
            active: true,
            allocation_received: false,
            contributions_paid: 0,
        };
        let mut members = storage::read_members(&env, pool_id);
        members.push_back(member.clone());
        pool.member_count = pool
            .member_count
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;

        storage::write_member(&env, pool_id, &member, &state);
        storage::write_members(&env, pool_id, &members);
        storage::write_pool(&env, &pool);

        MemberJoined {
            pool_id,
            member,
            joined_at,
        }
        .publish(&env);
        Ok(())
    }

    pub fn leave_pool(env: Env, member: Address, pool_id: u64) -> Result<(), ContractError> {
        member.require_auth();
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        if storage::read_member(&env, pool_id, &member).is_none() {
            return Err(ContractError::NotMember);
        }

        let mut members = storage::read_members(&env, pool_id);
        let mut found_index = None;
        for index in 0..members.len() {
            if members.get(index) == Some(member.clone()) {
                found_index = Some(index);
                break;
            }
        }
        let index = found_index.ok_or(ContractError::StorageInvariantViolation)?;
        members.remove(index);
        pool.member_count = pool
            .member_count
            .checked_sub(1)
            .ok_or(ContractError::StorageInvariantViolation)?;

        storage::remove_member(&env, pool_id, &member);
        storage::write_members(&env, pool_id, &members);
        storage::write_pool(&env, &pool);

        MemberLeft {
            pool_id,
            member,
            left_at: env.ledger().timestamp(),
        }
        .publish(&env);
        Ok(())
    }

    pub fn start_pool(
        env: Env,
        creator: Address,
        pool_id: u64,
        recipient_order: Vec<Address>,
    ) -> Result<(), ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if creator != pool.creator {
            return Err(ContractError::CreatorOnly);
        }
        creator.require_auth();
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        if pool.member_count != pool.member_limit {
            return Err(ContractError::PoolNotFull);
        }
        let funded_guarantee = storage::read_guarantee_balance(&env, pool_id);
        if funded_guarantee < pool.guarantee_required {
            return Err(ContractError::InsufficientGuarantee);
        }
        let verifier_policy = storage::read_verifier_policy(&env, pool_id)
            .ok_or(ContractError::VerifierPolicyMissing)?;
        validate_verifier_policy(
            &env,
            &pool,
            &verifier_policy.verifiers,
            verifier_policy.approval_quorum,
        )?;
        validate_recipient_order(&env, pool_id, pool.member_count, &recipient_order)?;

        let started_at = env.ledger().timestamp();
        let deadline = started_at
            .checked_add(pool.round_duration_secs)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let first_recipient = recipient_order
            .get(0)
            .ok_or(ContractError::InvalidRecipientOrder)?;
        let round = RoundState {
            index: 0,
            recipient: first_recipient,
            started_at,
            deadline,
            grace_deadline: None,
            deposit_count: 0,
            pot_amount: 0,
            status: RoundStatus::Collecting,
        };

        pool.status = PoolStatus::Active;
        pool.started_at = Some(started_at);
        storage::write_recipient_order(&env, pool_id, &recipient_order);
        storage::write_round(&env, pool_id, &round);
        storage::write_round_pot(&env, pool_id, 0, 0);
        storage::write_round_top_up(&env, pool_id, 0, 0);
        storage::write_pool(&env, &pool);

        PoolStarted {
            pool_id,
            started_at,
            first_deadline: deadline,
            funded_guarantee,
        }
        .publish(&env);
        Ok(())
    }

    pub fn propose_purchase(
        env: Env,
        member: Address,
        pool_id: u64,
        seller: Address,
        document_digest: BytesN<32>,
    ) -> Result<(), ContractError> {
        member.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }

        let round_index = pool.current_round;
        let round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.status != RoundStatus::Collecting {
            return Err(ContractError::RoundAlreadyFinalized);
        }
        if round.recipient != member {
            return Err(ContractError::CurrentRecipientOnly);
        }
        if storage::read_purchase(&env, pool_id, round_index).is_some() {
            return Err(ContractError::PurchaseAlreadyProposed);
        }

        let (_, _, expected, available) = round_funding(&env, &pool, &round)?;
        if available != expected {
            return Err(ContractError::RoundNotReady);
        }

        let policy = storage::read_verifier_policy(&env, pool_id)
            .ok_or(ContractError::VerifierPolicyMissing)?;
        if seller == env.current_contract_address()
            || seller == pool.creator
            || seller == pool.sponsor
            || seller == pool.token
            || storage::read_member(&env, pool_id, &seller).is_some()
            || is_verifier(&policy.verifiers, &seller)
        {
            return Err(ContractError::InvalidSeller);
        }
        if document_digest == BytesN::from_array(&env, &[0; 32]) {
            return Err(ContractError::InvalidDocumentDigest);
        }

        let purchase = PurchaseState {
            seller: seller.clone(),
            document_digest: document_digest.clone(),
            proposed_by: member.clone(),
            proposed_at: env.ledger().timestamp(),
            approval_count: 0,
            approved: false,
        };
        storage::write_purchase(&env, pool_id, round_index, &purchase);

        PurchaseProposed {
            pool_id,
            round: round_index,
            recipient: member,
            seller,
            document_digest,
        }
        .publish(&env);
        Ok(())
    }

    pub fn approve_purchase(
        env: Env,
        verifier: Address,
        pool_id: u64,
        round: u32,
    ) -> Result<u32, ContractError> {
        verifier.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        if round != pool.current_round {
            return Err(ContractError::WrongRound);
        }
        let round_state =
            storage::read_round(&env, pool_id, round).ok_or(ContractError::WrongRound)?;
        if round_state.status != RoundStatus::Collecting {
            return Err(ContractError::RoundAlreadyFinalized);
        }

        let policy = storage::read_verifier_policy(&env, pool_id)
            .ok_or(ContractError::VerifierPolicyMissing)?;
        if !is_verifier(&policy.verifiers, &verifier) {
            return Err(ContractError::UnauthorizedVerifier);
        }
        if storage::has_verifier_approval(&env, pool_id, round, &verifier) {
            return Err(ContractError::AlreadyApproved);
        }
        let mut purchase =
            storage::read_purchase(&env, pool_id, round).ok_or(ContractError::PurchaseNotFound)?;
        let approval_count = purchase
            .approval_count
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        if approval_count > policy.verifiers.len() {
            return Err(ContractError::StorageInvariantViolation);
        }
        purchase.approval_count = approval_count;
        purchase.approved = approval_count >= policy.approval_quorum;

        storage::write_verifier_approval(&env, pool_id, round, &verifier);
        storage::write_purchase(&env, pool_id, round, &purchase);

        PurchaseApproved {
            pool_id,
            round,
            verifier,
            approval_count,
        }
        .publish(&env);
        Ok(approval_count)
    }

    pub fn execute_round(env: Env, pool_id: u64) -> Result<i128, ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }

        let round_index = pool.current_round;
        let mut round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.status != RoundStatus::Collecting {
            return Err(ContractError::RoundAlreadyFinalized);
        }

        let payout_amount = pool
            .contribution_amount
            .checked_mul(i128::from(pool.member_count))
            .ok_or(ContractError::ArithmeticOverflow)?;
        let round_pot = storage::read_round_pot(&env, pool_id, round_index);
        if round.pot_amount != round_pot || round_pot < 0 {
            return Err(ContractError::StorageInvariantViolation);
        }
        let sponsor_top_up = storage::read_round_top_up(&env, pool_id, round_index);
        if sponsor_top_up < 0 {
            return Err(ContractError::StorageInvariantViolation);
        }
        let available_for_round = round_pot
            .checked_add(sponsor_top_up)
            .ok_or(ContractError::ArithmeticOverflow)?;
        if available_for_round < payout_amount {
            return Err(ContractError::RoundNotReady);
        }
        if available_for_round != payout_amount {
            return Err(ContractError::StorageInvariantViolation);
        }

        let purchase = storage::read_purchase(&env, pool_id, round_index)
            .ok_or(ContractError::PurchaseNotFound)?;
        if purchase.proposed_by != round.recipient {
            return Err(ContractError::StorageInvariantViolation);
        }
        let policy = storage::read_verifier_policy(&env, pool_id)
            .ok_or(ContractError::VerifierPolicyMissing)?;
        let mut recorded_approvals = 0_u32;
        for verifier in policy.verifiers.iter() {
            if storage::has_verifier_approval(&env, pool_id, round_index, &verifier) {
                recorded_approvals = recorded_approvals
                    .checked_add(1)
                    .ok_or(ContractError::ArithmeticOverflow)?;
            }
        }
        let quorum_met = recorded_approvals >= policy.approval_quorum;
        if recorded_approvals != purchase.approval_count || purchase.approved != quorum_met {
            return Err(ContractError::StorageInvariantViolation);
        }
        if !quorum_met {
            return Err(ContractError::PurchaseNotApproved);
        }

        let members = storage::read_members(&env, pool_id);
        if members.len() != pool.member_count {
            return Err(ContractError::StorageInvariantViolation);
        }
        let stored_total_liability = storage::read_total_refund_liability(&env, pool_id);
        let mut observed_total_liability = 0_i128;
        let mut released_liability = 0_i128;
        let mut recipient_found = false;
        for member in members.iter() {
            let state = storage::read_member(&env, pool_id, &member)
                .ok_or(ContractError::StorageInvariantViolation)?;
            if !state.active {
                return Err(ContractError::StorageInvariantViolation);
            }
            let liability = storage::read_refund_liability(&env, pool_id, &member);
            if liability < 0 {
                return Err(ContractError::StorageInvariantViolation);
            }
            observed_total_liability = observed_total_liability
                .checked_add(liability)
                .ok_or(ContractError::ArithmeticOverflow)?;

            let is_recipient = member == round.recipient;
            if is_recipient {
                if state.allocation_received {
                    return Err(ContractError::StorageInvariantViolation);
                }
                recipient_found = true;
            }
            if state.allocation_received || is_recipient {
                released_liability = released_liability
                    .checked_add(liability)
                    .ok_or(ContractError::ArithmeticOverflow)?;
            }
        }
        if !recipient_found || observed_total_liability != stored_total_liability {
            return Err(ContractError::StorageInvariantViolation);
        }
        let remaining_liability = observed_total_liability
            .checked_sub(released_liability)
            .ok_or(ContractError::StorageInvariantViolation)?;

        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id);
        if assigned_balance < payout_amount {
            return Err(ContractError::InsufficientPoolBalance);
        }
        let assigned_after = assigned_balance
            .checked_sub(payout_amount)
            .ok_or(ContractError::ArithmeticOverflow)?;
        if assigned_after < remaining_liability {
            return Err(ContractError::RefundSolvencyViolation);
        }

        let next_index = round_index
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let completed_at = env.ledger().timestamp();
        let next_round = if next_index < pool.member_count {
            let order = storage::read_recipient_order(&env, pool_id)
                .ok_or(ContractError::StorageInvariantViolation)?;
            let recipient = order
                .get(next_index)
                .ok_or(ContractError::StorageInvariantViolation)?;
            let deadline = completed_at
                .checked_add(pool.round_duration_secs)
                .ok_or(ContractError::ArithmeticOverflow)?;
            Some(RoundState {
                index: next_index,
                recipient,
                started_at: completed_at,
                deadline,
                grace_deadline: None,
                deposit_count: 0,
                pot_amount: 0,
                status: RoundStatus::Collecting,
            })
        } else {
            None
        };

        for member in members.iter() {
            let mut state = storage::read_member(&env, pool_id, &member)
                .ok_or(ContractError::StorageInvariantViolation)?;
            if state.allocation_received || member == round.recipient {
                storage::write_refund_liability(&env, pool_id, &member, 0);
            }
            if member == round.recipient {
                state.allocation_received = true;
                storage::write_member(&env, pool_id, &member, &state);
            }
        }
        storage::write_total_refund_liability(&env, pool_id, remaining_liability);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_after);
        round.status = RoundStatus::Settled;
        storage::write_round(&env, pool_id, &round);

        if let Some(state) = next_round {
            pool.current_round = next_index;
            storage::write_round(&env, pool_id, &state);
            storage::write_round_pot(&env, pool_id, next_index, 0);
            storage::write_round_top_up(&env, pool_id, next_index, 0);
        } else {
            pool.status = PoolStatus::Completed;
        }
        storage::write_pool(&env, &pool);

        token::TokenClient::new(&env, &pool.token).transfer(
            &env.current_contract_address(),
            &purchase.seller,
            &payout_amount,
        );

        RoundPaid {
            pool_id,
            round: round_index,
            recipient: round.recipient,
            seller: purchase.seller,
            amount: payout_amount,
            sponsor_top_up,
        }
        .publish(&env);
        if pool.status == PoolStatus::Completed {
            PoolCompleted {
                pool_id,
                completed_at,
            }
            .publish(&env);
        }

        Ok(payout_amount)
    }

    pub fn mark_overdue(env: Env, pool_id: u64) -> Result<u64, ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round_index = pool.current_round;
        let mut round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.status != RoundStatus::Collecting {
            return Err(ContractError::RoundAlreadyFinalized);
        }
        if env.ledger().timestamp() < round.deadline {
            return Err(ContractError::DeadlineNotReached);
        }
        let (_, _, expected, available) = round_funding(&env, &pool, &round)?;
        if available == expected {
            return Err(ContractError::RoundAlreadyReady);
        }
        if round.grace_deadline.is_some() {
            return Err(ContractError::StorageInvariantViolation);
        }

        let grace_deadline = round
            .deadline
            .checked_add(pool.grace_duration_secs)
            .ok_or(ContractError::ArithmeticOverflow)?;
        round.grace_deadline = Some(grace_deadline);
        pool.status = PoolStatus::Grace;
        storage::write_round(&env, pool_id, &round);
        storage::write_pool(&env, &pool);

        RoundOverdue {
            pool_id,
            round: round_index,
            grace_deadline,
        }
        .publish(&env);
        Ok(grace_deadline)
    }

    pub fn cure_payment(env: Env, member: Address, pool_id: u64) -> Result<i128, ContractError> {
        member.require_auth();
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Grace {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round = storage::read_round(&env, pool_id, pool.current_round)
            .ok_or(ContractError::StorageInvariantViolation)?;
        let grace_deadline = round
            .grace_deadline
            .ok_or(ContractError::StorageInvariantViolation)?;
        if env.ledger().timestamp() >= grace_deadline {
            return Err(ContractError::DeadlineReached);
        }

        let (round_index, amount, round_pot) = apply_member_payment(&env, &pool, &member)?;
        let updated_round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        let (_, _, expected, available) = round_funding(&env, &pool, &updated_round)?;
        if available == expected {
            pool.status = PoolStatus::Active;
            storage::write_pool(&env, &pool);
        }

        PaymentCured {
            pool_id,
            round: round_index,
            member,
            amount,
        }
        .publish(&env);
        Ok(round_pot)
    }

    pub fn top_up(env: Env, pool_id: u64, amount: i128) -> Result<i128, ContractError> {
        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Grace {
            return Err(ContractError::InvalidPoolStatus);
        }
        pool.sponsor.require_auth();
        let round_index = pool.current_round;
        let round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        let grace_deadline = round
            .grace_deadline
            .ok_or(ContractError::StorageInvariantViolation)?;
        if env.ledger().timestamp() >= grace_deadline {
            return Err(ContractError::DeadlineReached);
        }
        let (_, current_top_up, expected, available) = round_funding(&env, &pool, &round)?;
        if available == expected {
            return Err(ContractError::RoundAlreadyReady);
        }
        let shortfall = expected
            .checked_sub(available)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if amount != shortfall {
            return Err(ContractError::InvalidTopUpAmount);
        }
        let total_top_up = current_top_up
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id)
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;

        pool.status = PoolStatus::Active;
        storage::write_round_top_up(&env, pool_id, round_index, total_top_up);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_balance);
        storage::write_pool(&env, &pool);
        token::TokenClient::new(&env, &pool.token).transfer(
            &pool.sponsor,
            &env.current_contract_address(),
            &amount,
        );

        SponsorTopUp {
            pool_id,
            round: round_index,
            sponsor: pool.sponsor,
            amount,
        }
        .publish(&env);
        Ok(total_top_up)
    }

    pub fn pause_pool(env: Env, pool_id: u64) -> Result<(), ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Grace {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round_index = pool.current_round;
        let round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        let grace_deadline = round
            .grace_deadline
            .ok_or(ContractError::StorageInvariantViolation)?;
        if env.ledger().timestamp() < grace_deadline {
            return Err(ContractError::DeadlineNotReached);
        }
        let (_, _, expected, available) = round_funding(&env, &pool, &round)?;
        if available == expected {
            return Err(ContractError::RoundAlreadyReady);
        }

        pool.status = PoolStatus::Paused;
        storage::write_pool(&env, &pool);
        PoolPaused {
            pool_id,
            round: round_index,
        }
        .publish(&env);
        Ok(())
    }

    pub fn abort_pool(env: Env, pool_id: u64) -> Result<(), ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Paused {
            return Err(ContractError::AbortNotAllowed);
        }

        let round_index = pool.current_round;
        pool.status = PoolStatus::Aborted;
        storage::write_pool(&env, &pool);

        PoolAborted {
            pool_id,
            round: round_index,
            reason: AbortReason::SafetyRecovery,
        }
        .publish(&env);
        Ok(())
    }

    pub fn claim_refund(env: Env, member: Address, pool_id: u64) -> Result<i128, ContractError> {
        member.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Aborted {
            return Err(ContractError::RefundNotClaimable);
        }
        if storage::read_member(&env, pool_id, &member).is_none() {
            return Err(ContractError::NotMember);
        }
        if storage::has_refund_claimed(&env, pool_id, &member) {
            return Err(ContractError::RefundAlreadyClaimed);
        }
        let entitlement = storage::read_refund_liability(&env, pool_id, &member);
        if entitlement <= 0 {
            return Err(ContractError::RefundNotClaimable);
        }
        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id);
        let total_liability = storage::read_total_refund_liability(&env, pool_id);
        if assigned_balance < entitlement || total_liability < entitlement {
            return Err(ContractError::StorageInvariantViolation);
        }
        let assigned_after = assigned_balance
            .checked_sub(entitlement)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let total_after = total_liability
            .checked_sub(entitlement)
            .ok_or(ContractError::ArithmeticOverflow)?;

        storage::write_refund_claimed(&env, pool_id, &member);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_after);
        storage::write_total_refund_liability(&env, pool_id, total_after);

        token::TokenClient::new(&env, &pool.token).transfer(
            &env.current_contract_address(),
            &member,
            &entitlement,
        );

        RefundClaimed {
            pool_id,
            member,
            amount: entitlement,
        }
        .publish(&env);
        Ok(entitlement)
    }

    pub fn claim_sponsor_remainder(env: Env, pool_id: u64) -> Result<i128, ContractError> {
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Aborted {
            return Err(ContractError::SponsorRemainderNotClaimable);
        }
        pool.sponsor.require_auth();
        if storage::has_sponsor_remainder_claimed(&env, pool_id) {
            return Err(ContractError::SponsorRemainderNotClaimable);
        }
        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id);
        let total_liability = storage::read_total_refund_liability(&env, pool_id);
        if assigned_balance < total_liability {
            return Err(ContractError::StorageInvariantViolation);
        }
        let remainder = assigned_balance
            .checked_sub(total_liability)
            .ok_or(ContractError::ArithmeticOverflow)?;
        if remainder <= 0 {
            return Err(ContractError::SponsorRemainderNotClaimable);
        }

        storage::write_sponsor_remainder_claimed(&env, pool_id);
        storage::write_pool_assigned_balance(&env, pool_id, total_liability);

        token::TokenClient::new(&env, &pool.token).transfer(
            &env.current_contract_address(),
            &pool.sponsor,
            &remainder,
        );

        SponsorRemainderClaimed {
            pool_id,
            sponsor: pool.sponsor,
            amount: remainder,
        }
        .publish(&env);
        Ok(remainder)
    }

    pub fn version(env: Env) -> u32 {
        storage::extend_instance_ttl(&env);
        CONTRACT_VERSION
    }

    pub fn next_pool_id(env: Env) -> u64 {
        storage::read_next_pool_id(&env)
    }

    pub fn has_pool(env: Env, pool_id: u64) -> bool {
        storage::read_pool(&env, pool_id).is_some()
    }

    pub fn get_pool(env: Env, pool_id: u64) -> Result<Pool, ContractError> {
        storage::read_pool(&env, pool_id).ok_or(ContractError::PoolNotFound)
    }

    pub fn get_members(env: Env, pool_id: u64) -> Result<Vec<Address>, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_members(&env, pool_id))
    }

    pub fn get_member(
        env: Env,
        pool_id: u64,
        member: Address,
    ) -> Result<Option<MemberState>, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_member(&env, pool_id, &member))
    }

    pub fn get_recipient_order(
        env: Env,
        pool_id: u64,
    ) -> Result<Option<Vec<Address>>, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_recipient_order(&env, pool_id))
    }

    pub fn get_verifier_policy(
        env: Env,
        pool_id: u64,
    ) -> Result<Option<VerifierPolicy>, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_verifier_policy(&env, pool_id))
    }

    pub fn get_round(
        env: Env,
        pool_id: u64,
        round: u32,
    ) -> Result<Option<RoundState>, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_round(&env, pool_id, round))
    }

    pub fn has_deposit(env: Env, pool_id: u64, round: u32, member: Address) -> bool {
        storage::has_deposit(&env, pool_id, round, &member)
    }

    pub fn get_round_pot(env: Env, pool_id: u64, round: u32) -> i128 {
        storage::read_round_pot(&env, pool_id, round)
    }

    pub fn get_round_top_up(env: Env, pool_id: u64, round: u32) -> i128 {
        storage::read_round_top_up(&env, pool_id, round)
    }

    pub fn get_guarantee_balance(env: Env, pool_id: u64) -> Result<i128, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_guarantee_balance(&env, pool_id))
    }

    pub fn get_member_contribution_total(
        env: Env,
        pool_id: u64,
        member: Address,
    ) -> Result<i128, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_member_contribution_total(
            &env, pool_id, &member,
        ))
    }

    pub fn get_pool_assigned_balance(env: Env, pool_id: u64) -> Result<i128, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_pool_assigned_balance(&env, pool_id))
    }

    pub fn get_refund_liability(
        env: Env,
        pool_id: u64,
        member: Address,
    ) -> Result<i128, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_refund_liability(&env, pool_id, &member))
    }

    pub fn get_total_refund_liability(env: Env, pool_id: u64) -> Result<i128, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_total_refund_liability(&env, pool_id))
    }

    pub fn get_purchase(
        env: Env,
        pool_id: u64,
        round: u32,
    ) -> Result<Option<PurchaseState>, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_purchase(&env, pool_id, round))
    }

    pub fn has_verifier_approval(env: Env, pool_id: u64, round: u32, verifier: Address) -> bool {
        storage::has_verifier_approval(&env, pool_id, round, &verifier)
    }

    pub fn get_refund_claim(
        env: Env,
        pool_id: u64,
        member: Address,
    ) -> Result<bool, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::has_refund_claimed(&env, pool_id, &member))
    }

    pub fn get_sponsor_remainder_claimed(env: Env, pool_id: u64) -> Result<bool, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::has_sponsor_remainder_claimed(&env, pool_id))
    }
}

fn apply_member_payment(
    env: &Env,
    pool: &Pool,
    member: &Address,
) -> Result<(u32, i128, i128), ContractError> {
    let pool_id = pool.id;
    let round_index = pool.current_round;
    let mut member_state =
        storage::read_member(env, pool_id, member).ok_or(ContractError::NotMember)?;
    if !member_state.active {
        return Err(ContractError::NotMember);
    }
    let mut round = storage::read_round(env, pool_id, round_index)
        .ok_or(ContractError::StorageInvariantViolation)?;
    if round.status != RoundStatus::Collecting {
        return Err(ContractError::RoundAlreadyFinalized);
    }
    if storage::has_deposit(env, pool_id, round_index, member) {
        return Err(ContractError::AlreadyDeposited);
    }

    let (stored_round_pot, _, expected, available) = round_funding(env, pool, &round)?;
    if available == expected {
        return Err(ContractError::RoundAlreadyReady);
    }
    let amount = pool.contribution_amount;
    let round_pot = stored_round_pot
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let available_after = available
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;
    if available_after > expected {
        return Err(ContractError::RoundAlreadyReady);
    }
    let deposit_count = round
        .deposit_count
        .checked_add(1)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let contributions_paid = member_state
        .contributions_paid
        .checked_add(1)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let contribution_total = storage::read_member_contribution_total(env, pool_id, member)
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let refund_liability = storage::read_refund_liability(env, pool_id, member)
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let total_refund_liability = storage::read_total_refund_liability(env, pool_id)
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let assigned_balance = storage::read_pool_assigned_balance(env, pool_id)
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;

    round.deposit_count = deposit_count;
    round.pot_amount = round_pot;
    member_state.contributions_paid = contributions_paid;
    storage::write_deposit(env, pool_id, round_index, member);
    storage::write_round_pot(env, pool_id, round_index, round_pot);
    storage::write_round(env, pool_id, &round);
    storage::write_member(env, pool_id, member, &member_state);
    storage::write_member_contribution_total(env, pool_id, member, contribution_total);
    storage::write_refund_liability(env, pool_id, member, refund_liability);
    storage::write_total_refund_liability(env, pool_id, total_refund_liability);
    storage::write_pool_assigned_balance(env, pool_id, assigned_balance);

    token::TokenClient::new(env, &pool.token).transfer(
        member,
        &env.current_contract_address(),
        &amount,
    );
    Ok((round_index, amount, round_pot))
}

fn round_funding(
    env: &Env,
    pool: &Pool,
    round: &RoundState,
) -> Result<(i128, i128, i128, i128), ContractError> {
    let expected = pool
        .contribution_amount
        .checked_mul(i128::from(pool.member_count))
        .ok_or(ContractError::ArithmeticOverflow)?;
    let round_pot = storage::read_round_pot(env, pool.id, round.index);
    if round.pot_amount != round_pot || round_pot < 0 {
        return Err(ContractError::StorageInvariantViolation);
    }
    let top_up = storage::read_round_top_up(env, pool.id, round.index);
    if top_up < 0 {
        return Err(ContractError::StorageInvariantViolation);
    }
    let available = round_pot
        .checked_add(top_up)
        .ok_or(ContractError::ArithmeticOverflow)?;
    if available > expected {
        return Err(ContractError::StorageInvariantViolation);
    }
    Ok((round_pot, top_up, expected, available))
}

fn validate_pool_parameters(
    contribution_amount: i128,
    member_limit: u32,
    round_duration_secs: u64,
    grace_duration_secs: u64,
) -> Result<(), ContractError> {
    if contribution_amount <= 0 {
        return Err(ContractError::InvalidContributionAmount);
    }
    if !(MIN_MEMBERS..=MAX_MEMBERS).contains(&member_limit) {
        return Err(ContractError::InvalidMemberLimit);
    }
    if round_duration_secs == 0 {
        return Err(ContractError::InvalidRoundDuration);
    }
    if grace_duration_secs == 0 {
        return Err(ContractError::InvalidGraceDuration);
    }
    Ok(())
}

fn calculate_required_guarantee(
    member_limit: u32,
    contribution_amount: i128,
) -> Result<i128, ContractError> {
    let members = i128::from(member_limit);
    let peak_factor = members
        .checked_mul(members)
        .ok_or(ContractError::ArithmeticOverflow)?
        / 4;
    peak_factor
        .checked_mul(contribution_amount)
        .ok_or(ContractError::ArithmeticOverflow)
}

fn validate_recipient_order(
    env: &Env,
    pool_id: u64,
    member_count: u32,
    recipient_order: &Vec<Address>,
) -> Result<(), ContractError> {
    if recipient_order.len() != member_count {
        return Err(ContractError::InvalidRecipientOrder);
    }

    for index in 0..recipient_order.len() {
        let candidate = recipient_order
            .get(index)
            .ok_or(ContractError::InvalidRecipientOrder)?;
        let state = storage::read_member(env, pool_id, &candidate)
            .ok_or(ContractError::InvalidRecipientOrder)?;
        if !state.active {
            return Err(ContractError::InvalidRecipientOrder);
        }
        for previous in 0..index {
            if recipient_order.get(previous) == Some(candidate.clone()) {
                return Err(ContractError::DuplicateRecipient);
            }
        }
    }
    Ok(())
}

fn validate_verifier_policy(
    env: &Env,
    pool: &Pool,
    verifiers: &Vec<Address>,
    approval_quorum: u32,
) -> Result<(), ContractError> {
    if !(MIN_VERIFIERS..=MAX_VERIFIERS).contains(&verifiers.len()) {
        return Err(ContractError::InvalidVerifierSet);
    }
    if approval_quorum < MIN_VERIFIERS || approval_quorum > verifiers.len() {
        return Err(ContractError::InvalidApprovalQuorum);
    }

    for index in 0..verifiers.len() {
        let candidate = verifiers
            .get(index)
            .ok_or(ContractError::InvalidVerifierSet)?;
        if candidate == pool.creator
            || candidate == pool.sponsor
            || candidate == pool.token
            || candidate == env.current_contract_address()
            || storage::read_member(env, pool.id, &candidate).is_some()
        {
            return Err(ContractError::VerifierCannotBeParticipant);
        }
        for previous in 0..index {
            if verifiers.get(previous) == Some(candidate.clone()) {
                return Err(ContractError::DuplicateVerifier);
            }
        }
    }
    Ok(())
}

fn is_verifier(verifiers: &Vec<Address>, candidate: &Address) -> bool {
    for index in 0..verifiers.len() {
        if verifiers.get(index) == Some(candidate.clone()) {
            return true;
        }
    }
    false
}

fn get_pool_or_error(env: &Env, pool_id: u64) -> Result<Pool, ContractError> {
    storage::read_pool(env, pool_id).ok_or(ContractError::PoolNotFound)
}

fn ensure_pool_exists(env: &Env, pool_id: u64) -> Result<(), ContractError> {
    if storage::read_pool(env, pool_id).is_none() {
        return Err(ContractError::PoolNotFound);
    }
    Ok(())
}

#[cfg(test)]
mod test;
