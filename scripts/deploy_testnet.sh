#!/usr/bin/env bash
# Stellerpool — reproducible Testnet deployment for the rotating_pool contract.
#
# What it does:
#   1. Builds the canonical optimized WASM with the Stellar CLI.
#   2. Generates (or reuses) a funded Testnet deployer identity.
#   3. Deploys the rotating_pool contract WASM to Testnet.
#   4. Resolves (or deploys, if missing) the Stellar Asset Contract (SAC)
#      wrapper for the configured settlement asset.
#   5. Prints a copy-pasteable summary and, unless --no-write is passed,
#      writes the resulting IDs into the top-level and frontend .env files.
#
# The contract itself never decides which Anchor/fiat provider is used;
# this script only prepares Testnet settlement-asset and contract state.
#
# Requirements: `stellar` (Stellar CLI) and `cargo` on PATH.
#
# Usage:
#   scripts/deploy_testnet.sh [--asset-code USDC] [--asset-issuer G...] \
#       [--identity stellerpool-deployer] [--no-write]
#
# Re-running is safe: an existing deployer identity is reused, and an
# already-deployed SAC wrapper is looked up instead of re-deployed.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

NETWORK="testnet"
IDENTITY="stellerpool-deployer"
ASSET_CODE="USDC"
ASSET_ISSUER="GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5"
WRITE_ENV="1"
WASM_PATH="target/wasm32v1-none/release/rotating_pool.wasm"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --identity) IDENTITY="$2"; shift 2 ;;
    --asset-code) ASSET_CODE="$2"; shift 2 ;;
    --asset-issuer) ASSET_ISSUER="$2"; shift 2 ;;
    --no-write) WRITE_ENV="0"; shift ;;
    -h|--help)
      grep -E '^#( |$)' "$0" | sed -E 's/^# ?//'
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

command -v stellar >/dev/null 2>&1 || {
  echo "error: 'stellar' CLI not found on PATH." >&2
  exit 1
}
command -v cargo >/dev/null 2>&1 || {
  echo "error: 'cargo' not found on PATH." >&2
  exit 1
}

echo "==> Building canonical optimized WASM"
stellar contract build

echo "==> Ensuring a funded Testnet deployer identity ('$IDENTITY')"
if ! stellar keys address "$IDENTITY" >/dev/null 2>&1; then
  stellar keys generate "$IDENTITY" --network "$NETWORK" --fund
else
  echo "    Identity '$IDENTITY' already exists; reusing it."
fi
DEPLOYER_ADDRESS="$(stellar keys address "$IDENTITY")"
echo "    Deployer address: $DEPLOYER_ADDRESS"

echo "==> Deploying rotating_pool contract to Testnet"
CONTRACT_ID="$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source "$IDENTITY" \
  --network "$NETWORK" | tail -n 1)"
echo "    Contract ID: $CONTRACT_ID"

WASM_HASH="$(stellar contract info interface --wasm "$WASM_PATH" --network "$NETWORK" 2>/dev/null \
  | grep -m1 -oE '[0-9a-f]{64}' || true)"

echo "==> Resolving settlement asset SAC ($ASSET_CODE:$ASSET_ISSUER)"
if ! TOKEN_ID="$(stellar contract id asset \
  --asset "$ASSET_CODE:$ASSET_ISSUER" \
  --network "$NETWORK" 2>/dev/null)"; then
  TOKEN_ID=""
fi
if [[ -z "$TOKEN_ID" ]] || ! stellar contract invoke \
    --id "$TOKEN_ID" --source "$IDENTITY" --network "$NETWORK" --send=no \
    -- name >/dev/null 2>&1; then
  echo "    Not yet deployed; deploying SAC wrapper now."
  TOKEN_ID="$(stellar contract asset deploy \
    --asset "$ASSET_CODE:$ASSET_ISSUER" \
    --source "$IDENTITY" \
    --network "$NETWORK" | tail -n 1)"
else
  echo "    Already deployed; reusing existing wrapper."
fi
echo "    Token contract ID: $TOKEN_ID"

echo "==> Read-only smoke test"
VERSION="$(stellar contract invoke --id "$CONTRACT_ID" --source "$IDENTITY" \
  --network "$NETWORK" --send=no -- version)"
echo "    version() -> $VERSION"

cat <<SUMMARY

===================== Deployment summary =====================
Network:            $NETWORK
Deployer identity:  $IDENTITY ($DEPLOYER_ADDRESS)
Contract ID:        $CONTRACT_ID
Token (SAC) ID:      $TOKEN_ID
Settlement asset:    $ASSET_CODE issued by $ASSET_ISSUER
================================================================

Reminder: contract-reported settlement is separate from real property
or vehicle delivery. Only the off-chain verifier/legal layer confirms
that a Testnet seller/document approval reflects anything real.
SUMMARY

if [[ "$WRITE_ENV" == "1" ]]; then
  echo "==> Writing deployed IDs into .env files"
  for ENV_FILE in ".env" "frontend/.env"; do
    if [[ ! -f "$ENV_FILE" ]]; then
      cp "$ENV_FILE.example" "$ENV_FILE"
    fi
    # Portable in-place edit (no GNU sed -i quirks across platforms).
    tmp="$(mktemp)"
    sed -E \
      -e "s|^ROTATING_POOL_CONTRACT_ID=.*|ROTATING_POOL_CONTRACT_ID=$CONTRACT_ID|" \
      -e "s|^TOKEN_CONTRACT_ID=.*|TOKEN_CONTRACT_ID=$TOKEN_ID|" \
      -e "s|^VITE_ROTATING_POOL_CONTRACT_ID=.*|VITE_ROTATING_POOL_CONTRACT_ID=$CONTRACT_ID|" \
      -e "s|^VITE_POOL_ASSET_CODE=.*|VITE_POOL_ASSET_CODE=$ASSET_CODE|" \
      -e "s|^VITE_POOL_ASSET_ISSUER=.*|VITE_POOL_ASSET_ISSUER=$ASSET_ISSUER|" \
      "$ENV_FILE" > "$tmp"
    mv "$tmp" "$ENV_FILE"
    echo "    Updated $ENV_FILE"
  done
fi

echo "Done."
