#!/usr/bin/env bash
# Stellerpool — API v12 Testnet demo. Requires a newly deployed v12 contract.
# Runs a two-member fixed-order happy path, a late-payment abort/refund,
# and a two-member draw. Members approve the automatic terms; purchases
# require no verifier role or second approval. The demo asset and seller
# are test-only and do not represent real property or money.
#
# Usage:
#   scripts/demo_testnet.sh --contract C... [--round-duration 40] [--grace-duration 40]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

NETWORK="testnet"
CONTRACT_ID="${ROTATING_POOL_CONTRACT_ID:-}"
HAPPY_ROUND_DURATION="1800"  # Pool A: generous, no deadline pressure.
HAPPY_PURCHASE_DURATION="900"
DEFAULT_ROUND_DURATION="40"  # Pool B: short and deliberate.
DEFAULT_GRACE_DURATION="40"
DEFAULT_PURCHASE_DURATION="900"
SETUP_WINDOW="3600"
ASSET_CODE="STLP"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --contract) CONTRACT_ID="$2"; shift 2 ;;
    --round-duration) DEFAULT_ROUND_DURATION="$2"; shift 2 ;;
    --grace-duration) DEFAULT_GRACE_DURATION="$2"; shift 2 ;;
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
version="$(stellar contract invoke --id "$CONTRACT_ID" --source stellerpool-deployer --network "$NETWORK" --send=no -- version | tr -d '"[:space:]')"
if [[ "$version" != "12" ]]; then
  echo "error: demo_testnet.sh requires API v12; contract reports version $version." >&2
  exit 1
fi

echo "==> Generating/funding demo identities"
for name in demo-member1 demo-member2 demo-seller1; do
  stellar keys address "$name" >/dev/null 2>&1 || stellar keys generate "$name" --network "$NETWORK" --fund
done
MEMBER1="$(addr demo-member1)"; MEMBER2="$(addr demo-member2)"
SELLER1="$(addr demo-seller1)"

echo "==> Establishing $ASSET_CODE trustlines and issuing balances"
for holder in demo-member1 demo-member2 demo-seller1; do
  stellar tx new change-trust --source-account "$holder" --network "$NETWORK" --line "$ASSET" >/dev/null 2>&1 || true
done
for dest in "$MEMBER1" "$MEMBER2"; do
  stellar tx new payment --source-account stellerpool-deployer --network "$NETWORK" \
    --destination "$dest" --asset "$ASSET" --amount 500000000 >/dev/null
done

echo "==> Resolving/deploying the $ASSET_CODE Stellar Asset Contract wrapper"
TOKEN_ID="$(stellar contract id asset --asset "$ASSET" --network "$NETWORK" 2>/dev/null || true)"
if [[ -z "$TOKEN_ID" ]] || ! stellar contract invoke \
    --id "$TOKEN_ID" --source stellerpool-deployer --network "$NETWORK" --send=no \
    -- name >/dev/null 2>&1; then
  TOKEN_ID="$(stellar contract asset deploy --asset "$ASSET" --source stellerpool-deployer --network "$NETWORK" | tail -n 1)"
fi
echo "    Token contract ID: $TOKEN_ID"

digest() { printf '%02x%.0s' $(seq 1 32) | sed "s/02/0$1/g"; }

create_and_start_pool() {
  local round_dur="$1" grace_dur="$2" purchase_dur="$3" order_mode="${4:-Fixed}"
  local setup_deadline pool_id version
  setup_deadline=$(( $(now_ts) + SETUP_WINDOW ))
  pool_id="$(invoke "$CONTRACT_ID" stellerpool-deployer create_pool \
    --creator "$ISSUER" --token "$TOKEN_ID" \
    --contribution_amount 10000000 --member_limit 2 --order_mode "$order_mode" --down_payment 0 \
    --round_duration "$round_dur" --grace_duration "$grace_dur" \
    --purchase_duration "$purchase_dur" --setup_deadline "$setup_deadline" \
    --demo_seller "$SELLER1" | tail -n 1)"
  invoke "$CONTRACT_ID" demo-member1 join_pool --member "$MEMBER1" --pool_id "$pool_id" >/dev/null
  invoke "$CONTRACT_ID" demo-member2 join_pool --member "$MEMBER2" --pool_id "$pool_id" >/dev/null
  version=1  # join_pool records the automatic terms when the final member joins.
  invoke "$CONTRACT_ID" demo-member1 approve_terms --approver "$MEMBER1" --pool_id "$pool_id" --version "$version" >/dev/null
  invoke "$CONTRACT_ID" demo-member2 approve_terms --approver "$MEMBER2" --pool_id "$pool_id" --version "$version" >/dev/null
  invoke "$CONTRACT_ID" stellerpool-deployer start_pool --pool_id "$pool_id" >/dev/null
  echo "$pool_id"
}

settle_round() {
  local pool_id="$1" recipient_source="$2" recipient_addr="$3" marker="$4"
  invoke "$CONTRACT_ID" "$recipient_source" propose_purchase \
    --member "$recipient_addr" --pool_id "$pool_id" --seller "$SELLER1" --asset "$TOKEN_ID" \
    --amount 20000000 --doc_hash "$(digest "$marker")" >/dev/null
  invoke "$CONTRACT_ID" stellerpool-deployer execute_round --pool_id "$pool_id" >/dev/null
}

echo
echo "==> [Pool A: happy path + cure] create_pool (round=${HAPPY_ROUND_DURATION}s)"
POOL_A="$(create_and_start_pool "$HAPPY_ROUND_DURATION" "$HAPPY_ROUND_DURATION" "$HAPPY_PURCHASE_DURATION")"
echo "    pool_id: $POOL_A"
invoke "$CONTRACT_ID" demo-member1 deposit --member "$MEMBER1" --pool_id "$POOL_A" >/dev/null
invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$POOL_A" >/dev/null
settle_round "$POOL_A" demo-member1 "$MEMBER1" 1
echo "    Round 1 paid the demo seller; round 2 (member2's turn) now open."
invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$POOL_A" >/dev/null
invoke "$CONTRACT_ID" demo-member1 deposit --member "$MEMBER1" --pool_id "$POOL_A" >/dev/null
settle_round "$POOL_A" demo-member2 "$MEMBER2" 2
echo "    Round 2 paid the demo seller; pool $POOL_A is Completed."

echo
echo "==> [Pool B: plan demo scenario] create_pool (round=${DEFAULT_ROUND_DURATION}s grace=${DEFAULT_GRACE_DURATION}s)"
POOL_B="$(create_and_start_pool "$DEFAULT_ROUND_DURATION" "$DEFAULT_GRACE_DURATION" "$DEFAULT_PURCHASE_DURATION")"
echo "    pool_id: $POOL_B"
invoke "$CONTRACT_ID" demo-member1 deposit --member "$MEMBER1" --pool_id "$POOL_B" >/dev/null
invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$POOL_B" >/dev/null
settle_round "$POOL_B" demo-member1 "$MEMBER1" 3
echo "    Round 1 paid the demo seller (member1 received their allocation)."
echo "    Round 2: member2 pays; member1 (already delivered) stops paying entirely."
invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$POOL_B" >/dev/null
echo "    Waiting for the round-2 deadline (~${DEFAULT_ROUND_DURATION}s, real Testnet ledger time)..."
until invoke "$CONTRACT_ID" stellerpool-deployer mark_overdue --pool_id "$POOL_B" 2>/dev/null | grep -q .; do
  sleep 5
done
echo "    Round marked overdue; pool $POOL_B round 2 is in Grace."
echo "    Waiting for the grace deadline (~${DEFAULT_GRACE_DURATION}s, real Testnet ledger time)..."
until invoke "$CONTRACT_ID" stellerpool-deployer abort_pool --pool_id "$POOL_B" 2>/dev/null | grep -q .; do
  sleep 5
done
echo "    Pool $POOL_B is Aborted (SafetyRecovery)."
invoke "$CONTRACT_ID" demo-member2 claim_refund --member "$MEMBER2" --pool_id "$POOL_B" >/dev/null
echo "    member2 claimed their round-2 deposit. member1's round-1 allocation and round-2"
echo "    non-payment are both unrecoverable by design — the plan's accepted economic risk."

echo
echo "==> [Pool C: Draw mode, API v12] create_pool (round=${HAPPY_ROUND_DURATION}s, order_mode=Draw)"
POOL_C="$(create_and_start_pool "$HAPPY_ROUND_DURATION" "$HAPPY_ROUND_DURATION" "$HAPPY_PURCHASE_DURATION" Draw)"
echo "    pool_id: $POOL_C"
invoke "$CONTRACT_ID" demo-member1 deposit --member "$MEMBER1" --pool_id "$POOL_C" >/dev/null
invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$POOL_C" >/dev/null
echo "    Both members paid round 1; drawing the recipient (anyone may call draw_recipient)."
WINNER_1="$(invoke "$CONTRACT_ID" stellerpool-deployer draw_recipient --caller "$ISSUER" --pool_id "$POOL_C" | tail -n 1 | tr -d '"')"
echo "    Round 1 recipient drawn: $WINNER_1"
if [[ "$WINNER_1" == "$MEMBER1" ]]; then WINNER_1_SRC="demo-member1"; else WINNER_1_SRC="demo-member2"; fi
settle_round "$POOL_C" "$WINNER_1_SRC" "$WINNER_1" 4
echo "    Round 1 paid the demo seller; round 2 open (recipient not yet drawn)."
invoke "$CONTRACT_ID" demo-member1 deposit --member "$MEMBER1" --pool_id "$POOL_C" >/dev/null
invoke "$CONTRACT_ID" demo-member2 deposit --member "$MEMBER2" --pool_id "$POOL_C" >/dev/null
WINNER_2="$(invoke "$CONTRACT_ID" stellerpool-deployer draw_recipient --caller "$ISSUER" --pool_id "$POOL_C" | tail -n 1 | tr -d '"')"
echo "    Round 2 recipient drawn: $WINNER_2 (deterministic: the only member who has not received yet)"
if [[ "$WINNER_2" == "$MEMBER1" ]]; then WINNER_2_SRC="demo-member1"; else WINNER_2_SRC="demo-member2"; fi
settle_round "$POOL_C" "$WINNER_2_SRC" "$WINNER_2" 5
echo "    Round 2 paid the demo seller; pool $POOL_C is Completed."

echo
echo "Done. Copy the pool IDs and transaction hashes printed above into"
echo "docs/IMPLEMENTATION_LOG.md as this run's submission evidence."
