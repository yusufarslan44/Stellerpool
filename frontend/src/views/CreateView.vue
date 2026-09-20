<script setup lang="ts">
import { StrKey } from '@stellar/stellar-sdk'
import { computed, onMounted, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import CoinSpinner from '@/components/CoinSpinner.vue'
import Illo from '@/components/Illo.vue'
import Scene3D from '@/components/Scene3D.vue'
import StepIndicator from '@/components/StepIndicator.vue'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatStroops, parseAmount, toPlainAmount } from '@/lib/format'
import { contributionFor, goalMembers, GOALS, priceFor } from '@/lib/goals'
import type { Goal } from '@/lib/goals'
import { explorerTx, poolAsset, poolContractId, poolTokenContractId } from '@/lib/stellar'
import { createPool, getContractCapabilities } from '@/services/pool'
import type { ContractCapabilities } from '@/services/pool'
import { useWalletStore } from '@/stores/wallet'
import { MAX_MEMBERS, MIN_MEMBERS } from '@/types/pool'
import type { OrderMode } from '@/types/pool'

const wallet = useWalletStore()
const router = useRouter()
const token = poolAsset.getCode()

const DAY = 24 * 60 * 60
const PRESETS = [
  { id: 'demo', icon: 'rocket', label: 'Hızlı demo', hint: 'Dakikalar içinde denemek için', round: 3 * 60, grace: 10 * 60, purchase: 30 * 60, setup: 60 * 60 },
  { id: 'day', icon: 'alarm', label: 'Günlük', hint: 'Her tur için bir gün', round: DAY, grace: DAY, purchase: DAY, setup: DAY },
  { id: 'week', icon: 'memo', label: 'Haftalık', hint: 'Her tur için yedi gün', round: 7 * DAY, grace: 7 * DAY, purchase: 7 * DAY, setup: 7 * DAY },
  { id: 'month', icon: 'calendar', label: 'Aylık', hint: 'Aylık tur, haftalık ek süre', round: 30 * DAY, grace: 7 * DAY, purchase: 7 * DAY, setup: 7 * DAY },
] as const

const ROUND_OPTIONS = [
  { label: '3 dakika (demo)', seconds: 3 * 60 },
  { label: '1 saat', seconds: 60 * 60 },
  { label: '1 gün', seconds: DAY },
  { label: '7 gün', seconds: 7 * DAY },
  { label: '30 gün (aylık)', seconds: 30 * DAY },
]

/** Tasarruf finansman şirketleri (Fuzul Ev/Oto, Eminevim gibi) ile fark; kamuya açık sayfalardan özetlenmiştir. */
const COMPARE_ROWS = [
  { topic: 'Peşinat', company: 'İsteğe bağlı; şirkete yatar, teslimi öne çeker ya da vadeyi kısaltır.', ours: '' },
  { topic: 'Grup büyüklüğü', company: 'Şirket belirler, müşteri seçmez.', ours: 'Otomatik: havuz hedefi ÷ ödeyebileceğin taksit (2–30 kişi); elle de belirlenebilir.' },
  { topic: 'Ücret', company: 'Tek seferlik organizasyon ücreti (yaklaşık %7–14), cayınca iade edilmez.', ours: 'Yok. Kimseye pay ayrılmaz.' },
  { topic: 'Vade', company: 'Genelde 40–240 ay.', ours: 'Kişi sayısı kadar tur (2–30).' },
  { topic: 'Kura', company: 'Çekilişli modelde her ay, noter kontrolünde; çekilişsizde tarih sözleşmede sabit.', ours: 'Her tur, herkes ödeyince. Kontrat seçer, herkes çağırabilir; noter yok.' },
  { topic: 'Teslim', company: 'Şirket tahsisatı hak edene öder; sonra ipotek/rehin.', ours: 'Doğrulayıcı onayıyla yalnızca izinli satıcıya. İpotek/rehin yok.' },
  { topic: 'Ödeme aksarsa', company: '6 aya kadar taksit dondurma, teslim ertelenir.', ours: 'Ek süre, sonra iptal; yalnızca mevcut tur iade edilir.' },
] as const

const STEPS = [{ label: 'Plan' }, { label: 'Süreler' }, { label: 'Kişiler' }, { label: 'Onay' }]

// Ana sayfadaki hesaplama aracından gelen değerler (?amount=…&members=…).
const route = useRoute()
const queryAmount = typeof route.query.amount === 'string' ? route.query.amount : ''
const queryMembers = Number.parseInt(String(route.query.members ?? ''), 10)
const queryMode: OrderMode = route.query.mode === 'Draw' ? 'Draw' : 'Fixed'

const step = ref(0)
const dir = ref<'next' | 'prev'>('next')

const queryGoal = typeof route.query.goal === 'string' ? route.query.goal : ''
// Hesaplayıcıdan özel bir plan geldiyse ("Demo" çipi yanlış seçili görünmesin) hiçbir amaç seçili olmaz.
const goal = ref<string>(GOALS.some((g) => g.id === queryGoal) ? queryGoal : queryAmount ? '' : 'demo')
const CENT = 100_000n // 0,01 birim (7 ondalık)
const parseOrNull = (v: string): bigint | null => {
  if (!v.trim()) return null
  try {
    return parseAmount(v)
  } catch {
    return null
  }
}

/** Toplam bedel: ev veya araç değeri. */
const price = ref('')
/** Peşinat: alıcının satıcıya HAVUZ DIŞINDA doğrudan ödediği kısım. Kontrat bunu yönetmez ve doğrulamaz. */
const downPayment = ref('0')
/** Son elle seçilen peşinat oranı çipi; bedel değişince oran korunur. */
const downMode = ref<'pct' | 'abs'>('pct')
const downPctChosen = ref(0)
/** Ödeyebileceğin aylık taksit (istek). Kişi sayısı ve vade buradan otomatik çıkar. */
const amount = ref(queryAmount || '10')
/** Kişi sayısı otomatik mi (hedef ÷ taksit) yoksa elle mi belirleniyor? */
const autoMembers = ref(!Number.isInteger(queryMembers))
const memberLimit = ref(
  Number.isInteger(queryMembers) && queryMembers >= MIN_MEMBERS && queryMembers <= MAX_MEMBERS ? queryMembers : 4,
)
const orderMode = ref<OrderMode>(queryMode)

// Kontratın gerçek yetenekleri zincirden okunur; kura yoksa seçenek kapatılır.
const caps = ref<ContractCapabilities | null>(null)
const drawUnavailable = computed(() => caps.value !== null && !caps.value.supportsDraw)
const legacyContract = computed(() => caps.value?.legacySponsor === true)
/** Kontrat peşinatı zincirde tutuyor mu (v11+)? Değilse peşinat yalnızca plan hesabıdır. */
const downOnChain = computed(() => caps.value?.supportsDownPayment === true)
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
})
const preset = ref<string>('demo')
const custom = ref(false)
const duration = ref(3 * 60)
const graceDuration = ref(10 * 60)
const purchaseDuration = ref(30 * 60)
const setupDuration = ref(60 * 60)
const demoSellerInput = ref('')
const busy = ref(false)
const error = ref<string | null>(null)
const txHash = ref<string | null>(null)

function pickGoal(g: Goal) {
  goal.value = g.id
  orderMode.value = g.mode === 'Draw' && drawUnavailable.value ? 'Fixed' : g.mode
  const pot = parseAmount(g.pot)
  const total = priceFor(pot, g.down)
  price.value = toPlainAmount(total)
  downMode.value = 'pct'
  downPctChosen.value = g.down
  downPayment.value = toPlainAmount(total - pot)
  autoMembers.value = true
  memberLimit.value = goalMembers(g)
  const c = contributionFor(pot, goalMembers(g))
  if (c !== null) amount.value = toPlainAmount(c)
  const p = PRESETS.find((x) => x.id === g.interval)
  if (p) pickPreset(p)
}

const DOWN_OPTIONS = [0, 10, 20, 30] as const
/** Peşinatı toplam bedelin yüzdesi olarak ayarlar (0,01 birime aşağı yuvarlanır). */
function setDownPct(pct: number) {
  downMode.value = 'pct'
  downPctChosen.value = pct
  const p = priceStroops.value
  if (p === null) return
  downPayment.value = toPlainAmount(((p * BigInt(pct)) / 100n / CENT) * CENT)
}
function onPriceInput() {
  if (downMode.value === 'pct') setDownPct(downPctChosen.value)
}
function onDownInput() {
  downMode.value = 'abs'
}

function pickPreset(p: (typeof PRESETS)[number]) {
  preset.value = p.id
  custom.value = false
  duration.value = p.round
  graceDuration.value = p.grace
  purchaseDuration.value = p.purchase
  setupDuration.value = p.setup
}

function stepMembers(delta: number) {
  memberLimit.value = Math.min(MAX_MEMBERS, Math.max(MIN_MEMBERS, (Number(memberLimit.value) || 4) + delta))
}

const priceStroops = computed(() => {
  const v = parseOrNull(price.value)
  return v !== null && v > 0n ? v : null
})
const downStroops = computed(() => (downPayment.value.trim() === '' ? 0n : parseOrNull(downPayment.value)))
/** Havuzdan karşılanan tutar (hedef tutar) = toplam bedel − peşinat. */
const targetStroops = computed(() => {
  const p = priceStroops.value
  const d = downStroops.value
  return p !== null && d !== null && d >= 0n && d < p ? p - d : null
})
const downPctShown = computed(() =>
  priceStroops.value !== null && downStroops.value !== null
    ? Math.round(Number((downStroops.value * 10000n) / priceStroops.value)) / 100
    : 0,
)
const requestedInstallment = computed(() => {
  const v = parseOrNull(amount.value)
  return v !== null && v > 0n ? v : null
})
/** Ödenebilecek taksite göre gereken kişi sayısı (üst/alt sınıra kırpılmadan). */
const rawMembers = computed(() => {
  const t = targetStroops.value
  const c = requestedInstallment.value
  return t === null || c === null ? null : Number((t + c - 1n) / c)
})
// Fuzul Ev'deki gibi: kişi sayısı ve vade, hedef ÷ ödenebilir taksitten otomatik çıkar (2–30 arası).
watch(
  [rawMembers, autoMembers],
  () => {
    if (!autoMembers.value || rawMembers.value === null) return
    memberLimit.value = Math.min(MAX_MEMBERS, Math.max(MIN_MEMBERS, rawMembers.value))
  },
  { immediate: true },
)
const membersOk = computed(
  () => Number.isInteger(memberLimit.value) && memberLimit.value >= MIN_MEMBERS && memberLimit.value <= MAX_MEMBERS,
)
/** Gerçek taksit: hedef tutar ÷ kişi sayısı, 0,01 birime aşağı yuvarlanır (kontrata giden değer). */
const contribution = computed(() =>
  targetStroops.value === null || !membersOk.value ? null : contributionFor(targetStroops.value, memberLimit.value),
)
// Elle kişi sayısı seçilirse taksit alanı gerçek taksidi gösterir.
watch(contribution, (c) => {
  if (!autoMembers.value && c !== null) amount.value = toPlainAmount(c)
})
const pot = computed(() =>
  contribution.value === null || !membersOk.value ? null : contribution.value * BigInt(memberLimit.value),
)
/** Otomatik kip: istenen taksit gerçek taksitten farklıysa nedeni (sınır ya da yuvarlama). */
const installmentAdjusted = computed(() => {
  if (!autoMembers.value || contribution.value === null || requestedInstallment.value === null) return null
  if (contribution.value === requestedInstallment.value) return null
  const raw = rawMembers.value ?? 0
  const reason =
    raw > MAX_MEMBERS
      ? `Grup en fazla ${MAX_MEMBERS} kişi olabildiği için`
      : raw < MIN_MEMBERS
        ? `Grup en az ${MIN_MEMBERS} kişi olması gerektiği için`
        : 'Tutar kişi sayısına tam bölünmediği için'
  return { value: contribution.value, reason }
})
/** Taksit aralığına göre etiketler ve toplam vade metni. */
const intervalInfo = computed(() => {
  const d = duration.value
  if (d === 30 * DAY) return { unit: 'ay', adj: 'Aylık', term: (n: number) => (n >= 12 ? `${n} ay (${Number.isInteger(n / 12) ? n / 12 : (n / 12).toFixed(1)} yıl)` : `${n} ay`) }
  if (d === 7 * DAY) return { unit: 'hafta', adj: 'Haftalık', term: (n: number) => `${n} hafta` }
  if (d === DAY) return { unit: 'gün', adj: 'Günlük', term: (n: number) => `${n} gün` }
  return { unit: 'tur', adj: 'Tur başı', term: (n: number) => `${n} tur · her tur ${durationLabel(d)}` }
})
/** Hedef tutar kişi sayısına tam bölünmüyorsa taksit aşağı yuvarlanır; gerçek tur tutarı hedeften küçük olur. */
const roundedNote = computed(() =>
  pot.value !== null && targetStroops.value !== null && pot.value !== targetStroops.value ? pot.value : null,
)
/** Karşılaştırma tablosu; peşinat satırı kontratın gerçek yeteneğine göre yazılır. */
const compareRows = computed(() =>
  COMPARE_ROWS.map((r) =>
    r.topic === 'Peşinat'
      ? {
          ...r,
          ours: downOnChain.value
            ? 'Katılırken kontrata yatırılır (üye başına), sıran gelince alımına eklenip satıcıya gider; iptalde henüz almadıysan iade edilir.'
            : 'Havuza yatmaz: alıcı satıcıya kendi öder, havuz bedel − peşinatı karşılar. Kontrat doğrulamaz.',
        }
      : r,
  ),
)
/** Bir turun adımları; süreler seçilen takvime göre canlı hesaplanır. */
const roundTimeline = computed(() => {
  const draw = orderMode.value === 'Draw'
  return [
    {
      title: 'Taksit toplanır',
      text: `En fazla ${durationLabel(duration.value)}. Herkes ödeyince tur beklemeden sonraki adıma geçer.`,
    },
    {
      title: draw ? 'Kura çekilir' : 'Alıcı bellidir',
      text: draw
        ? 'Herkes taksidini yatırınca herkesin çağırabileceği bir işlemle, henüz almamış üyeler arasından kontrat seçer.'
        : 'Sıra havuz başlamadan önce üyelerce onaylanır; sıradaki üye tur başında bellidir.',
    },
    {
      title: 'Alım onayı',
      text: `Alıcı satıcıyı ve belgeyi önerir, doğrulayıcılar onaylar. ${draw ? 'Kura ve alım onayı birlikte' : 'Bu adım'} en fazla ${durationLabel(purchaseDuration.value)}.`,
    },
    {
      title: 'Satıcıya ödeme',
      text: 'Tutar satıcıya gider, yeni tur başlar. Son turdan sonra havuz tamamlanır.',
    },
  ]
})
/** "Kura ne zaman?" sorusunun kısa cevabı. */
const drawWhen = computed(() =>
  orderMode.value === 'Draw'
    ? `Her turda herkes taksidini yatırınca (en geç ${durationLabel(duration.value)} + ${durationLabel(graceDuration.value)} ek süre içinde)`
    : 'Kura yok, sıra baştan onaylanır',
)
const termText = computed(() => (membersOk.value ? intervalInfo.value.term(memberLimit.value) : '—'))
const demoSeller = computed(() => demoSellerInput.value.trim())
const demoSellerValid = computed(
  () =>
    StrKey.isValidEd25519PublicKey(demoSeller.value) &&
    demoSeller.value !== wallet.address,
)
const durationsOk = computed(
  () => duration.value > 0 && graceDuration.value > 0 && purchaseDuration.value > 0 && setupDuration.value > 0,
)

const stepValid = computed(() => {
  switch (step.value) {
    case 0:
      return (
        priceStroops.value !== null &&
        targetStroops.value !== null &&
        contribution.value !== null &&
        membersOk.value &&
        (!autoMembers.value || requestedInstallment.value !== null)
      )
    case 1:
      return durationsOk.value
    case 2:
      return demoSellerValid.value
    default:
      return contribution.value !== null && membersOk.value && durationsOk.value && demoSellerValid.value
  }
})

const stepHint = computed(() => {
  if (stepValid.value) return null
  switch (step.value) {
    case 0:
      if (priceStroops.value === null) return 'Devam etmek için geçerli bir toplam bedel gir.'
      if (targetStroops.value === null) return 'Peşinat toplam bedelden küçük olmalı.'
      if (autoMembers.value && requestedInstallment.value === null) return 'Devam etmek için geçerli bir taksit tutarı gir.'
      return contribution.value === null ? 'Tutar bu kişi sayısına bölünemiyor; taksidi ya da bedeli değiştir.' : `Üye sayısı ${MIN_MEMBERS} ile ${MAX_MEMBERS} arasında olmalı.`
    case 2:
      return 'Satıcı için geçerli ve kurucudan farklı bir Stellar adresi gerekli.'
    default:
      return null
  }
})

// İlk değerler: hedef tutar taksitten türetilir; hesaplayıcıdan bir amaç gelirse önerdiği taksit aralığı seçilir.
const initialGoal = GOALS.find((g) => g.id === queryGoal)
if (initialGoal && !queryAmount && !Number.isInteger(queryMembers)) {
  // Yalnızca amaç geldiyse örnek değerlerini (bedel, peşinat, taksit, sıra, aralık) eksiksiz uygula.
  pickGoal(initialGoal)
} else {
  if (queryAmount && Number.isInteger(queryMembers)) {
    // Hesaplayıcıdan gelen plan: peşinat yok, bedel = taksit × kişi.
    const c = parseOrNull(queryAmount)
    if (c !== null) price.value = toPlainAmount(c * BigInt(memberLimit.value))
  } else if (!initialGoal) {
    pickGoal(GOALS.find((g) => g.id === 'demo')!)
  }
  const initial = PRESETS.find((x) => x.id === initialGoal?.interval)
  if (initial) pickPreset(initial)
}

function go(to: number) {
  dir.value = to > step.value ? 'next' : 'prev'
  step.value = Math.max(0, Math.min(STEPS.length - 1, to))
}
function next() {
  if (step.value < STEPS.length - 1 && stepValid.value) go(step.value + 1)
}

function durationLabel(seconds: number): string {
  if (seconds % DAY === 0) return `${seconds / DAY} gün`
  if (seconds % 3600 === 0) return `${seconds / 3600} saat`
  return `${Math.round(seconds / 60)} dakika`
}

async function submit() {
  if (!wallet.address || contribution.value === null || !stepValid.value) return
  busy.value = true
  error.value = null
  txHash.value = null
  try {
    const result = await createPool(
      { address: wallet.address, signTransaction: wallet.signTransaction },
      {
        token: poolTokenContractId,
        contributionAmount: contribution.value,
        memberLimit: memberLimit.value,
        orderMode: orderMode.value,
        downPayment: downOnChain.value ? (downStroops.value ?? 0n) : undefined,
        roundDuration: duration.value,
        graceDuration: graceDuration.value,
        purchaseDuration: purchaseDuration.value,
        setupDeadline: Math.floor(Date.now() / 1000) + setupDuration.value,
        demoSeller: demoSeller.value,
      },
    )
    txHash.value = result.hash
    if (result.poolId !== null) router.push(`/pool/${result.poolId}`)
  } catch (e) {
    if (!isUserRejection(e)) error.value = errorMessage(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="mx-auto max-w-5xl space-y-6">
    <div class="flex flex-wrap items-end justify-between gap-3">
      <div>
        <RouterLink to="/" class="inline-flex items-center gap-1 text-sm font-medium text-brand-700 hover:underline">
          <AppIcon name="back" class="!size-4" /> Ana sayfa
        </RouterLink>
        <h1 class="mt-1 text-4xl font-extrabold sm:text-5xl">Yeni havuz kur</h1>
        <p class="mt-1 text-stone-600">Dört kısa adım. İstediğin zaman geri dönüp değiştirebilirsin.</p>
      </div>
    </div>

    <p
      v-if="!poolContractId"
      role="status"
      class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950"
    >
      <Illo name="bulb" :size="28" />
      <span>
        Havuz sözleşmesi henüz yapılandırılmadı. Formu gezebilirsin; sözleşme yayınlanıp adresi
        <code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> olarak eklenince havuz oluşturulabilir.
      </span>
    </p>

    <p
      v-if="legacyContract"
      role="alert"
      class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950"
    >
      <Illo name="warning" :size="28" />
      <span>
        Yapılandırılan kontrat eski sponsorlu sürüm. Arayüz sponsorsuz modele göre yazıldığı için havuz
        oluşturma şimdilik kapalı; kontrat yeniden yayınlanınca açılacak.
      </span>
    </p>

    <div class="grid gap-5 lg:grid-cols-[1.5fr_1fr]">
      <!-- SİHİRBAZ -->
      <form class="card space-y-6 !p-5 sm:!p-7" @submit.prevent="step === STEPS.length - 1 ? submit() : next()">
        <StepIndicator :steps="STEPS" :current="step" @goto="go" />

        <div class="overflow-hidden">
          <Transition :name="`step-${dir}`" mode="out-in">
            <div :key="step" class="min-h-[22rem] space-y-6">
              <!-- 1 · PLAN -->
              <template v-if="step === 0">
                <div>
                  <h2 class="text-2xl font-extrabold">Ne için, ne kadara biriktiriyorsunuz?</h2>
                  <p class="mt-1 text-sm text-stone-600">
                    Bir başlangıç seç; toplam bedeli, peşinatı ve ödeyebileceğin taksidi gir. Kişi sayısını ve vadeyi
                    biz hesaplarız. Seçim yalnızca örnek değerleri doldurur; gerçek ev veya araç teslimi yoktur.
                  </p>
                </div>
                <div class="flex flex-wrap gap-2.5" role="group" aria-label="Amaç">
                  <button
                    v-for="g in GOALS"
                    :key="g.id"
                    type="button"
                    class="inline-flex min-h-12 cursor-pointer items-center gap-2.5 rounded-full border-2 py-1.5 pr-4 pl-1.5 text-left transition-[background-color,border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 active:scale-95"
                    :class="
                      goal === g.id
                        ? 'border-brand-600 bg-brand-50 shadow-[0_10px_22px_-12px_rgb(20_128_90/0.55)]'
                        : 'border-stone-200 bg-white hover:border-brand-300'
                    "
                    :aria-pressed="goal === g.id"
                    @click="pickGoal(g)"
                  >
                    <span class="grid size-10 place-items-center rounded-full bg-white/80"><Illo :name="g.icon" :size="28" /></span>
                    <span class="leading-tight">
                      <span class="block font-display text-sm font-bold">{{ g.label }}</span>
                      <span class="block text-xs text-stone-600">{{ goalMembers(g) }} kişi · {{ g.mode === 'Draw' && !drawUnavailable ? 'kura' : 'sıralı' }}</span>
                    </span>
                  </button>
                </div>

                <div class="grid gap-5 sm:grid-cols-2">
                  <div>
                    <label class="label" for="price">Toplam bedel ({{ token }})</label>
                    <input
                      id="price"
                      v-model="price"
                      class="input text-xl font-bold"
                      type="text"
                      inputmode="decimal"
                      autocomplete="off"
                      required
                      @input="onPriceInput"
                    />
                    <p v-if="priceStroops === null" class="mt-1 text-xs text-rose-700">Geçerli bir tutar gir (en fazla 7 ondalık).</p>
                    <p v-else class="mt-1 text-xs text-stone-600">Almak istediğin ev veya aracın değeri.</p>
                  </div>
                  <div>
                    <label class="label" for="down-payment">
                      Peşinat ({{ token }}) <span class="font-normal text-stone-600">· %{{ downPctShown }}</span>
                    </label>
                    <input
                      id="down-payment"
                      v-model="downPayment"
                      class="input text-xl font-bold"
                      type="text"
                      inputmode="decimal"
                      autocomplete="off"
                      @input="onDownInput"
                    />
                    <div class="mt-2 flex flex-wrap gap-2" role="group" aria-label="Peşinat oranı">
                      <button
                        v-for="o in DOWN_OPTIONS"
                        :key="o"
                        type="button"
                        class="min-h-11 cursor-pointer rounded-full border-2 px-3.5 text-sm font-semibold transition-colors duration-200"
                        :class="downMode === 'pct' && downPctChosen === o ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
                        :aria-pressed="downMode === 'pct' && downPctChosen === o"
                        @click="setDownPct(o)"
                      >
                        {{ o === 0 ? 'Peşinatsız' : `%${o}` }}
                      </button>
                    </div>
                    <p v-if="downStroops === null || (priceStroops !== null && downStroops >= priceStroops)" class="mt-1 text-xs text-rose-700">
                      Peşinat, toplam bedelden küçük geçerli bir tutar olmalı.
                    </p>
                    <p v-else-if="downOnChain" class="mt-1 text-xs leading-relaxed text-stone-600">
                      Katılırken kontrata yatırılır (herkes aynı tutarı yatırır). Sıran gelince alımına eklenip satıcıya
                      gider; havuz iptal olursa henüz almadıysan iade edilir.
                    </p>
                    <p v-else class="mt-1 text-xs leading-relaxed text-stone-600">
                      Havuza yatmaz: sıran gelince satıcıya kendin ödersin, havuz yalnızca kalan kısmı karşılar.
                      Kontrat peşinatı doğrulamaz.
                    </p>
                  </div>
                </div>

                <div>
                  <label class="label" for="amount">Ne kadar {{ intervalInfo.adj.toLowerCase() }} taksit ödeyebilirsin? ({{ token }})</label>
                  <input
                    id="amount"
                    v-model="amount"
                    class="input text-xl font-bold sm:max-w-xs"
                    type="text"
                    inputmode="decimal"
                    autocomplete="off"
                    :readonly="!autoMembers"
                    required
                  />
                  <p v-if="autoMembers && requestedInstallment === null" class="mt-1 text-xs text-rose-700">
                    Geçerli bir tutar gir (en fazla 7 ondalık).
                  </p>
                  <p v-else-if="installmentAdjusted" class="mt-1 text-xs text-amber-800">
                    {{ installmentAdjusted.reason }} taksit {{ formatStroops(installmentAdjusted.value) }} {{ token }} olarak ayarlandı.
                  </p>
                  <p v-else class="mt-1 text-xs text-stone-600">
                    {{ autoMembers ? 'Kişi sayısı ve vade buna göre otomatik belirlenir.' : 'Kişi sayısını elle belirlediğin için taksit, hedef ÷ kişi olarak hesaplanır.' }}
                  </p>
                </div>

                <dl class="grid gap-3 sm:grid-cols-3" aria-label="Hesaplanan plan">
                  <div class="rounded-2xl bg-brand-50/70 p-3.5">
                    <dt class="text-xs text-stone-600">Havuzdan karşılanan</dt>
                    <dd class="font-display text-lg font-extrabold tabular-nums">{{ targetStroops !== null ? formatStroops(targetStroops) : '—' }}</dd>
                    <dd class="text-xs text-stone-600">toplam bedel − peşinat</dd>
                  </div>
                  <div class="rounded-2xl bg-brand-50/70 p-3.5">
                    <dt class="text-xs text-stone-600">Grup büyüklüğü</dt>
                    <dd class="font-display text-lg font-extrabold tabular-nums">{{ membersOk ? `${memberLimit} kişi` : '—' }}</dd>
                    <dd class="text-xs text-stone-600">{{ autoMembers ? 'otomatik belirlendi' : 'elle belirlendi' }}</dd>
                  </div>
                  <div class="rounded-2xl bg-brand-50/70 p-3.5">
                    <dt class="text-xs text-stone-600">Vade</dt>
                    <dd class="font-display text-lg font-extrabold">{{ termText }}</dd>
                    <dd class="text-xs text-stone-600">{{ membersOk ? memberLimit : '—' }} taksit</dd>
                  </div>
                </dl>
                <p v-if="roundedNote !== null" class="text-xs text-amber-800">
                  Kişi sayısına tam bölünmediği için gerçek tur tutarı {{ formatStroops(roundedNote) }} {{ token }} olur.
                </p>

                <div>
                  <button type="button" class="min-h-11 text-sm font-semibold text-brand-700 underline" @click="autoMembers = !autoMembers">
                    {{ autoMembers ? 'Kişi sayısını elle belirle' : 'Kişi sayısını otomatik belirle' }}
                  </button>
                  <div v-if="!autoMembers" class="mt-1">
                    <p id="members-label" class="label">Kaç kişi? (kişi sayısı = taksit sayısı)</p>
                    <div class="flex items-center gap-3" role="group" aria-labelledby="members-label">
                      <button
                        type="button"
                        class="btn-secondary !size-11 !p-0 text-xl"
                        :disabled="memberLimit <= MIN_MEMBERS"
                        aria-label="Bir kişi azalt"
                        @click="stepMembers(-1)"
                      >
                        −
                      </button>
                      <input
                        v-model.number="memberLimit"
                        class="input !w-20 text-center text-xl font-bold"
                        type="number"
                        :min="MIN_MEMBERS"
                        :max="MAX_MEMBERS"
                        step="1"
                        aria-label="Üye sayısı"
                        required
                      />
                      <button
                        type="button"
                        class="btn-secondary !size-11 !p-0 text-xl"
                        :disabled="memberLimit >= MAX_MEMBERS"
                        aria-label="Bir kişi artır"
                        @click="stepMembers(1)"
                      >
                        +
                      </button>
                    </div>
                    <p class="mt-1 text-xs text-stone-600">Kişi sayısı kadar tur olur; herkes bir kez alır.</p>
                  </div>
                </div>

                <div>
                  <p id="interval-label" class="label">Taksit aralığı</p>
                  <div class="flex flex-wrap gap-2" role="group" aria-labelledby="interval-label">
                    <button
                      v-for="p in PRESETS"
                      :key="p.id"
                      type="button"
                      class="inline-flex min-h-11 cursor-pointer items-center gap-2 rounded-full border-2 px-4 text-sm font-semibold transition-colors duration-200"
                      :class="!custom && preset === p.id ? 'border-brand-600 bg-brand-50 text-brand-900' : 'border-stone-200 bg-white hover:border-brand-300'"
                      :aria-pressed="!custom && preset === p.id"
                      @click="pickPreset(p)"
                    >
                      <Illo :name="p.icon" :size="20" /> {{ p.label }}
                    </button>
                  </div>
                  <p class="mt-1 text-xs text-stone-600">
                    Süre ayrıntılarını (ek süre, alım onayı) bir sonraki adımda değiştirebilirsin.
                  </p>
                </div>

                <dl class="divide-y divide-stone-100 rounded-2xl border border-stone-200 bg-sand/40 text-sm" aria-label="Plan özeti">
                  <div class="flex items-center justify-between gap-3 p-3.5">
                    <dt class="text-stone-600">Toplam bedel</dt>
                    <dd class="text-right font-semibold tabular-nums">
                      {{ priceStroops !== null ? formatStroops(priceStroops) : '—' }} {{ token }}
                      <span class="block text-xs font-normal text-stone-600">
                        peşinat {{ downStroops !== null ? formatStroops(downStroops) : '—' }} + havuz {{ targetStroops !== null ? formatStroops(targetStroops) : '—' }}
                      </span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5">
                    <dt class="text-stone-600">Havuza ödeyeceğin</dt>
                    <dd class="text-right font-semibold tabular-nums">
                      {{ pot !== null ? formatStroops(pot) : '—' }} {{ token }}
                      <span class="block text-xs font-normal text-stone-600">
                        {{ membersOk ? memberLimit : '—' }} taksit × {{ contribution !== null ? formatStroops(contribution) : '—' }}, ek ücret yok
                      </span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5">
                    <dt class="text-stone-600">İlk ödeme</dt>
                    <dd class="text-right font-semibold tabular-nums">
                      {{ contribution !== null ? formatStroops(contribution) : '—' }} {{ token }}
                      <span class="block text-xs font-normal text-stone-600">
                        havuz başlayınca 1. taksit<template v-if="downOnChain && downStroops !== null && downStroops > 0n">;
                        katılırken ayrıca {{ formatStroops(downStroops) }} peşinat</template>
                      </span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5">
                    <dt class="text-stone-600">Peşinat</dt>
                    <dd class="text-right font-semibold tabular-nums">
                      {{ downStroops !== null && downStroops > 0n ? `${formatStroops(downStroops)} ${token}` : 'Yok' }}
                      <span class="block text-xs font-normal text-stone-600">
                        {{
                          downStroops === null || downStroops <= 0n
                            ? 'peşinatsız plan'
                            : downOnChain
                              ? 'katılırken kontrata yatırılır, sıran gelince alımına eklenir'
                              : 'havuza yatmaz, sıran gelince satıcıya sen ödersin'
                        }}
                      </span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5">
                    <dt class="text-stone-600">Organizasyon / kayıt ücreti</dt>
                    <dd class="text-right font-semibold">
                      Yok
                      <span class="block text-xs font-normal text-stone-600">kimseye ücret ayrılmaz</span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5">
                    <dt class="text-stone-600">Kura ne zaman?</dt>
                    <dd class="text-right font-semibold">
                      {{ orderMode === 'Draw' ? 'Her tur, herkes ödeyince' : 'Yok, sabit sıra' }}
                      <span class="block text-xs font-normal text-stone-600">{{ drawWhen }}</span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5">
                    <dt class="text-stone-600">Sıran gelince</dt>
                    <dd class="text-right font-semibold tabular-nums">
                      <template v-if="downOnChain && pot !== null && downStroops !== null">
                        {{ formatStroops(pot + downStroops) }} {{ token }} satıcıya
                        <span class="block text-xs font-normal text-stone-600">
                          havuz {{ formatStroops(pot) }} + kendi peşinatın {{ formatStroops(downStroops) }}, kontrat öder
                        </span>
                      </template>
                      <template v-else>
                        {{ pot !== null ? formatStroops(pot) : '—' }} {{ token }} havuzdan satıcıya
                        <span class="block text-xs font-normal text-stone-600">
                          {{ downStroops !== null && downStroops > 0n ? `+ ${formatStroops(downStroops)} peşinatı satıcıya sen ödersin` : 'sana değil, satıcıya ödenir' }}
                        </span>
                      </template>
                    </dd>
                  </div>
                </dl>

                <div class="rounded-2xl border border-stone-200 p-4">
                  <h3 class="font-display font-bold">Bir tur nasıl geçer?</h3>
                  <ol class="mt-3 space-y-3">
                    <li v-for="(t, i) in roundTimeline" :key="t.title" class="flex items-start gap-3 text-sm">
                      <span class="grid size-7 shrink-0 place-items-center rounded-full bg-brand-600 text-xs font-bold text-white">{{ i + 1 }}</span>
                      <span><strong>{{ t.title }}.</strong> {{ t.text }}</span>
                    </li>
                  </ol>
                  <p class="mt-3 text-xs leading-relaxed text-stone-600">
                    Bir üye taksidini yatırmazsa {{ durationLabel(graceDuration) }} ek süre başlar; o da biterse tur durur,
                    yalnızca o turun katkıları iade edilir. Her turun sonunda yeni bir tur açılır, kişi sayısı kadar tur olur.
                  </p>
                </div>

                <details class="group rounded-2xl border border-stone-200 p-4">
                  <summary class="flex min-h-11 cursor-pointer list-none items-center justify-between gap-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
                    Fuzul Ev / Oto gibi şirketlerden farkı ne?
                    <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
                  </summary>
                  <div class="mt-3 overflow-x-auto">
                    <table class="w-full min-w-[30rem] text-left text-sm">
                      <caption class="sr-only">Tasarruf finansman şirketleri ile Stellerpool karşılaştırması</caption>
                      <thead>
                        <tr class="text-xs text-stone-600">
                          <th scope="col" class="py-2 pr-3 font-semibold"></th>
                          <th scope="col" class="py-2 pr-3 font-semibold">Şirketlerde tipik</th>
                          <th scope="col" class="py-2 font-semibold">Stellerpool</th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-stone-100 align-top">
                        <tr v-for="row in compareRows" :key="row.topic">
                          <th scope="row" class="py-2.5 pr-3 font-semibold">{{ row.topic }}</th>
                          <td class="py-2.5 pr-3 text-stone-700">{{ row.company }}</td>
                          <td class="py-2.5 text-stone-900">{{ row.ours }}</td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                  <p class="mt-3 text-xs leading-relaxed text-stone-600">
                    Şirketlerde peşinat vadeyi kısaltır. Burada da aynı etki var: peşinat yükseldikçe havuzun karşılayacağı
                    tutar küçülür, verdiğin taksitle <strong>daha az kişi ve daha kısa vade</strong> çıkar.
                    {{ downOnChain
                      ? 'Fark: peşinat şirkete değil kontrata yatar ve yalnızca kendi alımına gider; birikime sayılmaz.'
                      : 'Fark: peşinat havuza yatmaz ve kontrat onu doğrulamaz; alım belgesinde (doğrulayıcıların onayladığı) yazılması gerekir.' }}
                    Rakamlar şirketlerin kamuya açık sayfalarından özetlenmiştir ve sözleşmeye göre değişir; bu bir tavsiye değildir.
                  </p>
                </details>

                <p v-if="memberLimit > 12" role="note" class="flex items-start gap-2 rounded-2xl bg-gold-100/80 p-3 text-sm text-amber-950">
                  <Illo name="warning" :size="22" />
                  <span>
                    Büyük grupta bir turda satıcıya giden tutar da büyür. Erken teslim alan sonraki katkıyı bırakırsa
                    bekleyenlerin önceki tur ödemeleri geri alınamaz; grup büyüdükçe bu açık küçülmez.
                  </span>
                </p>

                <div>
                  <p id="mode-label" class="label">Sıra kimde? Alıcı nasıl belirlensin?</p>
                  <div class="grid gap-3 sm:grid-cols-2" role="radiogroup" aria-labelledby="mode-label">
                    <button
                      type="button"
                      role="radio"
                      class="choice"
                      :class="orderMode === 'Fixed' ? '!border-brand-600 bg-brand-50' : ''"
                      :aria-checked="orderMode === 'Fixed'"
                      @click="orderMode = 'Fixed'"
                    >
                      <span class="grid size-12 shrink-0 place-items-center rounded-xl bg-brand-50"><Illo name="memo" :size="34" /></span>
                      <span>
                        <span class="block font-display font-bold">Sabit sıra</span>
                        <span class="block text-xs text-stone-600">Üyeler sırayı birlikte onaylar (altın günü gibi).</span>
                      </span>
                    </button>
                    <button
                      type="button"
                      role="radio"
                      class="choice disabled:cursor-not-allowed disabled:opacity-55"
                      :class="orderMode === 'Draw' ? '!border-brand-600 bg-brand-50' : ''"
                      :aria-checked="orderMode === 'Draw'"
                      :disabled="drawUnavailable"
                      @click="orderMode = 'Draw'"
                    >
                      <span class="grid size-12 shrink-0 place-items-center rounded-xl bg-brand-50"><Illo name="dice" :size="34" /></span>
                      <span>
                        <span class="block font-display font-bold">Kura</span>
                        <span class="block text-xs text-stone-600">
                          {{ drawUnavailable ? 'Yayındaki kontrat henüz kura desteklemiyor.' : 'Her tur, henüz almamış üyeler arasından çekilir.' }}
                        </span>
                      </span>
                    </button>
                  </div>
                  <p v-if="orderMode === 'Draw'" class="mt-2 text-xs leading-relaxed text-stone-600">
                    Kura, tüm üyeler kendi katkısını yatırdıktan sonra çekilir; kazanan zaten payını ödemiş olur.
                    Zincir üstü rastgelelik hackathon düzeyindedir, yüksek tutarlı gerçek kullanım için yetmez.
                  </p>
                </div>
              </template>

              <!-- 2 · SÜRELER -->
              <template v-else-if="step === 1">
                <div>
                  <h2 class="text-2xl font-extrabold">Ne kadar süre tanınsın?</h2>
                  <p class="mt-1 text-sm text-stone-600">Hazır bir takvim seç. Ayrıntıyı istersen değiştirebilirsin.</p>
                </div>
                <div class="grid gap-3 sm:grid-cols-2" role="group" aria-label="Takvim">
                  <button
                    v-for="p in PRESETS"
                    :key="p.id"
                    type="button"
                    class="choice"
                    :class="!custom && preset === p.id ? '!border-brand-600 bg-brand-50' : ''"
                    :aria-pressed="!custom && preset === p.id"
                    @click="pickPreset(p)"
                  >
                    <span class="grid size-12 shrink-0 place-items-center rounded-xl bg-brand-50"><Illo :name="p.icon" :size="34" /></span>
                    <span>
                      <span class="block font-display font-bold">{{ p.label }}</span>
                      <span class="block text-xs text-stone-600">{{ p.hint }}</span>
                    </span>
                  </button>
                </div>

                <button type="button" class="text-sm font-semibold text-brand-700 underline" @click="custom = !custom">
                  {{ custom ? 'Ayrıntıyı gizle' : 'Süreleri tek tek ayarla' }}
                </button>

                <div v-if="custom" class="grid gap-4 sm:grid-cols-2">
                  <div>
                    <label class="label" for="duration">Katkı süresi</label>
                    <select id="duration" v-model.number="duration" class="input">
                      <option v-for="d in ROUND_OPTIONS" :key="d.seconds" :value="d.seconds">{{ d.label }}</option>
                    </select>
                    <p class="mt-1 text-xs text-stone-600">Herkesin bu tur payını yatırması için süre.</p>
                  </div>
                  <div>
                    <label class="label" for="grace-duration">Ek süre</label>
                    <select id="grace-duration" v-model.number="graceDuration" class="input">
                      <option :value="600">10 dakika (demo)</option>
                      <option :value="86400">1 gün</option>
                      <option :value="604800">7 gün</option>
                    </select>
                    <p class="mt-1 text-xs text-stone-600">Geciken üyeye tanınan son şans.</p>
                  </div>
                  <div>
                    <label class="label" for="purchase-duration">Alım onayı süresi</label>
                    <select id="purchase-duration" v-model.number="purchaseDuration" class="input">
                      <option :value="1800">30 dakika (demo)</option>
                      <option :value="86400">1 gün</option>
                      <option :value="604800">7 gün</option>
                    </select>
                    <p class="mt-1 text-xs text-stone-600">Satıcı ve doğrulayıcı onayı için süre.</p>
                  </div>
                  <div>
                    <label class="label" for="setup-duration">Kuruluş süresi</label>
                    <select id="setup-duration" v-model.number="setupDuration" class="input">
                      <option :value="3600">1 saat (demo)</option>
                      <option :value="86400">1 gün</option>
                      <option :value="604800">7 gün</option>
                    </select>
                    <p class="mt-1 text-xs text-stone-600">Üyeler katılıp onaylamazsa havuz iptal edilebilir.</p>
                  </div>
                </div>

                <dl v-else class="grid gap-2 rounded-2xl bg-sand/60 p-4 text-sm sm:grid-cols-2">
                  <div class="flex justify-between gap-3"><dt class="text-stone-600">Katkı süresi</dt><dd class="font-semibold">{{ durationLabel(duration) }}</dd></div>
                  <div class="flex justify-between gap-3"><dt class="text-stone-600">Ek süre</dt><dd class="font-semibold">{{ durationLabel(graceDuration) }}</dd></div>
                  <div class="flex justify-between gap-3"><dt class="text-stone-600">Alım onayı</dt><dd class="font-semibold">{{ durationLabel(purchaseDuration) }}</dd></div>
                  <div class="flex justify-between gap-3"><dt class="text-stone-600">Kuruluş</dt><dd class="font-semibold">{{ durationLabel(setupDuration) }}</dd></div>
                </dl>
              </template>

              <!-- 3 · KİŞİLER -->
              <template v-else-if="step === 2">
                <div>
                  <h2 class="text-2xl font-extrabold">Kimler yer alacak?</h2>
                  <p class="mt-1 text-sm text-stone-600">Bu Testnet örneğinde ödemenin gideceği satıcı adresini seç.</p>
                </div>

                <div class="space-y-2 rounded-2xl border-2 border-stone-200 p-4">
                  <label class="flex items-center gap-2 font-display font-bold" for="demo-seller">
                    <Illo name="store" :size="24" /> İzinli demo satıcısı
                  </label>
                  <p class="text-sm text-stone-600">
                    Tur tutarı yalnızca bu adrese gönderilebilir. Testnet’te ikinci bir cüzdan adresi
                    kullanabilirsin. Kurucu adresiyle aynı olamaz.
                  </p>
                  <input
                    id="demo-seller"
                    v-model="demoSellerInput"
                    class="input font-mono"
                    type="text"
                    placeholder="G… demo satıcısı"
                    autocomplete="off"
                    required
                  />
                  <p v-if="demoSeller && !demoSellerValid" class="text-xs text-rose-700">
                    Satıcı geçerli bir adres olmalı; kurucu adresiyle aynı olamaz.
                  </p>
                  <p class="text-xs text-stone-500">
                    {{ orderMode === 'Draw' ? 'Doğrulayıcılar' : 'Sıra ve doğrulayıcılar' }}, üyeler katıldıktan sonra ayrı bir adımda önerilip onaylanır.
                  </p>
                </div>
              </template>

              <!-- 4 · ONAY -->
              <template v-else>
                <div>
                  <h2 class="text-2xl font-extrabold">Her şey doğru mu?</h2>
                  <p class="mt-1 text-sm text-stone-600">Göz at, bir şey yanlışsa ilgili adıma geri dön.</p>
                </div>

                <dl class="divide-y divide-stone-100 rounded-2xl border border-stone-200">
                  <div class="flex items-center justify-between gap-3 p-3.5 text-sm">
                    <dt class="text-stone-600">Toplam bedel</dt>
                    <dd class="text-right font-semibold">
                      {{ priceStroops !== null ? formatStroops(priceStroops) : '—' }} {{ token }}
                      <button type="button" class="ml-2 font-medium text-brand-700 underline" @click="go(0)">değiştir</button>
                      <span class="block text-xs font-normal text-stone-600">
                        peşinat {{ downStroops !== null ? formatStroops(downStroops) : '—' }} ({{ downOnChain ? 'katılırken kontrata yatırılır' : 'havuz dışı, alıcı öder' }}) · havuz {{ pot !== null ? formatStroops(pot) : '—' }} · {{ memberLimit }} kişi
                      </span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5 text-sm">
                    <dt class="text-stone-600">Taksit</dt>
                    <dd class="text-right font-semibold">
                      {{ intervalInfo.adj.toLowerCase() }} {{ contribution !== null ? formatStroops(contribution) : '—' }} {{ token }} · vade {{ termText }}
                      <span class="block text-xs font-normal text-stone-600">ilk taksit havuz başlayınca, organizasyon/kayıt ücreti yok</span>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5 text-sm">
                    <dt class="text-stone-600">Alıcı</dt>
                    <dd class="text-right font-semibold">
                      {{ orderMode === 'Draw' ? 'Kura (tüm katkılar gelince)' : 'Sabit sıra (üyeler onaylar)' }}
                      <button type="button" class="ml-2 font-medium text-brand-700 underline" @click="go(0)">değiştir</button>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5 text-sm">
                    <dt class="text-stone-600">Süreler</dt>
                    <dd class="text-right font-semibold">
                      katkı {{ durationLabel(duration) }} · ek {{ durationLabel(graceDuration) }}
                      <button type="button" class="ml-2 font-medium text-brand-700 underline" @click="go(1)">değiştir</button>
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3 p-3.5 text-sm">
                    <dt class="text-stone-600">Satıcı</dt>
                    <dd class="text-right font-mono font-semibold break-all">
                      {{ demoSeller ? `${demoSeller.slice(0, 6)}…${demoSeller.slice(-6)}` : '—' }}
                    </dd>
                  </div>
                </dl>

                <p class="flex items-start gap-2 rounded-2xl bg-sand/70 p-3.5 text-sm text-stone-700">
                  <Illo name="bulb" :size="24" />
                  Havuz oluşunca üyeler katılır.
                  {{ orderMode === 'Draw' ? 'Doğrulayıcılar' : 'Sıra ve doğrulayıcılar' }} herkesin onayıyla belirlenir.
                  {{ downOnChain
                    ? 'Peşinat katılırken kontrata yatırılır; sıran gelince alımına eklenir, havuz iptal olursa henüz almadıysan iade edilir.'
                    : 'Peşinat havuza yatmaz; sıran gelince satıcıya kendin ödersin ve alım belgesinde yazılmalıdır.' }}
                  Katkı eksikse tur durur; geçmişte tamamlanmış turların ödemesi geri alınamaz.
                  Bu bir Testnet simülasyonudur, gerçek para veya ev/araç teslimi yoktur.
                </p>
              </template>
            </div>
          </Transition>
        </div>

        <p v-if="stepHint" role="status" class="text-sm text-amber-800">{{ stepHint }}</p>

        <p v-if="error" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ error }}</p>
        <p v-if="txHash" class="pop rounded-2xl bg-sage-50 p-3 text-sm text-sage-800">
          İşlem gönderildi:
          <a :href="explorerTx(txHash)" target="_blank" rel="noopener noreferrer" class="font-mono underline">
            {{ txHash.slice(0, 8) }}…
          </a>
          Havuz numarası okunamadıysa ana sayfadan aç.
        </p>

        <div class="flex items-center justify-between gap-3 border-t border-stone-100 pt-5">
          <button type="button" class="btn-secondary" :class="step === 0 ? 'invisible' : ''" @click="go(step - 1)">
            <AppIcon name="back" class="!size-4" /> Geri
          </button>

          <button v-if="step < STEPS.length - 1" type="submit" class="btn-primary btn-lg" :disabled="!stepValid">
            Devam <AppIcon name="arrow" class="!size-4" />
          </button>
          <template v-else>
            <button
              v-if="!wallet.isConnected"
              type="button"
              class="btn-primary btn-lg"
              :disabled="wallet.busy"
              @click="wallet.connect()"
            >
              Önce cüzdan bağla
            </button>
            <button v-else type="submit" class="btn-primary btn-lg" :disabled="busy || !stepValid || !poolContractId || legacyContract">
              <CoinSpinner v-if="busy" :size="22" />
              {{ busy ? 'Cüzdanı onayla…' : 'Havuzu oluştur' }}
            </button>
          </template>
        </div>
      </form>

      <!-- CANLI ÖNİZLEME -->
      <aside class="order-last lg:order-none" aria-label="Havuz önizlemesi">
        <div class="card sunrise relative overflow-hidden !p-0 lg:sticky lg:top-24">
          <div class="relative h-48 sm:h-64 lg:h-72">
            <Scene3D
              :coins="membersOk ? memberLimit : 4"
              :label="`${memberLimit} üyeli havuzu temsil eden ${memberLimit} altın para`"
            />
          </div>
          <dl class="grid grid-cols-2 gap-2 p-4 text-center sm:grid-cols-3 lg:grid-cols-1 lg:text-left">
            <div class="rounded-2xl bg-white/80 p-3">
              <dt class="text-xs text-stone-600">Havuz hedefi</dt>
              <dd class="font-display text-lg font-extrabold tabular-nums">
                {{ pot !== null ? formatStroops(pot) : '—' }}
              </dd>
            </div>
            <div class="rounded-2xl bg-white/80 p-3">
              <dt class="text-xs text-stone-600">{{ downOnChain ? 'Peşinat (kontratta)' : 'Peşinat (havuz dışı)' }}</dt>
              <dd class="font-display text-lg font-extrabold tabular-nums">
                {{ downStroops !== null ? formatStroops(downStroops) : '—' }}
              </dd>
            </div>
            <div class="rounded-2xl bg-white/80 p-3">
              <dt class="text-xs text-stone-600">{{ intervalInfo.adj }} taksit</dt>
              <dd class="font-display text-lg font-extrabold tabular-nums">
                {{ contribution !== null ? formatStroops(contribution) : '—' }}
              </dd>
            </div>
            <div class="rounded-2xl bg-white/80 p-3">
              <dt class="text-xs text-stone-600">Alıcı</dt>
              <dd class="text-sm font-semibold">{{ orderMode === 'Draw' ? 'Kura ile' : 'Sabit sıra' }}</dd>
            </div>
            <div class="rounded-2xl bg-white/80 p-3">
              <dt class="text-xs text-stone-600">Risk sınırı</dt>
              <dd class="text-sm font-semibold text-amber-900">Geçmiş turlar iade edilmez</dd>
            </div>
            <div class="rounded-2xl bg-white/80 p-3">
              <dt class="text-xs text-stone-600">Vade · grup</dt>
              <dd class="text-sm font-semibold">{{ termText }} · {{ membersOk ? memberLimit : '—' }} kişi</dd>
            </div>
          </dl>
        </div>
      </aside>
    </div>
  </div>
</template>
