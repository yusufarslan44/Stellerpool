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

Runs two full scenarios against an already-deployed contract, self-issuing
a demo settlement asset (`STLP`) so it needs no external testnet-USDC
faucet:

- **Pool A — happy path**: two members join, the creator proposes terms
  (recipient order + verifiers), every member and the sponsor approve that
  terms version, the sponsor funds the guarantee, anyone starts the pool,
  both rounds are fully funded, proposed against the pool's fixed demo
  seller, approved by both verifiers, and executed — the pool reaches
  `Completed`.
- **Pool B — default/abort/refund**: same members, short round/grace
  durations. Only one member deposits, the round goes overdue (round-level
  `Grace`, not a pool-level pause), anyone aborts the pool once grace
  expires, and the paying member's refund plus the sponsor's freed
  guarantee remainder are both claimed.

Pool B genuinely waits out its round and grace deadlines in real time
(Soroban deadlines are wall-clock, not simulatable) — the whole run takes
a few minutes. Every ID, event, and transaction hash the script prints is
real Testnet evidence for `docs/IMPLEMENTATION_LOG.md`.

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
