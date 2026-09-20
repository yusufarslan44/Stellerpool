# Pitch content draft (Pro Hackathon 2026)

**Version note:** This draft still contains historical v10/v11 demo proofs. The current product flow is the verifier-free **API v12** (contract `CB5O6WCGCSA5PKH5HXWKO3WFEDHNEKJ5ZNW6EMNSERYDK6J4VYKLG46O`, live on Testnet, 33 unit tests). For the current product flow, [plan](plan.md) and the [README](../README.md) are authoritative.

This is a slide-by-slide content draft, ready to paste into the official Stellar
presentation template (make a copy and fill it in — do not edit the original). If the
template's own slide order or headings differ, distribute the content accordingly: the
content matters, not the structure.

**Language note:** The pitch is written in English so that a mixed/international jury can
follow it directly.

---

## 1. Title / Team

**Stellarpool** — a transparent, smart-contract-based group savings pool (Stellar/Soroban)

> Save together. Verify everything.

Track: **Genesis**

---

## 2. Problem (Narrative Why)

Saving in a group for a home, a car or a shared goal is very common in Turkey:
"gold days" circles and the draw-based/ordered groups of savings-finance companies
(such as Eminevim and Fuzul).

**The shared problem is trust:**
- The money is collected in the hands of one person or one institution.
- The rules (who receives when, what happens if someone stops paying) are usually not transparent.
- A participant has no proof that answers "where is my money, and are the rules really
  being applied?" — it is a system based on trust.

**Target user:** small groups who know each other and want to save toward a shared
purchase, by rotation or by draw (friends, family, colleagues).

---

## 3. Solution / Value proposition

Stellarpool entrusts this coordination not to a person or an institution but to a
**Soroban smart contract**:

- Contributions are collected directly in the contract per round — the founder **never**
  has the authority to withdraw the shared money alone.
- In the API v12 flow, payment goes to the registered demo seller — not to the
  recipient's pocket — after the **rules approved by all members**, the complete round
  contributions and the recipient's purchase record.
- The recipient is chosen by fixed order or by **draw** (groups of up to 30 members are supported).
- If a payment stalls, the round stops; only **that round's** contributions are refunded
  to their owners — the rules are in code and anyone can verify them on-chain.

While approaching the experience that existing products such as Fuzul/Eminevim offer
(saving for a home or car by order/draw), it offers a model that says "verify" instead of
"trust me".

---

## 4. Live demo

- **Demo URL:** https://stellerpool.arslanyusuf.com (Testnet, connected to the live contract)
- **Contract:** `CB5O6WCGCSA5PKH5HXWKO3WFEDHNEKJ5ZNW6EMNSERYDK6J4VYKLG46O` (API v12,
  Stellar Testnet)
- Flow to show: enter a plan → join a matching pool or open one → members approve the
  terms → start → deposit contributions → (if draw) hold the draw → record the purchase →
  pay the seller.

*(Add 2–3 screenshots or a short GIF/video here: pool creation, the moment of the draw,
a completed pool.)*

---

## 5. Technical architecture

**Smart contract (Soroban, Rust):**
- The `rotating_pool` contract — multi-pool, with a full state machine:
  `Filling → Active → Completed | Aborted`; round: `Collecting → Grace →
  (AwaitingDraw) → AwaitingPurchase → Settled`.
- It evolved through several generations under real production pressure: from a sponsor
  model to a sponsor-free model, then draw mode + 30 members, then a down payment, then
  purchase without verifiers — at every step with a **live Testnet deployment and real
  transaction proof** (`docs/IMPLEMENTATION_LOG.md`).
- **33 unit tests**, including the full 30-member draw flow; every invocation is
  automatically checked against Mainnet resource limits (`InvocationResourceLimits::mainnet()`).
- Checks-effects-interactions pattern, role-based authorization with `require_auth`,
  persistent storage with TTL management.

**Anchor (SEP-1/10/24) — our own backend:**
- Instead of depending on `testanchor.stellar.org`, we wrote **our own SEP-1/10/24 service**
  (`backend/`, Node.js/TypeScript).
- A real Testnet asset (`TRYT`, which represents TRY) was issued; wallet-signed SEP-10
  login, SEP-24 interactive deposit, and a flow that retries the `pending_trust` state by
  itself — verified end-to-end with real Testnet transactions.
- The anchor is no longer a separate showcase; it sits **inside the contribution step of
  the pool** ("Not enough balance? Load it with the anchor").

**Frontend:** Vue 3 + TypeScript, real wallet connection through the Stellar Wallets Kit,
and a capability-detection layer that reads the contract's on-chain interface and turns
features such as draw and capacity on and off automatically.

---

## 6. Stellar ecosystem fit

- **Integration partner:** Stellar Wallets Kit (wallet connection, a core part of every
  action in the app).
- **Anchor / local payments:** our own SEP-1/10/24 service, a real protocol flow (with a
  test asset — not a real TRY/bank integration, and this is stated explicitly).
- **Stellar Skills used** *(full list and rationale: `docs/STELLAR_SKILLS_USED.md`)*:
  - `skills.stellar.org/skills/smart-contracts/SKILL.md`
  - `skills.stellar.org/skills/dapp/SKILL.md`
  - `skills.stellar.org/skills/assets/SKILL.md`
  - `skills.stellar.org/skills/standards/SKILL.md` (SEP selection)
  - `skills.stellar.org/skills/data/SKILL.md`
  - The Anchors skill (SEP-1/6/10/12/24/31/38) — the direct reference for the SEP-10/24
    server implementation.

---

## 7. Known limits (to be stated honestly)

- There is not yet a real, licensed TRY anchor — this is a deliberate scope decision, and
  the next step is clear (see item 8).
- If an early recipient (by order or by draw) stops paying in a later round, the
  contributions of the earlier round do not come back — this is an economic risk the
  product accepts, and group size does not solve it. It is documented transparently
  (`docs/plan.md`).
- On-chain randomness in draw mode is hackathon-grade (the Soroban PRNG, open to
  validator influence) — commit-reveal would be needed for real use.

---

## 8. Traction & next step (roadmap)

- Several phases of real iteration: from the sponsor model to the sponsor-free model, to
  the draw and 30 members, to the down payment and verifier-free purchase — each
  documented with live Testnet proof.
- Building our own anchor and verifying it end-to-end took a concrete step toward the
  "real TRY deposit" goal (with a test asset, but a real protocol).
- **Next step:** talk to a real, licensed TRY anchor provider (SEP-24 compatible),
  KYC/AML integration, the move to mainnet, and an SCF/InstaAward application.

---

## 8b. Likely jury questions about the anchor (preparation note, not part of the pitch)

⚠️ **Check right before the presentation**: is the live site (`stellerpool.arslanyusuf.com`)
currently connected to our own backend, or does it still fall back to the SDF's public
test anchor (the `usingTestAnchor` flag in `frontend/src/lib/anchor.ts` says so — it
likely shows in the interface as a badge/warning). At the time this document was written,
Yusuf's server deployment (`backend/DEPLOY.md`) was not yet complete; if it is complete,
the "short answer" below can be used, and if not, use the "not connected yet" answer —
both are honest and defensible; what matters is knowing which one is true and saying it.

**Short answer (if the backend is connected to the live site):**
> We wrote our own SEP-1/10/24 anchor server — instead of depending on the SDF's public
> test anchor. The real protocol flow works: SEP-10 login, SEP-24 interactive deposit,
> and our own test asset that represents TRY (`TRYT`) reaches the user through real
> on-chain transactions. We verified this both on its own and with our contract (through
> a real pool), end-to-end, with real Testnet transactions. The only non-real part: the
> "I deposited TRY" confirmation does not come from a bank, it is our server's own test
> confirmation — a real bank integration is deliberately out of scope, and it is the
> first step of our roadmap.

**If the backend is not yet connected to the live site:**
> The backend code is written and was verified with real Testnet transactions (two
> separate end-to-end flows, and together with our contract) — [show tx hash]. Connecting
> it to the live site is the last operational step, and we are working on it.

**Likely follow-up questions:**

| Question | Answer |
|---|---|
| Is this real TRY? | No. `TRYT` is a test asset that represents TRY. Real TRY requires a licensed anchor/bank integration — deliberately out of scope, and our next step. |
| Why didn't you use the SDF's test anchor? | The handbook says the anchor should be at the core of the product and is the most heavily weighted criterion. Connecting to a generic third-party test anchor did not show that; building our own SEP infrastructure and proving it end-to-end is stronger technical evidence. |
| Which SEPs does it support? | SEP-1 (stellar.toml), SEP-10 (authentication), SEP-24 (interactive deposit). SEP-12/KYC and withdrawal are out of scope and on the roadmap. |
| How does it connect to the contract? | The balance coming from the anchor is the same asset the pool uses (the code + issuer match is checked) — the user deposits first, then contributes to the pool directly with that balance; it is not a separate showcase, it sits inside the contribution step. |
| Which Stellar Skill did you use? | The Anchors skill (SEP-1/6/10/12/24/31/38) — the direct reference for our SEP-10/24 server implementation; the full list is in `docs/STELLAR_SKILLS_USED.md`. |

## 9. Team

*(Names, roles, contact details — must match the submission form.)*

---

## 10. Closing / Questions

**Stellarpool: "you don't have to trust, you can verify."**
