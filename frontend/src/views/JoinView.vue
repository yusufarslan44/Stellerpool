<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import CoinSpinner from '@/components/CoinSpinner.vue'
import Illo from '@/components/Illo.vue'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatStroops, parseAmount, toPlainAmount } from '@/lib/format'
import { contributionFor, goalMembers, GOALS, priceFor } from '@/lib/goals'
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
  month: { unit: 'ay', round: 30 * DAY, grace: 7 * DAY, purchase: 7 * DAY, setup: 7 * DAY },
  demo: { unit: 'tur', round: 3 * 60, grace: 10 * 60, purchase: 30 * 60, setup: 60 * 60 },
} as const
type CalendarId = keyof typeof CALENDARS

const DOWN_OPTIONS = [0, 10, 20, 30] as const
/** Hızlı vade seçimi (kişi sayısı = taksit sayısı). */
const TERM_OPTIONS = [6, 12, 18, 24, 30].filter((n) => n >= MIN_MEMBERS && n <= MAX_MEMBERS)
const CENT = 100_000n // 0,01 birim (7 ondalık)

// Ana sayfadaki hesaplama aracından gelen değerler (?amount=…&members=…&mode=…&goal=…).
const route = useRoute()
const queryAmount = typeof route.query.amount === 'string' ? route.query.amount : ''
const queryMembers = Number.parseInt(String(route.query.members ?? ''), 10)
const queryGoal = typeof route.query.goal === 'string' ? route.query.goal : ''

const parseOrNull = (v: string): bigint | null => {
  if (!v.trim()) return null
  try {
    return parseAmount(v)
  } catch {
    return null
  }
}

const goal = ref('')
const price = ref('')
const downPct = ref(0)
/** Vade = kişi sayısı = taksit sayısı; kullanıcı vadeyi seçer, taksit hesaplanır. */
const term = ref(4)
const orderMode = ref<OrderMode>('Fixed')
const calendar = ref<CalendarId>('demo')

const caps = ref<ContractCapabilities | null>(null)
const drawUnavailable = computed(() => caps.value !== null && !caps.value.supportsDraw)
const legacyContract = computed(() => caps.value?.legacySponsor === true)
/** Kontrat peşinatı zincirde tutuyor mu (v11+)? */
const downOnChain = computed(() => caps.value?.supportsDownPayment === true)

function pickGoal(g: Goal) {
  goal.value = g.id
  orderMode.value = g.mode === 'Draw' && drawUnavailable.value ? 'Fixed' : g.mode
  calendar.value = g.interval === 'demo' ? 'demo' : 'month'
  downPct.value = g.down
  term.value = goalMembers(g)
  price.value = toPlainAmount(priceFor(parseAmount(g.pot), g.down))
}

// İlk değerler: hesaplayıcıdan gelen plan (bedel = taksit × kişi) ya da seçilen/varsayılan amaç.
const initialGoal = GOALS.find((g) => g.id === queryGoal)
if (queryAmount !== '' && Number.isInteger(queryMembers)) {
  const c = parseOrNull(queryAmount)
  const members = Math.min(MAX_MEMBERS, Math.max(MIN_MEMBERS, queryMembers))
  term.value = members
  if (c !== null) price.value = toPlainAmount(c * BigInt(members))
  orderMode.value = route.query.mode === 'Draw' ? 'Draw' : 'Fixed'
  if (initialGoal) {
    goal.value = initialGoal.id
    calendar.value = initialGoal.interval === 'demo' ? 'demo' : 'month'
  }
} else {
  pickGoal(initialGoal ?? GOALS.find((g) => g.id === 'demo')!)
}

const cal = computed(() => CALENDARS[calendar.value])

function stepTerm(delta: number) {
  term.value = Math.min(MAX_MEMBERS, Math.max(MIN_MEMBERS, (Number(term.value) || MIN_MEMBERS) + delta))
}

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
const termOk = computed(() => Number.isInteger(term.value) && term.value >= MIN_MEMBERS && term.value <= MAX_MEMBERS)
/** Taksit: hedef tutar ÷ vade, 0,01 birime aşağı yuvarlanır (kontrata giden değer). */
const installment = computed(() =>
  targetStroops.value === null || !termOk.value ? null : contributionFor(targetStroops.value, term.value),
)
const pot = computed(() => (installment.value === null || !termOk.value ? null : installment.value * BigInt(term.value)))

function termLabel(n: number): string {
  if (calendar.value === 'demo') return `${n} tur`
  return n >= 12 ? `${n} ay (${Number.isInteger(n / 12) ? n / 12 : (n / 12).toFixed(1)} yıl)` : `${n} ay`
}
const installmentLabel = computed(() => (calendar.value === 'demo' ? 'Tur başı taksit' : 'Aylık taksit'))

const sellerValid = computed(() => wallet.address !== demoSellerAddress)

/** Kontrat yetenekleri okunmadan eşleştirme yapılmaz (peşinat zincirde mi bilinmiyor). */
const plan = computed<PoolPlan | null>(() => {
  if (caps.value === null || installment.value === null || !termOk.value) return null
  return {
    token: poolTokenContractId,
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
      if (!c.supportsDraw && orderMode.value === 'Draw') orderMode.value = 'Fixed'
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
const searching = computed(() => (plan.value === null ? !poolsFailed.value : poolsLoading.value && allPools.value === null))

const planValid = computed(() => priceStroops.value !== null && installment.value !== null && termOk.value)
const canSubmit = computed(
  () => planValid.value && plan.value !== null && Boolean(poolContractId) && !legacyContract.value && sellerValid.value,
)
const hint = computed(() => {
  if (priceStroops.value === null) return 'Geçerli bir toplam bedel gir.'
  if (installment.value === null) return 'Bu tutar bu vadeye bölünemiyor; bedeli ya da vadeyi değiştir.'
  if (!sellerValid.value) return 'Bu cüzdan demo satıcı adresi; başka bir cüzdanla dene.'
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
        throw new Error('Havuz açıldı ama numarası okunamadı. Sayfayı yenile; açılan havuza yönlendirilirsin.')
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
    return phase.value === 'creating' ? '1/2 · Havuz açılıyor…' : phase.value === 'joining' ? 'Katılıyorsun…' : 'Hazırlanıyor…'
  }
  if (best.value) return `Havuz #${best.value.id}${drawn.value ? ' kurasına' : '’e'} katıl`
  return drawn.value ? 'Kuraya katıl' : 'Havuza katıl'
})
</script>

<template>
  <div class="mx-auto max-w-xl space-y-5">
    <div>
      <RouterLink to="/" class="inline-flex items-center gap-1 text-sm font-medium text-brand-700 hover:underline">
        <AppIcon name="back" class="!size-4" /> Ana sayfa
      </RouterLink>
      <h1 class="mt-1 text-4xl font-extrabold sm:text-5xl">Havuza katıl</h1>
      <p class="mt-1 text-stone-600">Tutarı ve vadeyi seç, sana uyan havuza yönlendirelim.</p>
    </div>

    <p v-if="!poolContractId" role="status" class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950">
      <Illo name="bulb" :size="28" />
      <span>
        Havuz sözleşmesi henüz yapılandırılmadı. Formu gezebilirsin; adres
        <code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> olarak eklenince katılabilirsin.
      </span>
    </p>
    <p v-if="legacyContract" role="alert" class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950">
      <Illo name="warning" :size="28" />
      <span>Yapılandırılan kontrat eski sponsorlu sürüm; katılım şimdilik kapalı.</span>
    </p>

    <form class="card space-y-6 !p-5 sm:!p-7" @submit.prevent="submit()">
      <!-- Ne için -->
      <div class="flex flex-wrap gap-2" role="group" aria-label="Amaç">
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

      <!-- Tutar + peşinat -->
      <div>
        <label class="label" for="price">Toplam bedel ({{ token }})</label>
        <input id="price" v-model="price" class="input text-2xl font-bold" type="text" inputmode="decimal" autocomplete="off" />
        <p v-if="priceStroops === null" class="mt-1 text-xs text-rose-700">Geçerli bir tutar gir (en fazla 7 ondalık).</p>
        <div class="mt-3 flex flex-wrap items-center gap-2" role="group" aria-label="Peşinat">
          <span class="mr-1 text-sm text-stone-600">Peşinat</span>
          <button
            v-for="o in DOWN_OPTIONS"
            :key="o"
            type="button"
            class="min-h-11 cursor-pointer rounded-full border-2 px-3.5 text-sm font-semibold transition-colors duration-200"
            :class="downPct === o ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
            :aria-pressed="downPct === o"
            @click="downPct = o"
          >
            {{ o === 0 ? 'Yok' : `%${o}` }}
          </button>
          <span v-if="downStroops !== null && downStroops > 0n" class="text-sm font-semibold tabular-nums">{{ formatStroops(downStroops) }} {{ token }}</span>
        </div>
      </div>

      <!-- Vade -->
      <div>
        <div class="flex items-baseline justify-between gap-3">
          <label class="label" for="term">Vade</label>
          <p class="font-display text-xl font-extrabold tabular-nums" data-testid="term-value">{{ termOk ? termLabel(term) : '—' }}</p>
        </div>
        <div class="flex items-center gap-3">
          <button type="button" class="btn-secondary !size-11 shrink-0 !p-0 text-xl" :disabled="term <= MIN_MEMBERS" aria-label="Vadeyi kısalt" @click="stepTerm(-1)">−</button>
          <input
            id="term"
            v-model.number="term"
            class="h-11 w-full cursor-pointer accent-brand-600"
            type="range"
            :min="MIN_MEMBERS"
            :max="MAX_MEMBERS"
            step="1"
            :aria-valuetext="`${term} taksit`"
          />
          <button type="button" class="btn-secondary !size-11 shrink-0 !p-0 text-xl" :disabled="term >= MAX_MEMBERS" aria-label="Vadeyi uzat" @click="stepTerm(1)">+</button>
        </div>
        <div class="mt-1 flex flex-wrap gap-2" role="group" aria-label="Hızlı vade">
          <button
            v-for="n in TERM_OPTIONS"
            :key="n"
            type="button"
            class="min-h-11 cursor-pointer rounded-full border-2 px-3.5 text-sm font-semibold transition-colors duration-200"
            :class="term === n ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
            :aria-pressed="term === n"
            @click="term = n"
          >
            {{ n }} {{ cal.unit }}
          </button>
        </div>
      </div>

      <!-- Sıra -->
      <div class="flex flex-wrap items-center gap-2" role="radiogroup" aria-label="Teslim sırası">
        <span class="mr-1 text-sm text-stone-600">Teslim</span>
        <button
          type="button"
          role="radio"
          class="min-h-11 cursor-pointer rounded-full border-2 px-4 text-sm font-semibold transition-colors duration-200 disabled:cursor-not-allowed disabled:opacity-50"
          :class="orderMode === 'Draw' ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
          :aria-checked="orderMode === 'Draw'"
          :disabled="drawUnavailable"
          @click="orderMode = 'Draw'"
        >
          Kura
        </button>
        <button
          type="button"
          role="radio"
          class="min-h-11 cursor-pointer rounded-full border-2 px-4 text-sm font-semibold transition-colors duration-200"
          :class="orderMode === 'Fixed' ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
          :aria-checked="orderMode === 'Fixed'"
          @click="orderMode = 'Fixed'"
        >
          Sabit sıra
        </button>
        <span class="text-xs text-stone-600">{{ drawn ? 'Her tur, henüz almamışlar arasından çekilir.' : 'Sırayı üyeler birlikte onaylar.' }}</span>
      </div>

      <!-- Sonuç -->
      <div class="rounded-2xl bg-brand-50/80 p-4" aria-live="polite">
        <p class="text-xs text-stone-600">{{ installmentLabel }}</p>
        <p class="font-display text-4xl font-extrabold tabular-nums" data-testid="installment">
          {{ installment !== null ? formatStroops(installment) : '—' }} <span class="text-lg">{{ token }}</span>
        </p>
        <p class="mt-1 text-xs leading-relaxed text-stone-600">
          {{ termOk ? term : '—' }} taksit · havuz {{ pot !== null ? formatStroops(pot) : '—' }}
          <template v-if="downStroops !== null && downStroops > 0n">
            · {{ formatStroops(downStroops) }} peşinat {{ downOnChain ? 'katılırken kontrata yatar, sıran gelince alımına eklenir' : 'havuz dışı, satıcıya sen ödersin' }}
          </template>
          · ücret yok
        </p>
      </div>

      <!-- Uygun havuz -->
      <div class="text-sm" aria-live="polite" data-testid="match-panel">
        <p v-if="!poolContractId" class="text-stone-600">Sözleşme yapılandırılınca uygun havuz burada görünür.</p>
        <p v-else-if="!planValid" class="text-stone-600">Planı tamamlayınca uygun havuz aranır.</p>
        <p v-else-if="searching" class="flex items-center gap-2 text-stone-600"><CoinSpinner :size="18" /> Uygun havuz aranıyor…</p>
        <div v-else-if="best" class="rounded-2xl border-2 border-brand-600 bg-white p-3.5">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <p class="font-semibold">Havuz #{{ best.id }} · <span data-testid="match-count">{{ best.members.length }}/{{ best.memberLimit }} üye</span></p>
            <RouterLink :to="`/pool/${best.id}`" class="font-semibold text-brand-700 underline">İncele</RouterLink>
          </div>
          <div class="mt-2 h-2 overflow-hidden rounded-full bg-brand-50" role="progressbar" :aria-valuenow="best.members.length" :aria-valuemin="0" :aria-valuemax="best.memberLimit" aria-label="Havuz doluluğu">
            <div class="h-full rounded-full bg-brand-600" :style="{ width: `${(best.members.length / best.memberLimit) * 100}%` }" />
          </div>
        </div>
        <p v-else-if="ownPool" class="rounded-2xl bg-sand/70 p-3.5" data-testid="match-own">
          Bu planda zaten <strong>Havuz #{{ ownPool.id }}</strong> içindesin ({{ ownPool.members.length }}/{{ ownPool.memberLimit }} üye).
        </p>
        <p v-else class="text-stone-700" data-testid="match-none">
          <template v-if="poolsFailed">Mevcut havuzlar okunamadı. <button type="button" class="font-semibold text-brand-700 underline" @click="loadPools()">Tekrar dene</button></template>
          <template v-else>Bu plana açık havuz yok; senin için yeni bir havuz açılır, aynı planı seçenler ona yönlenir.</template>
        </p>
      </div>

      <p v-if="hint" role="status" class="text-sm text-amber-800">{{ hint }}</p>
      <p v-if="error" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ error }}</p>
      <p v-if="txHash" class="pop rounded-2xl bg-sage-50 p-3 text-sm text-sage-800">
        İşlem gönderildi:
        <a :href="explorerTx(txHash)" target="_blank" rel="noopener noreferrer" class="font-mono underline">{{ txHash.slice(0, 8) }}…</a>
      </p>

      <div class="space-y-3">
        <button v-if="!wallet.isConnected" type="button" class="btn-primary btn-lg w-full" :disabled="wallet.busy" @click="wallet.connect()">
          Önce cüzdan bağla
        </button>
        <RouterLink v-else-if="ownPool" :to="`/pool/${ownPool.id}`" class="btn-primary btn-lg w-full" data-testid="go-own-pool">
          Havuz #{{ ownPool.id }}’e git <AppIcon name="arrow" class="!size-4" />
        </RouterLink>
        <button v-else type="submit" class="btn-primary btn-lg w-full" :disabled="busy || !canSubmit" data-testid="join-button">
          <CoinSpinner v-if="busy" :size="22" />
          {{ buttonText }}
        </button>
        <p class="text-xs leading-relaxed text-stone-600">
          <template v-if="!ownPool">{{ best ? 'Cüzdanında tek onay istenir.' : 'Havuz önce açılır, sonra katılırsın; iki onay istenir.' }}</template>
          Testnet simülasyonu: gerçek para ya da ev/araç teslimi yok. Erken teslim alan sonraki taksitleri bırakırsa
          bekleyenlerin önceki tur ödemeleri geri alınamaz.
        </p>
      </div>
    </form>
  </div>
</template>
