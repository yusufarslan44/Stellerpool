<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import AnchorDemo from '@/components/AnchorDemo.vue'
import AppIcon from '@/components/AppIcon.vue'
import CoinSpinner from '@/components/CoinSpinner.vue'
import Illo from '@/components/Illo.vue'
import PoolCalculator from '@/components/PoolCalculator.vue'
import Scene3D from '@/components/Scene3D.vue'
import HowItWorks from '@/components/HowItWorks.vue'
import StorySim from '@/components/StorySim.vue'
import StepIndicator from '@/components/StepIndicator.vue'
import { useNetworkStatus } from '@/composables/useNetworkStatus'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatDecimalString, parseAmount, shortAddress } from '@/lib/format'
import {
  circleFaucetUrl,
  config,
  explorerAccount,
  explorerTx,
  poolAsset,
  poolContractId,
} from '@/lib/stellar'
import { addTrustline, fundWithFriendbot, loadAccount } from '@/services/account'
import type { AccountInfo } from '@/services/account'
import { useWalletStore } from '@/stores/wallet'
import type { IlloName } from '@/lib/icon-data'

const wallet = useWalletStore()
const router = useRouter()
const { ledger, online } = useNetworkStatus()
const token = poolAsset.getCode()

// --- Başlangıç sihirbazı: hesap durumu -----------------------------------------------------
const account = ref<AccountInfo | null>(null)
const loading = ref(false)
const busy = ref<'fund' | 'trust' | null>(null)
const error = ref<string | null>(null)
const lastTx = ref<string | null>(null)
const copied = ref(false)
const poolIdInput = ref('')

async function refresh() {
  error.value = null
  if (!wallet.address) {
    account.value = null
    return
  }
  loading.value = true
  try {
    account.value = await loadAccount(wallet.address)
  } catch (e) {
    error.value = errorMessage(e)
  } finally {
    loading.value = false
  }
}
watch(() => wallet.address, refresh, { immediate: true })

async function fund() {
  if (!wallet.address) return
  busy.value = 'fund'
  error.value = null
  try {
    await fundWithFriendbot(wallet.address)
    await refresh()
  } catch (e) {
    error.value = errorMessage(e)
  } finally {
    busy.value = null
  }
}

async function trust() {
  if (!wallet.address) return
  busy.value = 'trust'
  error.value = null
  try {
    lastTx.value = await addTrustline(wallet.address, wallet.signTransaction)
    await refresh()
  } catch (e) {
    if (!isUserRejection(e)) error.value = errorMessage(e)
  } finally {
    busy.value = null
  }
}

async function copyAddress() {
  if (!wallet.address) return
  await navigator.clipboard?.writeText(wallet.address)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}

function openPool() {
  const id = Number.parseInt(poolIdInput.value, 10)
  if (Number.isInteger(id) && id >= 0) router.push(`/pool/${id}`)
}

const hasBalance = computed(() => {
  const a = account.value?.asset
  if (a == null) return false
  try {
    return parseAmount(a) > 0n
  } catch {
    return false
  }
})

const SETUP = computed(() => [
  { key: 'wallet', label: 'Cüzdan', done: wallet.isConnected },
  { key: 'activate', label: 'Hesap', done: account.value?.exists === true },
  { key: 'trust', label: `${token} kabul`, done: account.value?.hasTrustline === true },
  { key: 'faucet', label: `Test ${token}`, done: hasBalance.value },
])
const setupIndex = computed(() => {
  const i = SETUP.value.findIndex((s) => !s.done)
  return i === -1 ? SETUP.value.length : i
})
const setupKey = computed(() => SETUP.value[setupIndex.value]?.key ?? 'ready')

// Test varlığı adımında bakiye otomatik kontrol edilir (kullanıcı Circle sayfasından dönünce görünsün).
let poll: ReturnType<typeof setInterval> | undefined
watch(
  setupKey,
  (k) => {
    clearInterval(poll)
    if (k === 'faucet') poll = setInterval(() => void refresh(), 10_000)
  },
  { immediate: true },
)
onBeforeUnmount(() => clearInterval(poll))

const ROLES: { icon: IlloName; title: string; text: string; tone: string }[] = [
  {
    icon: 'seedling',
    title: 'Kurucu',
    text: 'Havuzu açar, sıra (ya da kura) ve doğrulayıcıları önerir. Parayı çekme yetkisi yoktur.',
    tone: 'bg-brand-100 text-brand-800',
  },
  {
    icon: 'lock',
    title: 'Kontrat',
    text: 'Katkıları tur bazında tutmayı ve koşullar sağlanınca yalnız izinli ödemeyi yapmayı hedefler.',
    tone: 'bg-gold-100 text-amber-900',
  },
  {
    icon: 'handshake',
    title: 'Üye',
    text: 'Katılır, her tur katkısını öder. Sırası gelince ya da kurada seçilince satıcıyı ve alım belgesini önerir.',
    tone: 'bg-sage-100 text-sage-800',
  },
  {
    icon: 'magnifier',
    title: 'Doğrulayıcı',
    text: 'Alım belgesini zincir dışında kontrol eder ve onaylar. Tutar ancak yeterli onayla çıkar.',
    tone: 'bg-stone-200 text-stone-800',
  },
]

const TRUST: { icon: IlloName; title: string; text: string }[] = [
  {
    icon: 'lock',
    title: 'Para sözleşmede',
    text: 'Planlanan Testnet akışında katkılar Soroban kontratına gider; kurucuya serbest çekim yetkisi verilmez.',
  },
  {
    icon: 'pin',
    title: 'Sıra ya da kura',
    text: 'Havuz başlayınca üyeler değişmez. Alıcı, onaylanan sıradan ya da tüm katkılar gelince çekilen kuradan çıkar.',
  },
  {
    icon: 'shield',
    title: 'Açık risk sınırı',
    text: 'Bir tur ödendikten sonra geçmiş katkılar kontrattan geri alınamaz; gelecek teslimat garanti edilmez.',
  },
  { icon: 'eyes', title: 'Herkes doğrular', text: 'Zincire yazılan demo işlemler Stellar Expert’te görülebilir.' },
]

const FAQ = [
  {
    q: 'Organizatör parayı alıp kaçabilir mi?',
    a: 'Hedef kontratta kurucunun serbest çekim yetkisi olmayacak. Kontrat henüz yazılıp yayınlanmadı; bu koruma şu anda çalışır durumda değil.',
  },
  {
    q: 'Sıram sonradan değiştirilebilir mi?',
    a: 'Hedef kurala göre havuz başladıktan sonra üyeler ve (sabit sıra modunda) sıra kilitlenecek. Bu kuralın kontrat ve testlerle doğrulanması gerekiyor.',
  },
  {
    q: 'Kura nasıl çalışıyor?',
    a: 'Kura modunda önceden sıra yoktur. Her tur herkes kendi katkısını yatırır; hepsi tamamlanınca herkesin çağırabileceği bir işlemle, henüz teslim almamış üyeler arasından alıcıyı kontrat seçer. Kazanan zaten payını ödemiştir. Zincir üstü rastgelelik hackathon düzeyindedir, yüksek tutarlı gerçek kullanım için yetmez. Kontratın kura desteği henüz yayınlanmadı.',
  },
  {
    q: 'Fuzul Ev veya Eminevim ile aynı mı?',
    a: 'Hayır. Onlar lisanslı tasarruf finansman şirketleridir: ayrılmış fon havuzu, sözleşme, ipotek/rehin ve şirket taahhüdüyle çalışırlar. Stellerpool bir Testnet prototipidir. Ortak yanı grup katkısı, sıra ya da kura ve ödeme aksayınca durmadır; katkılar tur bazında kontratta kilitli kalır ve kurallar herkese görünür. Farkı ise ipotek, şirket garantisi, gerçek ev/araç teslimi ve tahsilat olmamasıdır.',
  },
  {
    q: 'Kaç kişilik grup kurulabilir?',
    a: 'Yayındaki Testnet kontratı şu an 2 ile 12 üyeyi destekliyor; hedef 30 üye. Büyük grupta bir turda satıcıya giden tutar da büyür; erken teslim alan sonraki katkıyı bırakırsa açık da büyür. Grubu büyütmek bu riski ortadan kaldırmaz.',
  },
  {
    q: 'Biri ödemeyi bırakırsa ne olur?',
    a: 'Planlanan Testnet akışında ek süre sonunda havuz sonlandırılabilir. Yalnızca henüz ödenmemiş turun katkıları iade edilebilir; önceki turlar geri alınamaz. Kontrat henüz çalışmıyor.',
  },
  {
    q: 'Ayşe ilk turda alıp sonra bırakırsa ne olur?',
    a: 'Dört kişi 10’ar birim yatırırsa ilk tur 40 birim satıcıya gider ve havuzda o turun parası kalmaz. Ayşe sonraki turu ödemezse ikinci tur durur; yalnızca ikinci turda yatırılan katkılar iade edilir. Mehmet, Zeynep ve Can’ın ilk tur payları kontrattan geri alınamaz. Bunu “Bir turu kendin dene” bölümünde adım adım görebilirsin.',
  },
  {
    q: 'Sponsor, sigorta ya da teslimat garantisi var mı?',
    a: 'Hayır. Ayrı sponsor, başkası adına avans ve platform garantisi yok. Daha fazla kişi katılması da erken teslim alanın gelecekteki ödeme riskini ortadan kaldırmaz. Bu risk yalnızca tanıdık, kapalı bir grupta ve ayrı sözleşmelerle yönetilebilir.',
  },
  {
    q: 'Param kime gider?',
    a: 'Testnet planında doğrulayıcı onayı sonrası yalnızca demo satıcısına test varlığı gönderilecek. Şu anda havuz kontratı ve satıcı ödemesi yok.',
  },
  {
    q: 'Faiz veya vade farkı var mı?',
    a: 'Örnek hesapta faiz veya vade farkı modellenmiyor. Henüz çalışan sözleşme veya anchor bulunmuyor; gerçek bir ürünün ücretleri ayrıca belirlenir.',
  },
  {
    q: 'Gerçek para mı kullanılıyor?',
    a: 'Hayır. Bu, Stellar Testnet arayüzüdür; havuz kontratı henüz yayınlanmadı. Gerçek para, ev veya araç teslimi yoktur.',
  },
]
</script>

<template>
  <div class="space-y-24">
    <!-- 1 · KARŞILAMA -->
    <section class="sunrise relative isolate overflow-hidden rounded-[2.25rem] px-5 py-10 sm:px-10 sm:py-14 lg:py-16">
      <div class="grid items-center gap-6 lg:grid-cols-[1.05fr_1fr]">
        <div>
          <div class="glass inline-flex items-center gap-2 rounded-full px-3.5 py-1.5 text-xs font-medium text-stone-700">
            <span
              class="size-2 rounded-full"
              :class="online === false ? 'bg-rose-500' : 'animate-pulse bg-sage-500'"
              aria-hidden="true"
            />
            <span v-if="online === false">Ağa ulaşılamıyor</span>
            <span v-else-if="ledger !== null" class="tabular-nums">
              Stellar {{ config.label }} canlı · defter #{{ ledger.toLocaleString('tr-TR') }}
            </span>
            <span v-else>Ağa bağlanılıyor…</span>
          </div>

          <h1 class="mt-5 text-5xl leading-[1.02] font-extrabold sm:text-6xl lg:text-7xl">
            Birlikte biriktir.
            <span class="block bg-gradient-to-r from-brand-600 via-brand-500 to-gold-500 bg-clip-text text-transparent">
              Her şeyi doğrula.
            </span>
          </h1>
          <p class="mt-5 max-w-xl text-lg leading-relaxed text-stone-700">
            Ev, araç ya da ortak bir hedef için sırayla ya da kurayla birikim yap. Hedef: paran bir kişinin
            cebinde değil, herkesin görebildiği kuralların içinde dursun.
          </p>

          <div class="mt-8 flex flex-wrap gap-3">
            <a href="#basla" class="btn-primary btn-lg">
              Adım adım başla
              <AppIcon name="arrow" class="!size-4" />
            </a>
            <a href="#nasil" class="btn-secondary btn-lg">Nasıl çalışır?</a>
          </div>

          <ol class="mt-9 flex flex-wrap items-center gap-2 text-sm font-medium text-stone-700" aria-label="Üç adımda özet">
            <li class="flex items-center gap-2 rounded-full bg-white/70 py-1.5 pr-3.5 pl-1.5">
              <span class="grid size-7 place-items-center rounded-full bg-brand-600 text-xs font-bold text-white">1</span>
              Cüzdanını bağla
            </li>
            <AppIcon name="arrow" class="!size-4 text-brand-500" />
            <li class="flex items-center gap-2 rounded-full bg-white/70 py-1.5 pr-3.5 pl-1.5">
              <span class="grid size-7 place-items-center rounded-full bg-brand-600 text-xs font-bold text-white">2</span>
              Havuzu kur
            </li>
            <AppIcon name="arrow" class="!size-4 text-brand-500" />
            <li class="flex items-center gap-2 rounded-full bg-white/70 py-1.5 pr-3.5 pl-1.5">
              <span class="grid size-7 place-items-center rounded-full bg-brand-600 text-xs font-bold text-white">3</span>
              Birlikte biriktir
            </li>
          </ol>
        </div>

        <!-- 3B sahne: ortada havuz, çevresinde üyeleri temsil eden paralar -->
        <div class="relative mx-auto h-[340px] w-full max-w-xl sm:h-[420px] lg:h-[480px]">
          <Scene3D :coins="4" label="Ortada büyük bir havuz parası, çevresinde dönen dört üye parası" />
          <div class="float glass absolute top-6 left-0 flex items-center gap-2 rounded-2xl px-3.5 py-2 text-xs font-semibold text-ink sm:left-2">
            <Illo name="lock" :size="22" /> Hedef: kurallar kilitli
          </div>
          <div class="float glass absolute right-0 bottom-10 flex items-center gap-2 rounded-2xl px-3.5 py-2 text-xs font-semibold text-ink [animation-delay:-2.4s] sm:right-2">
            <Illo name="shield" :size="22" /> Hedef: para sözleşmede
          </div>
        </div>
      </div>
    </section>

    <p
      v-if="!poolContractId"
      role="status"
      class="-mt-12 flex items-start gap-3 rounded-3xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm leading-relaxed text-amber-950 sm:items-center sm:p-5"
    >
      <Illo name="bulb" :size="30" />
      <span>
        <strong>Durum:</strong> Havuz ve ödeme akışı hedef tasarımdır. Soroban havuz kontratı henüz
        yayınlanmadı; şu anda test varlığıyla havuz açılamaz veya tahsisat yapılamaz. Gerçek para yoktur.
      </span>
    </p>

    <HowItWorks />

    <!-- 2b · HİKÂYE: dokunarak oynanan örnek tur -->
    <section id="hikaye" class="scroll-mt-28 space-y-8" aria-labelledby="hikaye-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Dene</p>
        <h2 id="hikaye-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Bir turu kendin dene</h2>
        <p class="mt-3 text-stone-600">
          Dört arkadaş, bir havuz. Dokun, öde, onayla; “ya biri ödemezse?” sorusunu da dene.
        </p>
      </div>
      <div v-reveal><StorySim /></div>
    </section>

    <!-- 3 · BAŞLA (hesap hazırlama sihirbazı) -->
    <section id="basla" class="scroll-mt-28 space-y-8" aria-labelledby="basla-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Başla</p>
        <h2 id="basla-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Hesabını dört adımda hazırla</h2>
        <p class="mt-3 text-stone-600">Hepsi {{ config.label }} üzerinde, gerçek para yok. Sırayla ilerle.</p>
      </div>

      <div v-reveal class="card mx-auto max-w-3xl space-y-6 !p-5 sm:!p-8">
        <StepIndicator :steps="SETUP" :current="setupIndex" />

        <Transition name="step-next" mode="out-in">
          <div :key="setupKey" class="min-h-56">
            <!-- 1 · Cüzdan -->
            <div v-if="setupKey === 'wallet'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-brand-100 text-brand-700">
                  <Illo name="purse" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">1. Cüzdanını bağla</h3>
                  <p class="text-stone-600">Cüzdan, hesabın ve imzan demektir. Şifreni kimseyle paylaşmayız.</p>
                </div>
              </div>
              <button type="button" class="btn-primary btn-lg" :disabled="wallet.busy" @click="wallet.connect()">
                {{ wallet.busy ? 'Bağlanıyor…' : 'Cüzdan bağla' }}
              </button>
              <p v-if="wallet.error" role="alert" class="text-sm text-rose-700">{{ wallet.error }}</p>
              <p class="text-sm text-stone-600">
                Freighter, xBull, Albedo, LOBSTR, Hana veya Rabet kullanabilirsin. Uzantı yüklemek
                istemezsen Albedo web üzerinden çalışır.
              </p>
            </div>

            <!-- Hesap okunuyor -->
            <div v-else-if="loading && !account" class="grid min-h-56 place-items-center">
              <div class="flex flex-col items-center gap-3 text-stone-600">
                <CoinSpinner :size="52" />
                Hesabın okunuyor…
              </div>
            </div>

            <!-- 2 · Hesabı etkinleştir -->
            <div v-else-if="setupKey === 'activate'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-gold-100 text-amber-800">
                  <Illo name="sparkles" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">2. Hesabını etkinleştir</h3>
                  <p class="text-stone-600">
                    Stellar hesabı ilk kez oluşurken az miktarda işlem ücreti (XLM) gerekir. Testnet’te bunu
                    ücretsiz bir musluktan (Friendbot) alırız.
                  </p>
                </div>
              </div>
              <button type="button" class="btn-primary btn-lg" :disabled="busy !== null" @click="fund">
                {{ busy === 'fund' ? 'Etkinleştiriliyor…' : `${config.label} hesabını etkinleştir` }}
              </button>
            </div>

            <!-- 3 · Kabul aç -->
            <div v-else-if="setupKey === 'trust'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-sage-100 text-sage-700">
                  <Illo name="coin" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">3. {{ token }} kabul etmeyi aç</h3>
                  <p class="text-stone-600">
                    Hesabının {{ token }} tutabilmesi için bir kez izin vermen gerekir (trustline). Cüzdanın
                    imza isteyecek.
                  </p>
                </div>
              </div>
              <button type="button" class="btn-primary btn-lg" :disabled="busy !== null" @click="trust">
                {{ busy === 'trust' ? 'Cüzdanı onayla…' : `${token} kabul etmeyi aç` }}
              </button>
            </div>

            <!-- 4 · Test varlığı al -->
            <div v-else-if="setupKey === 'faucet'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-brand-100 text-brand-700">
                  <Illo name="moneybag" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">4. Test {{ token }} al</h3>
                  <p class="text-stone-600">
                    Deneme için gerçek olmayan test parası gerekir. Circle’ın sayfasında “Stellar Testnet”
                    ağını seç, adresini yapıştır.
                  </p>
                </div>
              </div>
              <div class="flex flex-wrap gap-3">
                <a :href="circleFaucetUrl" target="_blank" rel="noopener noreferrer" class="btn-primary btn-lg">
                  Circle’ı aç <AppIcon name="external" class="!size-4" />
                </a>
                <button type="button" class="btn-secondary btn-lg" @click="copyAddress">
                  <AppIcon name="copy" class="!size-4" />
                  {{ copied ? 'Kopyalandı ✓' : 'Adresimi kopyala' }}
                </button>
                <button type="button" class="btn-secondary btn-lg" :disabled="loading" @click="refresh">
                  <AppIcon name="refresh" class="!size-4" />
                  Bakiyemi kontrol et
                </button>
              </div>
              <p class="text-sm text-stone-600">Bakiye gelince bu ekran kendiliğinden bir sonraki adıma geçer.</p>
            </div>

            <!-- Hazır -->
            <div v-else class="flex flex-col items-center gap-4 py-4 text-center">
              <Illo name="party" :size="84" class="pop" />
              <h3 class="text-3xl font-extrabold">Hazırsın!</h3>
              <p class="max-w-md text-stone-600">
                Cüzdanın bağlı, hesabın etkin ve test {{ token }} elinde. Şimdi ilk havuzunu kurabilirsin.
              </p>
              <RouterLink to="/create" class="btn-primary btn-lg">
                Havuz oluştur <AppIcon name="arrow" class="!size-4" />
              </RouterLink>
            </div>
          </div>
        </Transition>

        <!-- Hesap özeti -->
        <dl v-if="account && wallet.address" class="grid gap-3 border-t border-stone-100 pt-5 sm:grid-cols-3">
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">Adres</dt>
            <dd class="mt-0.5 font-mono text-sm">
              <a
                :href="explorerAccount(wallet.address)"
                target="_blank"
                rel="noopener noreferrer"
                class="text-brand-700 underline"
              >
                {{ shortAddress(wallet.address, 6) }}
              </a>
            </dd>
          </div>
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">XLM (işlem ücreti)</dt>
            <dd class="mt-0.5 text-lg font-bold tabular-nums">
              {{ account.exists ? formatDecimalString(account.xlm) : '—' }}
            </dd>
          </div>
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">{{ token }}</dt>
            <dd class="mt-0.5 text-lg font-bold tabular-nums">
              {{ account.asset !== null ? formatDecimalString(account.asset) : '—' }}
            </dd>
          </div>
        </dl>

        <p v-if="lastTx" class="text-sm text-sage-800">
          İşlem gönderildi:
          <a :href="explorerTx(lastTx)" target="_blank" rel="noopener noreferrer" class="font-mono underline">
            {{ lastTx.slice(0, 8) }}…
          </a>
        </p>
        <p v-if="error" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ error }}</p>
      </div>

      <form v-reveal class="mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-3 px-2" @submit.prevent="openPool">
        <div>
          <h3 class="text-lg font-bold">Havuz numaran var mı?</h3>
          <p class="text-sm text-stone-600">Sana verilen numarayı girip doğrudan havuza git.</p>
        </div>
        <div class="flex gap-2">
          <label class="sr-only" for="pool-id">Havuz numarası</label>
          <input
            id="pool-id"
            v-model="poolIdInput"
            class="input w-36"
            type="number"
            min="0"
            step="1"
            inputmode="numeric"
            placeholder="Havuz no"
            required
          />
          <button type="submit" class="btn-primary">Aç</button>
        </div>
      </form>
    </section>

    <!-- 4 · KİM NE YAPAR -->
    <section class="space-y-8" aria-labelledby="roller-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Roller</p>
        <h2 id="roller-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Havuzda kim ne yapar?</h2>
      </div>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <article v-for="(r, i) in ROLES" :key="r.title" v-reveal="i" v-tilt class="bento h-full">
          <span class="grid size-16 place-items-center rounded-3xl" :class="r.tone">
            <Illo :name="r.icon" :size="44" />
          </span>
          <h3 class="mt-4 text-xl font-bold">{{ r.title }}</h3>
          <p class="mt-1.5 text-sm leading-relaxed text-stone-600">{{ r.text }}</p>
        </article>
      </div>
    </section>

    <!-- 5 · HESAPLA -->
    <section id="hesapla" class="scroll-mt-28 space-y-8" aria-labelledby="hesapla-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Hesapla</p>
        <h2 id="hesapla-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Planını birlikte çizelim</h2>
        <p class="mt-3 text-stone-600">Amacını seç, tutarı ve kişi sayısını ayarla. Sağdaki paralar üye sayısını gösterir.</p>
      </div>
      <div v-reveal>
        <PoolCalculator />
      </div>
    </section>

    <!-- 6 · GÜVEN -->
    <section class="espresso relative overflow-hidden rounded-[2.25rem] px-5 py-12 text-white sm:px-10 sm:py-16" aria-labelledby="guven-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-gold-300">Güven modeli</p>
        <h2 id="guven-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Neden güvenli olması hedefleniyor?</h2>
        <div class="mt-6 flex flex-wrap items-center justify-center gap-2 text-sm font-semibold" aria-label="Para akışı">
          <span class="glass-dark rounded-full px-4 py-2">Üyeler</span>
          <AppIcon name="arrow" class="!size-4 text-gold-300" />
          <span class="rounded-full bg-gold-400 px-4 py-2 text-ink">Sözleşme</span>
          <AppIcon name="arrow" class="!size-4 text-gold-300" />
          <span class="glass-dark rounded-full px-4 py-2">Doğrulanmış satıcı</span>
        </div>
      </div>
      <div class="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <article
          v-for="(t, i) in TRUST"
          :key="t.title"
          v-reveal="i"
          class="glass-dark rounded-3xl p-5 transition-transform duration-300 hover:-translate-y-1"
        >
          <span class="grid size-14 place-items-center rounded-2xl bg-white/10">
            <Illo :name="t.icon" :size="38" />
          </span>
          <h3 class="mt-4 text-lg font-bold">{{ t.title }}</h3>
          <p class="mt-1.5 text-sm leading-relaxed text-stone-200">{{ t.text }}</p>
        </article>
      </div>
      <p class="mt-8 text-center text-xs text-stone-300">
        Bunlar hedef tasarımdır. Kontrat yayınlanıp denetlenmeden gerçek para için kullanılmamalıdır.
      </p>
    </section>

    <!-- 7 · ANCHOR SİMÜLASYONU -->
    <div v-reveal><AnchorDemo /></div>

    <!-- 8 · SSS -->
    <section id="sss" class="scroll-mt-28 space-y-8" aria-labelledby="sss-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Aklındakiler</p>
        <h2 id="sss-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Sık sorulan sorular</h2>
        <p class="mt-3 text-sm text-stone-600">
          Cevaplar hedef tasarımı anlatır. Sözleşme yayınlanıp denetlenmeden gerçek para için kullanılmamalıdır.
        </p>
      </div>
      <div class="mx-auto grid max-w-4xl gap-3 md:grid-cols-2">
        <details v-for="(item, i) in FAQ" :key="item.q" v-reveal="i % 2" class="group card cursor-pointer !p-0 open:shadow-[0_14px_32px_-16px_rgb(20_128_90/0.4)]">
          <summary
            class="flex min-h-14 list-none items-center justify-between gap-3 px-5 py-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden"
          >
            {{ item.q }}
            <AppIcon
              name="chevron"
              class="text-brand-600 transition-transform duration-300 group-open:rotate-180"
            />
          </summary>
          <p class="px-5 pb-5 text-sm leading-relaxed text-stone-600">{{ item.a }}</p>
        </details>
      </div>
    </section>
  </div>
</template>
