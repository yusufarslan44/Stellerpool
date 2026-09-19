#![no_std]

mod error;
mod events;
mod storage;
mod types;

pub use error::ContractError;
pub use events::*;
pub use types::{
    AbortReason, MemberState, MemberStatusView, Pool, PoolStatus, RoundPhase, RoundState,
};

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec};

pub const CONTRACT_VERSION: u32 = 8;
pub const MIN_MEMBERS: u32 = 2;
pub const MAX_MEMBERS: u32 = 12;
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
        round_duration: u64,
        grace_duration: u64,
        purchase_duration: u64,
        setup_deadline: u64,
        demo_seller: Address,
    ) -> Result<u64, ContractError> {
        creator.require_auth();
        let created_at = env.ledger().timestamp();
        validate_pool_parameters(
            contribution_amount,
            member_limit,
            round_duration,
            grace_duration,
            purchase_duration,
            setup_deadline,
            created_at,
        )?;
        if demo_seller == creator || demo_seller == sponsor {
            return Err(ContractError::SellerCannotBeParticipant);
        }

        let pool_id = storage::read_next_pool_id(&env);
        let next_pool_id = pool_id
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let required_guarantee = calculate_required_guarantee(member_limit, contribution_amount)?;
        let pool = Pool {
            id: pool_id,
            creator: creator.clone(),
            sponsor: sponsor.clone(),
            token: token.clone(),
            contribution_amount,
            member_limit,
            members: Vec::new(&env),
            recipient_order: Vec::new(&env),
            verifiers: Vec::new(&env),
            approval_threshold: 0,
            terms_version: 0,
            terms_approvals: Vec::new(&env),
            current_round: 0,
            status: PoolStatus::Filling,
            round_duration,
            grace_duration,
            purchase_duration,
            setup_deadline,
            demo_seller: demo_seller.clone(),
            required_guarantee,
            guarantee_deposited: 0,
            created_at,
            started_at: None,
        };

        storage::write_pool(&env, &pool);
        storage::write_pool_assigned_balance(&env, pool_id, 0);
        storage::write_total_refund_liability(&env, pool_id, 0);
        storage::write_next_pool_id(&env, next_pool_id);

        PoolCreated {
            pool_id,
            creator,
            sponsor,
            token,
            contribution_amount,
            required_guarantee,
            member_limit,
            round_duration,
            grace_duration,
            purchase_duration,
            setup_deadline,
            demo_seller,
        }
        .publish(&env);

        Ok(pool_id)
    }

    pub fn fund_guarantee(
        env: Env,
        pool_id: u64,
        sponsor: Address,
        amount: i128,
    ) -> Result<i128, ContractError> {
        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if sponsor != pool.sponsor {
            return Err(ContractError::SponsorOnly);
        }
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        sponsor.require_auth();

        let total_guarantee = pool
            .guarantee_deposited
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id)
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;

        token::TokenClient::new(&env, &pool.token).transfer(
            &sponsor,
            &env.current_contract_address(),
            &amount,
        );
        pool.guarantee_deposited = total_guarantee;
        storage::write_pool(&env, &pool);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_balance);

        GuaranteeFunded {
            pool_id,
            sponsor,
            amount,
            total_guarantee,
        }
        .publish(&env);

        Ok(total_guarantee)
    }

    pub fn join_pool(env: Env, member: Address, pool_id: u64) -> Result<(), ContractError> {
        member.require_auth();
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        if pool.members.len() >= pool.member_limit {
            return Err(ContractError::PoolFull);
        }
        if storage::read_member(&env, pool_id, &member).is_some() {
            return Err(ContractError::AlreadyMember);
        }
        if member == pool.demo_seller {
            return Err(ContractError::SellerCannotBeParticipant);
        }
        if is_address_in(&pool.verifiers, &member) {
            return Err(ContractError::VerifierCannotBeParticipant);
        }

        let joined_at = env.ledger().timestamp();
        let state = MemberState {
            joined_at,
            active: true,
            received: false,
            contributions_paid: 0,
        };
        pool.members.push_back(member.clone());
        storage::write_member(&env, pool_id, &member, &state);
        storage::write_pool(&env, &pool);

        MemberJoined {
            pool_id,
            member,
            joined_at,
        }
        .publish(&env);
        Ok(())
    }

    pub fn propose_terms(
        env: Env,
        creator: Address,
        pool_id: u64,
        recipient_order: Vec<Address>,
        verifiers: Vec<Address>,
    ) -> Result<u32, ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if creator != pool.creator {
            return Err(ContractError::CreatorOnly);
        }
        creator.require_auth();
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        validate_recipient_order(&pool, &recipient_order)?;
        validate_verifier_policy(&env, &pool, &verifiers)?;
        let approval_threshold = compute_quorum(verifiers.len());

        let version = pool
            .terms_version
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        pool.recipient_order = recipient_order;
        let verifier_count = verifiers.len();
        pool.verifiers = verifiers;
        pool.approval_threshold = approval_threshold;
        pool.terms_version = version;
        pool.terms_approvals = Vec::new(&env);
        storage::write_pool(&env, &pool);

        TermsProposed {
            pool_id,
            creator,
            version,
            verifier_count,
            approval_threshold,
        }
        .publish(&env);
        Ok(version)
    }

    pub fn approve_terms(
        env: Env,
        approver: Address,
        pool_id: u64,
        version: u32,
    ) -> Result<u32, ContractError> {
        approver.require_auth();
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        if pool.terms_version == 0 {
            return Err(ContractError::TermsNotProposed);
        }
        if version != pool.terms_version {
            return Err(ContractError::TermsVersionMismatch);
        }
        let is_member = storage::read_member(&env, pool_id, &approver).is_some();
        if !is_member && approver != pool.sponsor {
            return Err(ContractError::NotApprover);
        }
        if storage::has_terms_approval(&env, pool_id, version, &approver) {
            return Err(ContractError::AlreadyApprovedTerms);
        }

        storage::write_terms_approval(&env, pool_id, version, &approver);
        pool.terms_approvals.push_back(approver.clone());
        let approval_count = pool.terms_approvals.len();
        storage::write_pool(&env, &pool);

        TermsApproved {
            pool_id,
            approver,
            version,
        }
        .publish(&env);
        Ok(approval_count)
    }

    pub fn start_pool(env: Env, pool_id: u64) -> Result<(), ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        let now = env.ledger().timestamp();
        if now >= pool.setup_deadline {
            return Err(ContractError::SetupDeadlineReached);
        }
        if pool.members.len() != pool.member_limit {
            return Err(ContractError::PoolNotFull);
        }
        if pool.terms_version == 0 {
            return Err(ContractError::TermsNotProposed);
        }
        if pool.recipient_order.len() != pool.member_limit {
            return Err(ContractError::InvalidRecipientOrder);
        }
        validate_recipient_order(&pool, &pool.recipient_order)?;
        if !(MIN_VERIFIERS..=MAX_VERIFIERS).contains(&pool.verifiers.len()) {
            return Err(ContractError::InvalidVerifierSet);
        }
        for member in pool.members.iter() {
            if !storage::has_terms_approval(&env, pool_id, pool.terms_version, &member) {
                return Err(ContractError::TermsNotFullyApproved);
            }
        }
        if !storage::has_terms_approval(&env, pool_id, pool.terms_version, &pool.sponsor) {
            return Err(ContractError::TermsNotFullyApproved);
        }
        if pool.guarantee_deposited < pool.required_guarantee {
            return Err(ContractError::InsufficientGuarantee);
        }

        let started_at = now;
        let collect_deadline = started_at
            .checked_add(pool.round_duration)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let first_recipient = pool
            .recipient_order
            .get(0)
            .ok_or(ContractError::InvalidRecipientOrder)?;
        let round = RoundState {
            round: 1,
            phase: RoundPhase::Collecting,
            recipient: first_recipient,
            started_at,
            collect_deadline,
            grace_deadline: None,
            purchase_deadline: None,
            paid: Vec::new(&env),
            sponsor_advanced: Vec::new(&env),
            pot: 0,
            seller: None,
            asset: None,
            amount: None,
            doc_hash: None,
            purchase_version: 0,
            approvals: Vec::new(&env),
        };

        pool.status = PoolStatus::Active;
        pool.started_at = Some(started_at);
        pool.current_round = 1;
        let funded_guarantee = pool.guarantee_deposited;
        storage::write_round(&env, pool_id, &round);
        storage::write_pool(&env, &pool);

        PoolStarted {
            pool_id,
            started_at,
            first_deadline: collect_deadline,
            funded_guarantee,
        }
        .publish(&env);
        Ok(())
    }

    pub fn cancel_unstarted_pool(env: Env, pool_id: u64) -> Result<i128, ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        let now = env.ledger().timestamp();
        if now < pool.setup_deadline {
            return Err(ContractError::SetupDeadlineNotReached);
        }

        let refund_amount = pool.guarantee_deposited;
        pool.status = PoolStatus::Aborted;
        pool.guarantee_deposited = 0;
        let sponsor = pool.sponsor.clone();
        let token_address = pool.token.clone();
        storage::write_pool_assigned_balance(&env, pool_id, 0);
        storage::write_pool(&env, &pool);

        if refund_amount > 0 {
            token::TokenClient::new(&env, &token_address).transfer(
                &env.current_contract_address(),
                &sponsor,
                &refund_amount,
            );
        }

        PoolCancelled {
            pool_id,
            refunded_guarantee: refund_amount,
        }
        .publish(&env);
        Ok(refund_amount)
    }

    pub fn deposit(env: Env, member: Address, pool_id: u64) -> Result<i128, ContractError> {
        member.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round = storage::read_round(&env, pool_id, pool.current_round)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.phase != RoundPhase::Collecting {
            return Err(ContractError::InvalidPoolStatus);
        }
        let (round_index, amount, pot) = apply_member_payment(&env, &pool, &member)?;

        ContributionDeposited {
            pool_id,
            round: round_index,
            member,
            amount,
        }
        .publish(&env);
        Ok(pot)
    }

    pub fn cure_payment(env: Env, member: Address, pool_id: u64) -> Result<i128, ContractError> {
        member.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round = storage::read_round(&env, pool_id, pool.current_round)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.phase != RoundPhase::Grace {
            return Err(ContractError::InvalidPoolStatus);
        }
        let (round_index, amount, pot) = apply_member_payment(&env, &pool, &member)?;

        PaymentCured {
            pool_id,
            round: round_index,
            member,
            amount,
        }
        .publish(&env);
        Ok(pot)
    }

    pub fn top_up(
        env: Env,
        pool_id: u64,
        sponsor: Address,
        member: Address,
    ) -> Result<i128, ContractError> {
        let pool = get_pool_or_error(&env, pool_id)?;
        if sponsor != pool.sponsor {
            return Err(ContractError::SponsorOnly);
        }
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        sponsor.require_auth();
        let round_index = pool.current_round;
        let mut round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        let now = env.ledger().timestamp();
        match round.phase {
            RoundPhase::Collecting => {
                if now >= round.collect_deadline {
                    return Err(ContractError::DeadlineReached);
                }
            }
            RoundPhase::Grace => {
                let grace_deadline = round
                    .grace_deadline
                    .ok_or(ContractError::StorageInvariantViolation)?;
                if now >= grace_deadline {
                    return Err(ContractError::DeadlineReached);
                }
            }
            _ => return Err(ContractError::RoundAlreadyFinalized),
        }
        if member == round.recipient {
            return Err(ContractError::TopUpNotAllowedForRecipient);
        }
        if storage::read_member(&env, pool_id, &member).is_none() {
            return Err(ContractError::NotMember);
        }
        if storage::has_deposit(&env, pool_id, round_index, &member)
            || storage::has_advance_covered(&env, pool_id, round_index, &member)
        {
            return Err(ContractError::AlreadyDeposited);
        }

        let amount = pool.contribution_amount;
        let expected = expected_pot(&pool)?;
        let pot_after = round
            .pot
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;
        if pot_after > expected {
            return Err(ContractError::StorageInvariantViolation);
        }

        let advance_total = storage::read_sponsor_advance(&env, pool_id, &member)
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id)
            .checked_add(amount)
            .ok_or(ContractError::ArithmeticOverflow)?;

        round.pot = pot_after;
        round.sponsor_advanced.push_back(member.clone());
        storage::write_advance_covered(&env, pool_id, round_index, &member);
        maybe_transition_round(&env, &pool, &mut round)?;

        storage::write_sponsor_advance(&env, pool_id, &member, advance_total);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_balance);
        storage::write_round(&env, pool_id, &round);

        token::TokenClient::new(&env, &pool.token).transfer(
            &sponsor,
            &env.current_contract_address(),
            &amount,
        );

        SponsorAdvanced {
            pool_id,
            round: round_index,
            member,
            sponsor,
            amount,
            total_advance: advance_total,
        }
        .publish(&env);
        Ok(advance_total)
    }

    pub fn repay_advance(env: Env, member: Address, pool_id: u64) -> Result<i128, ContractError> {
        member.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        let owed = storage::read_sponsor_advance(&env, pool_id, &member);
        if owed <= 0 {
            return Err(ContractError::NoOutstandingAdvance);
        }

        storage::write_sponsor_advance(&env, pool_id, &member, 0);

        if pool.status == PoolStatus::Active {
            let round_index = pool.current_round;
            if let Some(mut round) = storage::read_round(&env, pool_id, round_index) {
                if round.recipient == member
                    && (round.phase == RoundPhase::Collecting || round.phase == RoundPhase::Grace)
                {
                    maybe_transition_round(&env, &pool, &mut round)?;
                    storage::write_round(&env, pool_id, &round);
                }
            }
        }

        token::TokenClient::new(&env, &pool.token).transfer(&member, &pool.sponsor, &owed);

        AdvanceRepaid {
            pool_id,
            member,
            amount: owed,
        }
        .publish(&env);
        Ok(owed)
    }

    pub fn propose_purchase(
        env: Env,
        member: Address,
        pool_id: u64,
        seller: Address,
        asset: Address,
        amount: i128,
        doc_hash: BytesN<32>,
    ) -> Result<u32, ContractError> {
        member.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round_index = pool.current_round;
        let mut round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.phase != RoundPhase::AwaitingPurchase {
            return Err(ContractError::RoundNotReady);
        }
        if round.recipient != member {
            return Err(ContractError::CurrentRecipientOnly);
        }
        if seller != pool.demo_seller {
            return Err(ContractError::InvalidSeller);
        }
        if asset != pool.token {
            return Err(ContractError::InvalidAsset);
        }
        let expected_amount = expected_pot(&pool)?;
        if amount != expected_amount {
            return Err(ContractError::InvalidPurchaseAmount);
        }
        if doc_hash == BytesN::from_array(&env, &[0; 32]) {
            return Err(ContractError::InvalidDocumentDigest);
        }

        let version = round
            .purchase_version
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        round.seller = Some(seller.clone());
        round.asset = Some(asset.clone());
        round.amount = Some(amount);
        round.doc_hash = Some(doc_hash.clone());
        round.purchase_version = version;
        round.approvals = Vec::new(&env);
        storage::write_round(&env, pool_id, &round);

        PurchaseProposed {
            pool_id,
            round: round_index,
            recipient: member,
            seller,
            asset,
            amount,
            doc_hash,
            version,
        }
        .publish(&env);
        Ok(version)
    }

    pub fn approve_purchase(
        env: Env,
        verifier: Address,
        pool_id: u64,
        round: u32,
        proposal_version: u32,
    ) -> Result<u32, ContractError> {
        verifier.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        if round != pool.current_round {
            return Err(ContractError::WrongRound);
        }
        let mut round_state =
            storage::read_round(&env, pool_id, round).ok_or(ContractError::WrongRound)?;
        if round_state.phase != RoundPhase::AwaitingPurchase {
            return Err(ContractError::RoundNotReady);
        }
        if round_state.purchase_version == 0 {
            return Err(ContractError::PurchaseNotFound);
        }
        if proposal_version != round_state.purchase_version {
            return Err(ContractError::PurchaseVersionMismatch);
        }
        if !is_address_in(&pool.verifiers, &verifier) {
            return Err(ContractError::UnauthorizedVerifier);
        }
        if storage::has_purchase_approval(&env, pool_id, round, proposal_version, &verifier) {
            return Err(ContractError::AlreadyApproved);
        }

        storage::write_purchase_approval(&env, pool_id, round, proposal_version, &verifier);
        round_state.approvals.push_back(verifier.clone());
        let approval_count = round_state.approvals.len();
        storage::write_round(&env, pool_id, &round_state);

        PurchaseApproved {
            pool_id,
            round,
            verifier,
            version: proposal_version,
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
        if round.phase != RoundPhase::AwaitingPurchase {
            return Err(ContractError::RoundNotReady);
        }
        if !storage::has_deposit(&env, pool_id, round_index, &round.recipient) {
            return Err(ContractError::RecipientMustSelfPay);
        }
        if storage::read_sponsor_advance(&env, pool_id, &round.recipient) != 0 {
            return Err(ContractError::RecipientOwesAdvance);
        }

        let payout_amount = expected_pot(&pool)?;
        if round.pot != payout_amount {
            return Err(ContractError::StorageInvariantViolation);
        }

        let seller = round
            .seller
            .clone()
            .ok_or(ContractError::PurchaseNotFound)?;
        if seller != pool.demo_seller {
            return Err(ContractError::StorageInvariantViolation);
        }
        let purchase_amount = round.amount.ok_or(ContractError::PurchaseNotFound)?;
        if purchase_amount != payout_amount {
            return Err(ContractError::StorageInvariantViolation);
        }

        let mut recorded_approvals = 0_u32;
        for verifier in pool.verifiers.iter() {
            if storage::has_purchase_approval(
                &env,
                pool_id,
                round_index,
                round.purchase_version,
                &verifier,
            ) {
                recorded_approvals = recorded_approvals
                    .checked_add(1)
                    .ok_or(ContractError::ArithmeticOverflow)?;
            }
        }
        if recorded_approvals != round.approvals.len() {
            return Err(ContractError::StorageInvariantViolation);
        }
        if recorded_approvals < pool.approval_threshold {
            return Err(ContractError::PurchaseNotApproved);
        }

        let stored_total_liability = storage::read_total_refund_liability(&env, pool_id);
        let mut observed_total_liability = 0_i128;
        let mut released_liability = 0_i128;
        let mut recipient_found = false;
        for member in pool.members.iter() {
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
                if state.received {
                    return Err(ContractError::StorageInvariantViolation);
                }
                recipient_found = true;
            }
            if state.received || is_recipient {
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

        let completed_at = env.ledger().timestamp();
        let next_index = round_index
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let next_round = if next_index <= pool.member_limit {
            let recipient = pool
                .recipient_order
                .get(next_index - 1)
                .ok_or(ContractError::StorageInvariantViolation)?;
            let collect_deadline = completed_at
                .checked_add(pool.round_duration)
                .ok_or(ContractError::ArithmeticOverflow)?;
            Some(RoundState {
                round: next_index,
                phase: RoundPhase::Collecting,
                recipient,
                started_at: completed_at,
                collect_deadline,
                grace_deadline: None,
                purchase_deadline: None,
                paid: Vec::new(&env),
                sponsor_advanced: Vec::new(&env),
                pot: 0,
                seller: None,
                asset: None,
                amount: None,
                doc_hash: None,
                purchase_version: 0,
                approvals: Vec::new(&env),
            })
        } else {
            None
        };

        for member in pool.members.iter() {
            let mut state = storage::read_member(&env, pool_id, &member)
                .ok_or(ContractError::StorageInvariantViolation)?;
            if state.received || member == round.recipient {
                storage::write_refund_liability(&env, pool_id, &member, 0);
            }
            if member == round.recipient {
                state.received = true;
                storage::write_member(&env, pool_id, &member, &state);
            }
        }
        storage::write_total_refund_liability(&env, pool_id, remaining_liability);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_after);
        round.phase = RoundPhase::Settled;
        storage::write_round(&env, pool_id, &round);

        if let Some(state) = next_round {
            pool.current_round = next_index;
            storage::write_round(&env, pool_id, &state);
        } else {
            pool.status = PoolStatus::Completed;
        }
        storage::write_pool(&env, &pool);

        token::TokenClient::new(&env, &pool.token).transfer(
            &env.current_contract_address(),
            &seller,
            &payout_amount,
        );

        RoundPaid {
            pool_id,
            round: round_index,
            recipient: round.recipient.clone(),
            seller,
            amount: payout_amount,
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
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round_index = pool.current_round;
        let mut round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.phase != RoundPhase::Collecting {
            return Err(ContractError::RoundAlreadyFinalized);
        }
        let now = env.ledger().timestamp();
        if now < round.collect_deadline {
            return Err(ContractError::DeadlineNotReached);
        }

        let grace_deadline = round
            .collect_deadline
            .checked_add(pool.grace_duration)
            .ok_or(ContractError::ArithmeticOverflow)?;
        round.phase = RoundPhase::Grace;
        round.grace_deadline = Some(grace_deadline);
        storage::write_round(&env, pool_id, &round);

        RoundOverdue {
            pool_id,
            round: round_index,
            grace_deadline,
        }
        .publish(&env);
        Ok(grace_deadline)
    }

    pub fn abort_pool(env: Env, pool_id: u64) -> Result<(), ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        let round_index = pool.current_round;
        let round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        let now = env.ledger().timestamp();

        let reason = match round.phase {
            RoundPhase::Grace => {
                let grace_deadline = round
                    .grace_deadline
                    .ok_or(ContractError::StorageInvariantViolation)?;
                if now < grace_deadline {
                    return Err(ContractError::DeadlineNotReached);
                }
                AbortReason::SafetyRecovery
            }
            RoundPhase::AwaitingPurchase => {
                let purchase_deadline = round
                    .purchase_deadline
                    .ok_or(ContractError::StorageInvariantViolation)?;
                if now < purchase_deadline {
                    return Err(ContractError::DeadlineNotReached);
                }
                AbortReason::BlockedSettlement
            }
            _ => return Err(ContractError::AbortNotAllowed),
        };

        pool.status = PoolStatus::Aborted;
        storage::write_pool(&env, &pool);

        PoolAborted {
            pool_id,
            round: round_index,
            reason,
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

    pub fn claim_sponsor_remainder(
        env: Env,
        sponsor: Address,
        pool_id: u64,
    ) -> Result<i128, ContractError> {
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Aborted {
            return Err(ContractError::SponsorRemainderNotClaimable);
        }
        if sponsor != pool.sponsor {
            return Err(ContractError::SponsorOnly);
        }
        sponsor.require_auth();
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
            &sponsor,
            &remainder,
        );

        SponsorRemainderClaimed {
            pool_id,
            sponsor,
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

    pub fn get_round(env: Env, pool_id: u64, round: u32) -> Result<RoundState, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        storage::read_round(&env, pool_id, round).ok_or(ContractError::RoundNotFound)
    }

    pub fn get_member_status(
        env: Env,
        pool_id: u64,
        member: Address,
    ) -> Result<MemberStatusView, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        let refundable = storage::read_refund_liability(&env, pool_id, &member);
        let received = storage::read_member(&env, pool_id, &member)
            .map(|state| state.received)
            .unwrap_or(false);
        Ok(MemberStatusView {
            refundable,
            received,
        })
    }

    pub fn get_sponsor_advance(
        env: Env,
        pool_id: u64,
        member: Address,
    ) -> Result<i128, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::read_sponsor_advance(&env, pool_id, &member))
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
    let now = env.ledger().timestamp();
    match round.phase {
        RoundPhase::Collecting => {
            if now >= round.collect_deadline {
                return Err(ContractError::DeadlineReached);
            }
        }
        RoundPhase::Grace => {
            let grace_deadline = round
                .grace_deadline
                .ok_or(ContractError::StorageInvariantViolation)?;
            if now >= grace_deadline {
                return Err(ContractError::DeadlineReached);
            }
        }
        _ => return Err(ContractError::RoundAlreadyFinalized),
    }
    if storage::has_deposit(env, pool_id, round_index, member)
        || storage::has_advance_covered(env, pool_id, round_index, member)
    {
        return Err(ContractError::AlreadyDeposited);
    }

    let amount = pool.contribution_amount;
    let expected = expected_pot(pool)?;
    let pot_after = round
        .pot
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;
    if pot_after > expected {
        return Err(ContractError::StorageInvariantViolation);
    }

    member_state.contributions_paid = member_state
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

    round.pot = pot_after;
    round.paid.push_back(member.clone());
    storage::write_deposit(env, pool_id, round_index, member);
    maybe_transition_round(env, pool, &mut round)?;

    storage::write_member(env, pool_id, member, &member_state);
    storage::write_member_contribution_total(env, pool_id, member, contribution_total);
    storage::write_refund_liability(env, pool_id, member, refund_liability);
    storage::write_total_refund_liability(env, pool_id, total_refund_liability);
    storage::write_pool_assigned_balance(env, pool_id, assigned_balance);
    storage::write_round(env, pool_id, &round);

    token::TokenClient::new(env, &pool.token).transfer(
        member,
        &env.current_contract_address(),
        &amount,
    );
    Ok((round_index, amount, round.pot))
}

/// Promotes a round from Collecting/Grace to AwaitingPurchase once the pot is fully
/// funded and the current recipient has personally paid their own contribution and
/// closed any earlier sponsor advance. Sponsor top-ups can never satisfy this for the
/// recipient themselves (`top_up` rejects `member == round.recipient`), so a recorded
/// self-deposit here is always genuinely the recipient's own money.
fn maybe_transition_round(
    env: &Env,
    pool: &Pool,
    round: &mut RoundState,
) -> Result<(), ContractError> {
    if round.phase != RoundPhase::Collecting && round.phase != RoundPhase::Grace {
        return Ok(());
    }
    let expected = expected_pot(pool)?;
    if round.pot != expected {
        return Ok(());
    }
    if !storage::has_deposit(env, pool.id, round.round, &round.recipient) {
        return Ok(());
    }
    if storage::read_sponsor_advance(env, pool.id, &round.recipient) != 0 {
        return Ok(());
    }

    let now = env.ledger().timestamp();
    let purchase_deadline = now
        .checked_add(pool.purchase_duration)
        .ok_or(ContractError::ArithmeticOverflow)?;
    round.phase = RoundPhase::AwaitingPurchase;
    round.purchase_deadline = Some(purchase_deadline);

    RoundAwaitingPurchase {
        pool_id: pool.id,
        round: round.round,
        purchase_deadline,
    }
    .publish(env);
    Ok(())
}

fn expected_pot(pool: &Pool) -> Result<i128, ContractError> {
    pool.contribution_amount
        .checked_mul(i128::from(pool.member_limit))
        .ok_or(ContractError::ArithmeticOverflow)
}

fn validate_pool_parameters(
    contribution_amount: i128,
    member_limit: u32,
    round_duration: u64,
    grace_duration: u64,
    purchase_duration: u64,
    setup_deadline: u64,
    created_at: u64,
) -> Result<(), ContractError> {
    if contribution_amount <= 0 {
        return Err(ContractError::InvalidContributionAmount);
    }
    if !(MIN_MEMBERS..=MAX_MEMBERS).contains(&member_limit) {
        return Err(ContractError::InvalidMemberLimit);
    }
    if round_duration == 0 {
        return Err(ContractError::InvalidRoundDuration);
    }
    if grace_duration == 0 {
        return Err(ContractError::InvalidGraceDuration);
    }
    if purchase_duration == 0 {
        return Err(ContractError::InvalidPurchaseDuration);
    }
    if setup_deadline <= created_at {
        return Err(ContractError::InvalidSetupDeadline);
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

fn validate_recipient_order(pool: &Pool, order: &Vec<Address>) -> Result<(), ContractError> {
    if order.len() != pool.members.len() {
        return Err(ContractError::InvalidRecipientOrder);
    }
    for index in 0..order.len() {
        let candidate = order
            .get(index)
            .ok_or(ContractError::InvalidRecipientOrder)?;
        if !is_address_in(&pool.members, &candidate) {
            return Err(ContractError::InvalidRecipientOrder);
        }
        for previous in 0..index {
            if order.get(previous) == Some(candidate.clone()) {
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
) -> Result<(), ContractError> {
    if !(MIN_VERIFIERS..=MAX_VERIFIERS).contains(&verifiers.len()) {
        return Err(ContractError::InvalidVerifierSet);
    }
    for index in 0..verifiers.len() {
        let candidate = verifiers
            .get(index)
            .ok_or(ContractError::InvalidVerifierSet)?;
        if candidate == pool.creator
            || candidate == pool.sponsor
            || candidate == pool.token
            || candidate == pool.demo_seller
            || candidate == env.current_contract_address()
            || is_address_in(&pool.members, &candidate)
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

/// Demo quorum: at least 2/3 of verifiers, rounded up.
fn compute_quorum(verifier_count: u32) -> u32 {
    (verifier_count.saturating_mul(2).saturating_add(2)) / 3
}

fn is_address_in(list: &Vec<Address>, candidate: &Address) -> bool {
    for index in 0..list.len() {
        if list.get(index) == Some(candidate.clone()) {
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
