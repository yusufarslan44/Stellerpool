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

Runs three scenarios against a **new API v12** contract, self-issuing a
Testnet demo asset (`STLP`). It checks `version() == 12` before creating pools.

- **Pool A — fixed order:** the last join records the delivery order; both
  members approve the same terms, fund both rounds and record each purchase.
  Payment then goes straight to the registered demo seller.
- **Pool B — missed contribution:** after the first payout, the first
  recipient stops paying. The next round enters grace, then aborts; only
  that round's contribution is refundable.
- **Pool C — draw:** after every member pays, the recipient is drawn from
  members who have not received yet. Both rounds settle without a verifier
  approval step.

All pools genuinely wait out real deadlines where relevant (Soroban
deadlines are wall-clock, not simulatable) — the whole run takes a few
minutes. Every ID, event, and transaction hash the script prints is real
Testnet evidence for the README proof section.

Secret keys are never printed; they stay in the Stellar CLI's local
identity store (`stellar keys ls` / `stellar keys address <name>`).

## What is intentionally not here

- Anchor/SEP integration: this script exercises the contract directly; the
  separate Testnet anchor is implemented in `backend/`.
- A real TRY/fiat on/off-ramp, or any claim that a Testnet purchase
  record reflects real property/vehicle delivery — `demo_testnet.sh`
  is contract-only, simulated evidence.
- A wallet-driven click-through demo through the frontend UI; these
  scripts exercise the contract directly via the Stellar CLI.
