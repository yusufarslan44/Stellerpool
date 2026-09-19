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

## Phase 11 - Realign Contract to the Revised docs/plan.md and the Frontend's Client

### Goal

Reconcile the deployed contract with two things that moved out from under it after Phase 10: a substantially revised `docs/plan.md` (commit `aa0bc42`) and a teammate's frontend contract client (`frontend/src/services/pool.ts`, `frontend/src/types/pool.ts`, commit `5060119`) written against that revised plan's *assumed* API before any matching contract existed. The user's explicit instruction was to revise the contract to match the plan the frontend already follows, so the frontend needs zero changes and the two sides stay in sync going forward.

### Changes Made

- Rewrote `contracts/rotating_pool/src/types.rs`: `PoolStatus` is now `Filling | Active | Completed | Aborted` (Grace/Paused removed from the pool level). New `RoundPhase` (`Collecting | Grace | AwaitingPurchase | Settled`) carries per-round deadline state instead. `Pool` now embeds `members`, `recipient_order`, `verifiers`, `approval_threshold`, `terms_version`, `terms_approvals`, `purchase_duration`, `setup_deadline`, and `demo_seller` directly. `RoundState` embeds `paid`, `sponsor_advanced`, `approvals`, `seller`, `asset`, `amount`, `doc_hash`, `purchase_version`, `collect_deadline`, `grace_deadline`, and `purchase_deadline` directly, matching the frontend's single-read expectation for `get_pool`/`get_round`. Added `MemberStatusView { refundable, received }`.
- Rewrote `contracts/rotating_pool/src/storage.rs`: added `SponsorAdvance`, `TermsApproval` (keyed by version, so a version bump auto-invalidates stale approvals without an explicit clear step), `PurchaseApproval` (keyed by round *and* purchase version, same auto-invalidation trick), and `AdvanceCovered` (distinguishes a sponsor-covered round contribution from a self-paid one). Removed the old `PoolMembers`/`RecipientOrder`/`VerifierPolicy`/`RoundPot`/`RoundTopUp`/`Purchase`/`VerifierApproval` keys, superseded by the denormalized `Pool`/`RoundState` structs.
- Rewrote `contracts/rotating_pool/src/error.rs` and `events.rs`: added errors/events for the terms-approval flow, cancellation, sponsor advances/repayment, the purchase-version binding, and the two now-distinct abort reasons.
- Rewrote `contracts/rotating_pool/src/lib.rs` (every entrypoint): `create_pool` (new `purchase_duration`/`setup_deadline`/`demo_seller` params, validated), `fund_guarantee`, `join_pool` (now rejects the demo seller's address as a member), new `propose_terms`/`approve_terms`, `start_pool` (now permissionless; requires full membership, a fully-approved terms version from every member and the sponsor, sufficient guarantee, and `now < setup_deadline`), new `cancel_unstarted_pool`, `deposit`/`cure_payment` (now gated by `RoundPhase` instead of `PoolStatus`, sharing an `apply_member_payment` helper that also drives the phase transition), new `top_up` (single-member, no amount argument, rejects the current recipient, credits `SponsorAdvance` instead of refund liability) and `repay_advance`, `propose_purchase`/`approve_purchase` (seller pinned to `demo_seller`, asset pinned to `pool.token`, amount pinned to the round's fixed payout, approvals bound to `purchase_version`), `execute_round` (re-validates the recipient personally paid and owes no advance before paying the seller), `mark_overdue` (round-level, no pool-level Grace), `abort_pool` (permissionless from `Grace` past its deadline with `AbortReason::SafetyRecovery`, or from `AwaitingPurchase` past its `purchase_deadline` with `AbortReason::BlockedSettlement`), `claim_refund`/`claim_sponsor_remainder` (sponsor address now an explicit argument, matching the frontend call shape), and read APIs `get_pool`/`get_round`/`get_member_status`/`get_sponsor_advance`/`get_refund_claim`/`get_sponsor_remainder_claimed`. Removed `leave_pool` and `configure_verifiers` (absent from both the revised plan and the frontend client). Verifier quorum is no longer creator-supplied; it is computed as `ceil(2 * verifier_count / 3)`. `MAX_MEMBERS` lowered from 20 to 12.
- Rewrote `contracts/rotating_pool/src/test.rs` from scratch: 24 tests covering create-pool validation (including the new duration/deadline/demo-seller checks), no-fund join with seller exclusion, terms propose/approve/version-reset/role-eligibility, permissionless start gated on full approval, setup-deadline cancellation with full sponsor refund, deposit/cure round-phase gating and the Collecting->AwaitingPurchase transition, the recipient-cannot-be-topped-up rule and advance bookkeeping, advance repayment unblocking a stalled round, purchase proposal/approval validation (seller/asset/amount/digest/version binding), quorum, full two-round happy-path completion, insufficient-balance and failed-transfer rollback, grace-then-abort (`SafetyRecovery`), cure-during-grace recovery, purchase-deadline-expiry abort (`BlockedSettlement`), delivered/not-delivered refund plus order-independent sponsor remainder, stored-role-auth requirements on both claim entrypoints, multi-pool isolation, and verifier-policy exclusion/bounds.
- Deployed the rewritten contract to Testnet as a **new** instance (the Phase 9/10 instance is API-incompatible and left untouched as a historical artifact) and ran the full new lifecycle against it live, using the exact call shapes `frontend/src/services/pool.ts` uses.
- Rewrote `scripts/demo_testnet.sh` to drive the new API (terms propose/approve, the fixed demo seller, `asset`/`amount` on purchase proposals, round-level grace instead of a pool-level pause). `scripts/deploy_testnet.sh` needed no changes — it only builds/deploys the WASM and resolves a settlement asset, it never calls `create_pool`. Recreated `scripts/README.md` (lost from git history in an earlier stash/merge round-trip) and updated it for the new demo flow.
- Updated `.env` and `frontend/.env` (local, gitignored) with the new contract ID.
- Updated `README.md`'s delivery checklist and removed the now-resolved plan/contract mismatch warning; updated `docs/IMPLEMENTATION_PLAN.md`'s status header, "Documentation Conflicts" entry, and added this phase to the phase plan.

### Files Changed

- `contracts/rotating_pool/src/types.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/events.rs`
- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `scripts/demo_testnet.sh`
- `scripts/README.md`
- `.env` (local only, gitignored)
- `frontend/.env` (local only, gitignored)
- `README.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo fmt --all` / `cargo fmt --all -- --check`
- `stellar contract build`
- `stellar contract info interface --wasm target/wasm32v1-none/release/rotating_pool.wasm` (used to cross-check every method/field name and type against `frontend/src/services/pool.ts` and `frontend/src/types/pool.ts` by hand, one by one)
- `stellar contract deploy --wasm ... --source stellerpool-deployer --network testnet --alias rotating_pool_v2`
- `stellar contract invoke --id <new contract> ...` for the full live lifecycle: `create_pool`, `join_pool` x2, `propose_terms`, `approve_terms` x3, `fund_guarantee`, `start_pool`, `deposit` x2 per round, `propose_purchase`, `approve_purchase` x2 per round, `execute_round` x2, `get_pool`, `get_round`

### Test Results

- Unit tests passed: 24 passed, 0 failed (down from 29 because `leave_pool`/`configure_verifiers`-specific tests no longer apply; coverage of the equivalent and new behaviors is broader than before per-function).
- Canonical Protocol 28 WASM build passed. Optimized WASM size: 39,823 bytes. WASM SHA-256: `f60503d00631e6af6e42ef862f5a066708dc07a7edafe40a64260b50b82ab854`. Exported functions: 27 (all 22 the frontend calls, plus 5 extra read/utility methods it doesn't use).
- `stellar contract info interface` output was compared field-by-field against `frontend/src/services/pool.ts`'s `PoolMethods`/`EXPECTED_METHODS` and `frontend/src/types/pool.ts`'s `PoolInfo`/`RoundInfo`/`MemberStatus`: every method name, parameter name, return type, struct field name, and enum variant (`PoolStatus`, `RoundPhase`) matches exactly.
- **Live Testnet deployment**: new contract ID `CBC5DGFAQMEGVJVC6W3Z3J7SNMMFK5LQO3LMZVVJE5YFOPSF4CKW3U4Q` (WASM upload tx `bbac73ed4c01c55e42ba65283a4778816d6cc24dd13343d2000db74ad5701cb4`, instance creation tx `7da6288c0b3fbdd69ca903bb680453afbc3828809a6d4a5c69f70372fabd46b0`). Reused the Phase 10 demo identities and the `STLP` demo asset (SAC `CAOV35NPIJXHWA7QPXXERRJQ4ZTDUGAEHA7FKB6QIOTIOTI62B35ZNWI`), which still held sufficient balances.
- **Live full happy-path run** (`pool_id: 1`, contribution 1 STLP, member_limit 2, required guarantee 1 STLP, `round_duration: 1800`, `grace_duration: 1800`, `purchase_duration: 900`): `create_pool` tx `ce27118919a93b59ed0ac419be1303780d5a283ec136f8e67e092602bca93124`; both `join_pool` calls; `propose_terms` tx `ea0e3d2d41506920abc31a68d60d2865a4a26d6917d03b61dd819da41e441f7f` (`approval_threshold: 2` for 2 verifiers, confirming the `ceil(2n/3)` quorum formula); three `approve_terms` calls (both members + sponsor); `fund_guarantee` tx moving 1 STLP; `start_pool` tx `b5a946545f0c1b46f0130d36b8799a7115d77371dda67628fffff6e005afffc0` (permissionless, called by the deployer identity, which is neither a member nor the sponsor); round 1 both deposits, after which a live `get_round` read showed `phase: "AwaitingPurchase"` and `purchase_deadline` set automatically; `propose_purchase` tx with `asset` and `amount` fields populated and echoed correctly; two `approve_purchase` calls; `execute_round` tx `0dbda0d13092733c714d571108e79fb1224fa0227d07e878ce004b832095b114` paying the demo seller exactly 2 STLP; round 2 repeated the same sequence; final `execute_round` tx `017785e64c77e974ecf9e97786a9c98b4580de6edeeb2975acdc17da16fb22ef` paid the seller again and emitted `PoolCompleted`. A final `get_pool` read confirmed `status: "Completed"` with every field (`members`, `recipient_order`, `verifiers`, `terms_approvals`, `approval_threshold`, etc.) present in exactly the shape `frontend/src/services/pool.ts`'s `mapPool` expects.
- Total live seller payout across both rounds: 4 STLP (2 STLP x 2 rounds), matching `contribution_amount * member_limit` exactly each time.

### Decisions

- Rewrote rather than patched: the pool/round state machine changed shape enough (Grace moving from pool-level to round-level, a new AwaitingPurchase phase, denormalized reads) that incremental edits to the Phase 1-10 contract would have been harder to get right than a clean rewrite built directly against the frontend's already-fixed interface.
- Treated `frontend/src/services/pool.ts` and `frontend/src/types/pool.ts` as the literal interface contract (method names, parameter names, field names, enum variants) rather than re-deriving an interface from `docs/plan.md` prose alone, since the user's instruction was specifically to match what the frontend already expects and prevent future drift.
- `TermsApproval` and `PurchaseApproval` are keyed by version (`(pool_id, version, approver)` / `(pool_id, round, version, verifier)`) rather than storing a separate "approvals" list that must be manually cleared on every re-proposal; a version bump alone makes every prior approval key permanently stale, which is simpler and can't be forgotten in some future code path.
- `RoundState.paid`/`sponsor_advanced`/`approvals` are stored as bounded `Vec<Address>` directly on the round (bounded by `MAX_MEMBERS`=12 / `MAX_VERIFIERS`=10) rather than reconstructed from flags at read time, specifically so `get_round` remains the single call the frontend expects rather than requiring N extra reads.
- Sponsor top-up money is deliberately never added to `RefundLiability`/`MemberContributionTotal` for the covered member — only to `PoolAssignedBalance`, `RoundTopUp`-equivalent pot accounting, and the member's own `SponsorAdvance` debt — so it flows back to the sponsor via `claim_sponsor_remainder` if the pool later aborts, per `docs/plan.md`'s "Sponsor avansı üyenin iade hesabına katılmaz."
- `repay_advance` requires paying the full outstanding debt in one call (no partial-repayment amount parameter), matching the frontend's method signature (`{ pool_id, member }`, no `amount`) and the existing codebase's "exact amount" idiom used elsewhere (`cure_payment`, the old `top_up`).
- Deployed a fresh contract instance rather than trying to migrate the Phase 9/10 instance's state, since Soroban contracts have no upgrade mechanism by design (documented decision since Phase 0) and the API is a breaking change regardless.
- `cancel_unstarted_pool` refunds the sponsor immediately and directly inside the call itself (not via a later `claim_sponsor_remainder`), since `docs/plan.md` frames it as an immediate return ("sponsor kilitlediği tutarın tamamını alır") and no member has paid anything yet at the Filling stage to complicate the accounting.

### Risks / Open Questions

- `scripts/README.md` had never actually been committed to git in any earlier phase (confirmed via `git log -- scripts/README.md` returning nothing) despite being written and referenced in this conversation twice before; it was silently dropped somewhere in an earlier stash/pop/merge sequence. It has been recreated from scratch this phase. Worth double-checking after any future stash operation that every intended file actually landed in the commit, not just the ones git happened to report as conflicted.
- The live Testnet run exercised the happy path only; the new `cancel_unstarted_pool`, `top_up`/`repay_advance`, and `BlockedSettlement` abort paths are covered by the 24 unit tests but not re-proven live in this phase (Phase 10's live abort/refund run was against the now-superseded Phase 9/10 contract instance, not this one). A future phase could re-run `demo_testnet.sh`'s Pool B scenario against the new instance for live abort evidence too.
- `asset`/`amount` on `propose_purchase` are currently required to exactly equal `pool.token` and the round's fixed payout; the frontend passes them but never reads them back from `get_round`'s response, so they function as write-only, checked audit fields for now.
- The old Phase 9/10 contract instance (`CACZQBHHQ3TY33AJIC52MHMCPEKUYO4KQFURHQGPJJ3LNDJZAT6LG4II`) remains live on Testnet with its own pool history; it is no longer the one referenced by `.env`/`frontend/.env` but nothing decommissions it (Soroban contracts cannot be deleted). Should not be confused with the current instance in any submission material.
- The frontend itself was not run (no Node/browser session in this environment); compatibility was verified by direct comparison of `stellar contract info interface` output against the TypeScript source, plus live CLI calls using the exact same field names/shapes the frontend's client constructs. A real wallet-driven click-through test of the frontend against this contract has not been performed.

### Stellar Skills Used

- Stellar Smart Contracts: full state-machine redesign (round-level phases, version-keyed approval invalidation, denormalized read structs), permissionless lifecycle transitions, checked arithmetic throughout, generated-interface cross-checking against an external TypeScript client, and canonical build/deploy.
- Stellar Assets & SAC: reused the existing self-issued demo asset and SAC wrapper; peer-to-peer `repay_advance` transfer (member to sponsor) using the same `require_auth` pattern as every other member-initiated transfer in the contract.
- RPC & Horizon APIs: `stellar contract info interface` used as a machine-readable compatibility check, not just documentation.
- Soroban Common Mistakes: no upgrade-in-place attempt on a deployed contract (fresh instance instead); version-keyed storage instead of manual list-clearing to prevent stale-approval bugs; checks-effects-interactions preserved in every rewritten entrypoint.

### Definition of Done

- Contract rewritten to match the current `docs/plan.md`: completed.
- Contract rewritten to match the frontend's existing client field-for-field: completed and cross-verified via generated interface inspection.
- New unit test coverage for every new mechanism (terms approval, cancellation, sponsor advances, purchase-version binding, both abort reasons): completed, 24/24 passing.
- Canonical WASM build: passed.
- Redeployed to Testnet as a new instance: completed.
- Live end-to-end happy-path run against the new instance with frontend-shaped calls and reads: completed.
- Deployment/demo scripts and docs updated to match: completed.
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 12 - Sponsorless Realignment

### Goal

Remove the sponsor model entirely from the contract, matching `docs/plan.md`'s 19 Eylül sponsorless revision and the frontend client a teammate had already rewritten to that spec (`frontend/src/services/pool.ts`, `frontend/src/types/pool.ts`). A teammate had also already written the target API and scope directly into `docs/IMPLEMENTATION_PLAN.md` (section 6 and the "PHASE 12" entry) before this phase started; that specification was followed literally.

### Changes Made

- Rewrote `contracts/rotating_pool/src/types.rs`: dropped `sponsor` from `Pool`; dropped `required_guarantee`/`guarantee_deposited`; dropped `sponsor_advanced`/`asset`/`amount` from `RoundState` (asset/amount are validated once at `propose_purchase` time against `pool.token`/the round's fixed payout and never need to be persisted, since neither can change before execution); `MemberState` reduced to `{joined_at, received}` (no more `active`/`contributions_paid`, both unused without `leave_pool` or cross-round totals).
- Rewrote `contracts/rotating_pool/src/storage.rs`: removed `GuaranteeBalance`, `RoundTopUp`, `SponsorAdvance`, `AdvanceCovered`, `SponsorRemainderClaimed`, `MemberContributionTotal`, and `TotalRefundLiability` keys entirely. `RefundLiability(pool_id, member)` now means only "this member's deposit into the current, not-yet-settled round" — no cumulative/cross-round tracking, no cached total (each member's own entry is authoritative; nothing needs cross-checking against a stored aggregate since `execute_round`/`claim_refund` keep `PoolAssignedBalance` consistent by construction).
- Rewrote `contracts/rotating_pool/src/error.rs` and `events.rs`: removed every sponsor/guarantee/advance/top-up variant and event (`SponsorOnly`, `InsufficientGuarantee`, `InvalidTopUpAmount`, `TopUpNotAllowedForRecipient`, `RecipientOwesAdvance`, `RecipientMustSelfPay`, `NoOutstandingAdvance`, `SponsorRemainderNotClaimable`, `GuaranteeFunded`, `SponsorAdvanced`, `AdvanceRepaid`, `SponsorRemainderClaimed`).
- Rewrote `contracts/rotating_pool/src/lib.rs`: dropped `fund_guarantee`, `top_up`, `repay_advance`, `claim_sponsor_remainder`, `get_sponsor_advance`, `get_sponsor_remainder_claimed`, and the `sponsor` parameter of `create_pool`/event. `approve_terms`/`start_pool` now only ever check member approval (no sponsor branch, no guarantee gate). `cancel_unstarted_pool` no longer transfers anything (nothing is ever locked during `Filling` now that there is no guarantee) — it just flips the pool to `Aborted` and emits a parameterless `PoolCancelled`. The shared `apply_member_payment` helper lost the advance-covered branch entirely: a round transitions `Collecting`/`Grace` -> `AwaitingPurchase` the moment `round.pot` reaches every member's contribution, full stop — no more "recipient must have self-paid and be debt-clear" special case, because there is no other way for a contribution to enter the pot. `execute_round` now clears **every** member's `RefundLiability` to `0` when the round settles (previously only the recipient's and already-delivered members'), matching "refund entitlement no longer spans rounds," and no longer computes a cross-round `remaining_liability`/solvency figure — the round's assigned balance is exactly its pot, consumed exactly by the payout.
- Rewrote `contracts/rotating_pool/src/test.rs` from scratch: 19 tests covering create-pool validation, no-fund join with seller exclusion, member-only terms propose/approve/version-reset, permissionless start gated on full membership and full **member** approval (no guarantee check), setup-deadline cancellation with **no transfer at all**, deposit/cure round-phase gating and the simplified all-N-members-paid AwaitingPurchase transition, purchase proposal/approval validation and quorum, full two-round happy-path completion, insufficient-balance and failed-transfer rollback, grace-then-abort (`SafetyRecovery`), cure-during-grace recovery, purchase-deadline-expiry abort (`BlockedSettlement`), stored-role-auth requirements, multi-pool isolation, verifier-policy exclusion/bounds, and — the plan's canonical scenario, written as its own test — round 1 fully pays and settles, round 2's already-delivered member stops paying entirely, only the still-paying member's round-2 deposit is refundable, and the never-claimable first member's attempt is rejected.
- Deployed the rewritten contract to Testnet as a **new** instance (the Phase 9-11 sponsor-based instances are API-incompatible and left untouched as historical artifacts).
- Rewrote `scripts/demo_testnet.sh` for the sponsorless flow (no sponsor identity, no guarantee funding, no remainder claim); combined the plan's canonical demo scenario and a live cure-during-Grace demonstration into the same two-pool run to keep the live-evidence pass efficient. `scripts/deploy_testnet.sh` needed no changes (it only builds/deploys the WASM and resolves a settlement asset). Updated `scripts/README.md` to match.
- Updated `.env` and `frontend/.env` (local, gitignored) with the new contract ID; updated `README.md`'s delivery checklist and `docs/IMPLEMENTATION_PLAN.md`'s status/section 1 to reflect Phase 12 completion.

### Files Changed

- `contracts/rotating_pool/src/types.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/events.rs`
- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `scripts/demo_testnet.sh`
- `scripts/README.md`
- `.env` (local only, gitignored)
- `frontend/.env` (local only, gitignored)
- `README.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/IMPLEMENTATION_LOG.md`

### Commands Run

- `cargo test --workspace`
- `cargo fmt --all` / `cargo fmt --all -- --check`
- `stellar contract build`
- `stellar contract info interface --wasm target/wasm32v1-none/release/rotating_pool.wasm` (cross-checked `Pool`/`RoundState`/`MemberStatusView` field names one by one against `frontend/src/services/pool.ts`'s `mapPool`/`mapRound`/`mapMember`)
- `stellar contract deploy --wasm ... --source stellerpool-deployer --network testnet --alias rotating_pool_v3`
- Full live lifecycle via `stellar contract invoke --id <new contract> ...`: `create_pool`, `join_pool` x2, `propose_terms`, `approve_terms` x2 (members only), `start_pool`, `deposit`, `mark_overdue`, `cure_payment`, `propose_purchase`, `approve_purchase` x2, `execute_round`, `deposit` (round 2), `mark_overdue`, `abort_pool`, `claim_refund` x2, `get_pool`, `get_round`, `get_member_status`

### Test Results

- Unit tests passed: 19 passed, 0 failed.
- Canonical Protocol 28 WASM build passed. Optimized WASM size: 31,165 bytes (down from 39,823 bytes in the sponsor-based v8 build, reflecting the removed sponsor code paths). WASM SHA-256: `8535d21fc83b242486fd5865b08ae433908a5cf77067215e5b27fdd5f1a55b2f`. Exported functions: 21 — exactly the 17 the frontend's `EXPECTED_METHODS` lists plus `version`/`next_pool_id`/`has_pool`/`get_refund_claim`. No `sponsor`/`guarantee`/`advance`/`top_up` symbol anywhere in the interface.
- `stellar contract info interface` output compared field-by-field against `frontend/src/types/pool.ts`: `Pool` (20 fields, no `sponsor`/`requiredGuarantee`/`guaranteeDeposited`), `RoundState` (13 fields, no `sponsorAdvanced`), `MemberStatusView` (`{refundable, received}`) — exact match.
- **Live Testnet deployment**: new contract ID `CCAKOEC34WVBKQ427KT5PI5GMPPKSGBHCWBI7FO4GNG247KKDUH67AZH` (WASM upload tx `83bbb0895b7a7952d5347603b296c7dfd3845444e3c53940682e1a76426a456c`, instance creation tx `730248d5e027d91422114620df41351dc7348e30177db3685e9fc45889783d97`). Reused the existing funded demo identities and `STLP` demo asset (SAC `CAOV35NPIJXHWA7QPXXERRJQ4ZTDUGAEHA7FKB6QIOTIOTI62B35ZNWI`).
- **Live combined cure-path + plan-demo-scenario run** (`pool_id: 1`, contribution 1 STLP, member_limit 2, `round_duration: 40`, `grace_duration: 40`, `purchase_duration: 900`):
  - `create_pool` tx `486f97b0d56b08ddb5473586b63ea235add8bbd98dca1c2a6a060ec03200f2dc`; both `join_pool`; `propose_terms` (`approval_threshold: 2` for 2 verifiers); two `approve_terms` (members only, no sponsor call anywhere in the sequence); `start_pool` tx `0d829021ab4cdec22ba824c23bff9fd7fdb93e4e3627f1576dd2f3d45a310abf` (permissionless, no guarantee check).
  - Round 1: member1 deposits immediately, member2 does not; after the real collect deadline passed, `mark_overdue` tx `9e2db4eec262987cdf77f3c4d79b470f8aa77f95b8ab45ec324533ff1b497ba1` moved the round to `Grace`; member2 then called `cure_payment`, which both paid their contribution **and** transitioned the round straight to `AwaitingPurchase` in the same transaction (confirmed via `RoundAwaitingPurchase` in the same event log) — the cure path proven live, not just in unit tests.
  - `propose_purchase`/two `approve_purchase`/`execute_round` (tx `fcf9190504f4a9d0a1c84a3d32cfa217ed977f5feab9160eb99c9215564513ab`) paid the demo seller 2 STLP and advanced to round 2.
  - Round 2: only member2 (this round's recipient) deposits; member1 (round 1's already-delivered recipient) never pays. After the real collect deadline, `mark_overdue` tx `05838e108c3555e1986ec0ae9a6ccec8c728583a828ca10e98f175953c8e5b7f` moved round 2 to `Grace`; after the real grace deadline, `abort_pool` tx `3ccb735023d0a124f42bbf624fd94bb0f94e3e7f31c6ba83e4196a2d78610cb9` aborted the pool with `reason: "SafetyRecovery"`.
  - Read back before any claim: `get_member_status(member1) = {refundable: 0, received: true}`, `get_member_status(member2) = {refundable: 10000000, received: false}` — exactly the plan's rule (round 1's recipient has nothing left to reclaim; the still-paying member's current-round deposit is fully refundable).
  - `claim_refund(member1)` failed with contract error `#21` (`RefundNotClaimable`, entitlement `0`) as expected. `claim_refund(member2)` succeeded, transferring exactly `10000000` stroops back and emitting `RefundClaimed`.

### Decisions

- Followed the teammate's already-written `docs/IMPLEMENTATION_PLAN.md` section 6/12 specification literally rather than re-deriving scope from `docs/plan.md` prose independently, since it was written specifically to match the frontend client this phase had to stay compatible with, and cross-checking two independently-written specs against each other would have only introduced risk of a spurious mismatch.
- Combined the "cure path" and "plan demo scenario" live-evidence requirements into a single two-pool script run (Pool A: happy path ending with an explicit cure-during-Grace step in its first round; Pool B: the canonical round-1-settles/round-2-defaults/abort/refund scenario) rather than four separate runs, to keep the live verification pass efficient while still exercising every required path.
- `asset`/`amount` on `propose_purchase` remain in the function signature and event (the frontend sends and needs to send them) but are not persisted on `RoundState`, since both are fully determined by immutable pool fields (`pool.token`, `contribution_amount * member_limit`) at the moment they're validated — persisting them would be redundant state that could theoretically drift from the values that were actually checked.
- Deployed a fresh contract instance rather than attempting any kind of migration from the v8 sponsor-based instance; Soroban has no upgrade mechanism by design (a standing decision since Phase 0), and the storage layout changed regardless.

### Risks / Open Questions

- The now three live Testnet contract instances (Phase 9/10's `CACZQBHHQ3TY33AJIC52MHMCPEKUYO4KQFURHQGPJJ3LNDJZAT6LG4II`, Phase 11's `CBC5DGFAQMEGVJVC6W3Z3J7SNMMFK5LQO3LMZVVJE5YFOPSF4CKW3U4Q`, and this phase's `CCAKOEC34WVBKQ427KT5PI5GMPPKSGBHCWBI7FO4GNG247KKDUH67AZH`) all remain live and queryable; only the last is referenced by `.env`/`frontend/.env`. None can be deleted (no Soroban contract-deletion mechanism); submission material must be careful to cite only the current one.
- The accepted economic risk this phase's tests and live run demonstrate — an early recipient who stops paying in a later round leaves that round's other payer(s) needing to individually claim a refund, and the early recipient's own already-settled allocation is never clawed back — is by design per `docs/plan.md`, not a bug; it must stay visible in any demo or pitch material, not just in this log.
- No dedicated TTL test exists yet for the sponsorless storage layout (same gap as every prior phase; Phase 8's full security/invariant suite was explicitly skipped for hackathon time pressure and that decision still stands).
- The frontend itself was not run (no Node/browser session in this environment); compatibility rests on the generated-interface-vs-TypeScript-source comparison plus live CLI calls using the frontend's exact field shapes, not an actual wallet-driven click-through.

### Stellar Skills Used

- Stellar Smart Contracts: further state-machine simplification (dropping a whole accounting dimension — cross-round liability tracking — once the product rule that motivated it went away), generated-interface cross-checking against an external TypeScript client, and canonical build/deploy.
- Stellar Assets & SAC: reused the existing self-issued demo asset and SAC wrapper across three contract generations without re-issuing anything.
- Soroban Common Mistakes: no partial/duplicate storage removal (every sponsor-era key, error, and event was deleted together, not left dangling); fresh contract instance instead of an unsupported in-place upgrade; checks-effects-interactions preserved in every rewritten entrypoint.

### Definition of Done

- Contract matches `docs/IMPLEMENTATION_PLAN.md` section 6's 17-method API plus the four read helpers, with no sponsor/guarantee/advance/top-up symbol remaining: completed, verified via `stellar contract info interface`.
- Parameter and field names match the frontend client exactly: completed, verified by direct comparison.
- Plan demo scenario passes as a unit test and live on Testnet: completed.
- Cure path runs live: completed.
- New contract ID, WASM hash, and key transaction hashes recorded: completed (above).
- Frontend untouched: completed.

### Status

NEEDS REVIEW

## Phase 13 - Draw Mode and 30 Members

### Goal

Implement `docs/CONTRACT_HANDOFF.md` in full: add Draw-mode (kura) recipient selection alongside the existing Fixed order, and raise `MAX_MEMBERS` from 12 to 30 while removing the per-member storage loops that would no longer scale. A teammate had written the handoff (and a matching frontend, gated behind on-chain capability detection) before this phase started; that specification was followed literally, same working pattern as Phase 12.

### Changes Made

- `contracts/rotating_pool/src/types.rs`: added `OrderMode { Fixed, Draw }`; added `RoundPhase::AwaitingDraw` between `Grace` and `AwaitingPurchase`; added `Pool.order_mode: OrderMode` (declared right after `member_limit`, matching `create_pool`'s new parameter position); changed `RoundState.recipient` from `Address` to `Option<Address>` (`None` in Draw mode until drawn).
- `contracts/rotating_pool/src/storage.rs`: deleted the `RefundLiability(pool_id, member)` key and its `read_refund_liability`/`write_refund_liability` helpers entirely — no replacement key was added, since the value they held (a member's current-round deposit) is already fully recoverable from the existing `Deposit(pool_id, round, member)` flag plus `pool.current_round`/`contribution_amount`.
- `contracts/rotating_pool/src/error.rs`: added `NotDrawPool = 64` (`draw_recipient` called on a `Fixed`-order pool).
- `contracts/rotating_pool/src/events.rs`: added `order_mode: OrderMode` to `PoolCreated`; added `RoundAwaitingDraw { pool_id, round, purchase_deadline }` (Draw-mode's equivalent of `RoundAwaitingPurchase`, fired when a round becomes fully funded) and `RecipientDrawn { pool_id, round, recipient }`.
- `contracts/rotating_pool/src/lib.rs`:
  - `create_pool` gained an `order_mode: OrderMode` parameter (positioned right after `member_limit`, per the handoff and the frontend's `pool.ts`).
  - New `draw_recipient(caller, pool_id) -> Address`: `caller.require_auth()` but no membership/role gate ("anyone may call it"); requires `pool.status == Active`, `pool.order_mode == Draw` (else `NotDrawPool`), and `round.phase == AwaitingDraw` (else `RoundNotReady` — this one error code now covers both "not fully funded yet" and "already drawn this round," since a successful draw immediately advances the phase); builds the candidate list from `pool.members` filtered to `MemberState.received == false`; picks uniformly via `env.prng().gen_range::<u64>(0..candidates.len())`, or deterministically (no PRNG call) when exactly one candidate remains; sets `round.recipient`, flips the phase to `AwaitingPurchase`, and emits `RecipientDrawn`.
  - `apply_member_payment`'s pot-complete branch now matches on `pool.order_mode`: `Fixed` behaves exactly as before (→ `AwaitingPurchase`, emits `RoundAwaitingPurchase`); `Draw` sets `round.purchase_deadline` at the same moment but moves to `AwaitingDraw` and emits `RoundAwaitingDraw` instead — the single `purchase_duration` window covers both the draw and the purchase steps for Draw pools, per the handoff.
  - `propose_purchase` unwraps `round.recipient` (now `Option`) once and compares it to the caller.
  - `execute_round`: unwraps `round.recipient`; builds the next round's recipient as `Some(...)` for `Fixed` or `None` for `Draw`; the per-member `has_deposit` check loop and the `recipient_found` loop were both deleted (`round.pot == payout_amount`, checked immediately above, already implies exactly `member_limit` distinct deposits, and `round.recipient` is guaranteed `Some` by the phase gate), replaced by a single cheap in-memory check `round.paid.len() != pool.member_limit` (`round.paid` was already loaded, so this adds no storage reads); the per-member `RefundLiability` reset loop is gone — nothing needs resetting now that refund entitlement is derived, not stored.
  - `claim_refund` and `get_member_status`: `refundable`/entitlement is now computed as `has_deposit(pool_id, pool.current_round, member) && !has_refund_claimed(...) ? contribution_amount : 0` instead of reading a stored `RefundLiability`. This is exactly equivalent to the old semantics (verified by every pre-existing refund-related test passing unchanged) because a settled round's deposits are never re-examined (only `pool.current_round`, the pool's still-open round when it aborts, is ever checked) and `RefundLiability` was already only ever written to the current-round deposit amount or zero.
  - `start_pool`: the per-member `has_terms_approval` storage-read loop was replaced by `pool.terms_approvals.len() == pool.member_limit`, sound because `approve_terms` only ever appends a member once per terms version (guarded by both the `NotApprover`/membership check and the `AlreadyApprovedTerms` check), so a duplicate-free subset of `pool.members` whose length equals `pool.member_limit` (already confirmed equal to `pool.members.len()` by the preceding `PoolNotFull` check) must be the full member set.
  - `validate_recipient_order` now branches on `pool.order_mode`: `Draw` requires an empty order; `Fixed` keeps the existing full-permutation, no-duplicates check.
  - `MAX_MEMBERS` raised 12 → 30; `CONTRACT_VERSION` bumped to 10.
- `contracts/rotating_pool/src/test.rs`: extended from 19 to 27 tests. New: `draw_mode_full_flow_excludes_repeat_winners_with_four_members` and `..._with_thirty_members` (every member wins exactly once, no repeats, the pool reaches `Completed`, and the last round is necessarily deterministic since only one candidate remains); `draw_recipient_rejected_before_round_fully_funded_and_after_first_draw`; `draw_recipient_rejected_on_fixed_order_pool`; `propose_terms_recipient_order_must_match_order_mode` (empty/duplicate rejected for `Fixed`, non-empty rejected for `Draw`); `draw_mode_awaiting_draw_deadline_aborts_and_refunds_current_round_only`; `draw_mode_plan_demo_scenario_early_winner_defaults_next_round` (the plan's canonical scenario, replayed for Draw mode: the round-1 draw winner stops paying in round 2, so round 2 never reaches `AwaitingDraw` and only the still-paying members' round-2 deposits are refundable); `thirty_member_round_operations_stay_within_mainnet_resource_limits` (captures and prints real `InvocationResources` for `deposit`/`draw_recipient`/`execute_round` at `member_limit = 30`). Two pre-existing tests were adjusted for the `Option<Address>` recipient type and the raised `MAX_MEMBERS` bound; no other pre-existing test needed behavioral changes.
- `scripts/demo_testnet.sh`: `create_and_start_pool` gained an `order_mode` argument (passed through to `create_pool --order_mode` and used to choose between an empty or a full `propose_terms --recipient_order`); added a third scenario, **Pool C**, exercising Draw mode live: two members pay into round 1, `draw_recipient` is called by an outsider account (demonstrating the "anyone may call it" rule), the drawn recipient's purchase is proposed/approved/executed, and round 2 repeats — deterministically drawing the one remaining member.
- `scripts/README.md`: documented Pool C and the v10 capabilities.
- Deployed the rewritten contract to Testnet as a **new** instance (the Phase 9-12 instances are left untouched as historical artifacts; Soroban has no upgrade/migration mechanism).
- Updated `.env`/`frontend/.env` (local, gitignored) with the new contract ID and `VITE_MAX_MEMBERS=30`; updated `.env.example`'s default from 12 to 30. `docs/IMPLEMENTATION_PLAN.md`'s status header, section 1, and the (already-drafted) Phase 13 entry were updated to "completed." `docs/CONTRACT_HANDOFF.md`'s status line was updated to point at the new contract ID and this log entry; its task-definition body was left unchanged as the historical spec.
- **`README.md` was intentionally not touched**, per an explicit standing instruction from the user earlier in this session ("readme güncellemlerini iptal et artık readmeye dokunma" — stop touching README.md). Its "Kontrat ve dağıtım kanıtı" section still needs the new contract ID added by whoever next edits it.

### Files Changed

- `contracts/rotating_pool/src/types.rs`
- `contracts/rotating_pool/src/storage.rs`
- `contracts/rotating_pool/src/error.rs`
- `contracts/rotating_pool/src/events.rs`
- `contracts/rotating_pool/src/lib.rs`
- `contracts/rotating_pool/src/test.rs`
- `contracts/rotating_pool/test_snapshots/**`
- `scripts/demo_testnet.sh`
- `scripts/README.md`
- `.env` (local only, gitignored)
- `frontend/.env` (local only, gitignored)
- `.env.example`
- `docs/CONTRACT_HANDOFF.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/IMPLEMENTATION_LOG.md`

Deliberately not changed: `README.md` (standing instruction, see above), `docs/plan.md` (frontend/product-owned source of truth), `scripts/deploy_testnet.sh` (still only builds/deploys the WASM and resolves a settlement asset; never calls `create_pool`), and everything under `frontend/src/` (separate workstream, already written and merged ahead of this phase).

### Commands Run

- `cargo test -p rotating-pool` (twice: before and after `cargo fmt`)
- `cargo fmt --all` / `cargo fmt --all -- --check`
- `cargo test -p rotating-pool thirty_member_round_operations -- --nocapture` (to capture the printed resource numbers below)
- `stellar contract build`
- `stellar contract info interface --wasm target/wasm32v1-none/release/rotating_pool.wasm` (cross-checked against `frontend/src/services/pool.ts` / `types/pool.ts` field-for-field)
- `stellar contract bindings typescript --wasm target/wasm32v1-none/release/rotating_pool.wasm --output-dir ...` (independent second cross-check of method/parameter names and order against the same frontend files)
- `scripts/deploy_testnet.sh` (builds, deploys the new instance, resolves the existing Testnet-USDC SAC, writes `.env`/`frontend/.env`)
- `scripts/demo_testnet.sh --contract CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB` (live: Pool A happy path, Pool B canonical default/abort/refund, Pool C Draw mode — full output is the evidence below)

### Test Results

- Unit tests: **27 passed, 0 failed** (19 pre-existing + 8 new Draw/30-member tests).
- Canonical Protocol 28 WASM build passed. Optimized WASM size: 34,070 bytes (up from 31,165 bytes in the v9 build, reflecting the added Draw-mode logic). WASM SHA-256: `6cc5e1a031fbf30575042461d21c8bf02a7df8f9d9a9f48ec8592388dd2512b4`. Exported functions: 22 — the previous 21 plus `draw_recipient`.
- `stellar contract info interface` and `stellar contract bindings typescript` both matched `frontend/src/services/pool.ts` / `types/pool.ts` exactly: `create_pool(creator, token, contribution_amount, member_limit, order_mode, round_duration, grace_duration, purchase_duration, setup_deadline, demo_seller)`; `draw_recipient({caller, pool_id}) -> string`; `Pool.order_mode: OrderMode`; `RoundState.recipient: Option<string>`; `RoundPhase` includes `"AwaitingDraw"`; no `fund_guarantee`/`top_up` symbol anywhere (so the frontend's `legacySponsor` capability check reads `false` and `supportsDraw` reads `true`).
- **30-member resource check** (`thirty_member_round_operations_stay_within_mainnet_resource_limits`, real measured `InvocationResources`, native-Rust execution — a lower bound on the real Wasm-host cost, not the authoritative figure): against `InvocationResourceLimits::mainnet()` (`instructions: 400,000,000`, `mem_bytes: 41,943,040`, `disk_read_entries`/`write_entries: 200` each) —
  - `deposit` (30th of 30, the one that also flips the round to `AwaitingDraw`): `instructions: 1,102,647`, `mem_bytes: 283,850`, `disk_read_entries: 0`, `write_entries: 6`.
  - `draw_recipient` (30 candidates read down to the winner): `instructions: 1,622,427`, `mem_bytes: 208,059`, `disk_read_entries: 0`, `write_entries: 2`.
  - `execute_round`: `instructions: 1,351,920`, `mem_bytes: 360,814`, `disk_read_entries: 1`, `write_entries: 7`.
  - All three are ~0.3-0.4% of the Mainnet instruction budget and single-digit-percent of the entry-count budgets — comfortable margin, with `draw_recipient`'s ~30 `MemberState` reads (one per candidate, needed to know who has already received) as the dominant but still trivial cost. soroban-sdk 28's `Env::default()` also enforces these same Mainnet limits on every invocation in every test by default, so the 4-member and 30-member Draw full-flow tests (27+ contract calls each) are themselves passing resource checks, not just this dedicated test.
- **Live Testnet deployment**: new contract ID `CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB` (WASM upload tx `d0f9e5cbbf023a9dde46ab2017b362f9ea8dc6a5ac643b109cabd304ba647867`, instance creation tx `5c7ae11430ec06055f612027ab046888f88ddcd83d3addd6c6328bd92d7a91a1`); read-only smoke test `version()` returned `10`. Reused the existing Testnet-USDC SAC (`CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA`, for `deploy_testnet.sh`'s settlement-asset resolution) and the existing self-issued `STLP` demo asset (SAC `CAOV35NPIJXHWA7QPXXERRJQ4ZTDUGAEHA7FKB6QIOTIOTI62B35ZNWI`, for the live demo below).
- **Live combined demo run** (`scripts/demo_testnet.sh`, contribution 1 STLP, `member_limit: 2`):
  - **Pool A** (`pool_id: 1`, Fixed, `round_duration: 1800s`, happy path): `create_pool` tx `7f14ac89fcc63b43396c92a1770258fd9b4e9e1a6db5499c26e4c0d1fb3ee94e` (event shows `order_mode: "Fixed"`); both rounds fully funded and settled without needing Grace — round 1 `RoundPaid` tx `78bc813cae56605d08dedf5cf303e158f5a0731517663e1ac2177ae2ef177d2b`, round 2 `RoundPaid`+`PoolCompleted` tx `ad5a8e6e9a379816c18fb23f84d6a49f497b42a54987f2c861c9968bb3993a01`.
  - **Pool B** (`pool_id: 2`, Fixed, `round_duration: 40s`, `grace_duration: 40s`, the plan's canonical scenario): `create_pool` tx `5c20843227aea3656ca45c6a1644b6bf95df1fec033e3e4d67514d13e23755cd`; round 1 both pay and settle (`RoundPaid` tx `a00b3cf770d21634f64bebbbf75b46a897840061c20d6a2d3be33db06eb70ad6`); round 2 only the non-recipient member pays (`ContributionDeposited` tx `f513244596c4ba07d929fa8820d9aa92e1965b2ae5efdfe589b985528fa048fc`); after the real collect and grace deadlines the script's polling loop called `mark_overdue` then `abort_pool` (their individual tx hashes were not captured — the script pipes that polling call's stdout through `grep -q` to detect success without printing it — but the resulting state change is directly confirmed: `PoolAborted`/`SafetyRecovery` took effect, since the very next call, `claim_refund`, succeeded); `claim_refund(member2)` tx `d4171a4866f9e7fa15db371c00c26737ed132c33baba0b3363f482bcf4d60fd1` transferred exactly `10000000` stroops (only the round-2 deposit — round 1's payout stayed with the seller, unrecoverable by design).
  - **Pool C** (`pool_id: 3`, **Draw mode**, `round_duration: 1800s`) — the new capability, live: `create_pool` tx `38dec31c1c9cfc2388ce1f6adda32ad7d97c1ecd77e245b9f6226a52d195e554` (event shows `order_mode: "Draw"`); `propose_terms` was called with an empty `recipient_order`. Round 1: both members deposit, the second deposit fires `RoundAwaitingDraw` (tx `b1b05c4dfc1c3d40eb5b12572b41fa640c1385e491b592a774a872798756f4d4`, `purchase_deadline` already set); `draw_recipient`, called from the **deployer/issuer account** — not a pool member or verifier, demonstrating the "anyone may call it" rule — tx `11845a01bbfe010755af7e74588b408b469066ee8f9247dd668d12fdd0ea2822`, emitting `RecipientDrawn(round: 1, recipient: GDANLW6SJYQDUMVVFT77IJCOZ4X2CGL5GAUVBC2PQMVTE666YMU57LXH)`; that recipient's purchase settles via `execute_round` tx `a253b14aee6bbd21329eb085ac7532fb984cabe3a92ee44f0bda698961e82609`. Round 2: both members deposit again (everyone pays every round, including the round-1 winner) — `RoundAwaitingDraw` tx `6e0f34ffa9e2b81110a032d22553824f32555bed3208b6b47b4524955f5bb8a3`; `draw_recipient` tx `a9266d236b33ab5b262942e5c9734f81e6dbcb641eb040d199866f7193ed74f9` emits `RecipientDrawn(round: 2, recipient: GAMTMCPL2AVYRLOGJKPJPDXIXSYEPXFHRQPBTLJ5MV2C5RIQHUDC7Z57)` — **the other member**, confirming live, on real Testnet state, that the round-1 winner was correctly excluded from the round-2 candidate pool (deterministic here since exactly one candidate remained); round 2 settles via `execute_round` tx `22b4c62efd755a75ca4c4d1f8a467c9a02f48890c49dba1f3796b50054bb72fa`, emitting `PoolCompleted`.

### Decisions

- Followed `docs/CONTRACT_HANDOFF.md` literally rather than re-deriving scope, for the same reason as Phase 12: it was written specifically to match a frontend that was already built and merged against it, and second-guessing an already-agreed spec would only risk introducing a spurious mismatch.
- Deleted the `RefundLiability` storage key outright rather than keeping it and merely fixing the reset loop, because the value it held was always fully derivable from data the contract already stores (`Deposit` flags plus `pool.current_round`/`contribution_amount`); keeping a redundant, independently-writable copy would have reintroduced exactly the kind of "could theoretically drift from what was actually checked" risk Phase 12's log flagged for a different field. This is also what let `execute_round` drop its O(member_limit) reset loop entirely rather than merely reduce its cost.
- Used `caller.require_auth()` on `draw_recipient` without any membership/role check, rather than making it fully parameterless like `execute_round`/`mark_overdue`/`abort_pool`. The frontend's `pool.ts` already defines `draw_recipient` as taking an explicit `caller` argument (consistent with `deposit`/`cure_payment`'s pattern of an acting address that must sign), and the handoff's "herkes çağırabilir, özel yetki yok" ("anyone may call it, no special permission") reads as "no role/membership gate," not "no signature requirement." The live demo deliberately called it from the deployer/issuer identity — neither a pool member nor a verifier — to demonstrate this concretely.
- Reused `RoundNotReady` for both "round not yet fully funded" and "already drawn this round" in `draw_recipient`, rather than adding a second new error code, since both are instances of "the round isn't in `AwaitingDraw` right now" and every other entrypoint in the contract already reuses `RoundNotReady` the same way for analogous phase mismatches.
- `execute_round`'s deleted invariant loop was replaced with `round.paid.len() != pool.member_limit` (a free, already-in-memory check) rather than removed outright, to keep a cheap defense-in-depth sanity check without reintroducing any O(member_limit) storage reads.
- Deployed a fresh contract instance rather than any kind of migration, consistent with every prior phase (Soroban has no upgrade mechanism, and the storage/type layout changed regardless — `RoundState.recipient`'s type alone would make old rounds unreadable by the new contract).
- Did not touch `README.md`, per the user's explicit standing instruction earlier in this session to stop editing it; the handoff's instruction to record the new contract ID there is consequently only partially fulfilled (done in `IMPLEMENTATION_LOG.md`/`IMPLEMENTATION_PLAN.md`/`CONTRACT_HANDOFF.md`, not in `README.md`).

### Risks / Open Questions

- The now four live Testnet contract instances (Phase 9/10's `CACZQBHHQ3TY33AJIC52MHMCPEKUYO4KQFURHQGPJJ3LNDJZAT6LG4II`, Phase 11's `CBC5DGFAQMEGVJVC6W3Z3J7SNMMFK5LQO3LMZVVJE5YFOPSF4CKW3U4Q`, Phase 12's `CCAKOEC34WVBKQ427KT5PI5GMPPKSGBHCWBI7FO4GNG247KKDUH67AZH`, and this phase's `CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB`) all remain live and queryable; only the last is referenced by `.env`/`frontend/.env`. Submission material must cite only the current one — and since `README.md` was intentionally not updated this phase, whoever next edits it needs to swap in this ID.
- On-chain randomness stays hackathon-grade by design: `env.prng()` is seeded from consensus data that is public and not validator-secret (see the module-level warning in `soroban-sdk`'s own `prng` module, and `docs/CONTRACT_HANDOFF.md` section 1.5). A real deployment would need commit-reveal or an external randomness oracle; this is an explicit, documented limitation, not a gap to silently fix.
- The 30-member resource numbers are a lower bound, not the authoritative figure — they come from native (non-Wasm) test execution, which `soroban-sdk`'s own documentation says underestimates real Wasm-host cost. The margin to the Mainnet limits (roughly 250-350x on instructions) is large enough that this is very unlikely to matter, but a true 30-member live Testnet run (30 funded identities, real deposits) would be the fully authoritative check and was not performed this phase (impractical within the session's time budget; the live demo instead used 2-member pools to keep the run fast while still proving the Draw mechanism live).
- The accepted economic risk carried over from Phase 12 — an early recipient (whether by Fixed order or by draw) who stops paying in a later round leaves that round's other payer(s) needing to individually claim a refund, and the early recipient's own already-settled allocation is never clawed back — is unchanged by Draw mode or by raising `MAX_MEMBERS`, exactly as `docs/plan.md` section 2 and the handoff's section 5 both state. It must stay visible in any demo or pitch material.
- The frontend itself was not run (no Node/browser session in this environment); compatibility rests on the generated-interface and generated-TypeScript-bindings comparisons against the frontend's source, plus the live CLI demo using the frontend's exact field/parameter shapes — not an actual wallet-driven click-through.

### Stellar Skills Used

- Stellar Smart Contracts: `env.prng()` for on-chain pseudo-randomness with an explicit documented limitation, a new round sub-phase (`AwaitingDraw`) threaded through the existing state machine without duplicating its deadline/abort logic, and a storage-key deletion (`RefundLiability`) in favor of deriving the same value from already-authoritative data.
- Resource/fee metering: `env.cost_estimate()` (`resources()`/`budget()`) and `InvocationResourceLimits::mainnet()` (enforced by default in `soroban-sdk` 28 test `Env`s) to get concrete, checkable evidence that raising `MAX_MEMBERS` to 30 stays well inside real network limits, instead of relying on code inspection alone.
- Stellar Assets & SAC: reused the existing self-issued demo asset and both SAC wrappers across a fourth contract generation without re-issuing anything.
- Soroban Common Mistakes: no partial storage removal (the `RefundLiability` key, its read/write helpers, and every reference to it were deleted together); fresh contract instance instead of an unsupported in-place upgrade; checks-effects-interactions preserved in `draw_recipient` and every rewritten entrypoint; verified interface compatibility two independent ways (`info interface` and generated TypeScript bindings) rather than trusting one tool's output.

### Definition of Done

- `cargo test` covers Fixed and Draw, including a 30-member full flow: completed (27/27 passing).
- Testnet resource simulation for a 30-member pool stays under the read/write-entry and CPU limits, recorded in this log: completed (native-execution lower bound; see Risks above for the authoritative-figure caveat).
- Frontend generated-bindings comparison shows no parameter/field mismatch: completed, via both `stellar contract info interface` and `stellar contract bindings typescript`.
- New contract ID, WASM hash, and key transaction hashes recorded: completed (above).
- Frontend untouched: completed.
- `README.md` untouched (standing instruction this session): completed — flagged above as still needing a follow-up edit by whoever next updates it.

### Status

NEEDS REVIEW

## Phase 14 - Anchor Backend (SEP-1/10/24, TRYT test varlığı)

### Goal

Doldurmak için `docs/HACKATHON_SUBMISSION_TASKS.md`'nin "Backend ekibi" P0 bölümünde
tanımlanan en kritik boşluğu: proje bir backend'e sahip değildi ve anchor entegrasyonu
`frontend/src/lib/anchor.ts` üzerinden doğrudan SDF'in `testanchor.stellar.org`'una
konuşuyordu (test varlığı, sunucu tarafı bize ait değil). Kendi işlettiğimiz minimal ama
gerçek bir SEP-1/10/24 anchor sunucusu kurmak — el kitabının en ağırlıklı alt kriteri
(Ecosystem Fit → Anchor/Local Payments) burada.

### Changes Made

- Yeni `backend/` (Node.js + TypeScript + Express, `@stellar/stellar-sdk ^17.1.0` —
  frontend'le aynı sürüm) projesi kuruldu: `package.json`, `tsconfig.json` (ESM,
  NodeNext), `.env.example`, `.gitignore` (`.env`/`node_modules`/`dist` hariç tutulur).
- `src/config.ts`: ortam değişkenlerinden yapılandırma; `homeDomain`/`webAuthEndpoint`/
  `sep24Server` `PUBLIC_BASE_URL`'den türetilir (tek değişkeni değiştirmek her şeyi
  günceller).
- `src/lib/stellar.ts`: `TRYT` varlık tanımı (issuer'dan); `sendTry()` dağıtım hesabından
  kullanıcıya gerçek bir Horizon `Payment` işlemi gönderir; hedefte trustline yoksa
  `NoTrustlineError` fırlatır (trustline'ı yalnızca hesap sahibi açabilir — bu istisnayı
  çağıran taraf SEP-24 `pending_trust` durumuna çevirir).
- `src/lib/jwt.ts`, `src/lib/authMiddleware.ts`: SEP-10 oturum JWT'si imzalama/doğrulama
  ve `Authorization: Bearer` middleware'i.
- `src/lib/store.ts`: SEP-24 işlem kayıtları için bellek-içi depo (bilinçli
  basitleştirme, bkz. `backend/README.md`).
- `src/routes/wellKnown.ts`: `GET /.well-known/stellar.toml` — `frontend/src/lib/anchor.ts`
  `resolveAnchor()`'ın okuduğu tüm alanlarla (`WEB_AUTH_ENDPOINT`, `TRANSFER_SERVER_SEP0024`,
  `SIGNING_KEY`, `NETWORK_PASSPHRASE`, `CURRENCIES`), varlığın test amaçlı olduğunu açıkça
  belirten `DOCUMENTATION`/`desc` alanlarıyla.
- `src/routes/auth.ts`: SEP-10 `GET /auth` (challenge üretimi, `WebAuth.buildChallengeTx`)
  ve `POST /auth` (`WebAuth.readChallengeTx` + `WebAuth.verifyChallengeTxSigners` ile
  doğrulama, JWT dönüşü) — `frontend/src/lib/anchor.ts`'nin zaten kullandığı aynı SDK
  modülünün (`WebAuth`) sunucu tarafı karşılığı.
- `src/routes/sep24.ts`: `GET /sep24/info`, `POST /sep24/transactions/deposit/interactive`
  (Bearer korumalı, işlem açar), `GET /sep24/transaction` (Bearer korumalı; durum
  `pending_trust` ise ödemeyi burada sessizce yeniden dener — ayrı bir "retry" ucu
  gerektirmez).
- `src/routes/interactive.ts`: sunucu tarafında render edilen basit bir HTML form
  (tutar girişi + "TRY yatırdım, onayla (test)" butonu, sayfanın kendisinde "gerçek banka
  entegrasyonu yoktur" uyarısı); onay sonrası gerçek ödemeyi dener, sonucu SEP-24 `callback=
  postMessage` sözleşmesine uygun şekilde `window.opener`'a bildirir ve pencereyi kapatır.
- `src/server.ts`: Express uygulaması, `cors`/`json`/`urlencoded` middleware'leri, `/health`.
- `scripts/setup-issuer.ts`: tek seferlik kurulum — issuer + dağıtım hesabı üretir,
  Friendbot ile fonlar, dağıtım hesabına `TRYT` trustline'ı açar, issuer'dan dağıtıma
  1.000.000.000 birimlik arz gönderir, `.env`'e yapıştırılacak satırları yazdırır.
- `scripts/test-flow.mjs`: uçtan uca doğrulama scripti (aşağıda, gerçek Testnet
  işlemleriyle çalıştırıldı).

### Files Changed

- `backend/**` (yeni — `DEPLOY.md` ve `.tools/` için `.gitignore` girdisi dahil)
- `frontend/.env` (local only, gitignored) — `VITE_ANCHOR_HOME_DOMAIN` tünel adresine yazıldı
- `docs/IMPLEMENTATION_LOG.md` (bu bölüm)
- `docs/HACKATHON_SUBMISSION_TASKS.md` (backend bölümü güncellendi)

### Commands Run

- `npm install` (backend/)
- `npx tsc --noEmit -p tsconfig.json` (temiz)
- `npm run setup-issuer` — gerçek Testnet issuer/dağıtım hesabı üretti ve `TRYT` ihraç etti
- `npm run start`, ardından `curl http://localhost:3001/.well-known/stellar.toml` ve
  `/health` ile sağlık kontrolü
- `node scripts/test-flow.mjs <issuer>` — tam SEP-10 + SEP-24 uçtan uca akışı

### Test Results

- **Gerçek Testnet issuer/dağıtım kurulumu**: issuer `GAOPTL4Q34VQWE5PWWVYEX7QQLOSO2DVYUVURCHHXTZFKASS66YL7YJ3`,
  dağıtım `GCXVNHNXFKMQHUILBFMSVW2FBOGXFGLU4Y5P42RGOW5H3HP7BQBMIOIG`, varlık `TRYT`, arz
  `1,000,000,000` dağıtım hesabında.
- **Uçtan uca canlı test** (`scripts/test-flow.mjs`, rastgele üretilip Friendbot ile
  fonlanan gerçek bir Testnet hesabıyla):
  1. SEP-10: challenge alındı, imzalandı, `POST /auth` JWT döndürdü.
  2. SEP-24 `/info`: `{"deposit":{"TRYT":{"enabled":true,"min_amount":10,"max_amount":100000}},...}`.
  3. `POST /transactions/deposit/interactive` bir işlem id'si ve interaktif form adresi
     döndürdü.
  4. Form, kullanıcı henüz `TRYT` trustline'ı açmadan onaylandı → durum doğru şekilde
     `pending_trust` oldu (gerçek bir Horizon `op_no_trust` hatası yakalanıp bu duruma
     çevrildi, sahte/simüle bir hata değil).
  5. Kullanıcı gerçek bir on-chain `changeTrust` işlemiyle trustline açtı.
  6. Bir sonraki `GET /sep24/transaction` sorgusunda sunucu ödemeyi **kendiliğinden**
     yeniden denedi ve başarılı oldu — durum `completed`, gerçek bir Stellar işlem hash'i
     (`415c3f687f7f74d7385dbdc7b9a73c9202c12b411b4873d93eabb339793073a8`) döndü.
  7. Kullanıcının nihai bakiyesi doğrulandı: `250.0000000 TRYT` — talep edilen tutarla
     birebir.
- Bu, `pending_trust` kendi kendini iyileştirme mantığının gerçek bir hata koşulunda
  (trustline yokluğu) ve gerçek bir kurtarmada (trustline sonrası otomatik yeniden deneme)
  çalıştığının doğrudan kanıtı — uydurulmuş bir mutlu yol değil.
- **Genel https tünel üzerinden ikinci bir doğrulama**: `cloudflared` (hesapsız "quick
  tunnel" modu, tek dosya, kurulum gerektirmez — winget'in MSI kurulumu bir UAC izin
  penceresinde takılınca bu yola geçildi) ile `https://projection-democratic-leslie-orders.trycloudflare.com`
  üzerinden dışarı açıldı, `PUBLIC_BASE_URL` buna güncellenip sunucu yeniden başlatıldı
  (`stellar.toml`'daki `WEB_AUTH_ENDPOINT`/`TRANSFER_SERVER_SEP0024`'ün artık tünel
  adresini gösterdiği doğrulandı), ve `scripts/test-flow.mjs` bu genel adrese karşı
  ikinci kez uçtan uca çalıştırıldı — yine gerçek bir Testnet işlem hash'iyle
  (`a7170ca00d6da1272d089fb30bf1faf421d8fa7fe78c61423dba8c5c6c4a5133`) ve doğru
  `250.0000000 TRYT` bakiyesiyle tamamlandı. `frontend/.env`'deki `VITE_ANCHOR_HOME_DOMAIN`
  bu tünel adresine yazıldı. Bu, yalnızca geliştiricinin makinesi açıkken çalışan **geçici**
  bir teşhir — kalıcı çözüm için bkz. aşağıdaki `backend/DEPLOY.md`.
- `scripts/test-flow.mjs`, hem yerel hem tünel adresine karşı tekrar kullanılabilsin diye
  `baseUrl`/`issuerPublicKey`'i komut satırı argümanı olarak almak üzere genelleştirildi
  (önceden `localhost:3001` sabitti).
- Yeni `backend/DEPLOY.md`: Yusuf'un sunucusuna kalıcı deploy talimatı — en kritik uyarı,
  **issuer/dağıtım sırlarının yeniden üretilmemesi** (aksi halde farklı bir `TRYT` varlığı
  doğar ve bu fazın tüm kanıtları geçersiz kalır); sırların git dışında güvenli bir
  kanaldan taşınması; alt alan adı + reverse proxy (Caddy/nginx) ile TLS; pm2/systemd ile
  sürekli çalışma; `scripts/test-flow.mjs` ile deploy sonrası doğrulama adımı.
- **Ek doğrulama — kontrat × `TRYT` uyumu (`docs/HACKATHON_SUBMISSION_TASKS.md` madde 1
  P0)**: `TRYT`'nin SAC sarmalayıcısı ilk kez `stellar contract asset deploy` ile
  yayınlandı (`CDCMFG35MOQPNQAJ4LEBSE427JM57SONZDSEGW4TDKLMBO6ZPMBTL2QB`), ardından
  gerçek bir Testnet havuzu (`pool_id: 4`, v10 kontrat) `token` olarak bu SAC ile
  sıfırdan oluşturulup tam bir tur boyunca çalıştırıldı: `create_pool → join_pool ×2 →
  propose_terms → approve_terms ×2 → start_pool → deposit ×2 → propose_purchase →
  approve_purchase ×2 → execute_round`. Satıcı gerçekten `20000000` stroop TRYT aldı
  (`execute_round` tx `3379aedf3903041e6d0428b1e09614d45898827ef0aece377bbc6636b7031cc6`),
  havuz `current_round: 2`'ye geçti. Kod değişikliği gerekmedi — kontrat zaten
  asset-agnostik tasarlanmıştı (bkz. Phase 12). Yol boyunca satıcı hesabında henüz
  trustline yokken ilk `execute_round` denemesi gerçek bir `op_no_trust` (`#13`) hatasıyla
  başarısız oldu; `get_round` ile round durumunun bozulmadan kaldığı (`AwaitingPurchase`,
  `pot: 20000000`, onaylar sağlam) doğrulandı, trustline eklenip tekrar denendiğinde
  sorunsuz tamamlandı — checks-effects-interactions'ın canlı ağda beklendiği gibi
  çalıştığının ek bir kanıtı (Phase 12'nin `failed_seller_transfer_rolls_back_round_and_liability_updates`
  birim testiyle aynı davranış, bu kez gerçek bir Testnet hatasıyla).

### Decisions

- **Kendi anchor'ımızı kurmayı**, SDF'in test anchor'ına bağlı kalmaya tercih ettik:
  `docs/HACKATHON_SUBMISSION_TASKS.md`'de tespit edildiği gibi el kitabı Ecosystem Fit
  içinde Anchor/Local Payments'a en yüksek ağırlığı veriyor ve bunun "ürünün çekirdeğinde"
  olmasını istiyor; kendi sunucumuz olmadan bu, sadece SDF'in genel demosuna bir bağlantı
  olarak kalırdı.
- **Node.js/TypeScript/Express** seçildi (Rust/Python değil): frontend zaten aynı dilde
  (`@stellar/stellar-sdk`'nin aynı sürümü), takımın hackathon süresince yeni bir dil
  öğrenmesini gerektirmiyor, ve SEP-10 yardımcı fonksiyonları (`WebAuth.buildChallengeTx`
  vb.) frontend'in zaten kullandığı SDK modülünün birebir aynısı — istemci/sunucu arasında
  kavramsal tutarlılık sağlıyor.
- **`Utils` değil `WebAuth`**: ilk yazımda `Utils.buildChallengeTx` vb. varsayıldı ama
  `@stellar/stellar-sdk@17.1.0`'da bu fonksiyonlar `WebAuth` ad alanında —
  `frontend/src/lib/anchor.ts`'nin zaten `import { WebAuth } from '@stellar/stellar-sdk'`
  yaptığı fark edilip düzeltildi (tip denetimi bunu hemen yakaladı).
- **`pending_trust`'ı GET /transaction içinde sessizce yeniden deneme**, ayrı bir webhook/
  callback ucu yerine: SEP-24 istemcileri zaten durumu poll'luyor (frontend'in
  `getTransaction`'ı ve `TERMINAL_STATUSES` mantığı buna göre yazılmış), bu yüzden var olan
  poll döngüsünü "self-healing" yapmak, yeni bir entegrasyon noktası eklemekten daha az
  hareketli parça demek.
- **Bellek-içi işlem deposu**: hackathon süresi için kalıcı bir veritabanı kurmanın getirisi
  yok; bu `backend/README.md`'de açıkça bir basitleştirme olarak işaretlendi, gizlenmedi.
- **Varlık kodu `TRYT`** (yalnızca `TRY` değil): kullanıcının veya bir judge'ın bunu gerçek
  TL zannetmesini SDK/arayüz seviyesinde daha zor kılmak için — isim zaten "test" içeriyor.

### Risks / Open Questions

- **Hâlâ gerçek banka/ödeme kuruluşu entegrasyonu yok** — bu bilinçli bir kapsam kararı
  (madde: Decisions), el kitabının "gerçek TL girişi" beklentisini tam karşılamıyor.
  `docs/altin-gunu-legal-boundary.md`'deki değerlendirme bu fazdan sonra da geçerliliğini
  koruyor: testnet + gerçek para yokluğu düşük risk, ama "gerçek TRY" iddiası hâlâ yanlış
  olur — kod ve dokümantasyon bunu her katmanda (toml, form sayfası, README) açıkça
  reddediyor.
- **Anchor henüz havuz akışına bağlı değil** (`docs/HACKATHON_SUBMISSION_TASKS.md` madde 3,
  frontend P0) — bu faz backend'i ve genel erişimi teslim eder; `CreateView.vue`/
  `PoolView.vue`'ya bağlanması ayrı, henüz yapılmamış bir adımdır.
- **`PUBLIC_BASE_URL` şu an bir `cloudflared` quick tunnel adresi** (hesapsız, tek dosya) —
  gerçek bir cüzdanla https üzerinden deneme artık mümkün, ama bu tünel yalnızca
  geliştiricinin makinesi açıkken yaşar ve her yeniden başlatmada adres değişir. **Teslim
  için yeterli değil** — kalıcı çözüm `backend/DEPLOY.md`'de belgelendi, Yusuf'un
  sunucusuna uygulanması bekleniyor.
- **Tek imzalı hesap varsayımı** (`verifyChallengeTxSigners`, eşik/çoklu imza yok) —
  hackathon demosu için yeterli, üretim için yetersiz; `backend/README.md`'de açıkça
  "bilinçli basitleştirme" olarak işaretli.
- **`ISSUER_SECRET`/`DISTRIBUTION_SECRET` yalnızca yerel `backend/.env`'de** (gitignore'lu,
  hiç commit edilmedi) — bu sırların takım içinde nasıl paylaşılacağı (ör. güvenli bir
  kanaldan) bu fazın kapsamı dışında, teslim öncesi netleştirilmeli.

### Stellar Skills Used

- Anchors (SEP-1/6/10/12/24/31/38 skill dosyası, `skills.stellar.org`): SEP-10 challenge
  üretimi/doğrulaması ve SEP-24 interactive deposit akışının doğrudan referansı.
- Stellar Assets & SAC: yeni bir classic varlık ihracı (issuer + dağıtım hesabı ayrımı,
  trustline), Soroban tarafında ekstra işlem gerekmeden aynı bakiyenin SAC sarmalayıcısı
  üzerinden kontrat tarafından da kullanılabilir olması (`stellar contract id asset` ile
  hesaplanabilir SAC adresi, kontrat `token` parametresine geçilebilir).

### Definition of Done

- Gerçek SEP-1/10/24 akışı, kendi sunucumuzda, uçtan uca ve gerçek bir Testnet ödemesiyle
  doğrulandı: **tamamlandı** (yukarıdaki test kanıtı, iki ayrı gerçek tx hash ile).
- Genel https erişim (geçici): **tamamlandı** (`cloudflared` quick tunnel). **Kalıcı
  değil** — sunucuya deploy `backend/DEPLOY.md`'de belgelendi, uygulanması bekleniyor.
- Anchor'ın ana havuz akışına bağlanması: **tamamlanmadı** — ayrı bir sonraki adım
  (frontend tarafı, `docs/HACKATHON_SUBMISSION_TASKS.md`).
- Gerçek bir cüzdanla (https üzerinden) canlı deneme: **tamamlanmadı** — sonraki adım.
- Dürüstlük/etiketleme (gerçek TRY olmadığının her katmanda açık olması): **tamamlandı**.

### Status

NEEDS REVIEW
