<script setup lang="ts">
import { config } from '@/lib/stellar'

type Tone = 'none' | 'small' | 'note'
interface Fee {
  item: string
  amount: string
  tone: Tone
  detail: string
  goesTo: string
}

// Every row was verified against the code: the contract has no fee field/recipient, and the anchor uses `fee: { enabled: false }`.
const FEES: Fee[] = [
  {
    item: 'Organization fee',
    amount: 'None',
    tone: 'none',
    detail: 'Savings-finance companies typically charge a one-time fee of roughly 7–14%. Here, no fee is set aside for anyone.',
    goesTo: '—',
  },
  {
    item: 'Stellarpool commission',
    amount: 'None',
    tone: 'none',
    detail: 'The contract has no fee field and no address that could collect a fee; money leaves the contract only to you or to the seller.',
    goesTo: '—',
  },
  {
    item: 'Interest or term surcharge',
    amount: 'None',
    tone: 'none',
    detail: 'The installment is the total price divided by the number of people; nothing is added on top.',
    goesTo: '—',
  },
  {
    item: 'Down payment',
    amount: 'Not a fee',
    tone: 'note',
    detail: 'Paid when you join, held in the contract, and added only to your own purchase when your turn comes. If the pool is cancelled, the unspent part is refunded.',
    goesTo: 'Your own purchase',
  },
  {
    item: 'Network transaction fee',
    amount: 'Very small',
    tone: 'small',
    detail: `A small amount of XLM is paid to the ${config.label} network with every signed transaction. On Testnet, Friendbot provides it for free.`,
    goesTo: 'Stellar network',
  },
  {
    item: 'Anchor fee',
    amount: 'None on test',
    tone: 'none',
    detail: 'Fees are turned off on our test anchor. A real anchor sets its own deposit/withdrawal fees.',
    goesTo: 'Anchor',
  },
]

const TONE: Record<Tone, string> = {
  none: 'bg-sage-100 text-sage-800',
  small: 'bg-gold-100 text-amber-900',
  note: 'bg-stone-200 text-stone-800',
}
</script>

<template>
  <div class="fees">
    <div class="fees-table" role="table" aria-label="Fees and cost items">
      <div class="fees-head" role="row">
        <span role="columnheader">Item</span>
        <span role="columnheader">How much</span>
        <span role="columnheader">What it means</span>
        <span role="columnheader">Goes to</span>
      </div>
      <div v-for="f in FEES" :key="f.item" class="fees-row" role="row">
        <strong role="cell" class="fees-item">{{ f.item }}</strong>
        <span role="cell"><b class="fees-pill" :class="TONE[f.tone]">{{ f.amount }}</b></span>
        <p role="cell" class="fees-detail">{{ f.detail }}</p>
        <span role="cell" class="fees-goes"><i>Goes to:</i> {{ f.goesTo }}</span>
      </div>
    </div>
    <p class="fees-note">
      This is a Testnet prototype; the fees of a real product would be set separately. Company rates are typical values summarized from public pages,
      not advice.
    </p>
  </div>
</template>

<style scoped>
.fees-table { border: 1px solid #dfe4d1; border-radius: 24px; background: #fffff9; overflow: hidden; }
.fees-head, .fees-row { display: grid; grid-template-columns: 1.1fr .8fr 2.6fr .8fr; gap: 16px; align-items: center; padding: 16px 24px; }
.fees-head { background: #f0f2e4; color: #7a8368; font-size: 10px; letter-spacing: .12em; text-transform: uppercase; }
.fees-row { border-top: 1px solid #e8ecdb; }
.fees-item { color: #365037; font: 600 15px var(--font-display); }
.fees-pill { display: inline-block; padding: 4px 11px; border-radius: 999px; font-size: 12px; font-weight: 600; white-space: nowrap; }
.fees-detail { color: #6f7862; font-size: 12px; line-height: 1.75; }
.fees-goes { color: #5c6a4d; font-size: 12px; }
.fees-goes i { display: none; font-style: normal; color: #9a9f88; }
.fees-note { margin-top: 14px; color: #828973; font-size: 11px; line-height: 1.8; text-align: center; }
@media (max-width: 767px) {
  .fees-head { display: none; }
  .fees-row { grid-template-columns: 1fr auto; gap: 6px 12px; padding: 16px 18px; }
  .fees-detail { grid-column: 1 / -1; }
  .fees-goes { grid-column: 1 / -1; }
  .fees-goes i { display: inline; }
}
</style>
