# Implementation Log

## Phase 0 - Discovery, Skills, Architecture Plan

### Goal

Inspect the repository, read all Markdown/project context, review required Stellar Skills, and produce an implementation plan without writing contract/backend/frontend code.

### Changes Made

- Added the backend/Soroban implementation plan.
- Added the Stellar Skills usage record in the required format.
- Added this Phase 0 implementation log.

### Files Changed

- `docs/IMPLEMENTATION_PLAN.md`
- `docs/STELLAR_SKILLS_USED.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `rg --files`
- `Get-ChildItem -Recurse -File -Include *.md,README*,*.toml,package.json,.env.example,*.rs,*.ts,*.js`
- `Get-Content README.md`
- `Get-Content docs\plan.md`
- `Get-Content docs\stellar-skills.md`
- `Get-Content .env.example`
- `Get-ChildItem contracts -Recurse -Force`
- `Get-ChildItem backend -Recurse -Force`
- `Get-ChildItem scripts -Recurse -Force`
- `Get-ChildItem frontend -Recurse -Force`
- `git status --short`

### Tests Run

None. Phase 0 is documentation-only and the repository has no contract/backend test suite yet.

### Test Results

Not applicable.

### Decisions

- Keep the project on a single multi-pool contract unless later evidence requires a factory.
- Keep backend non-authoritative for all financial decisions.
- Treat provider-specific Anchor integration as blocked until a real provider/home domain and `/info` capabilities are known.
- Plan SAC/classic asset support through a token contract address rather than custom settlement token logic.
- Keep collateral and round deposits in separate storage/accounting paths.
- Default to no contract upgrade mechanism for the hackathon MVP.

### Risks / Open Questions

- Anchor provider and SEP support are unknown.
- Whether sandbox/test anchor local-payment flow satisfies judging criteria is unknown.
- SDK/protocol/Stellar CLI versions must be verified before Phase 1 dependency pinning.
- `abort_pool` may be too large for MVP if implemented after all core safety paths; blocked-pool behavior must still be explicit.

### Stellar Skills Used

- Stellar Smart Contracts
- Stellar Assets & SAC
- SEPs, CAPs & Ecosystem
- RPC & Horizon APIs
- Anchors community skill
- Soroban Common Mistakes

### Definition of Done

- Repository state documented: completed.
- Required Skills reviewed and recorded: completed.
- Implementation plan created: completed.
- Open questions listed: completed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 1 - Contract Workspace, Domain Model, Storage, Events, Errors, Test Skeleton

### Goal

Create a compile-ready Soroban workspace and the contract's non-financial foundation without implementing token transfers or lifecycle mutations.

### Changes Made

- Added a Rust workspace pinned to Protocol 28 and `soroban-sdk 28.0.0`.
- Added the `wasm32v1-none` Rust target configuration and release profiles with overflow checks.
- Added pool, member, round, lifecycle, and abort domain types.
- Added typed persistent/instance storage keys and TTL extension helpers.
- Kept collateral and round-pot keys separate in the storage schema.
- Added typed contract errors and candidate typed events.
- Added a read-only contract API for version, pool, membership, recipient order, round, deposit, pot, and collateral state.
- Added storage round-trip and contract registration tests.
- Retained the SDK-generated test snapshots as differential regression artifacts.
- Installed Rust 1.98.1, Visual C++ Build Tools, and Stellar CLI 28.0.0 on the local development machine to complete verification.
- Built the optimized contract WASM with the canonical Stellar CLI command.

### Files Changed

- `Cargo.toml`
- `Cargo.lock`
- `rust-toolchain.toml`
- `contracts/rotating_pool/Cargo.toml`
- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/types.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/events.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/STELLAR_SKILLS_USED.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `rustc --version`
- `cargo --version`
- `stellar --version`
- `winget install --id Rustlang.Rustup`
- `rustup component add rustfmt --toolchain 1.98.1`
- `rustup target add wasm32v1-none --toolchain 1.98.1`
- `winget install --id Microsoft.VisualStudio.2022.BuildTools ...`
- `winget install --id Stellar.StellarCLI`
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo build --workspace --target wasm32v1-none --release`
- `stellar contract build`
- `git status --short`

### Tests Run

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `stellar contract build`

### Test Results

- Formatting check passed.
- Unit tests passed: 3 passed, 0 failed.
- Canonical WASM build passed with Stellar CLI 28.0.0.
- Optimized WASM size: 9,769 bytes.
- WASM SHA-256 reported by Stellar CLI: `ab46b517c150c2915f9e9378abca3bea9c43ca2f73c13672777dd18f600d6be6`.
- Exported functions: 11, all read-only in Phase 1.
- A direct `cargo build --target wasm32v1-none` was intentionally rejected by `soroban-sdk`; the required `stellar contract build` path passed.

### Decisions

- Pin `soroban-sdk` exactly to `28.0.0`, matching the released Protocol 28 toolchain discovered during dependency resolution.
- Pin Rust to 1.98.1 for reproducible local builds.
- Use one multi-pool contract with typed keys instead of a factory.
- Use instance storage only for the next pool ID and persistent storage for pool/member/round financial state.
- Keep bounded member and recipient vectors per pool; the member cap will be enforced when creation logic is added.
- Stage write helpers now but expose no public financial mutation until Phase 2.
- Use the Stellar CLI canonical build path for deployable WASM.

### Risks / Open Questions

- Live Testnet protocol/RPC compatibility must be rechecked before Phase 8 deployment even though local tooling is on Protocol 28.
- TTL durations are initial policy values and need lifecycle tests before deployment.
- `AbortReason` and abort/refund keys are schema placeholders; Phase 6 must define strict economics before exposing recovery methods.
- Events are typed but not emitted until their corresponding mutation entrypoints exist.
- The real Anchor provider and settlement asset remain unknown.

### Stellar Skills Used

- Stellar Smart Contracts: workspace layout, `#![no_std]`, typed storage, TTL, errors, events, tests, and canonical WASM build.
- Soroban Common Mistakes: exact SDK pin, overflow checks, bounded-storage design, separated accounting keys, and no premature admin/upgrade path.

### Definition of Done

- Contract workspace compiles: completed.
- Protocol/toolchain versions are pinned: completed.
- Domain, storage, error, and event schemas exist: completed.
- Read API and unit-test scaffold exist: completed.
- Unit tests pass: completed.
- Canonical optimized WASM build passes: completed.
- No financial function moves funds: completed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 2 - Sponsor Model Foundation and Filling Lifecycle

### Goal

Adopt `docs/plan.md` as the controlling product/economic plan, replace the member-collateral model with a separate sponsor guarantee, and implement the safe Filling lifecycle.

### Changes Made

- Rewrote `docs/IMPLEMENTATION_PLAN.md` around the sponsor-guarantee, seller-verification, Grace/Paused, and abort/refund model in `docs/plan.md`.
- Removed member collateral fields, storage keys, read methods, and collateral events.
- Added sponsor, required guarantee, configurable grace duration, and `Grace`/`Paused` lifecycle states.
- Added checked `floor(N^2 / 4) * contribution_amount` guarantee calculation.
- Added bounded membership with a 2-member minimum and 20-member maximum.
- Implemented authenticated `create_pool`.
- Implemented sponsor-authenticated `fund_guarantee` with SEP-41/SAC token transfer and separate per-pool accounting.
- Implemented authenticated, no-collateral `join_pool` and Filling-only `leave_pool`.
- Implemented creator-authenticated `start_pool` with full-membership, minimum-guarantee, exact-permutation, no-duplicate, and deadline checks.
- Added/updated typed events for guarantee, purchase verification, overdue/recovery, refunds, and sponsor remainder.
- Bumped contract schema/API version from 1 to 2.
- Replaced Phase 1 tests with SAC-backed sponsor-guarantee and Filling-lifecycle tests.
- Regenerated SDK differential test snapshots.

### Files Changed

- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/types.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/events.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/STELLAR_SKILLS_USED.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `Get-Content docs/plan.md`
- `Get-Content docs/IMPLEMENTATION_PLAN.md`
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `stellar contract build`
- `git status --short`

### Tests Run

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `stellar contract build`

### Test Results

- Formatting check passed.
- Unit tests passed: 9 passed, 0 failed.
- Tests cover creator/sponsor/member auth, invalid parameters, guarantee overflow, N=4/C=10 guarantee 40, no-fund member join/leave, duplicate membership, capacity, pool-isolated guarantees, insufficient guarantee, invalid/duplicate recipient order, active-pool leave rejection, and successful start.
- Canonical Protocol 28 WASM build passed.
- Optimized WASM size: 18,379 bytes.
- WASM SHA-256 reported by Stellar CLI: `b710453ff8f617c77af95e2b6e9e6ca11e5d12f47b3898f4602622aa75590ea5`.
- Exported functions: 16.

### Decisions

- `docs/plan.md` now overrides older member-collateral assumptions.
- Members do not transfer funds when joining.
- Sponsor guarantee is a separate per-pool ledger value backed by an actual token transfer to the contract.
- Guarantee funding may occur in multiple transfers and may exceed the minimum; later refund/remainder logic determines what the sponsor can reclaim.
- Pool creation stores `grace_duration_secs` now so recovery timing cannot be introduced as an active-pool mutable parameter later.
- Membership loops are capped at 20 to bound recipient-order validation costs.
- One multi-pool contract remains the MVP architecture; token balance accounting must remain isolated by `pool_id`.

### Risks / Open Questions

- Abort, member refund, and sponsor remainder functions are not implemented yet. A never-started funded pool is therefore not production-ready until the later recovery phase is complete.
- Verifier selection and quorum remain open and must be immutable by pool start before purchase approval is implemented.
- Guarantee funding records per-pool balances, but full pool-assigned balance/refund-liability accounting arrives in Phase 3.
- Contract join validates a typed Stellar address but cannot guarantee every future classic-account trustline condition; readiness remains an off-chain preflight plus token-transfer enforcement concern.
- Sponsor legal identity, loss allocation, debt creditor, and regulatory status remain external product/legal decisions.

### Stellar Skills Used

- Stellar Smart Contracts: role auth, checked arithmetic, typed storage/events/errors, lifecycle validation, bounded loops, tests, and canonical build.
- Stellar Assets & SAC: `TokenClient` guarantee transfer and `register_stellar_asset_contract_v2` test setup.
- Soroban Common Mistakes: wrong-auth rejection, no global-balance accounting, overflow tests, duplicate-state prevention, and pool-scoped keys.

### Definition of Done

- `docs/plan.md` reflected in the implementation plan: completed.
- Member collateral removed: completed.
- Sponsor guarantee formula and funding implemented: completed.
- Create/join/leave/start lifecycle implemented: completed.
- Joining moves no member funds: verified.
- Exact member order and guarantee start gates: verified.
- Unit tests and canonical WASM build pass: completed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 3 - Contribution and Liability Accounting

### Goal

Implement authenticated member contributions with atomic, pool-scoped accounting and establish the preliminary refund-liability ledger required for later solvent seller payouts and abort claims.

### Changes Made

- Bumped the contract API/schema version from 2 to 3.
- Added authenticated `deposit(member, pool_id)` for Active pools and active members.
- Enforced one contribution per member per round and the pool's exact configured contribution amount.
- Added member contribution totals, per-member refund liabilities, total pool refund liability, and pool-assigned balance storage helpers.
- Made sponsor guarantee funding increase the pool-assigned balance as well as the guarantee ledger.
- Updated round pot, round deposit count, and member contribution count atomically after a successful SEP-41 transfer.
- Added read APIs for member totals, assigned balance, per-member liabilities, and total liabilities.
- Added tests for deposit accounting, duplicate prevention, state/auth/membership checks, failed-transfer rollback, and same-token multi-pool isolation.
- Regenerated SDK differential test snapshots and built the optimized WASM.

### Files Changed

- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/STELLAR_SKILLS_USED.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `cargo fmt --all -- --check`
- `cargo fmt --all`
- `cargo test --workspace`
- `stellar contract build`
- `stellar contract info interface --wasm target/wasm32v1-none/release/rotating_pool.wasm`

### Tests Run

- Rust formatting check.
- Full workspace unit tests with Soroban/SAC test utilities.
- Canonical Stellar CLI optimized WASM build.
- Generated contract interface inspection.

### Test Results

- Unit tests passed: 13 passed, 0 failed.
- Failed token transfers were verified to leave deposit markers, pots, counters, contribution totals, liabilities, and assigned balance unchanged.
- Two pools sharing one SAC were verified to retain independent assigned balances, pots, and liabilities.
- Canonical Protocol 28 WASM build passed.
- Optimized WASM size: 21,433 bytes.
- WASM SHA-256: `fb71d28c8dae1841dc42d7551e2e29ed152b379a26b52d9530f79cc2cf3116b6`.
- Exported functions: 21.

### Decisions

- `deposit` accepts no amount argument; it always transfers the immutable pool contribution amount.
- Deposits are accepted only while the pool is Active and the current round is Collecting. Grace-period cure remains a separate Phase 6 action.
- Every successful deposit is initially recorded as a member refund liability. Phase 5 payout logic will adjust liabilities only when allocation delivery and solvency rules permit it.
- `PoolAssignedBalance` tracks tokens attributed to a specific pool. The contract's aggregate token balance is not financial truth for any one pool.
- All arithmetic is checked before the external token call; state writes occur only after the transfer succeeds.

### Risks / Open Questions

- Preliminary refund liabilities are intentionally conservative until Phase 5 implements seller payout and delivered-allocation reclassification.
- Verifier selection and quorum remain the next open protocol decision and must be immutable by pool start.
- Abort and sponsor-remainder claims are still unavailable, so the contract is not production-ready.

### Stellar Skills Used

- Stellar Smart Contracts: typed persistent storage, auth, atomic state transitions, checked arithmetic, events, interface inspection, and canonical build.
- Stellar Assets & SAC: SEP-41 contribution transfers and SAC-backed balance/rollback tests.
- Soroban Common Mistakes: double-payment prevention, transfer-failure rollback, per-pool accounting, and cross-pool isolation.

### Definition of Done

- Exact authenticated contribution transfer: completed.
- One payment per member per round: completed.
- Round/member/liability/assigned-balance accounting: completed.
- Failed-transfer atomicity: verified.
- Same-token pool isolation: verified.
- Unit tests and canonical WASM build: passed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 4 - Purchase Proposal and Independent Approval

### Goal

Fix the MVP verifier policy before pool start and implement an immutable seller/document proposal with authenticated, replay-protected independent approvals.

### Changes Made

- Bumped the contract API/schema version from 3 to 4.
- Added `VerifierPolicy` and `PurchaseState` contract types.
- Added pool-scoped verifier-policy, round-purchase, and verifier-approval storage.
- Implemented creator-authenticated `configure_verifiers` for Filling pools.
- Required a valid verifier policy before `start_pool` can activate a pool.
- Enforced 2 to 10 unique verifiers with a configurable `2..=N` approval quorum.
- Excluded creator, sponsor, members, the token contract, and the pool contract from verifier roles.
- Prevented configured verifiers from joining as members.
- Implemented current-recipient-authenticated `propose_purchase` after the complete round pot is available.
- Rejected participant, verifier, token, and contract addresses as sellers and rejected the all-zero document digest.
- Made each pool/round purchase immutable after its first proposal.
- Implemented authenticated allowlisted approvals, one per verifier, with stored approval count and quorum status.
- Added policy, purchase, and approval read APIs plus typed configuration/proposal/approval events.
- Added tests for policy validation and immutability, start gating, seller restrictions, proposal ownership/readiness, role auth, unauthorized/duplicate approvals, and quorum.

### Files Changed

- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/types.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/events.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/STELLAR_SKILLS_USED.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `cargo fmt --all -- --check`
- `cargo fmt --all`
- `cargo test --workspace`
- `stellar contract build`
- `stellar contract info interface --wasm target/wasm32v1-none/release/rotating_pool.wasm`

### Tests Run

- Rust formatting check.
- Full workspace Soroban/SAC unit-test suite.
- Canonical Stellar CLI optimized WASM build.
- Generated contract interface inspection.

### Test Results

- Unit tests passed: 17 passed, 0 failed.
- Missing, undersized, duplicate, participant-controlled, low-quorum, and active-pool policy mutations were rejected.
- Non-recipient proposals, participant/verifier sellers, zero digests, duplicate proposals, unauthorized approvals, wrong-round approvals, and duplicate approvals were rejected.
- Explicit missing-auth tests passed for proposal and approval entrypoints.
- Canonical Protocol 28 WASM build passed.
- Optimized WASM size: 28,517 bytes.
- WASM SHA-256: `a10c9babedf7f5085dc31cf295160ce82a80484b60f5b3468f8db14db6fc3a29`.
- Exported functions: 27.

### Decisions

- The MVP uses 2 to 10 independent verifier addresses and a creator-selected quorum from 2 through the verifier count.
- The creator may replace the policy only while the pool is Filling; start makes it immutable.
- Creator, sponsor, active members, configured verifiers, token contract, and pool contract cannot be the recorded seller.
- A purchase may be proposed only after all current-round member deposits form the exact expected pot. Sponsor-backed readiness will be integrated when top-ups are implemented.
- The first purchase proposal for a pool/round is final, so approvals cannot be replayed onto a replacement seller or digest.
- A non-zero digest is only a commitment to off-chain test evidence; it does not prove seller identity, title, deed, registration, mortgage, or lien status.

### Risks / Open Questions

- Verifier organizations, credentialing, legal authority, key custody, and collusion controls remain off-chain product/legal responsibilities.
- Phase 5 must consume only an approved purchase and preserve refund solvency before transferring anything to the seller.
- The stricter two-verifier minimum is an MVP safety policy; any future governance change requires a separately reviewed contract design.

### Stellar Skills Used

- Stellar Smart Contracts: bounded vectors, role auth, typed purchase/policy state, pool/round storage, events, tests, and generated interface inspection.
- Soroban Common Mistakes: immutable active configuration, duplicate-approval prevention, participant-role separation, checked counters, and replay-resistant storage keys.

### Definition of Done

- Verifier set and quorum fixed before start: completed.
- Independent role separation: completed.
- Current-recipient seller/document proposal: completed.
- Authenticated allowlist approvals and quorum: completed.
- Proposal and approval anti-replay: completed.
- Unit tests and canonical WASM build: passed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 5 - Solvent Seller Payout and Round Transition

### Goal

Pay exactly one approved round allocation to its recorded seller while preserving every pool-scoped member refund liability, then advance the immutable recipient order or complete the pool.

### Changes Made

- Bumped the contract API/schema version from 4 to 5.
- Implemented permissionless `execute_round(pool_id)`.
- Added round-top-up storage helpers and initialized pot/top-up ledgers for every new round.
- Revalidated round pot, future top-up amount, immutable purchase, verifier approval keys, stored approval count, and quorum before payout.
- Recomputed the aggregate member refund ledger before every seller transfer.
- Cleared completed-round liabilities for the current recipient and earlier allocation recipients while preserving future recipients' cumulative liabilities.
- Enforced `PoolAssignedBalance - payout >= remaining refund liabilities` without consulting the contract-wide token balance.
- Transferred exactly `member_count * contribution_amount` to the approved seller.
- Marked the paid round Settled, recorded the recipient allocation, and created the next fixed-order round or marked the pool Completed.
- Emitted `RoundPaid` and final `PoolCompleted` events.
- Added a read API for the separately recorded round top-up.
- Added two-round lifecycle, approval gate, pool-ledger solvency, seller-transfer rollback, liability reclassification, and completion tests.

### Files Changed

- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/STELLAR_SKILLS_USED.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `cargo fmt --all -- --check`
- `cargo fmt --all`
- `cargo test --workspace`
- `stellar contract build`
- `stellar contract info interface --wasm target/wasm32v1-none/release/rotating_pool.wasm`

### Tests Run

- Rust formatting check.
- Full workspace Soroban/SAC unit-test suite.
- Two complete approved rounds through pool completion.
- Deliberately corrupted pool-assigned ledger with a higher aggregate contract token balance.
- Deliberately insufficient SAC balance during seller transfer to verify transaction rollback.
- Canonical Stellar CLI optimized WASM build and generated interface inspection.

### Test Results

- Unit tests passed: 20 passed, 0 failed.
- Missing purchase, insufficient quorum, pool-ledger insolvency, replay, and invalid lifecycle execution were rejected.
- The solvency test proved that an aggregate contract balance of 30 cannot authorize a payout when that pool's assigned ledger is only 20 and would leave a 10-unit liability uncovered.
- A failed seller token transfer rolled back round settlement, pool advancement, allocation status, assigned balance, and refund-liability writes.
- The first rollback fixture attempted SAC deauthorization but the default test asset was not revocable; it was corrected to create an insufficient token balance without weakening the assertion.
- Canonical Protocol 28 WASM build passed.
- Optimized WASM size: 32,086 bytes.
- WASM SHA-256: `0fe932af532d54eb32915e52c87549bc6fe87405678a9c062756bef7ccc03892`.
- Exported functions: 29.

### Decisions

- `execute_round` is permissionless and accepts no caller identity or payout parameters.
- Payout amount and seller come only from immutable pool and approved purchase state.
- Approval count is not trusted alone; approval keys are recounted against the immutable verifier policy.
- Member liabilities are recomputed and compared with the stored total before any payout state is written.
- State is written before the external token transfer to block replay/reentry; Soroban atomicity restores it if the token transfer fails.
- Normal-round payout consumes the pool-assigned contribution/top-up amount but does not reduce the separately tracked sponsor guarantee.
- Historical round pots remain queryable after settlement; spendability comes only from current lifecycle and pool-assigned accounting.

### Risks / Open Questions

- Phase 6 must implement sponsor top-up transfers and update purchase readiness to accept an exact contribution-plus-top-up pot without marking missing member debt paid.
- Completed sponsor guarantee remains locked until the Phase 7 sponsor-remainder claim is implemented.
- An approved document digest and successful seller payment still do not prove physical delivery or legal title transfer.
- Abort/refund paths are not implemented yet, so the contract remains unsuitable for production funds.

### Stellar Skills Used

- Stellar Smart Contracts: permissionless execution, checked arithmetic, bounded invariant recomputation, checks-effects-interactions, typed events, and deterministic round transitions.
- Stellar Assets & SAC: exact seller transfer, token-failure rollback, and balance assertions.
- Soroban Common Mistakes: no aggregate-balance authorization, replay prevention, post-payment solvency, cross-contract failure rollback, and pool-scoped accounting.

### Definition of Done

- Permissionless ready-round execution: completed.
- Approved recorded-seller payout: completed.
- Pool-scoped post-payment solvency: completed.
- Liability reclassification and allocation state: completed.
- Next-round and Completed transitions: completed.
- Failed-transfer rollback: verified.
- Unit tests and canonical WASM build: passed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 6 - Overdue, Grace, Cure, and Sponsor Top-Up

### Goal

Enforce round deadlines and add a bounded recovery window in which missing members may cure or the sponsor may fund the exact shortfall without erasing member debt, followed by a permissionless Paused transition.

### Changes Made

- Bumped the contract API/schema version from 5 to 6.
- Added `grace_deadline` to `RoundState` and initialized it for every new round.
- Closed normal `deposit` calls exactly at the round deadline.
- Refactored member contribution accounting into one internal atomic payment path shared by normal deposits and cures.
- Implemented permissionless `mark_overdue` for underfunded rounds at or after their deadline.
- Fixed grace expiry at `round.deadline + grace_duration_secs`, independent of when overdue marking is triggered.
- Implemented authenticated `cure_payment` for one missing member contribution during Grace.
- Implemented sponsor-authenticated `top_up` for exactly the current remaining round shortfall.
- Kept sponsor top-up in `RoundTopUp` and `PoolAssignedBalance` without writing member deposit, contribution-total, or refund-liability records.
- Returned fully funded cure/top-up rounds to Active.
- Updated purchase readiness to accept exact `round pot + sponsor top-up` funding.
- Implemented permissionless `pause_pool` after grace expiry when the round remains underfunded.
- Added deadline, repeated transition, cure, top-up, debt-preservation, authorization, rollback, Active recovery, and Paused tests.

### Files Changed

- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/types.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/STELLAR_SKILLS_USED.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `cargo fmt --all -- --check`
- `cargo fmt --all`
- `cargo test --workspace`
- `stellar contract build`
- `stellar contract info interface --wasm target/wasm32v1-none/release/rotating_pool.wasm`

### Tests Run

- Rust formatting check.
- Full workspace Soroban/SAC unit-test suite.
- Exact round/grace deadline boundary tests.
- Member cure and sponsor top-up recovery through seller execution.
- Missing sponsor authorization and insufficient sponsor-balance rollback.
- Canonical Stellar CLI optimized WASM build and generated interface inspection.

### Test Results

- Unit tests passed: 24 passed, 0 failed.
- Early overdue/pause calls, late deposits/cures, repeated transitions, duplicate cures, and non-exact top-ups were rejected.
- Member cure restored normal contribution and refund accounting and safely returned the pool to Active.
- Sponsor top-up returned the pool to Active while the missing member remained unpaid in the on-chain deposit/contribution records.
- A failed sponsor transfer rolled back the tentative top-up, assigned balance, and Active transition.
- Canonical Protocol 28 WASM build passed.
- Optimized WASM size: 36,592 bytes.
- WASM SHA-256: `c6bd16cb1144eb8ebef80404918473a5c2f219b5de9f8335c9cfaf8bf753b58f`.
- Exported functions: 33.

### Decisions

- The grace deadline is derived from the original round deadline, so delayed automation cannot extend the recovery window.
- Normal deposits are rejected at `timestamp >= deadline`; cures and top-ups are rejected at `timestamp >= grace_deadline`.
- Sponsor top-up must equal the full current shortfall. Partial or excess top-ups are rejected to avoid dust and ambiguous later member payments.
- A sponsor-funded missing contribution remains absent from `Deposit`, `MemberContributionTotal`, and `RefundLiability`; the top-up is separately auditable in `RoundTopUp`.
- `Paused` closes cure/top-up execution for that round and becomes the Phase 7 abort entry state.
- Recovery calls write state before token transfer and rely on Soroban transaction rollback if transfer fails.

### Risks / Open Questions

- Off-chain collection rights and the creditor for sponsor-covered member debt still require legal/product agreements; the contract only preserves the unpaid deposit evidence.
- A Paused pool cannot recover or release funds until Phase 7 abort/refund claims are implemented.
- No background process is assumed; any account may trigger overdue and pause transitions when their ledger times arrive.

### Stellar Skills Used

- Stellar Smart Contracts: ledger-time gates, deterministic state transitions, role auth, shared atomic payment logic, persistent grace state, typed events, and tests.
- Stellar Assets & SAC: member cure and sponsor top-up transfers with failure rollback.
- Soroban Common Mistakes: boundary-time tests, no silent debt clearing, no excess top-up, replay-resistant transitions, and checks-effects-interactions.

### Definition of Done

- Deadline-enforced normal deposits: completed.
- Deterministic Grace transition: completed.
- Authenticated member cure: completed.
- Separate exact sponsor top-up: completed.
- Safe return to Active: completed.
- Permissionless Paused transition: completed.
- Failed-transfer rollback: verified.
- Unit tests and canonical WASM build: passed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 7 - Abort, Member Refunds, and Sponsor Remainder

### Goal

Implement permissionless abort from the Paused recovery state, pay each member exactly their delivered/not-delivered refund entitlement with double-claim protection, and let the sponsor claim only the guarantee remainder that is not reserved for an outstanding member liability.

### Changes Made

- Bumped the contract API/schema version from 6 to 7.
- Implemented permissionless `abort_pool(pool_id)`, allowed only from `Paused`, transitioning the pool to `Aborted` and emitting `PoolAborted` with `AbortReason::SafetyRecovery`.
- Relied on existing status guards on `deposit`, `cure_payment`, `top_up`, `mark_overdue`, `pause_pool`, `propose_purchase`, `approve_purchase`, and `execute_round` to freeze all financial mutation once a pool is `Aborted`, so refund entitlements need no separate snapshot step.
- Implemented authenticated `claim_refund(member, pool_id)`. Entitlement is read directly from the existing `RefundLiability` ledger, which Phase 5's `execute_round` already zeroes only for a member's own settled round, so it already equals the correct delivered/not-delivered amount at abort time.
- Added persistent per-member `RefundClaimed` and per-pool `SponsorRemainderClaimed` storage read/write helpers.
- Implemented sponsor-authenticated `claim_sponsor_remainder(pool_id)`, computed as `PoolAssignedBalance - TotalRefundLiability` so the sponsor is never blocked on individual members actually claiming, only on enough balance remaining reserved for outstanding liabilities.
- Added `get_refund_claim` and `get_sponsor_remainder_claimed` read APIs.
- Added abort-lifecycle, not-yet-delivered full refund, delivered-member partial (unfinished-round-only) refund, sponsor-remainder order-independence, double-claim, missing-auth, and failed-transfer-rollback tests.
- Regenerated SDK differential test snapshots and built the optimized WASM.

### Files Changed

- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `cargo fmt --all -- --check`
- `cargo fmt --all`
- `cargo test --workspace`
- `stellar contract build`

### Tests Run

- Rust formatting check.
- Full workspace Soroban/SAC unit-test suite.
- Abort eligibility gated to `Paused`, with a second abort and every other mutation rejected once `Aborted`.
- A member who never received an allocation reclaiming their full completed-round contribution while a never-depositing member has nothing to claim.
- A member who already received an allocation, then contributed again in the next round before abort, reclaiming only that unfinished round's contribution while the still-undelivered member reclaims their full historical contribution.
- Sponsor remainder claimed before and interleaved with a member's own refund claim, proving order-independence, and rejected once nothing remains unreserved.
- Double refund and double sponsor-remainder claims.
- Missing member/sponsor authorization on both claim entrypoints.
- A deliberately insufficient contract token balance during `claim_refund` to verify rollback of the claimed flag and every ledger.
- Canonical Stellar CLI optimized WASM build and generated interface inspection.

### Test Results

- Unit tests passed: 29 passed, 0 failed.
- Abort before `Paused`, a second abort after `Aborted`, and post-abort deposit/top-up/execute calls were all rejected.
- The not-yet-delivered member's refund matched their full accumulated contribution; the never-depositing member's claim was rejected as not claimable.
- The delivered member's refund matched only their post-delivery round contribution; the still-undelivered member's refund matched their full historical contribution, confirming the existing `RefundLiability` ledger already encodes the correct delivered/not-delivered rule without new bookkeeping.
- Sponsor remainder and member refund claims summed to exactly the pool's pre-claim assigned balance and drained the contract's token balance to zero in the fully-claimed scenario.
- A failed refund transfer left the claimed flag, refund liability, pool-assigned balance, and total liability exactly as they were before the call.
- Canonical Protocol 28 WASM build passed.
- Optimized WASM size: 39,542 bytes.
- WASM SHA-256: `0fc46a9ec0c3539723f40bff2640789917830c324d953943ee762f4b46dfb3a7`.
- Exported functions: 38.

### Decisions

- Abort eligibility is tied to `Paused` rather than a new independent timer: `Paused` is only reachable once the grace deadline has passed on an underfunded round, so it already is the "recovery conditions expired" state the plan describes, and Phase 6's log had already flagged it as the Phase 7 entry point.
- `AbortReason::SafetyRecovery` is the only reason emitted in this MVP; `AbortReason::BlockedSettlement` remains a reserved, unused variant for a future stuck-purchase abort path that is not part of the current scope.
- Refund entitlement is read directly from `RefundLiability` rather than snapshotting a separate frozen value at abort time, because every mutation that could change it is already blocked once the pool is `Aborted`; this avoids duplicate storage and keeps a single source of truth.
- Each member claim decrements `TotalRefundLiability` and `PoolAssignedBalance` by exactly that member's entitlement so the remaining reserved amount is always accurate for the sponsor-remainder computation, regardless of claim order.
- Sponsor remainder is computed as assigned balance minus outstanding (unclaimed) total liability, not minus original total liability, so the sponsor is never forced to wait for every member to actually submit a claim transaction, only for the contract to hold enough to cover whatever remains unclaimed.
- Claim state is written before the external token transfer, consistent with every prior phase, so a failed transfer rolls back atomically under Soroban's transaction semantics.

### Risks / Open Questions

- `AbortReason::BlockedSettlement` has no triggering path yet; if a future phase needs to abort a pool stuck on a never-approved purchase rather than an underfunded round, that will need its own eligibility rule and tests.
- A member who never claims their refund leaves funds permanently reserved out of the sponsor's reach; this is the intended conservative behavior but has no on-chain expiry/sweep mechanism in the MVP.
- Off-chain notification that a pool has aborted and refunds/remainder are claimable is not part of this contract-only workstream.
- Abort, refund, and sponsor-remainder claims are still unverified on live Testnet; only local Soroban/SAC test-utility coverage exists so far.

### Stellar Skills Used

- Stellar Smart Contracts: permissionless lifecycle-gated abort, reuse of existing status guards for entitlement freezing, persistent claim-replay storage, typed events, generated interface inspection, and canonical build.
- Stellar Assets & SAC: refund and sponsor-remainder SEP-41 transfers with failure-rollback tests.
- Soroban Common Mistakes: double-claim prevention, checks-effects-interactions ordering, no reliance on contract-wide token balance, and pool-scoped accounting reused rather than duplicated.

### Definition of Done

- Permissionless abort gated to the expired-recovery `Paused` state: completed.
- Refund entitlements frozen by lifecycle guards without duplicate storage: completed.
- Delivered/not-delivered member refund rule and double-claim protection: completed.
- Order-independent sponsor remainder available only for unreserved funds: completed.
- Failed-transfer rollback: verified.
- Unit tests and canonical WASM build: passed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 8 - Full Security and Invariant Suite

### Status

SKIPPED — explicit user decision on 2026-09-19 ("faz 8 geç" / "skip phase 8") to prioritize reaching a live Testnet deployment and demo evidence ahead of the Rise In x Stellar Pro Hackathon 2026 submission window (19-20 Eylül, İstanbul). This is a scope/priority call, not a claim that the contract needs no further review. The security posture the contract ships with is exactly what Phases 1-7 already built inline: 29 passing unit tests, checked arithmetic throughout, pool-scoped accounting, replay-blocking claim/approval flags, checks-effects-interactions ordering, and the Soroban Common Mistakes checklist applied at every phase. No dedicated cross-pool/TTL/race-condition audit pass or external tooling run has been done. Revisit this phase before any non-hackathon or production use of the contract.

## Phase 9 - Testnet Assets, Deployment Scripts, and Minimal Backend

### Goal

Produce reproducible Testnet deployment tooling for the `rotating_pool` contract and its settlement-asset token, then actually deploy to Testnet to obtain real IDs and transaction evidence for the hackathon submission. Leave the Anchor adapter and minimal backend out of scope, since the real provider/home domain is still unknown (see Phase 0/2 open questions).

### Changes Made

- Added `scripts/deploy_testnet.sh`: builds the canonical WASM, generates and Friendbot-funds a `stellerpool-deployer` Testnet identity (reused on re-run), deploys the contract, resolves or deploys the settlement-asset SAC wrapper, runs a read-only `version()` smoke test, prints a summary, and writes the resulting IDs into `.env` and `frontend/.env` (created from the `.example` templates if missing) unless `--no-write` is passed.
- Added `scripts/README.md` documenting prerequisites, usage, and what is intentionally out of scope.
- Ran the equivalent workflow manually against Stellar Testnet to produce the deployment evidence below, then populated the local (gitignored) `.env` and `frontend/.env` files with the resulting IDs so the frontend can be pointed at the live contract immediately.

### Files Changed

- `scripts/deploy_testnet.sh`
- `scripts/README.md`
- `.env` (local only, gitignored)
- `frontend/.env` (local only, gitignored)
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `stellar contract build`
- `stellar keys generate stellerpool-deployer --network testnet --fund`
- `stellar keys address stellerpool-deployer`
- `stellar contract deploy --wasm target/wasm32v1-none/release/rotating_pool.wasm --source stellerpool-deployer --network testnet --alias rotating_pool`
- `stellar contract asset deploy --asset USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5 --source stellerpool-deployer --network testnet` (failed: already deployed by someone else at the deterministic address, which is expected and handled)
- `stellar contract id asset --asset USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5 --network testnet`
- `stellar contract invoke --id <contract> --source stellerpool-deployer --network testnet --send=no -- version`
- `stellar contract invoke --id <contract> --source stellerpool-deployer --network testnet --send=no -- next_pool_id`
- `stellar contract invoke --id <contract> --source stellerpool-deployer --network testnet -- create_pool --creator <deployer> --sponsor <deployer> --token <token contract> --contribution_amount 10000000 --member_limit 2 --round_duration_secs 3600 --grace_duration_secs 3600`

### Test Results / Deployment Evidence

- Deployer identity (Testnet, Friendbot-funded): `GBHIRK6NXKLE46DB3ORLKAMIGG4XUOEL66VH2QAYDY5NUTZKSTSQURTB`.
- Rotating pool contract ID (Testnet): `CACZQBHHQ3TY33AJIC52MHMCPEKUYO4KQFURHQGPJJ3LNDJZAT6LG4II`.
- Deployed WASM hash: `0fc46a9ec0c3539723f40bff2640789917830c324d953943ee762f4b46dfb3a7` (matches the Phase 7 local canonical build).
- WASM upload transaction: `8ee0c1df83c80af809302e7e203e46b93cc7b76979995e2b36e7d5710a25b7c7`.
- Contract instance creation transaction: `37fabed3d5227da0fb942cdc76dd388d647d5f8267d5f31a4e007768827cf944`.
- Settlement asset: Testnet USDC (`USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5`, the same default the frontend already ships), SAC contract ID `CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA` — already wrapped on Testnet before this deployment; the script detects and reuses it instead of failing.
- Read-only smoke test: `version()` returned `7`, `next_pool_id()` returned `1`, both against the live deployed contract.
- Live write-path smoke test: `create_pool` transaction `b801fb2ae1c32cc4df2b254aeda54f091e20714d11a2605a515d84d42d54d90d` succeeded on Testnet, emitting `PoolCreated` (`pool_id: 1`, `guarantee_required: 10000000`, `member_limit: 2`) exactly as computed locally, confirming the full create-pool path (auth, checked guarantee formula, storage) works end to end on live Testnet, not only in the local unit-test harness.

### Decisions

- Reused the existing Testnet USDC SAC wrapper at its deterministic address instead of deploying a duplicate, matching the asset the frontend's `.env.example` already defaults to (`VITE_POOL_ASSET_CODE=USDC`, same issuer). No new asset invented.
- The deploy script never prints secret keys; the deployer's key stays in the Stellar CLI's local identity store, addressed only by alias/public address.
- `pool_id: 1` on this contract instance now holds real (harmless, unfunded) Testnet state from the smoke test. A future Phase 10 demo can either continue from this pool or redeploy a fresh contract instance via the same script — deployment is cheap and idempotent per run.
- Anchor adapter and minimal backend were intentionally left out of this phase; `ANCHOR_HOME_DOMAIN` is still blank, and Phase 0 already ruled out inventing a provider.

### Risks / Open Questions

- The `create_pool` smoke-test pool (`pool_id: 1` on the deployed contract) used the deployer address as both creator and sponsor purely to prove the write path; it is not a realistic demo scenario and should not be presented as one.
- No multi-member funded flow (join/deposit/propose/approve/execute, or an abort/refund run) has been exercised on Testnet yet — that is Phase 10's scope and needs several funded, trustlined accounts.
- The deployed contract has no upgrade mechanism by design; redeploying for any future contract change means a new contract ID and updated `.env` values.
- `.env`/`frontend/.env` are local and gitignored, so this deployment's IDs are not yet visible to teammates who have not pulled/re-run the script or been given the values directly; consider sharing the contract/token IDs above out-of-band (e.g. in the hackathon submission notes) since they are not secrets.

### Stellar Skills Used

- Stellar Smart Contracts: canonical `stellar contract build`/`deploy` workflow, Stellar CLI identity management, and a live read/write smoke test against a deployed contract.
- Stellar Assets & SAC: resolving an existing SAC wrapper via `stellar contract id asset` instead of re-deploying, and using the deterministic wrap address correctly.
- RPC & Horizon APIs: `soroban-testnet.stellar.org` RPC used throughout for simulation, submission, and read-only invocation.
- Soroban Common Mistakes: no secret material printed or committed, `.env` files kept out of version control, and the script is idempotent rather than assuming a fresh network state.

### Definition of Done

- Reproducible deployment script for contract + settlement asset: completed.
- Real Testnet contract ID and transaction evidence recorded: completed.
- Read-only and write-path smoke tests against the live deployment: completed.
- Anchor adapter and minimal backend: intentionally deferred, matches plan gating.
- Frontend untouched (only local, gitignored `.env` values populated): completed.

### Status

NEEDS REVIEW

## Phase 10 - End-to-End Demo and Submission Evidence

### Goal

Actually run both the normal and default/abort/refund lifecycles against the live Testnet deployment from Phase 9, capture real IDs/hashes as evidence, and leave a reusable script so the demo is reproducible rather than a one-off manual run.

### Changes Made

- Added `scripts/demo_testnet.sh`: generates/funds six demo identities (sponsor, two members, two verifiers, one seller), issues and trustlines a self-contained demo asset (`STLP`, issued by the same deployer identity from Phase 9) so the run needs no external Testnet-USDC faucet, resolves/deploys its SAC wrapper, then runs Pool A (happy path to `Completed`) and Pool B (forced default, `Grace` -> `Paused` -> `Aborted`, member refund + sponsor remainder claims) against an already-deployed contract.
- Ran the equivalent flow manually against Stellar Testnet (interactively, so each step's result could be checked before the next) to produce the evidence below, then wrote the script to match exactly what worked. Pool B's two real-time waits (round deadline, then grace deadline) were driven by polling `mark_overdue`/`pause_pool` every 5s in the background rather than a blind sleep, since Soroban deadlines are wall-clock time and cannot be simulated against a live network.
- Updated `scripts/README.md` with `demo_testnet.sh` usage.
- Updated `README.md`'s delivery checklist to reflect the completed Soroban contract, Testnet deployment, and eksik ödeme/durdurma/iptal demo items.

### Files Changed

- `scripts/demo_testnet.sh`
- `scripts/README.md`
- `README.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `stellar keys generate <demo-sponsor|demo-member1|demo-member2|demo-verifier1|demo-verifier2|demo-seller1> --network testnet --fund`
- `stellar tx new change-trust --source-account <holder> --network testnet --line STLP:<issuer>` (sponsor, both members, seller)
- `stellar tx new payment --source-account stellerpool-deployer --network testnet --destination <addr> --asset STLP:<issuer> --amount 500000000` (sponsor, both members)
- `stellar contract asset deploy --asset STLP:<issuer> --source stellerpool-deployer --network testnet`
- `stellar contract invoke --id <contract> --source <identity> --network testnet -- <create_pool|join_pool|configure_verifiers|fund_guarantee|start_pool|deposit|propose_purchase|approve_purchase|execute_round|mark_overdue|pause_pool|abort_pool|claim_refund|claim_sponsor_remainder|get_pool|get_refund_liability>` (full sequence for both pools)

### Test Results / Deployment Evidence

Demo settlement asset (self-issued, not Testnet USDC): `STLP` issued by `GBHIRK6NXKLE46DB3ORLKAMIGG4XUOEL66VH2QAYDY5NUTZKSTSQURTB` (the Phase 9 deployer). SAC contract ID: `CAOV35NPIJXHWA7QPXXERRJQ4ZTDUGAEHA7FKB6QIOTIOTI62B35ZNWI`.

Demo identities (Testnet, Friendbot-funded): sponsor `GBSZZFESXTBXM2KEKTAFHOLGYWA4KCHMPNNXX6WKU5UX2CM6RCUZQ62O`, member1 `GDANLW6SJYQDUMVVFT77IJCOZ4X2CGL5GAUVBC2PQMVTE666YMU57LXH`, member2 `GAMTMCPL2AVYRLOGJKPJPDXIXSYEPXFHRQPBTLJ5MV2C5RIQHUDC7Z57`, verifier1 `GDOBHXYPR54G7LVH6ACWMSWPTIWA372EPOSTJ2MAVPINVBXVVWPVUHEH`, verifier2 `GACDJC3GCYQLTI4CLPKNXFWXYYMJSDXHZ2Z6GHTFXLRYGDPNU7O3RIXI`, seller1 `GDD7ADYTZ7UAWP3Y44ZJWNAGSL4KOFYEFQRRWS6AHEYSRT445G7A4FAO`.

**Pool A - happy path** (contract `CACZQBHHQ3TY33AJIC52MHMCPEKUYO4KQFURHQGPJJ3LNDJZAT6LG4II`, `pool_id: 2`, contribution 10 STLP, member_limit 2, guarantee_required 10 STLP):

- `create_pool` tx `3abc9997ebfe434eb9e2dae72ce35a4403f595520e6dd6ab2864f391b44d742a`.
- `join_pool` member1 tx `35ace37d5b788c66cb46428234febfc2bada857db75241d84b335ea1c511250b`; member2 tx `edf08ab940b585c2152fd23c65982271fc18406b1eee6cb37968c0139e7a7bbe`.
- `configure_verifiers` tx `ec51f9f65204a61376251a888f5f51d96eeb1c5077b9ca17cbf31dca6cd27f96`.
- `fund_guarantee` tx `f5e8cc3462b35ab04c30ea12c6b239d299b9ae5056f15e9d355fd02f3eb73cab` (10 STLP).
- `start_pool` tx `902afefe97c3663c13997a2c9b2f95406e3a715c4ad13b0e1161036ea0fe720f`.
- Round 0: both deposits (tx `37986ddd79ee3d1723a4f41905d90e39cfb4c8cbd70267dc211282bdf73705e9`, `26b33308e3aac6b029cc02b334ba0aaa2717e5e0b5c6f63e2f6f21ad19c000fd`), `propose_purchase` tx `980137496b0fce629722d0edf870127af3d8c6cfefad12eb19f01ceded8a54fb`, approvals tx `95829594247a823810135f2b70e3ef70826bbb6179f6e2438a74eca240ff7615` and `d2220a119ab5c587ee0cdeb26abff82e66a74bfd785cf1a84cf1d7166cdea3f1`, `execute_round` tx `d698bc817c2515ae5e1963a41feda70a3a1b1e796b483896454de3e8a9f2af9e` — paid seller1 exactly 20 STLP, emitted `RoundPaid`.
- Round 1: both deposits, `propose_purchase` tx `975c272533c1859409f8c8cda25fb814d096acf00a7ab24710fd89a882083ea1`, approvals tx `75d00bb488c4f6b0c971c418efc04f75301183de5153e9a1773e14d9405d9f5a` and `fe50fcacde7678016585d43a5124be6e83c0323ad1d15386cf6c903221028db4`, `execute_round` tx `817f8141a6c184c26b58c1f2608ba549d02317f3499aec3c5d51b5049ba6d959` — paid seller1 another 20 STLP and emitted `PoolCompleted`.
- Final `get_pool(2).status` read back as `"Completed"`.

**Pool B - default/abort/refund** (same contract, `pool_id: 3`, contribution 10 STLP, member_limit 2, guarantee_required 10 STLP, `round_duration_secs: 40`, `grace_duration_secs: 40`, deliberately short to make the wait reproducible in minutes rather than days):

- `create_pool` tx `28f1963dc9385874a412b48ca10d89e163dd36abe8df28ed93745315823c0ca1`.
- `join_pool` member1 tx `f491caea9c2c0815528e99a2728e38c1dff6b9e2c32a6724bc08cd229868be62`; member2 tx `08fe545e47e18eddd7645d10279b5624586be90eae43f7480a8439558ca7f8a3`.
- `configure_verifiers` tx `b9529063b19a15bb48c4bae433007705b41e76c1d7cb56ec09b6f741b17be391`.
- `fund_guarantee` (10 STLP) and `start_pool` tx `34ef5c9222ca3106ab86408309be3c2e2ef66834f869d178fc425ebab5be11e5`.
- member1 deposits into round 0 (10 STLP); member2 deliberately does not deposit.
- After the real round deadline passed, `mark_overdue` tx `2f09f6f216e34a4c443961ec94329b9ec1bb4cf0e3fa9f74c9f957e01a7a469b` succeeded, emitting `RoundOverdue`.
- After the real grace deadline passed, `pause_pool` tx `f9055ff44f0831960d683307fc8ef379f19776f97097a5f2b0147eccde7a551d` succeeded, emitting `PoolPaused`.
- `abort_pool` tx `6a3a06fafe854c8d59450490dcc8e61e70c2fc47f94619ac7c0afaa0e5c12357`, emitting `PoolAborted` with `reason: SafetyRecovery`.
- `claim_refund(member1)` tx `5c9fb3d0f855b2ef6c027f5e4668d5105a081001a5fb96a7665dbb0b8cebeb6c` — paid member1 exactly their 10 STLP round-0 contribution back.
- `claim_sponsor_remainder` tx `a1d9fc086492fb0e10f00119a1b200711aa071860459605147256e012357c992` — paid the sponsor exactly the 10 STLP guarantee that was never needed to cover member1's liability.
- `get_refund_liability(3, member2)` read back as `0` (member2 never deposited, so has nothing to reclaim), and `get_pool(3).status` read back as `"Aborted"`.

### Decisions

- Used a self-issued demo asset (`STLP`) instead of the Testnet USDC already configured in `frontend/.env.example`, to keep the demo fully self-contained and independently re-runnable without depending on an external anchor/faucet having testnet USDC available on demand. Production/demo UI default remains Testnet USDC; this is documented as a deliberate substitution, not a silent asset change.
- Ran Pool B's abort scenario with short (40s) round/grace durations rather than the production-shaped example in `docs/plan.md`, purely so the real-time wait fits a reproducible few-minute demo; the contract logic itself is duration-agnostic (already proven at both the unit-test and now real-clock level).
- Waited for real deadlines by polling the actual state-changing call (`mark_overdue`, then `pause_pool`) every 5 seconds in a background process rather than a blind timed sleep, so the script advances the instant the ledger crosses each deadline instead of guessing a fixed wait.
- Reused Pool A's members/sponsor/verifiers for Pool B rather than minting a fresh set of identities, since nothing in the contract restricts an address from participating in multiple pools and it kept the demo shorter.

### Risks / Open Questions

- This is contract-only, simulated evidence: the seller, the "purchase document digest," and the demo asset are test fixtures, not a real seller, real title/deed/registration check, or real currency. Nothing here should be presented as proof of real property/vehicle delivery.
- No TRY/fiat anchor flow was exercised or could be, since `ANCHOR_HOME_DOMAIN` remains unset (Phase 9).
- The frontend was not driven through this demo (no browser/wallet-kit flow); this evidence is CLI/contract-level only. A wallet-driven click-through demo is still open work for the hackathon presentation itself.
- Testnet state (including these exact pools) can be reset or pruned by the network operator at any time; the recorded transaction hashes and contract/pool IDs above are the durable evidence, not the live queryable state.

### Stellar Skills Used

- Stellar Smart Contracts: full lifecycle invocation sequence (create/join/configure/fund/start/deposit/propose/approve/execute for the happy path; create/join/configure/fund/start/deposit/mark_overdue/pause/abort/claim for the default path) exercised live, not only in the unit-test harness.
- Stellar Assets & SAC: classic-layer trustline (`change-trust`) and issuance (`payment`) operations for a self-issued demo asset, then its SAC wrapper for use by the Soroban contract.
- RPC & Horizon APIs: Testnet RPC used for every simulate/submit/read call; deadlines observed against real ledger closeTime rather than a test harness clock.
- Soroban Common Mistakes: verified live (not just via unit tests) that a missing contribution cannot be silently absorbed, that the paying member and the sponsor both recover exactly their entitled amount and no more, and that the never-depositing member's claim is correctly `0`.

### Definition of Done

- Live normal (happy-path) Testnet scenario run to `Completed`: completed.
- Live default/abort/refund Testnet scenario run to `Aborted` with both claims paid: completed.
- IDs, hashes, and commands recorded: completed.
- Demo limitations and real-versus-simulated boundaries documented: completed.
- Reusable script for reproducing the demo: completed.
- README delivery checklist updated: completed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW
