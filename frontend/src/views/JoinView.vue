<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import CoinSpinner from '@/components/CoinSpinner.vue'
import Illo from '@/components/Illo.vue'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatStroops, parseAmount, toPlainAmount } from '@/lib/format'
import { contributionFor, GOALS, priceFor } from '@/lib/goals'
import type { Goal } from '@/lib/goals'
import { findMatches, findOwnPool } from '@/lib/matching'
import type { PoolPlan } from '@/lib/matching'
import { demoSellerAddress, explorerTx, poolAsset, poolContractId, poolTokenContractId } from '@/lib/stellar'
import { createPool, getContractCapabilities, joinPool, listRecentPools } from '@/services/pool'
import type { ContractCapabilities } from '@/services/pool'
import { useWalletStore } from '@/stores/wallet'
import { MAX_MEMBERS, MIN_MEMBERS } from '@/types/pool'
import type { OrderMode, PoolInfo } from '@/types/pool'

const wallet = useWalletStore()
const router = useRouter()
const token = poolAsset.getCode()

const DAY = 24 * 60 * 60
/** Takvim: hedefler aylık, demolar dakikalar içinde denenebilsin diye hızlı. Süreler eşleşme anahtarının parçasıdır. */
const CALENDARS = {
  month: { unit: 'months', round: 30 * DAY, grace: 7 * DAY, purchase: 7 * DAY, setup: 7 * DAY },
  demo: { unit: 'rounds', round: 3 * 60, grace: 10 * 60, purchase: 30 * 60, setup: 60 * 60 },
} as const
type CalendarId = keyof typeof CALENDARS

const DOWN_OPTIONS = [0, 10, 20, 30] as const
const CENT = 100_000n // 0.01 unit (7 decimals)

// Ana sayfadaki hesaplama aracından gelen plan değerleri.
const route = useRoute()
const queryAmount = typeof route.query.amount === 'string' ? route.query.amount : ''
const queryMembers = Number.parseInt(String(route.query.members ?? ''), 10)
const queryGoal = typeof route.query.goal === 'string' ? route.query.goal : ''
const queryPrice = typeof route.query.price === 'string' ? route.query.price : ''
const queryInstallment = typeof route.query.installment === 'string' ? route.query.installment : ''
const queryDown = Number.parseInt(String(route.query.down ?? ''), 10)

const parseOrNull = (v: string): bigint | null => {
  if (!v.trim()) return null
  try {
    return parseAmount(v)
  } catch {
    return null
  }
}

const goal = ref('home')
const price = ref('')
const downPct = ref(0)
const desiredInstallment = ref('')
const quickDemo = ref(false)
const calendar = computed<CalendarId>(() => quickDemo.value ? 'demo' : 'month')
const orderMode = computed<OrderMode>(() => {
  const preferred = quickDemo.value ? 'Fixed' : (GOALS.find((g) => g.id === goal.value)?.mode ?? 'Fixed')
  return preferred === 'Draw' && caps.value?.supportsDraw === false ? 'Fixed' : preferred
})

const caps = ref<ContractCapabilities | null>(null)
const legacyContract = computed(() => caps.value?.legacySponsor === true)
const oldFlow = computed(() => caps.value !== null && !caps.value.simpleTerms)
/** Kontrat peşinatı zincirde tutuyor mu (v11+)? */
const downOnChain = computed(() => caps.value?.supportsDownPayment === true)

function pickGoal(g: Goal) {
  goal.value = g.id
  quickDemo.value = false
  downPct.value = g.down
  price.value = toPlainAmount(priceFor(parseAmount(g.pot), g.down))
  const perMember = contributionFor(parseAmount(g.pot), g.members)
  desiredInstallment.value = perMember === null ? '' : toPlainAmount(perMember)
}

function pickQuickDemo() {
  quickDemo.value = true
  goal.value = 'demo'
  price.value = '40'
  downPct.value = 0
  desiredInstallment.value = '10'
}

const initialGoal = GOALS.find((g) => g.id === queryGoal)
if (queryGoal === 'demo' || route.query.demo === '1') pickQuickDemo()
else pickGoal(initialGoal ?? GOALS[0]!)
if (queryPrice && parseOrNull(queryPrice) !== null) price.value = queryPrice
if (queryInstallment && parseOrNull(queryInstallment) !== null) desiredInstallment.value = queryInstallment
if (DOWN_OPTIONS.some((value) => value === queryDown)) downPct.value = queryDown
if (queryAmount !== '' && Number.isInteger(queryMembers) && queryMembers > 0) {
  const amount = parseOrNull(queryAmount)
  if (amount !== null) {
    desiredInstallment.value = toPlainAmount(amount)
    const target = amount * BigInt(queryMembers)
    price.value = toPlainAmount(priceFor(target, downPct.value))
  }
}

const cal = computed(() => CALENDARS[calendar.value])

// --- Hesaplar ------------------------------------------------------------------------------
const priceStroops = computed(() => {
  const v = parseOrNull(price.value)
  return v !== null && v > 0n ? v : null
})
/** Peşinat: toplam bedelin yüzdesi, 0,01 birime aşağı yuvarlanır. */
const downStroops = computed(() =>
  priceStroops.value === null ? null : ((priceStroops.value * BigInt(downPct.value)) / 100n / CENT) * CENT,
)
/** Havuzdan karşılanan tutar = toplam bedel − peşinat. */
const targetStroops = computed(() =>
  priceStroops.value !== null && downStroops.value !== null ? priceStroops.value - downStroops.value : null,
)
const requestedStroops = computed(() => {
  const value = parseOrNull(desiredInstallment.value)
  return value !== null && value > 0n ? value : null
})
/** Kullanıcının ödeyebileceği tutara göre vade ve kişi sayısı. */
const neededMembers = computed(() =>
  targetStroops.value !== null && requestedStroops.value !== null
    ? (targetStroops.value + requestedStroops.value - 1n) / requestedStroops.value
    : null,
)
const term = computed(() =>
  neededMembers.value === null || neededMembers.value > BigInt(MAX_MEMBERS)
    ? null
    : Math.max(MIN_MEMBERS, Number(neededMembers.value)),
)
const installment = computed(() =>
  targetStroops.value === null || term.value === null ? null : contributionFor(targetStroops.value, term.value),
)
const pot = computed(() => installment.value === null || term.value === null ? null : installment.value * BigInt(term.value))
const termLabel = computed(() => term.value === null ? '—' : `${term.value} ${calendar.value === 'demo' ? 'rounds' : 'months'}`)
const installmentLabel = computed(() => calendar.value === 'demo' ? 'Installment per round' : 'Monthly installment')

const sellerValid = computed(() => wallet.address !== demoSellerAddress)

/** Kontrat yetenekleri okunmadan eşleştirme yapılmaz (peşinat zincirde mi bilinmiyor). */
const plan = computed<PoolPlan | null>(() => {
  if (caps.value === null || installment.value === null || term.value === null || !caps.value.simpleTerms) return null
  return {
    token: poolTokenContractId,
    demoSeller: demoSellerAddress,
    contributionAmount: installment.value,
    memberLimit: term.value,
    orderMode: orderMode.value,
    downPayment: downOnChain.value ? (downStroops.value ?? 0n) : 0n,
    roundDuration: cal.value.round,
    graceDuration: cal.value.grace,
    purchaseDuration: cal.value.purchase,
  }
})

// --- Uygun havuzu bulma --------------------------------------------------------------------
const allPools = ref<PoolInfo[] | null>(null)
const poolsLoading = ref(false)
const poolsFailed = ref(false)
const nowSec = ref(Math.floor(Date.now() / 1000))

async function loadPools() {
  if (!poolContractId) return
  poolsLoading.value = true
  try {
    allPools.value = await listRecentPools()
    poolsFailed.value = false
  } catch {
    poolsFailed.value = true
  } finally {
    nowSec.value = Math.floor(Date.now() / 1000)
    poolsLoading.value = false
  }
}

onMounted(() => {
  if (!poolContractId) return
  getContractCapabilities()
    .then((c) => {
      caps.value = c
    })
    .catch(() => {
      /* okunamazsa form açık kalır; gönderimde gerçek hata gösterilir */
    })
  void loadPools()
})

const matches = computed(() =>
  plan.value === null || allPools.value === null ? [] : findMatches(allPools.value, plan.value, wallet.address, nowSec.value),
)
const best = computed(() => matches.value[0] ?? null)
/** Aynı plana uyan bir havuza zaten üyeysem yeni havuz açmak yerine oraya gidilir. */
const ownPool = computed(() =>
  best.value !== null || plan.value === null || allPools.value === null
    ? null
    : findOwnPool(allPools.value, plan.value, wallet.address, nowSec.value),
)
const searching = computed(() => !oldFlow.value && (plan.value === null ? !poolsFailed.value : poolsLoading.value && allPools.value === null))

const planValid = computed(() => priceStroops.value !== null && installment.value !== null && term.value !== null)
const canSubmit = computed(
  () => planValid.value && plan.value !== null && Boolean(poolContractId) && !legacyContract.value && sellerValid.value,
)
const hint = computed(() => {
  if (priceStroops.value === null) return 'Enter a valid total price.'
  if (requestedStroops.value === null) return 'Enter the monthly installment you can pay.'
  if (neededMembers.value !== null && neededMembers.value > BigInt(MAX_MEMBERS)) return `This installment exceeds the ${MAX_MEMBERS}-person limit; raise the installment or lower the price.`
  if (installment.value === null) return 'This amount cannot be divided by the chosen installment.'
  if (!sellerValid.value) return 'This wallet is the demo seller address; try another wallet.'
  return null
})

// --- Katıl ---------------------------------------------------------------------------------
const busy = ref(false)
const phase = ref<'idle' | 'creating' | 'joining'>('idle')
const error = ref<string | null>(null)
const txHash = ref<string | null>(null)

/**
 * Uygun açık havuza katılır; yoksa önce yeni havuzu açar, sonra katılır (iki cüzdan onayı).
 * Liste gönderimden hemen önce yeniden okunur; bayat bir listeyle dolu havuza gidilmez.
 */
async function submit() {
  if (!wallet.address || !canSubmit.value || plan.value === null) return
  const p = plan.value
  const signer = { address: wallet.address, signTransaction: wallet.signTransaction }
  busy.value = true
  error.value = null
  txHash.value = null
  try {
    let pools = allPools.value ?? []
    try {
      pools = await listRecentPools()
      allPools.value = pools
      nowSec.value = Math.floor(Date.now() / 1000)
    } catch {
      /* elde tutulan listeyle devam; katılım zincirde zaten doğrulanır */
    }
    let poolId = findMatches(pools, p, wallet.address, nowSec.value)[0]?.id ?? null
    if (poolId === null) {
      phase.value = 'creating'
      const created = await createPool(signer, {
        token: p.token,
        contributionAmount: p.contributionAmount,
        memberLimit: p.memberLimit,
        orderMode: p.orderMode,
        downPayment: downOnChain.value ? p.downPayment : undefined,
        roundDuration: p.roundDuration,
        graceDuration: p.graceDuration,
        purchaseDuration: p.purchaseDuration,
        setupDeadline: Math.floor(Date.now() / 1000) + cal.value.setup,
        demoSeller: demoSellerAddress,
      })
      txHash.value = created.hash
      if (created.poolId === null) {
        throw new Error('The pool was opened but its number could not be read. Refresh the page; you will be routed to the pool that was opened.')
      }
      poolId = created.poolId
    }
    phase.value = 'joining'
    const joined = await joinPool(signer, poolId)
    txHash.value = joined.hash
    await router.push(`/pool/${poolId}`)
  } catch (e) {
    if (!isUserRejection(e)) error.value = errorMessage(e)
    void loadPools()
  } finally {
    busy.value = false
    phase.value = 'idle'
  }
}

const drawn = computed(() => orderMode.value === 'Draw')
const buttonText = computed(() => {
  if (busy.value) {
    return phase.value === 'creating' ? '1/2 · Opening the pool…' : phase.value === 'joining' ? 'Joining…' : 'Preparing…'
  }
  if (best.value) return `Join ${drawn.value ? 'the draw of ' : ''}pool #${best.value.id}`
  return drawn.value ? 'Join the draw' : 'Join a pool'
})
</script>

<template>
  <div class="mx-auto max-w-xl space-y-5">
    <div>
      <RouterLink to="/" class="inline-flex items-center gap-1 text-sm font-medium text-brand-700 hover:underline">
        <AppIcon name="back" class="!size-4" /> Ana sayfa
      </RouterLink>
      <h1 class="mt-1 text-4xl font-extrabold sm:text-5xl">Join a pool</h1>
      <p class="mt-1 text-stone-600">Enter the total price and the {{ quickDemo ? 'per-round' : 'monthly' }} installment you can afford; the term and group size are calculated automatically.</p>
    </div>

    <p v-if="!poolContractId" role="status" class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950">
      <Illo name="bulb" :size="28" />
      <span>
        The pool contract is not configured yet. You can browse the form; once the address
        <code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> is added, you can join.
      </span>
    </p>
    <p v-if="legacyContract" role="alert" class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950">
      <Illo name="warning" :size="28" />
      <span>The configured contract is an old sponsored version; joining is disabled for now.</span>
    </p>
    <p v-else-if="oldFlow" role="alert" class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950">
      <Illo name="warning" :size="28" />
      <span>The live contract uses the old verifier flow. Joining with this plan requires the v12 contract to be deployed; you can review the form for now.</span>
    </p>

    <form class="card space-y-6 !p-5 sm:!p-7" @submit.prevent="submit()">
      <!-- What for -->
      <div class="flex flex-wrap gap-2" role="group" aria-label="Goal">
        <button
          v-for="g in GOALS"
          :key="g.id"
          type="button"
          class="inline-flex min-h-11 cursor-pointer items-center gap-2 rounded-full border-2 py-1 pr-4 pl-1.5 text-sm font-semibold transition-colors duration-200"
          :class="goal === g.id ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
          :aria-pressed="goal === g.id"
          @click="pickGoal(g)"
        >
          <span class="grid size-8 place-items-center rounded-full bg-white/80"><Illo :name="g.icon" :size="22" /></span>
          {{ g.label }}
        </button>
      </div>

      <button type="button" class="inline-flex min-h-11 items-center gap-2 rounded-full border-2 px-4 text-sm font-semibold transition-colors" :class="quickDemo ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'" :aria-pressed="quickDemo" @click="pickQuickDemo">
        <Illo name="rocket" :size="22" /> Quick demo <span class="font-normal text-stone-600">· rounds of minutes</span>
      </button>

      <!-- Amount + down payment -->
      <div>
        <label class="label" for="price">Total price ({{ token }})</label>
        <input id="price" v-model="price" class="input text-2xl font-bold" type="text" inputmode="decimal" autocomplete="off" />
        <p v-if="priceStroops === null" class="mt-1 text-xs text-rose-700">Enter a valid amount (at most 7 decimals).</p>
        <div class="mt-3 flex flex-wrap items-center gap-2" role="group" aria-label="Down payment">
          <span class="mr-1 text-sm text-stone-600">Down payment</span>
          <button
            v-for="o in DOWN_OPTIONS"
            :key="o"
            type="button"
            class="min-h-11 cursor-pointer rounded-full border-2 px-3.5 text-sm font-semibold transition-colors duration-200"
            :class="downPct === o ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
            :aria-pressed="downPct === o"
            @click="downPct = o"
          >
            {{ o === 0 ? 'None' : `%${o}` }}
          </button>
          <span v-if="downStroops !== null && downStroops > 0n" class="text-sm font-semibold tabular-nums">{{ formatStroops(downStroops) }} {{ token }}</span>
        </div>
      </div>

      <!-- Monthly payment; the number of people and the term are calculated automatically. -->
      <div>
        <label class="label" for="desired-installment">{{ quickDemo ? 'Installment you can pay per round' : 'Monthly installment you can pay' }} ({{ token }})</label>
        <input id="desired-installment" v-model="desiredInstallment" class="input text-xl font-bold" type="text" inputmode="decimal" autocomplete="off" />
        <p class="mt-1 text-xs text-stone-600">{{ quickDemo ? 'In the quick demo, rounds advance within minutes.' : 'The normal plan is monthly.' }} The number of people and the term are calculated from this amount; at most {{ MAX_MEMBERS }} people. You pay {{ quickDemo ? 'each round’s contribution' : 'your monthly contribution' }} by approving it in your own wallet.</p>
      </div>
      <p class="text-sm text-stone-600">Delivery: <strong>{{ drawn ? 'Draw' : 'Join order' }}</strong>. {{ drawn ? 'Each round, one is drawn among members who have not received yet.' : 'When the pool is full, the fixed order is recorded automatically.' }}</p>

      <!-- Result -->
      <div class="rounded-2xl bg-brand-50/80 p-4" aria-live="polite">
        <p class="text-xs text-stone-600">{{ installmentLabel }} · {{ termLabel }} term · {{ term ?? '—' }} people</p>
        <p class="font-display text-4xl font-extrabold tabular-nums" data-testid="installment">
          {{ installment !== null ? formatStroops(installment) : '—' }} <span class="text-lg">{{ token }}</span>
        </p>
        <p class="mt-1 text-xs leading-relaxed text-stone-600">
          {{ term ?? '—' }} {{ quickDemo ? 'rounds' : 'months' }} · {{ term ?? '—' }} installments · pool {{ pot !== null ? formatStroops(pot) : '—' }}
          <template v-if="downStroops !== null && downStroops > 0n">
            · {{ formatStroops(downStroops) }} down payment {{ downOnChain ? 'paid into the contract when you join, added to your purchase when your turn comes' : 'outside the pool, you pay the seller yourself' }}
          </template>
          · no fees
        </p>
      </div>

      <!-- Matching pool -->
      <div class="text-sm" aria-live="polite" data-testid="match-panel">
        <p v-if="!poolContractId" class="text-stone-600">Once the contract is configured, a matching pool will appear here.</p>
        <p v-else-if="oldFlow" class="text-amber-800">New pools will open after the v12 deployment.</p>
        <p v-else-if="!planValid" class="text-stone-600">A matching pool is searched for once the plan is complete.</p>
        <p v-else-if="searching" class="flex items-center gap-2 text-stone-600"><CoinSpinner :size="18" /> Looking for a matching pool…</p>
        <div v-else-if="best" class="rounded-2xl border-2 border-brand-600 bg-white p-3.5">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <p class="font-semibold">Pool #{{ best.id }} · <span data-testid="match-count">{{ best.members.length }}/{{ best.memberLimit }} members</span></p>
            <RouterLink :to="`/pool/${best.id}`" class="font-semibold text-brand-700 underline">View</RouterLink>
          </div>
          <div class="mt-2 h-2 overflow-hidden rounded-full bg-brand-50" role="progressbar" :aria-valuenow="best.members.length" :aria-valuemin="0" :aria-valuemax="best.memberLimit" aria-label="Pool fill level">
            <div class="h-full rounded-full bg-brand-600" :style="{ width: `${(best.members.length / best.memberLimit) * 100}%` }" />
          </div>
        </div>
        <p v-else-if="ownPool" class="rounded-2xl bg-sand/70 p-3.5" data-testid="match-own">
          You are already in <strong>Pool #{{ ownPool.id }}</strong> with this plan ({{ ownPool.members.length }}/{{ ownPool.memberLimit }} members).
        </p>
        <p v-else class="text-stone-700" data-testid="match-none">
          <template v-if="poolsFailed">Existing pools could not be read. <button type="button" class="font-semibold text-brand-700 underline" @click="loadPools()">Try again</button></template>
          <template v-else>There is no open pool for this plan; a new pool will be opened for you, and others choosing the same plan will be routed to it.</template>
        </p>
      </div>

      <p v-if="hint" role="status" class="text-sm text-amber-800">{{ hint }}</p>
      <p v-if="error" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ error }}</p>
      <p v-if="txHash" class="pop rounded-2xl bg-sage-50 p-3 text-sm text-sage-800">
        Transaction sent:
        <a :href="explorerTx(txHash)" target="_blank" rel="noopener noreferrer" class="font-mono underline">{{ txHash.slice(0, 8) }}…</a>
      </p>

      <div class="space-y-3">
        <button v-if="oldFlow || legacyContract" type="button" class="btn-primary btn-lg w-full" disabled>
          Waiting for the new contract
        </button>
        <button v-else-if="!wallet.isConnected" type="button" class="btn-primary btn-lg w-full" :disabled="wallet.busy" @click="wallet.connect()">
          Connect a wallet first
        </button>
        <RouterLink v-else-if="ownPool" :to="`/pool/${ownPool.id}`" class="btn-primary btn-lg w-full" data-testid="go-own-pool">
          Go to pool #{{ ownPool.id }} <AppIcon name="arrow" class="!size-4" />
        </RouterLink>
        <button v-else type="submit" class="btn-primary btn-lg w-full" :disabled="busy || !canSubmit" data-testid="join-button">
          <CoinSpinner v-if="busy" :size="22" />
          {{ buttonText }}
        </button>
        <p class="text-xs leading-relaxed text-stone-600">
          <template v-if="!ownPool">{{ best ? 'Your wallet will ask for one approval.' : 'The pool is opened first, then you join; two approvals are requested.' }}</template>
          Testnet simulation: no real money or home/car delivery. If someone who received early stops paying later installments,
          the earlier-round payments of those still waiting cannot be recovered.
        </p>
      </div>
    </form>
  </div>
</template>
