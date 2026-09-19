#!/usr/bin/env bash
# Stellerpool — end-to-end Testnet demo for the rotating_pool contract (v8 API:
# sponsor guarantee + propose_terms/approve_terms + cancel_unstarted_pool +
# per-member sponsor advances + a separate purchase-execution deadline, per the
# current docs/plan.md).
#
# Runs two scenarios against an already-deployed contract (see
# scripts/deploy_testnet.sh):
#   Pool A — happy path: two members join, the creator proposes terms
#            (recipient order + verifiers), every member and the sponsor
#            approve that terms version, the sponsor funds the guarantee,
#            anyone starts the pool, both rounds are fully funded, proposed,
#            approved by both verifiers, and executed against the pool's
#            fixed demo seller — the pool reaches Completed.
#   Pool B — default/abort/refund: same members, a short round/grace window.
#            Only one member deposits, the round goes overdue, grace expires,
#            anyone aborts the pool (SafetyRecovery), and the paying member's
#            refund plus the sponsor's freed guarantee remainder are both
#            claimed.
#
# This script issues and funds its own demo settlement asset (STLP) from the
# deployer identity so the run is fully self-contained and does not depend on
# an external Testnet USDC faucet. It is illustrative evidence, not a
# production settlement asset. Every contract-reported "purchase approval"
# here is simulated test evidence (a fixed 32-byte digest and a pre-registered
# demo seller), not a real seller/title/deed check.
#
# Requirements: `stellar` CLI on PATH, a deployed contract ID (from
# scripts/deploy_testnet.sh or ROTATING_POOL_CONTRACT_ID in .env), and
# network access to Testnet. Pool B genuinely waits out its round and grace
# deadlines in real time (Soroban deadlines are wall-clock, not simulatable).
#
# Usage:
#   scripts/demo_testnet.sh [--contract C...] [--round-duration 40] [--grace-duration 40]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

NETWORK="testnet"
CONTRACT_ID="${ROTATING_POOL_CONTRACT_ID:-}"
HAPPY_ROUND_DURATION="1800"  # Pool A: generous, no deadline pressure.
HAPPY_PURCHASE_DURATION="900"
ABORT_ROUND_DURATION="40"    # Pool B: short and deliberate.
ABORT_GRACE_DURATION="40"
ABORT_PURCHASE_DURATION="900"
SETUP_WINDOW="3600"
ASSET_CODE="STLP"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --contract) CONTRACT_ID="$2"; shift 2 ;;
    --round-duration) ABORT_ROUND_DURATION="$2"; shift 2 ;;
    --grace-duration) ABORT_GRACE_DURATION="$2"; shift 2 ;;
    -h|--help)
      grep -E '^#( |$)' "$0" | sed -E 's/^# ?//'
      exit 0
      ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$CONTRACT_ID" ]]; then
  echo "error: no contract ID. Pass --contract or set ROTATING_POOL_CONTRACT_ID (see .env)." >&2
  exit 1
fi
command -v stellar >/dev/null 2>&1 || { echo "error: 'stellar' CLI not found on PATH." >&2; exit 1; }

invoke() { stellar contract invoke --id "$1" --source "$2" --network "$NETWORK" -- "${@:3}"; }
addr() { stellar keys address "$1"; }
now_ts() { date +%s; }

echo "==> Ensuring a funded deployer identity (issuer + pool creator)"
stellar keys address stellerpool-deployer >/dev/null 2>&1 \
  || stellar keys generate stellerpool-deployer --network "$NETWORK" --fund
ISSUER="$(addr stellerpool-deployer)"
ASSET="$ASSET_CODE:$ISSUER"

echo "==> Generating/funding demo identities"
for name in demo-sponsor demo-member1 demo-member2 demo-verifier1 demo-verifier2 demo-seller1; do
  stellar keys address "$name" >/dev/null 2>&1 || stellar keys generate "$name" --network "$NETWORK" --fund
done
SPONSOR="$(addr demo-sponsor)"; MEMBER1="$(addr demo-member1)"; MEMBER2="$(addr demo-member2)"
VERIFIER1="$(addr demo-verifier1)"; VERIFIER2="$(addr demo-verifier2)"; SELLER1="$(addr demo-seller1)"

echo "==> Establishing $ASSET_CODE trustlines and issuing balances"
for holder in demo-sponsor demo-member1 demo-member2 demo-seller1; do
  stellar tx new change-trust --source-account "$holder" --network "$NETWORK" --line "$ASSET" >/dev/null 2>&1 || true
done
for dest in "$SPONSOR" "$MEMBER1" "$MEMBER2"; do
  stellar tx new payment --source-account stellerpool-deployer --network "$NETWORK" \
    --destination "$dest" --asset "$ASSET" --amount 500000000 >/dev/null
done

echo "==> Resolving/deploying the $ASSET_CODE Stellar Asset Contract wrapper"
TOKEN_ID="$(stellar contract id asset --asset "$ASSET" --network "$NETWORK" 2>/dev/null || true)"
if [[ -z "$TOKEN_ID" ]]; then
  TOKEN_ID="$(stellar contract asset deploy --asset "$ASSET" --source stellerpool-deployer --network "$NETWORK" | tail -n 1)"
fi
echo "    Token contract ID: $TOKEN_ID"

digest() { printf '%02x%.0s' $(seq 1 32) | sed "s/02/0$1/g"; }

run_pool() {
  local label="$1" round_dur="$2" grace_dur="$3" purchase_dur="$4" auto_settle="$5"

  echo
  echo "==> [$label] create_pool (round=${round_dur}s grace=${grace_dur}s purchase=${purchase_dur}s)"
  local setup_deadline
  setup_deadline=$(( $(now_ts) + SETUP_WINDOW ))
  local pool_id
  pool_id="$(invoke "$CONTRACT_ID" stellerpool-deployer create_pool \
    --creator "$ISSUER" --sponsor "$SPONSOR" --token "$TOKEN_ID" \
    --contribution_amount 10000000 --member_limit 2 \
    --round_duration "$round_dur" --grace_duration "$grace_dur" \
    --purchase_duration "$purchase_dur" --setup_deadline "$setup_deadline" \
    --demo_seller "$SELLER1" | tail -n 1)"
  echo "    pool_id: $pool_id"

  invoke "$CONTRACT_ID" demo-member1 join_pool --member "$MEMBER1" --pool_id "$pool_id" >/dev/null
  invoke "$CONTRACT_ID" demo-member2 join_pool --member "$MEMBER2" --pool_id "$pool_id" >/dev/null

  echo "    Proposing and approving terms (recipient order + verifiers)."
  local version
  version="$(invoke "$CONTRACT_ID" stellerpool-deployer propose_terms \
    --creator "$ISSUER" --pool_id "$pool_id" \
    --recipient_order "[\"$MEMBER1\",\"$MEMBER2\"]" \
    --verifiers "[\"$VERIFIER1\",\"$VERIFIER2\"]" | tail -n 1)"
  invoke "$CONTRACT_ID" demo-member1 approve_terms --approver "$MEMBER1" --pool_id "$pool_id" --version "$version" >/dev/null
  invoke "$CONTRACT_ID" demo-member2 approve_terms --approver "$MEMBER2" --pool_id "$pool_id" --version "$version" >/dev/null
  invoke "$CONTRACT_ID" demo-sponsor approve_terms --approver "$SPONSOR" --pool_id "$pool_id" --version "$version" >/dev/null

  invoke "$CONTRACT_ID" demo-sponsor fund_guarantee --pool_id "$pool_id" --sponsor "$SPONSOR" --amount 10000000 >/dev/null
  invoke "$CONTRACT_ID" stellerpool-deployer start_pool --pool_id "$pool_id" >/dev/null
  echo "    Pool $pool_id is Active."

  invoke "$CONTRACT_ID" demo-member1 deposit --member "$MEMBER1" --pool_id "$pool_id" >/dev/null
  echo "    member1 deposited round 1."

  if [[ "$auto_settle" == "1" ]]; then
    invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$pool_id" >/dev/null
    invoke "$CONTRACT_ID" demo-member1 propose_purchase \
      --member "$MEMBER1" --pool_id "$pool_id" --seller "$SELLER1" --asset "$TOKEN_ID" \
      --amount 20000000 --doc_hash "$(digest 1)" >/dev/null
    invoke "$CONTRACT_ID" demo-verifier1 approve_purchase --verifier "$VERIFIER1" --pool_id "$pool_id" --round 1 --proposal_version 1 >/dev/null
    invoke "$CONTRACT_ID" demo-verifier2 approve_purchase --verifier "$VERIFIER2" --pool_id "$pool_id" --round 1 --proposal_version 1 >/dev/null
    invoke "$CONTRACT_ID" stellerpool-deployer execute_round --pool_id "$pool_id" >/dev/null
    echo "    Round 1 paid the demo seller; round 2 (member2's turn) now open."

    invoke "$CONTRACT_ID" demo-member1 deposit --member "$MEMBER1" --pool_id "$pool_id" >/dev/null
    invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$pool_id" >/dev/null
    invoke "$CONTRACT_ID" demo-member2 propose_purchase \
      --member "$MEMBER2" --pool_id "$pool_id" --seller "$SELLER1" --asset "$TOKEN_ID" \
      --amount 20000000 --doc_hash "$(digest 2)" >/dev/null
    invoke "$CONTRACT_ID" demo-verifier1 approve_purchase --verifier "$VERIFIER1" --pool_id "$pool_id" --round 2 --proposal_version 1 >/dev/null
    invoke "$CONTRACT_ID" demo-verifier2 approve_purchase --verifier "$VERIFIER2" --pool_id "$pool_id" --round 2 --proposal_version 1 >/dev/null
    invoke "$CONTRACT_ID" stellerpool-deployer execute_round --pool_id "$pool_id" >/dev/null
    echo "    Round 2 paid the demo seller; pool $pool_id is Completed."
  else
    echo "    member2 intentionally does NOT deposit — forcing an overdue round."
    echo "    Waiting for the round deadline (~${round_dur}s, real Testnet ledger time)..."
    until invoke "$CONTRACT_ID" stellerpool-deployer mark_overdue --pool_id "$pool_id" 2>/dev/null | grep -q .; do
      sleep 5
    done
    echo "    Round marked overdue; pool $pool_id is in round-level Grace."

    echo "    Waiting for the grace deadline (~${grace_dur}s, real Testnet ledger time)..."
    until invoke "$CONTRACT_ID" stellerpool-deployer abort_pool --pool_id "$pool_id" 2>/dev/null | grep -q .; do
      sleep 5
    done
    echo "    Pool $pool_id is Aborted (SafetyRecovery)."

    invoke "$CONTRACT_ID" demo-member1 claim_refund --member "$MEMBER1" --pool_id "$pool_id" >/dev/null
    echo "    member1 claimed their round-1 contribution refund."
    invoke "$CONTRACT_ID" demo-sponsor claim_sponsor_remainder --sponsor "$SPONSOR" --pool_id "$pool_id" >/dev/null
    echo "    sponsor claimed the freed guarantee remainder."
  fi
}

run_pool "Pool A: happy path" "$HAPPY_ROUND_DURATION" "$HAPPY_ROUND_DURATION" "$HAPPY_PURCHASE_DURATION" "1"
run_pool "Pool B: default/abort/refund" "$ABORT_ROUND_DURATION" "$ABORT_GRACE_DURATION" "$ABORT_PURCHASE_DURATION" "0"

echo
echo "Done. Copy the pool IDs and transaction hashes printed above into"
echo "docs/IMPLEMENTATION_LOG.md as this run's submission evidence."
