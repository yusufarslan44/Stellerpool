<script setup lang="ts">
import { StrKey } from '@stellar/stellar-sdk'
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import CoinSpinner from '@/components/CoinSpinner.vue'
import Illo from '@/components/Illo.vue'
import Scene3D from '@/components/Scene3D.vue'
import StepIndicator from '@/components/StepIndicator.vue'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatStroops, parseAmount, toPlainAmount } from '@/lib/format'
import { contributionFor, goalMembers, GOALS } from '@/lib/goals'
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
const amount = ref(queryAmount || '10')
const memberLimit = ref(
  Number.isInteger(queryMembers) && queryMembers >= MIN_MEMBERS && queryMembers <= MAX_MEMBERS ? queryMembers : 4,
)
const orderMode = ref<OrderMode>(queryMode)

// Kontratın gerçek yetenekleri zincirden okunur; kura yoksa seçenek kapatılır.
const caps = ref<ContractCapabilities | null>(null)
const drawUnavailable = computed(() => caps.value !== null && !caps.value.supportsDraw)
const legacyContract = computed(() => caps.value?.legacySponsor === true)
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
  memberLimit.value = goalMembers(g)
  orderMode.value = g.mode === 'Draw' && drawUnavailable.value ? 'Fixed' : g.mode
  try {
    const c = contributionFor(parseAmount(g.pot), goalMembers(g))
    if (c !== null) amount.value = toPlainAmount(c)
  } catch {
    /* örnek tutar geçerli; yine de form bozulmasın */
  }
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

const contribution = computed(() => {
  try {
    const v = parseAmount(amount.value)
    return v > 0n ? v : null
  } catch {
    return null
  }
})
const membersOk = computed(
  () => Number.isInteger(memberLimit.value) && memberLimit.value >= MIN_MEMBERS && memberLimit.value <= MAX_MEMBERS,
)
const pot = computed(() =>
  contribution.value === null || !membersOk.value ? null : contribution.value * BigInt(memberLimit.value),
)
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
      return contribution.value !== null && membersOk.value
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
      return contribution.value === null ? 'Devam etmek için geçerli bir tutar gir.' : `Üye sayısı ${MIN_MEMBERS} ile ${MAX_MEMBERS} arasında olmalı.`
    case 2:
      return 'Satıcı için geçerli ve kurucudan farklı bir Stellar adresi gerekli.'
    default:
      return null
  }
})

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
                  <h2 class="text-2xl font-extrabold">Ne için biriktiriyorsunuz?</h2>
                  <p class="mt-1 text-sm text-stone-600">
                    Bir başlangıç seç, tutarı sonra değiştirebilirsin. Seçim yalnızca örnek değerleri doldurur;
                    gerçek ev veya araç teslimi yoktur.
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
                    <label class="label" for="amount">Her turda herkes ne kadar yatırsın? ({{ token }})</label>
                    <input
                      id="amount"
                      v-model="amount"
                      class="input text-xl font-bold"
                      type="text"
                      inputmode="decimal"
                      autocomplete="off"
                      required
                    />
                    <p v-if="contribution === null" class="mt-1 text-xs text-rose-700">
                      Geçerli bir tutar gir (en fazla 7 ondalık).
                    </p>
                  </div>
                  <div>
                    <p id="members-label" class="label">Kaç kişi?</p>
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
                    <dt class="text-stone-600">Plan</dt>
                    <dd class="text-right font-semibold">
                      {{ memberLimit }} kişi · her tur {{ contribution !== null ? formatStroops(contribution) : '—' }} {{ token }}
                      <button type="button" class="ml-2 font-medium text-brand-700 underline" @click="go(0)">değiştir</button>
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
          <dl class="grid grid-cols-2 gap-2 p-4 text-center sm:grid-cols-4 lg:grid-cols-1 lg:text-left">
            <div class="rounded-2xl bg-white/80 p-3">
              <dt class="text-xs text-stone-600">Tur tutarı</dt>
              <dd class="font-display text-lg font-extrabold tabular-nums">
                {{ pot !== null ? formatStroops(pot) : '—' }}
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
              <dt class="text-xs text-stone-600">Toplam tur</dt>
              <dd class="font-display text-lg font-extrabold tabular-nums">{{ membersOk ? memberLimit : '—' }}</dd>
            </div>
          </dl>
        </div>
      </aside>
    </div>
  </div>
</template>
