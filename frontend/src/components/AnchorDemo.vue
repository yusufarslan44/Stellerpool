<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import AppIcon from '@/components/AppIcon.vue'
import Illo from '@/components/Illo.vue'
import {
  anchorDomain,
  authenticate,
  getTransaction,
  resolveAnchor,
  startInteractive,
  STATUS_LABELS,
  TERMINAL_STATUSES,
  usingTestAnchor,
} from '@/lib/anchor'
import type { AnchorInfo, AnchorTransaction, InteractiveSession } from '@/lib/anchor'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { explorerAccount, mainnetShowcase, poolAsset } from '@/lib/stellar'
import { useWalletStore } from '@/stores/wallet'

/**
 * Anchor akışı. Testnet'te gerçek SEP-1 (stellar.toml) → SEP-10 (cüzdan imzalı giriş) → SEP-24
 * (interaktif yatırma) protokolünü çalıştırır. Varsayılan sağlayıcı SDF test anchor'ıdır: test
 * varlığı üretir, Türk lirası DEĞİLDİR. Mainnet tanıtımı imza kabul etmez, yalnızca anlatım
 * simülasyonu gösterir. Çekme (Stellar → TRY) canlı akışta henüz yoktur.
 */
const live = !mainnetShowcase
const wallet = useWalletStore()

// --- Canlı akış --------------------------------------------------------------------------
const anchor = ref<AnchorInfo | null>(null)
const anchorError = ref<string | null>(null)
const loadingAnchor = ref(false)
const asset = ref('')
const busy = ref<'auth' | null>(null)
const error = ref<string | null>(null)
const session = ref<InteractiveSession | null>(null)
const tx = ref<AnchorTransaction | null>(null)
/** SEP-10 oturum anahtarı yalnızca bellekte tutulur; sayfa yenilenince silinir. */
let token: string | null = null
let poll: ReturnType<typeof setInterval> | undefined

const depositAssets = computed(() =>
  Object.entries(anchor.value?.deposit ?? {})
    .filter(([, v]) => v.enabled)
    .map(([code, v]) => ({ code, ...v })),
)
const selected = computed(() => depositAssets.value.find((a) => a.code === asset.value) ?? null)
const finished = computed(() => (tx.value ? TERMINAL_STATUSES.has(tx.value.status) : false))

async function loadAnchor() {
  loadingAnchor.value = true
  anchorError.value = null
  try {
    anchor.value = await resolveAnchor()
    const codes = depositAssets.value.map((a) => a.code)
    const preferred = poolAsset.getCode()
    asset.value = codes.includes(preferred) ? preferred : (codes[0] ?? '')
  } catch (e) {
    anchorError.value = errorMessage(e)
  } finally {
    loadingAnchor.value = false
  }
}

async function begin() {
  if (!anchor.value || !wallet.address || !asset.value) return
  busy.value = 'auth'
  error.value = null
  session.value = null
  tx.value = null
  try {
    token = await authenticate(anchor.value, wallet.address, (xdr, opts) => wallet.signTransaction(xdr, opts))
    session.value = await startInteractive(anchor.value, token, 'deposit', asset.value, wallet.address)
    startPolling()
  } catch (e) {
    if (!isUserRejection(e)) error.value = errorMessage(e)
  } finally {
    busy.value = null
  }
}

/** Popup engelleyicilere takılmamak için pencere, kullanıcının tıklamasıyla açılır. */
function openWindow() {
  if (!session.value) return
  window.open(session.value.url, 'stellerpool-anchor', 'popup,width=480,height=760,noopener=no')
}

async function refreshTx() {
  if (!anchor.value || !token || !session.value) return
  try {
    tx.value = await getTransaction(anchor.value, token, session.value.id)
    if (finished.value) stopPolling()
  } catch (e) {
    error.value = errorMessage(e)
    stopPolling()
  }
}
function startPolling() {
  stopPolling()
  void refreshTx()
  poll = setInterval(() => void refreshTx(), 5000)
}
function stopPolling() {
  if (poll) clearInterval(poll)
  poll = undefined
}

/** Anchor penceresi bitince `postMessage` ile bildirir; yalnızca açtığımız pencerenin kökeni dinlenir. */
function onMessage(event: MessageEvent) {
  if (!session.value || event.origin !== new URL(session.value.url).origin) return
  const t = (event.data as { transaction?: { status?: string } } | null)?.transaction
  if (t?.status) void refreshTx()
}

onMounted(() => {
  if (!live) return
  void loadAnchor()
  window.addEventListener('message', onMessage)
})
onBeforeUnmount(() => {
  stopPolling()
  window.removeEventListener('message', onMessage)
  token = null
})

// --- Anlatım simülasyonu (bağlantısız) -------------------------------------------------------
type Direction = 'deposit' | 'withdraw'

const direction = ref<Direction>('deposit')
const step = ref(0)

const flows = {
  deposit: [
    { title: 'TRY yatırma talebi', text: 'Gerçek bir hizmette kullanıcı yetkili anchor üzerinden talep açar. Bu ekranda talep gönderilmez.' },
    { title: 'Banka hareketi doğrulaması', text: 'Gerçek hizmette ödemeyi sağlayıcı doğrular. Bu simülasyon banka hesabına bağlanmaz.' },
    { title: 'Stellar varlığının gönderimi', text: 'Gerçek hizmette ihraççı varlığı cüzdana aktarabilir. Burada token basılmaz veya transfer edilmez.' },
    { title: 'Örnek akış bitti', text: 'Adımlar yalnızca anlatım içindir. TRY veya Stellar bakiyesi oluşmadı ve değişmedi.' },
  ],
  withdraw: [
    { title: 'Varlığı çekme talebi', text: 'Gerçek bir hizmette kullanıcı yetkili anchor üzerinden çekim talebi açar. Bu ekranda talep gönderilmez.' },
    { title: 'Varlık ve kimlik kontrolü', text: 'Gerçek hizmette sağlayıcı bakiye ve uygunluğu denetler. Bu simülasyon cüzdana bağlanmaz.' },
    { title: 'Banka hesabına TRY ödemesi', text: 'Gerçek hizmette sağlayıcı uygun talebi banka kanalıyla sonuçlandırır. Burada ödeme emri verilmez.' },
    { title: 'Örnek akış bitti', text: 'Adımlar yalnızca anlatım içindir. TRY veya Stellar bakiyesi oluşmadı ve değişmedi.' },
  ],
} as const

const current = computed(() => flows[direction.value][step.value]!)

function select(next: Direction) {
  direction.value = next
  step.value = 0
}

function advance() {
  if (step.value < flows[direction.value].length - 1) step.value += 1
  else step.value = 0
}
</script>

<template>
  <section class="card space-y-5" aria-labelledby="anchor-demo-title">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <p class="eyebrow text-brand-700">Anchor · SEP-1 / SEP-10 / SEP-24</p>
        <h2 id="anchor-demo-title" class="mt-1 text-2xl font-extrabold">TRY ile Stellar bağlantısı</h2>
      </div>
      <span v-if="live" class="badge" :class="usingTestAnchor ? 'bg-gold-100 text-amber-900' : 'bg-brand-100 text-brand-800'">
        {{ usingTestAnchor ? 'Test anchor · TRY değil' : anchorDomain }}
      </span>
      <span v-else class="badge bg-amber-100 text-amber-900">Yalnızca simülasyon</span>
    </div>

    <!-- CANLI AKIŞ -->
    <template v-if="live">
      <p class="text-sm leading-relaxed text-stone-600">
        Bu bölüm <strong>{{ anchorDomain }}</strong> ile gerçek anchor protokolünü çalıştırır: uç noktalar
        <code class="font-mono">stellar.toml</code>'dan okunur, cüzdanınla giriş imzalarsın, yatırma penceresi anchor'ın
        kendi arayüzünde açılır.
        <template v-if="usingTestAnchor || (anchor && !anchor.supportsTry)">
          <strong>Bu sağlayıcı test varlığı ({{ anchor?.assetCodes.join(', ') || 'USDC' }}) üretir, Türk lirası sunmaz.</strong>
          Hackathon'un gerçek TL şartı, TRY sunan doğrulanmış bir sağlayıcı bulununca aynı akışla karşılanır.
        </template>
      </p>

      <div v-if="loadingAnchor" class="skeleton h-24 rounded-2xl" aria-live="polite" />

      <div v-else-if="anchorError" role="alert" class="space-y-2 rounded-2xl bg-rose-50 p-4 text-sm text-rose-800">
        <p>Anchor'a ulaşılamadı: {{ anchorError }}</p>
        <button type="button" class="btn-secondary !min-h-10" @click="loadAnchor">Tekrar dene</button>
      </div>

      <template v-else-if="anchor">
        <div class="flex flex-wrap items-center gap-2 text-xs">
          <span class="badge bg-sage-100 text-sage-800"><AppIcon name="check" class="!size-3.5" /> stellar.toml okundu</span>
          <span class="badge bg-stone-100 text-stone-700">Varlıklar: {{ anchor.assetCodes.join(' · ') }}</span>
          <span class="badge" :class="anchor.supportsTry ? 'bg-sage-100 text-sage-800' : 'bg-gold-100 text-amber-900'">
            {{ anchor.supportsTry ? 'TRY destekleniyor' : 'TRY yok' }}
          </span>
        </div>

        <ol class="space-y-2 text-sm" aria-label="Anchor adımları">
          <li class="flex items-start gap-2.5">
            <span class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-full bg-sage-600 text-xs font-bold text-white">
              <AppIcon name="check" class="!size-3.5" />
            </span>
            <span><strong>Anchor tanınır.</strong> Uç noktalar ve desteklenen varlıklar okundu.</span>
          </li>
          <li class="flex items-start gap-2.5">
            <span class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-full text-xs font-bold text-white" :class="session ? 'bg-sage-600' : 'bg-brand-600'">
              <AppIcon v-if="session" name="check" class="!size-3.5" /><template v-else>2</template>
            </span>
            <span><strong>Cüzdanınla giriş yaparsın.</strong> Anchor'ın gönderdiği doğrulama işlemi (challenge) uygulamada doğrulanır ve sen imzalarsın. Ücret ödenmez.</span>
          </li>
          <li class="flex items-start gap-2.5">
            <span class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-full text-xs font-bold text-white" :class="finished && tx?.status === 'completed' ? 'bg-sage-600' : session ? 'bg-brand-600' : 'bg-stone-300'">3</span>
            <span><strong>Anchor penceresinde işlemi tamamlarsın.</strong> Durum burada canlı izlenir.</span>
          </li>
        </ol>

        <div v-if="!session" class="space-y-3 rounded-2xl bg-sand/60 p-4">
          <div class="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
            <div>
              <label class="label" for="anchor-asset">Yatırılacak varlık</label>
              <select id="anchor-asset" v-model="asset" class="input">
                <option v-for="a in depositAssets" :key="a.code" :value="a.code">
                  {{ a.code === 'native' ? 'XLM' : a.code }}{{ a.minAmount || a.maxAmount ? ` (${a.minAmount ?? '—'}–${a.maxAmount ?? '—'})` : '' }}
                </option>
              </select>
            </div>
            <button v-if="!wallet.isConnected" type="button" class="btn-primary" :disabled="wallet.busy" @click="wallet.connect()">
              <AppIcon name="wallet" class="!size-4" /> Önce cüzdan bağla
            </button>
            <button v-else type="button" class="btn-primary" :disabled="busy !== null || !selected" @click="begin">
              {{ busy === 'auth' ? 'Cüzdanı onayla…' : 'Anchor ile başla' }}
            </button>
          </div>
          <p v-if="asset && asset !== 'native'" class="text-xs text-stone-600">
            {{ asset }} alabilmen için hesabında bu varlığa güven (trustline) olmalı. Yukarıdaki kurulum sihirbazının “güven” adımı bunu yapar.
          </p>
        </div>

        <div v-else class="pop space-y-3 rounded-2xl border-2 border-brand-200 bg-brand-50/60 p-4" aria-live="polite">
          <div class="flex items-center gap-2">
            <Illo :name="tx?.status === 'completed' ? 'party' : 'hourglass'" :size="30" />
            <p class="font-display font-bold">{{ tx ? (STATUS_LABELS[tx.status] ?? tx.status) : 'Durum okunuyor…' }}</p>
          </div>
          <p v-if="tx" class="text-xs text-stone-600">
            İşlem no: <span class="font-mono">{{ tx.id.slice(0, 8) }}…</span>
            <template v-if="tx.amountIn"> · yatırılan {{ tx.amountIn }}</template>
            <template v-if="tx.amountOut"> · gelen {{ tx.amountOut }}</template>
          </p>
          <div v-if="!finished" class="flex flex-wrap items-center gap-3">
            <button type="button" class="btn-primary" @click="openWindow">
              Anchor penceresini aç <AppIcon name="external" class="!size-4" />
            </button>
            <span class="text-xs text-stone-600">
              Pencere <strong class="font-mono">{{ session.host }}</strong> adresinde açılır<template v-if="!session.sameDomain">
              (anchor'ın kendi arayüz sunucusu; adresi kontrol et)</template>.
            </span>
          </div>
          <div v-else class="flex flex-wrap items-center gap-3">
            <a v-if="wallet.address && tx?.status === 'completed'" :href="explorerAccount(wallet.address)" target="_blank" rel="noopener noreferrer" class="btn-secondary !min-h-10">
              Bakiyeyi Stellar Expert'te gör <AppIcon name="external" class="!size-4" />
            </a>
            <button type="button" class="btn-secondary !min-h-10" @click="session = null; tx = null">Yeni işlem</button>
          </div>
        </div>

        <p v-if="error" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ error }}</p>
        <p class="text-xs leading-relaxed text-stone-500">
          Oturum anahtarı yalnızca bu sayfanın belleğinde tutulur. Uygulama banka bilgisi, kimlik verisi veya gizli anahtar istemez;
          kimlik doğrulama (KYC) gerekirse anchor'ın kendi penceresinde istenir. Çekme (Stellar → TRY) akışı bu sürümde canlı değil.
        </p>
      </template>
    </template>

    <!-- ANLATIM SİMÜLASYONU -->
    <component :is="live ? 'details' : 'div'" class="group space-y-4 rounded-2xl" :class="live ? 'border border-stone-200 p-4' : ''">
      <summary v-if="live" class="flex min-h-11 cursor-pointer list-none items-center justify-between gap-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
        Anlatım simülasyonu (bağlantısız)
        <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
      </summary>

      <p class="text-sm leading-relaxed text-stone-600">
        Gerçek TRY anchor'ının nasıl işleyeceğini anlatan örnek. Kişisel bilgi istemez, ödeme talimatı veya zincir işlemi üretmez,
        kullanılabilir bakiye oluşturmaz.
      </p>

      <div class="flex flex-wrap gap-2" role="group" aria-label="Örnek akış yönü">
        <button type="button" class="btn-secondary" :aria-pressed="direction === 'deposit'" :class="direction === 'deposit' ? '!border-brand-600 !bg-brand-50 !text-brand-900' : ''" @click="select('deposit')">
          TRY → Stellar varlığı
        </button>
        <button type="button" class="btn-secondary" :aria-pressed="direction === 'withdraw'" :class="direction === 'withdraw' ? '!border-brand-600 !bg-brand-50 !text-brand-900' : ''" @click="select('withdraw')">
          Stellar varlığı → TRY
        </button>
      </div>

      <div :key="`${direction}-${step}`" class="pop rounded-2xl border border-gold-300/60 bg-gold-100/70 p-5" role="status" aria-live="polite">
        <div class="flex items-center gap-1.5" aria-hidden="true">
          <span
            v-for="n in 4"
            :key="n"
            class="h-1.5 flex-1 rounded-full transition-colors duration-300"
            :class="n <= step + 1 ? 'bg-brand-500' : 'bg-gold-300/50'"
          />
        </div>
        <p class="mt-3 text-xs font-semibold uppercase tracking-wide text-amber-900">Simülasyon · adım {{ step + 1 }} / 4</p>
        <h3 class="mt-2 font-semibold text-amber-950">{{ current.title }}</h3>
        <p class="mt-2 text-sm leading-relaxed text-amber-950">{{ current.text }}</p>
      </div>

      <button type="button" class="btn-primary" @click="advance">
        {{ step === 3 ? 'Örneği başa al' : 'Sonraki örnek adım' }}
      </button>
    </component>
  </section>
</template>
