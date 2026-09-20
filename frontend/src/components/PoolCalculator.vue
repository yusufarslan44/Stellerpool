<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import Illo from '@/components/Illo.vue'
import Scene3D from '@/components/Scene3D.vue'
import { formatStroops, parseAmount, toPlainAmount } from '@/lib/format'
import { contributionFor, GOALS, priceFor } from '@/lib/goals'
import type { Goal } from '@/lib/goals'
import { MAX_MEMBERS, MIN_MEMBERS } from '@/types/pool'
import { poolAsset } from '@/lib/stellar'

const token = poolAsset.getCode()
const goal = ref(GOALS[0]!.id)
const scenePaused = ref(false)
const price = ref(toPlainAmount(priceFor(parseAmount(GOALS[0]!.pot), GOALS[0]!.down)))
const downPct = ref(GOALS[0]!.down)
const desiredInstallment = ref(toPlainAmount(contributionFor(parseAmount(GOALS[0]!.pot), GOALS[0]!.members)!))
const DOWN_OPTIONS = [0, 10, 20, 30] as const
const CENT = 100_000n
function pick(g: Goal) {
  goal.value = g.id
  downPct.value = g.down
  price.value = toPlainAmount(priceFor(parseAmount(g.pot), g.down))
  desiredInstallment.value = toPlainAmount(contributionFor(parseAmount(g.pot), g.members)!)
}
function positiveAmount(value: string): bigint | null {
  try {
    const amount = parseAmount(value)
    return amount > 0n ? amount : null
  } catch {
    return null
  }
}
const priceAmount = computed(() => positiveAmount(price.value))
const requested = computed(() => positiveAmount(desiredInstallment.value))
const down = computed(() => priceAmount.value === null ? null : ((priceAmount.value * BigInt(downPct.value)) / 100n / CENT) * CENT)
const target = computed(() => priceAmount.value === null || down.value === null ? null : priceAmount.value - down.value)
const needed = computed(() => target.value === null || requested.value === null ? null : (target.value + requested.value - 1n) / requested.value)
const members = computed(() => needed.value === null || needed.value > BigInt(MAX_MEMBERS) ? null : Math.max(MIN_MEMBERS, Number(needed.value)))
const contribution = computed(() => target.value === null || members.value === null ? null : contributionFor(target.value, members.value))
const effectivePot = computed(() => contribution.value === null || members.value === null ? null : contribution.value * BigInt(members.value))
const selectedGoal = computed(() => GOALS.find((g) => g.id === goal.value) ?? GOALS[0]!)
const createLink = computed(() => ({
  path: '/join',
  query: { goal: goal.value, price: price.value, down: String(downPct.value), installment: desiredInstallment.value },
}))
</script>

<template>
  <section class="plan-calculator card overflow-hidden !p-0" aria-labelledby="hesap-araci">
    <div class="grid lg:grid-cols-[1.1fr_1fr]">
      <div class="space-y-6 p-5 sm:p-8">
        <div>
          <h3 id="hesap-araci" class="text-2xl font-extrabold">Calculate your plan</h3>
          <p class="mt-1 text-sm text-stone-600">See your monthly plan for a home, a car or other goals.</p>
        </div>

        <div>
          <p class="label" id="calc-goal">1. What are you saving for?</p>
          <div class="flex flex-wrap gap-2" role="group" aria-labelledby="calc-goal">
            <button v-for="g in GOALS" :key="g.id" type="button"
              class="inline-flex min-h-11 cursor-pointer items-center gap-1.5 rounded-full border-2 px-4 text-sm font-semibold transition-[background-color,border-color,transform] duration-200 active:scale-95"
              :class="goal === g.id ? 'border-brand-600 bg-brand-600 text-white' : 'border-stone-200 bg-white text-stone-700 hover:border-brand-300'"
              :aria-pressed="goal === g.id" @click="pick(g)">
              <Illo :name="g.icon" :size="22" />{{ g.label }}
            </button>
          </div>
        </div>

        <div class="grid gap-5 sm:grid-cols-2">
          <div>
            <label class="label" for="calc-price">2. Total price ({{ token }})</label>
            <input id="calc-price" v-model="price" class="input text-lg font-bold" type="text" inputmode="decimal" autocomplete="off" />
          </div>
          <div>
            <label class="label" for="calc-installment">3. What you can pay monthly ({{ token }})</label>
            <input id="calc-installment" v-model="desiredInstallment" class="input text-lg font-bold" type="text" inputmode="decimal" autocomplete="off" />
          </div>
        </div>
        <div class="flex flex-wrap items-center gap-2" role="group" aria-label="Down payment">
          <span class="text-sm text-stone-600">Down payment</span>
          <button v-for="o in DOWN_OPTIONS" :key="o" type="button" class="min-h-10 cursor-pointer rounded-full border px-3 text-sm font-semibold"
            :class="downPct === o ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white text-stone-700'"
            :aria-pressed="downPct === o" @click="downPct = o">{{ o === 0 ? 'None' : `%${o}` }}</button>
        </div>

        <dl v-if="contribution !== null && effectivePot !== null" class="grid grid-cols-2 gap-3">
          <div class="rounded-2xl bg-sand/70 p-3.5"><dt class="text-xs text-stone-600">Your monthly installment</dt><dd class="mt-0.5 font-display text-xl font-extrabold tabular-nums">{{ formatStroops(contribution) }}</dd></div>
          <div class="rounded-2xl bg-sand/70 p-3.5"><dt class="text-xs text-stone-600">Term and group</dt><dd class="mt-0.5 font-display text-xl font-extrabold tabular-nums">{{ members }} months · {{ members }} people</dd></div>
          <div class="rounded-2xl bg-brand-50 p-3.5 ring-1 ring-brand-100"><dt class="text-xs text-brand-800">Pool target</dt><dd class="mt-0.5 font-display text-xl font-extrabold tabular-nums text-brand-900">{{ formatStroops(effectivePot) }}</dd></div>
          <div class="rounded-2xl bg-sage-50 p-3.5 ring-1 ring-sage-100"><dt class="text-xs text-sage-800">How is the recipient chosen?</dt><dd class="mt-0.5 text-sm font-bold text-sage-900">{{ selectedGoal.mode === 'Draw' ? 'Draw, once all contributions are in' : 'Join order' }}</dd></div>
        </dl>
        <p v-else class="rounded-2xl bg-gold-100 p-3 text-sm text-amber-950" role="status">Enter a valid price and monthly installment; the plan can be at most {{ MAX_MEMBERS }} months.</p>

        <div>
          <RouterLink :to="createLink" class="btn-primary btn-lg w-full sm:w-auto" :class="contribution === null ? 'pointer-events-none opacity-50' : ''" :aria-disabled="contribution === null">
            Join a pool with this plan <AppIcon name="arrow" class="!size-4" />
          </RouterLink>
          <p class="mt-3 text-xs leading-relaxed text-stone-600">The monthly schedule is set automatically; each member pays the installment from their own wallet. A quick demo is chosen separately on the join screen. {{ token }} is a test asset; there is no real money or delivery guarantee.</p>
        </div>
      </div>

      <div class="calculator-scene sunrise relative min-h-[300px] border-t border-stone-100 lg:min-h-0 lg:border-t-0 lg:border-l">
        <div class="calculator-scene-top"><span>YOUR PLAN, VISUALIZED</span><button type="button" :aria-label="scenePaused ? 'Play plan animation' : 'Pause plan animation'" @click="scenePaused = !scenePaused">{{ scenePaused ? '▷' : 'Ⅱ' }}</button></div>
        <Scene3D :paused="scenePaused" :coins="members ?? 0" :label="`${members ?? 0} gold coins representing the members orbit the pool`" />
        <div class="glass absolute right-4 bottom-4 left-4 flex items-center justify-between gap-2 rounded-2xl px-4 py-2.5 text-sm font-semibold">
          <span>{{ members ?? '—' }} members · {{ members ?? '—' }} months</span>
          <span v-if="effectivePot !== null" class="tabular-nums text-brand-800">Round: {{ formatStroops(effectivePot) }} {{ token }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.plan-calculator { border-radius: 30px; border-color: #dce4cf; background: #fffef8; box-shadow: 0 18px 45px -30px #3b61343d; }
.calculator-scene { background: radial-gradient(ellipse at 50% 45%,#f8f8e5,#dde8ca); }
.calculator-scene-top { position: absolute; z-index: 1; top: 22px; left: 24px; right: 24px; display: flex; align-items: center; justify-content: space-between; color: #859572; font-size: 9px; letter-spacing: .14em; }
.calculator-scene-top button { display: grid; place-items: center; width: 29px; height: 29px; border: 1px solid #b5c79c; border-radius: 50%; color: #6c8354; font-size: 12px; cursor: pointer; }
.plan-calculator :deep(input) { border-color: #d4dfc4; }
.plan-calculator [aria-pressed] { border-radius: 12px; font-size: 12px; }
</style>
