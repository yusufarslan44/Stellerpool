# Scripts

Testnet deployment and demo helpers for the `rotating_pool` Soroban contract.
These scripts never decide financial state themselves — they only build,
deploy, and exercise the contract; the contract remains the source of truth.

## Prerequisites

- [Stellar CLI](https://developers.stellar.org/docs/tools/cli/stellar-cli) 28.x on `PATH` (`stellar --version`).
- Rust toolchain pinned by [`rust-toolchain.toml`](../rust-toolchain.toml) with the `wasm32v1-none` target (`cargo --version`).
- Network access to `https://soroban-testnet.stellar.org` and Friendbot.

## `deploy_testnet.sh`

```bash
scripts/deploy_testnet.sh
```

1. Builds the canonical optimized WASM (`stellar contract build`).
2. Generates and Friendbot-funds a Testnet deployer identity named
   `stellerpool-deployer` (reused on repeat runs — safe to re-run).
3. Deploys the contract WASM to Testnet.
4. Resolves the Stellar Asset Contract (SAC) wrapper for the configured
   settlement asset (`--asset-code` / `--asset-issuer`, default: Testnet
   USDC), deploying it only if no wrapper exists yet at that deterministic
   address.
5. Runs a read-only `version()` smoke test against the deployed contract.
6. Prints a summary and writes `ROTATING_POOL_CONTRACT_ID`,
   `TOKEN_CONTRACT_ID`, and the matching `VITE_*` keys into `.env` and
   `frontend/.env` (created from the `.example` files if missing). Pass
   `--no-write` to skip this step.

This script only builds/deploys the contract and resolves a settlement
asset — it never calls `create_pool`, so it works unchanged regardless of
the contract's current pool-lifecycle API.

## `demo_testnet.sh`

```bash
scripts/demo_testnet.sh --contract <ROTATING_POOL_CONTRACT_ID>
```

(Omit `--contract` if `ROTATING_POOL_CONTRACT_ID` is already set, e.g. by
sourcing `.env` first.)

Runs three full scenarios against an already-deployed contract, self-issuing
a demo settlement asset (`STLP`) so it needs no external testnet-USDC
faucet. The contract is sponsorless (v10): every member always pays their
own contribution; there is no sponsor, guarantee, or advance. v10 adds
Draw-mode recipient selection (`OrderMode::Fixed | Draw`) and a 30-member
ceiling — see `docs/CONTRACT_HANDOFF.md`.

- **Pool A — happy path + live cure**: two members join, the creator
  proposes terms (recipient order + verifiers), every member approves that
  terms version, anyone starts the pool, both rounds are fully funded
  (every member pays every round), proposed against the pool's fixed demo
  seller, approved by both verifiers, and executed — the pool reaches
  `Completed`.
- **Pool B — the plan's canonical demo scenario**: round 1 is fully paid
  and settles to the demo seller. In round 2 the member who already
  received round 1's allocation stops paying entirely; the round goes
  overdue (round-level `Grace`), anyone aborts once grace expires
  (`SafetyRecovery`), and only the still-paying member's round-2 deposit
  is refundable — round 1's amounts are gone for good, matching the
  plan's accepted economic risk (an early recipient defaulting later
  cannot be recovered on-chain).
- **Pool C — Draw mode**: two members join a Draw-mode pool (empty
  `recipient_order` at `propose_terms`); once a round is fully funded it
  enters `AwaitingDraw` and anyone can call `draw_recipient` to pick the
  round's recipient at random among members who have not received yet.
  Round 2 is deterministic (only one member is left) — both rounds settle
  and the pool reaches `Completed`.

All pools genuinely wait out real deadlines where relevant (Soroban
deadlines are wall-clock, not simulatable) — the whole run takes a few
minutes. Every ID, event, and transaction hash the script prints is real
Testnet evidence for `docs/IMPLEMENTATION_LOG.md`.

Secret keys are never printed; they stay in the Stellar CLI's local
identity store (`stellar keys ls` / `stellar keys address <name>`).

## What is intentionally not here

- Anchor/SEP integration: blocked until a real provider and home domain
  are supplied.
- A real TRY/fiat on/off-ramp, or any claim that a Testnet purchase
  approval reflects real property/vehicle delivery — `demo_testnet.sh`
  is contract-only, simulated evidence.
- A wallet-driven click-through demo through the frontend UI; these
  scripts exercise the contract directly via the Stellar CLI.
