#![no_std]

mod error;
mod events;
mod storage;
mod types;

pub use error::ContractError;
pub use events::*;
pub use types::{
    AbortReason, MemberState, MemberStatusView, OrderMode, Pool, PoolStatus, RoundPhase, RoundState,
};

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec};

pub const CONTRACT_VERSION: u32 = 11;
pub const MIN_MEMBERS: u32 = 2;
pub const MAX_MEMBERS: u32 = 30;
pub const MIN_VERIFIERS: u32 = 2;
pub const MAX_VERIFIERS: u32 = 10;

#[contract]
pub struct RotatingPoolContract;

#[contractimpl]
impl RotatingPoolContract {
    pub fn create_pool(
        env: Env,
        creator: Address,
        token: Address,
        contribution_amount: i128,
        member_limit: u32,
        order_mode: OrderMode,
        down_payment: i128,
        round_duration: u64,
        grace_duration: u64,
        purchase_duration: u64,
        setup_deadline: u64,
        demo_seller: Address,
    ) -> Result<u64, ContractError> {
        creator.require_auth();
        if down_payment < 0 {
            return Err(ContractError::InvalidDownPayment);
        }
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
        if demo_seller == creator {
            return Err(ContractError::SellerCannotBeParticipant);
        }

        let pool_id = storage::read_next_pool_id(&env);
        let next_pool_id = pool_id
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let pool = Pool {
            id: pool_id,
            creator: creator.clone(),
            token: token.clone(),
            contribution_amount,
            member_limit,
            order_mode,
            down_payment,
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
            created_at,
            started_at: None,
        };

        storage::write_pool(&env, &pool);
        storage::write_pool_assigned_balance(&env, pool_id, 0);
        storage::write_next_pool_id(&env, next_pool_id);

        PoolCreated {
            pool_id,
            creator,
            token,
            contribution_amount,
            member_limit,
            order_mode,
            down_payment,
            round_duration,
            grace_duration,
            purchase_duration,
            setup_deadline,
            demo_seller,
        }
        .publish(&env);

        Ok(pool_id)
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
            received: false,
        };
        pool.members.push_back(member.clone());
        storage::write_member(&env, pool_id, &member, &state);
        storage::write_pool(&env, &pool);

        // Down payment (peşinat): escrowed at join. Storage first, external token call last.
        // It is spent on this member's own purchase, or refunded if the pool aborts / is cancelled
        // before they receive (see `execute_round` and `claim_refund`).
        if pool.down_payment > 0 {
            let assigned_after = storage::read_pool_assigned_balance(&env, pool_id)
                .checked_add(pool.down_payment)
                .ok_or(ContractError::ArithmeticOverflow)?;
            storage::write_pool_assigned_balance(&env, pool_id, assigned_after);
            token::TokenClient::new(&env, &pool.token).transfer(
                &member,
                &env.current_contract_address(),
                &pool.down_payment,
            );
        }

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
        if storage::read_member(&env, pool_id, &approver).is_none() {
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
        if pool.order_mode == OrderMode::Fixed && pool.recipient_order.len() != pool.member_limit {
            return Err(ContractError::InvalidRecipientOrder);
        }
        validate_recipient_order(&pool, &pool.recipient_order)?;
        if !(MIN_VERIFIERS..=MAX_VERIFIERS).contains(&pool.verifiers.len()) {
            return Err(ContractError::InvalidVerifierSet);
        }
        // `terms_approvals` only grows through `approve_terms`, which requires the approver to
        // be an existing member and rejects a repeat approval for the same version, so it is a
        // duplicate-free subset of `pool.members`. Once its length equals `member_limit` (and
        // `pool.members.len()` already does, checked above), it must equal the full member set
        // — no need to re-read a per-member approval flag from storage for every member.
        if pool.terms_approvals.len() != pool.member_limit {
            return Err(ContractError::TermsNotFullyApproved);
        }

        let started_at = now;
        let collect_deadline = started_at
            .checked_add(pool.round_duration)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let first_recipient = match pool.order_mode {
            OrderMode::Fixed => Some(
                pool.recipient_order
                    .get(0)
                    .ok_or(ContractError::InvalidRecipientOrder)?,
            ),
            OrderMode::Draw => None,
        };
        let round = RoundState {
            round: 1,
            phase: RoundPhase::Collecting,
            recipient: first_recipient,
            started_at,
            collect_deadline,
            grace_deadline: None,
            purchase_deadline: None,
            paid: Vec::new(&env),
            pot: 0,
            seller: None,
            doc_hash: None,
            purchase_version: 0,
            approvals: Vec::new(&env),
        };

        pool.status = PoolStatus::Active;
        pool.started_at = Some(started_at);
        pool.current_round = 1;
        storage::write_round(&env, pool_id, &round);
        storage::write_pool(&env, &pool);

        PoolStarted {
            pool_id,
            started_at,
            first_deadline: collect_deadline,
        }
        .publish(&env);
        Ok(())
    }

    pub fn cancel_unstarted_pool(env: Env, pool_id: u64) -> Result<(), ContractError> {
        let mut pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Filling {
            return Err(ContractError::InvalidPoolStatus);
        }
        let now = env.ledger().timestamp();
        if now < pool.setup_deadline {
            return Err(ContractError::SetupDeadlineNotReached);
        }

        pool.status = PoolStatus::Aborted;
        storage::write_pool(&env, &pool);

        PoolCancelled { pool_id }.publish(&env);
        Ok(())
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
        let recipient = round
            .recipient
            .clone()
            .ok_or(ContractError::StorageInvariantViolation)?;
        if recipient != member {
            return Err(ContractError::CurrentRecipientOnly);
        }
        if seller != pool.demo_seller {
            return Err(ContractError::InvalidSeller);
        }
        if asset != pool.token {
            return Err(ContractError::InvalidAsset);
        }
        let expected_amount = purchase_amount(&pool)?;
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

    /// Draw-mode only. Once every member has paid into the round (`AwaitingDraw`), anyone may
    /// call this to pick the round's recipient uniformly among members who have not received a
    /// payout yet. Randomness is hackathon-grade (Soroban's validator-influenced PRNG, see
    /// `docs/CONTRACT_HANDOFF.md`) — not suitable for real funds.
    pub fn draw_recipient(
        env: Env,
        caller: Address,
        pool_id: u64,
    ) -> Result<Address, ContractError> {
        caller.require_auth();
        let pool = get_pool_or_error(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(ContractError::InvalidPoolStatus);
        }
        if pool.order_mode != OrderMode::Draw {
            return Err(ContractError::NotDrawPool);
        }
        let round_index = pool.current_round;
        let mut round = storage::read_round(&env, pool_id, round_index)
            .ok_or(ContractError::StorageInvariantViolation)?;
        if round.phase != RoundPhase::AwaitingDraw {
            return Err(ContractError::RoundNotReady);
        }

        let mut candidates: Vec<Address> = Vec::new(&env);
        for member in pool.members.iter() {
            let state = storage::read_member(&env, pool_id, &member)
                .ok_or(ContractError::StorageInvariantViolation)?;
            if !state.received {
                candidates.push_back(member);
            }
        }
        if candidates.is_empty() {
            return Err(ContractError::StorageInvariantViolation);
        }
        let winner = if candidates.len() == 1 {
            candidates
                .get(0)
                .ok_or(ContractError::StorageInvariantViolation)?
        } else {
            let index = env.prng().gen_range::<u64>(0..candidates.len() as u64) as u32;
            candidates
                .get(index)
                .ok_or(ContractError::StorageInvariantViolation)?
        };

        round.recipient = Some(winner.clone());
        round.phase = RoundPhase::AwaitingPurchase;
        storage::write_round(&env, pool_id, &round);

        RecipientDrawn {
            pool_id,
            round: round_index,
            recipient: winner.clone(),
        }
        .publish(&env);
        Ok(winner)
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

        // The seller receives the round pot plus the recipient's own escrowed down payment.
        let payout_amount = purchase_amount(&pool)?;
        if round.pot != expected_pot(&pool)? {
            return Err(ContractError::StorageInvariantViolation);
        }

        let seller = round
            .seller
            .clone()
            .ok_or(ContractError::PurchaseNotFound)?;
        if seller != pool.demo_seller {
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

        // `round.pot == payout_amount` (checked above) already implies exactly `member_limit`
        // distinct deposits, since every deposit adds exactly `contribution_amount` and a
        // second deposit from the same member is rejected. `round.paid.len()` (already loaded,
        // no extra storage reads) confirms this cheaply even at 30 members instead of
        // re-reading a `has_deposit` flag per member from storage.
        if pool.members.len() != pool.member_limit || round.paid.len() != pool.member_limit {
            return Err(ContractError::StorageInvariantViolation);
        }
        let recipient = round
            .recipient
            .clone()
            .ok_or(ContractError::StorageInvariantViolation)?;

        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id);
        if assigned_balance < payout_amount {
            return Err(ContractError::InsufficientPoolBalance);
        }
        let assigned_after = assigned_balance
            .checked_sub(payout_amount)
            .ok_or(ContractError::ArithmeticOverflow)?;

        let completed_at = env.ledger().timestamp();
        let next_index = round_index
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?;
        let next_round = if next_index <= pool.member_limit {
            let next_recipient = match pool.order_mode {
                OrderMode::Fixed => Some(
                    pool.recipient_order
                        .get(next_index - 1)
                        .ok_or(ContractError::StorageInvariantViolation)?,
                ),
                OrderMode::Draw => None,
            };
            let collect_deadline = completed_at
                .checked_add(pool.round_duration)
                .ok_or(ContractError::ArithmeticOverflow)?;
            Some(RoundState {
                round: next_index,
                phase: RoundPhase::Collecting,
                recipient: next_recipient,
                started_at: completed_at,
                collect_deadline,
                grace_deadline: None,
                purchase_deadline: None,
                paid: Vec::new(&env),
                pot: 0,
                seller: None,
                doc_hash: None,
                purchase_version: 0,
                approvals: Vec::new(&env),
            })
        } else {
            None
        };

        // Refund entitlement is derived at claim time from `has_deposit(pool, current_round,
        // member)` rather than stored per member, so settling a round needs no per-member
        // reset loop here — that is what keeps this O(1) in storage writes at 30 members.
        let mut recipient_state = storage::read_member(&env, pool_id, &recipient)
            .ok_or(ContractError::StorageInvariantViolation)?;
        recipient_state.received = true;
        storage::write_member(&env, pool_id, &recipient, &recipient_state);

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
            recipient,
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
            RoundPhase::AwaitingDraw | RoundPhase::AwaitingPurchase => {
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
        let member_state =
            storage::read_member(&env, pool_id, &member).ok_or(ContractError::NotMember)?;
        if storage::has_refund_claimed(&env, pool_id, &member) {
            return Err(ContractError::RefundAlreadyClaimed);
        }
        // Refund liability is not stored per member; it is derived from whether `member` paid
        // into the pool's current (unfinished, since the pool is Aborted) round. A settled
        // round already cleared its `RoundPhase` to `Settled` and advanced `current_round`, so
        // only the round active at abort time is ever refundable — matching the accepted risk
        // that an earlier round's payout cannot be recovered once it has settled.
        // A down payment is refundable only while unspent, i.e. until the member has received.
        let entitlement = refundable_amount(
            &pool,
            storage::has_deposit(&env, pool_id, pool.current_round, &member),
            member_state.received,
        )?;
        if entitlement <= 0 {
            return Err(ContractError::RefundNotClaimable);
        }
        let assigned_balance = storage::read_pool_assigned_balance(&env, pool_id);
        if assigned_balance < entitlement {
            return Err(ContractError::StorageInvariantViolation);
        }
        let assigned_after = assigned_balance
            .checked_sub(entitlement)
            .ok_or(ContractError::ArithmeticOverflow)?;

        storage::write_refund_claimed(&env, pool_id, &member);
        storage::write_pool_assigned_balance(&env, pool_id, assigned_after);

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
        let pool = get_pool_or_error(&env, pool_id)?;
        let member_state = storage::read_member(&env, pool_id, &member);
        let received = member_state.as_ref().map(|state| state.received).unwrap_or(false);
        let refundable = if member_state.is_none() || storage::has_refund_claimed(&env, pool_id, &member) {
            0
        } else {
            refundable_amount(
                &pool,
                storage::has_deposit(&env, pool_id, pool.current_round, &member),
                received,
            )?
        };
        Ok(MemberStatusView {
            refundable,
            received,
        })
    }

    pub fn get_refund_claim(
        env: Env,
        pool_id: u64,
        member: Address,
    ) -> Result<bool, ContractError> {
        ensure_pool_exists(&env, pool_id)?;
        Ok(storage::has_refund_claimed(&env, pool_id, &member))
    }
}

fn apply_member_payment(
    env: &Env,
    pool: &Pool,
    member: &Address,
) -> Result<(u32, i128, i128), ContractError> {
    let pool_id = pool.id;
    let round_index = pool.current_round;
    if storage::read_member(env, pool_id, member).is_none() {
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
    if storage::has_deposit(env, pool_id, round_index, member) {
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
    let assigned_balance = storage::read_pool_assigned_balance(env, pool_id)
        .checked_add(amount)
        .ok_or(ContractError::ArithmeticOverflow)?;

    round.pot = pot_after;
    round.paid.push_back(member.clone());
    storage::write_deposit(env, pool_id, round_index, member);
    if round.pot == expected {
        let purchase_deadline = now
            .checked_add(pool.purchase_duration)
            .ok_or(ContractError::ArithmeticOverflow)?;
        round.purchase_deadline = Some(purchase_deadline);
        match pool.order_mode {
            OrderMode::Fixed => {
                round.phase = RoundPhase::AwaitingPurchase;
                RoundAwaitingPurchase {
                    pool_id,
                    round: round_index,
                    purchase_deadline,
                }
                .publish(env);
            }
            OrderMode::Draw => {
                round.phase = RoundPhase::AwaitingDraw;
                RoundAwaitingDraw {
                    pool_id,
                    round: round_index,
                    purchase_deadline,
                }
                .publish(env);
            }
        }
    }
    storage::write_round(env, pool_id, &round);
    storage::write_pool_assigned_balance(env, pool_id, assigned_balance);

    token::TokenClient::new(env, &pool.token).transfer(
        member,
        &env.current_contract_address(),
        &amount,
    );
    Ok((round_index, amount, round.pot))
}

/// What the seller receives when a round settles: the round pot plus the recipient's down payment.
fn purchase_amount(pool: &Pool) -> Result<i128, ContractError> {
    expected_pot(pool)?
        .checked_add(pool.down_payment)
        .ok_or(ContractError::ArithmeticOverflow)
}

/// Refund entitlement of one member in an aborted pool: this round's contribution (if paid) plus
/// their down payment (if not yet spent on their own purchase).
fn refundable_amount(
    pool: &Pool,
    paid_current_round: bool,
    received: bool,
) -> Result<i128, ContractError> {
    let round_part = if paid_current_round {
        pool.contribution_amount
    } else {
        0
    };
    let down_part = if received { 0 } else { pool.down_payment };
    round_part
        .checked_add(down_part)
        .ok_or(ContractError::ArithmeticOverflow)
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

/// `Fixed` requires a full, duplicate-free permutation of the current members. `Draw` requires
/// an empty order — the recipient is resolved per round by `draw_recipient` instead.
fn validate_recipient_order(pool: &Pool, order: &Vec<Address>) -> Result<(), ContractError> {
    if pool.order_mode == OrderMode::Draw {
        if order.len() != 0 {
            return Err(ContractError::InvalidRecipientOrder);
        }
        return Ok(());
    }
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
