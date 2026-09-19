# Stellar Skills Used

Phase 0 reviewed the Stellar Skills directory and selected only skills relevant to the backend, Soroban contract, asset/SAC, data, and anchor scope.

## Skill

Path: https://skills.stellar.org/skills/smart-contracts/SKILL.md

Neden kullanildi: Soroban/Rust contract setup, storage/TTL, auth, typed events, errors, testing, deployment, and security workflow.

Projede nerede uygulandi: Applied to the root Rust workspace, `contracts/rotating_pool/**`, typed storage/events/errors, TTL helpers, unit tests, and the canonical WASM build.

Onemli kararlar: Use `#![no_std]`, Protocol 28 with exact `soroban-sdk 28.0.0`, `wasm32v1-none`, typed storage keys, persistent per-pool/per-member storage, typed errors/events, release overflow checks, proactive TTL extension, and explicit role authorization. Phase 2 applies `require_auth` to creator, sponsor, and member mutations. Phase 3 performs every checked accounting calculation before the token transfer and relies on Soroban transaction atomicity so a failed transfer cannot leave partial deposit state. Phase 4 stores a bounded verifier policy before start and binds each authenticated approval to one immutable pool/round purchase. Phase 5 revalidates all payout inputs, writes replay-blocking state before the token call, and relies on transaction rollback if that call fails. Phase 6 stores a deterministic grace deadline, separates member cure from sponsor top-up, and permits permissionless timeout transitions without privileged automation. Phase 7 gates permissionless `abort_pool` to the already-expired-recovery `Paused` state, reuses existing lifecycle guards instead of a separate entitlement freeze, and writes replay-blocking claim flags before each refund/remainder token transfer so a failed transfer rolls back atomically. Phase 11 redesigns the state machine (round-level `Grace`/`AwaitingPurchase` phases replacing a pool-level pause), keys terms/purchase approvals by version so a re-proposal invalidates stale approvals without a manual clear step, and cross-checks the generated `stellar contract info interface` output field-by-field against an independently-written TypeScript client to prove ABI compatibility rather than assuming it.

Ilgili dosyalar: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `contracts/rotating_pool/**`, `docs/IMPLEMENTATION_PLAN.md`, future deployment scripts.

## Skill

Path: https://skills.stellar.org/skills/assets/SKILL.md

Neden kullanildi: Settlement asset model, Stellar classic asset vs SAC vs custom token decision, trustline implications, and SEP-41 token transfer semantics.

Projede nerede uygulandi: Contract token-address design, sponsor-guarantee funding through `TokenClient`, SAC-backed tests, trustline/account readiness preflight responsibilities, and future deployment asset setup.

Onemli kararlar: Prefer classic Stellar asset through SAC for MVP settlement unless custom token logic becomes necessary. Contract remains asset agnostic and uses token contract address plus integer amounts. Sponsor guarantee, active/grace member contributions, sponsor shortfall top-ups, and exact approved-seller payouts use the SEP-41 token interface and are accounted per pool. Top-up never writes a member deposit or contribution record. Member join performs no token transfer. Contract-wide token balance is never used as an individual pool ledger or payout authorization source. TRY logic stays in the anchor layer. Phase 7 member refund and sponsor remainder claims are also plain SEP-41 transfers out of the pool-assigned balance, sized from the same per-pool liability ledger Phase 3/5 already maintain rather than any new asset-specific logic.

Ilgili dosyalar: `docs/IMPLEMENTATION_PLAN.md`, `contracts/rotating_pool/src/lib.rs`, `contracts/rotating_pool/src/storage.rs`, `contracts/rotating_pool/src/test.rs`, future `scripts/**`.

## Skill

Path: https://skills.stellar.org/skills/standards/SKILL.md

Neden kullanildi: SEP/CAP routing for anchors, token interfaces, auth, contract metadata, and upgrade guidance.

Projede nerede uygulandi: Anchor integration strategy, no-upgrade default, SEP selection, and contract/interface planning.

Onemli kararlar: SEP-1 for discovery, SEP-10 for auth when required, SEP-24 preferred for hosted anchor flows, SEP-6 fallback, SEP-12 and SEP-38 only when provider capability requires them.

Ilgili dosyalar: `docs/IMPLEMENTATION_PLAN.md`, future `backend/**`.

## Skill

Path: https://skills.stellar.org/skills/data/SKILL.md

Neden kullanildi: RPC usage, transaction polling, event querying, and understanding retention limits.

Projede nerede uygulandi: Backend read-helper strategy, scripts, transaction status diagnostics, and event/indexing assumptions.

Onemli kararlar: Use Stellar RPC for contract interactions and recent events. Do not rely on public RPC for long-term event history without an indexer or data provider.

Ilgili dosyalar: `docs/IMPLEMENTATION_PLAN.md`, future `backend/**`, future `scripts/**`.

## Skill

Path: https://github.com/CheesecakeLabs/stellar-anchor-skill/blob/main/SKILL.md

Neden kullanildi: Client-side anchor integration pitfalls and practical SEP-1/10/24/6/12/38 flow design.

Projede nerede uygulandi: Anchor adapter design and backend integration plan.

Onemli kararlar: Do not invent provider capabilities. `/info` is the contract. SEP-24 interactive URLs must not be iframe-only. Amounts and statuses must be handled as exact strings/state machines. Catch 401 and redo SEP-10 when needed.

Ilgili dosyalar: `docs/IMPLEMENTATION_PLAN.md`, future `backend/**`.

## Skill

Path: https://github.com/mariaelisaaraya/stellar-security-guide/blob/main/skills/soroban-common-mistakes/SKILL.md

Neden kullanildi: Soroban security review checklist for authorization, storage/TTL, arithmetic, token handling, events, tests, and secret hygiene.

Projede nerede uygulandi: Security risks, checked guarantee arithmetic, bounded membership/verifier sets, stored-role authorization tests, separate per-pool guarantee/contribution/top-up/liability accounting, failed-transfer rollback tests, same-token multi-pool isolation tests, immutable purchase records, approval anti-replay, participant/seller/verifier separation, payout solvency checks, deadline boundary tests, and future Phase 8 audit checklist.

Onemli kararlar: Prioritize missing `require_auth`, wrong storage type, missing TTL extension, unchecked arithmetic, missing state deduction, unsafe unwrap/panic patterns, hardcoded secrets, unbounded loops, duplicate approvals, mutable seller records, and participant-controlled verification. Phase 7 adds double-claim replay protection on both refund and sponsor-remainder paths, an order-independent remainder formula so a sponsor claim can never race ahead of reserved member liabilities, and a failed-transfer rollback test for the refund path.

Ilgili dosyalar: `docs/IMPLEMENTATION_PLAN.md`, future contract review docs/logs.
