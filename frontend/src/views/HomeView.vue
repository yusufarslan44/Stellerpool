<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import AnchorDemo from '@/components/AnchorDemo.vue'
import AppIcon from '@/components/AppIcon.vue'
import CoinSpinner from '@/components/CoinSpinner.vue'
import Illo from '@/components/Illo.vue'
import PoolCalculator from '@/components/PoolCalculator.vue'
import HomeHero from '@/components/HomeHero.vue'
import ContractFlow from '@/components/ContractFlow.vue'
import HowItWorks from '@/components/HowItWorks.vue'
import StorySim from '@/components/StorySim.vue'
import StepIndicator from '@/components/StepIndicator.vue'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatDecimalString, parseAmount, shortAddress } from '@/lib/format'
import {
  circleFaucetUrl,
  config,
  explorerAccount,
  explorerTx,
  poolAsset,
  poolAssetFromCircleFaucet,
  poolContractId,
} from '@/lib/stellar'
import { addTrustline, fundWithFriendbot, loadAccount } from '@/services/account'
import type { AccountInfo } from '@/services/account'
import { useWalletStore } from '@/stores/wallet'
import type { IlloName } from '@/lib/icon-data'

const wallet = useWalletStore()
const router = useRouter()
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
    text: 'Havuzu açar, sıra ve doğrulayıcıları önerir. Parayı çekme yetkisi yoktur.',
    tone: 'bg-brand-100 text-brand-800',
  },
  {
    icon: 'lock',
    title: 'Kontrat',
    text: 'Katkıları tur bazında tutar ve koşullar sağlanınca yalnızca izinli demo satıcısına ödeme yapar.',
    tone: 'bg-gold-100 text-amber-900',
  },
  {
    icon: 'handshake',
    title: 'Üye',
    text: 'Katılır, her tur katkısını öder. Sırası gelince satıcıyı ve alım belgesini önerir.',
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
    text: 'Testnet kontratında katkılar Soroban kontratına gider; kurucuda serbest çekim fonksiyonu yoktur.',
  },
  {
    icon: 'pin',
    title: 'Sabit sıra',
    text: 'Havuz başlayınca üyeler ve sıra değişmez; sıra yalnızca üyelerin onayladığı sürümle geçerlidir. İstersen sıra yerine kura seç.',
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
    a: 'Testnet kontratında kurucunun serbest çekim fonksiyonu yok: tur tutarı yalnızca havuzda kayıtlı izinli demo satıcısına gidebilir, iade yalnızca katkı sahibine yapılır. Kontrat birim testlerinden geçti ancak bağımsız güvenlik denetimi yapılmadı.',
  },
  {
    q: 'Sıram sonradan değiştirilebilir mi?',
    a: 'Hayır. Başlamadan önce sıra yalnızca üyelerin onayladığı sürümle geçerli olur, kurucu tek başına değiştiremez. Havuz başladıktan sonra üyeler ve sıra kilitlidir.',
  },
  {
    q: 'Kura nasıl çalışıyor?',
    a: 'Kura modunda önceden sıra yoktur. Her tur herkes kendi katkısını yatırır; hepsi tamamlanınca herkesin çağırabileceği bir işlemle, henüz teslim almamış üyeler arasından alıcıyı kontrat seçer. Kazanan zaten payını ödemiştir. Zincir üstü rastgelelik hackathon düzeyindedir, yüksek tutarlı gerçek kullanım için yetmez. Kura Testnet kontratında canlı denendi; oluşturma formunda “Kura” seçilebilir.',
  },
  {
    q: 'Peşinat veya organizasyon ücreti var mı?',
    a: 'Organizasyon ücreti yok: şirketler tek seferlik yaklaşık %7–14 alır, burada kimseye ücret ayrılmaz. Peşinat var ve kontratta tutulur: havuz kurulurken üye başına bir peşinat belirlenir, herkes katılırken yatırır. Sıran gelince o tutar alımına eklenip satıcıya gider; havuz iptal olursa henüz almadıysan iade edilir. Şirketlerdeki gibi peşinat yükseldikçe hedef küçülür, verdiğin taksitle daha az kişi ve daha kısa vade çıkar. Fark: peşinat şirkete değil kontrata yatar ve yalnızca kendi alımına gider, birikime sayılmaz.',
  },
  {
    q: 'Kura ne zaman çekilir?',
    a: 'Şirketlerde çekiliş genelde her ay noter kontrolünde yapılır. Burada kura tarihi sabit değildir: her turda herkes taksidini yatırır yatırmaz çekilebilir. Katkı süresi (aylık planda 30 gün) içinde herkes öderse hemen; ödemeyen olursa ek süre gelir, o da biterse tur durur ve iade başlar. Kura herkesin çağırabileceği bir işlemle zincirde yapılır, sonucu kontrat belirler ve herkes doğrulayabilir. Noter veya canlı yayın yoktur.',
  },
  {
    q: 'Fuzul Ev veya Eminevim ile aynı mı?',
    a: 'Hayır. Onlar lisanslı tasarruf finansman şirketleridir: ayrılmış fon havuzu, sözleşme, ipotek/rehin ve şirket taahhüdüyle çalışırlar. Stellerpool bir Testnet prototipidir. Ortak yanı grup katkısı, sıra ya da kura ve ödeme aksayınca durmadır; katkılar tur bazında kontratta kilitli kalır ve kurallar herkese görünür. Farkı ise ipotek, şirket garantisi, gerçek ev/araç teslimi ve tahsilat olmamasıdır.',
  },
  {
    q: 'Kaç kişilik grup kurulabilir?',
    a: 'Yayındaki Testnet kontratı 2 ile 30 üyeyi destekliyor. Büyük grupta bir turda satıcıya giden tutar da büyür; erken teslim alan sonraki katkıyı bırakırsa açık da büyür. Grubu büyütmek bu riski ortadan kaldırmaz.',
  },
  {
    q: 'Biri ödemeyi bırakırsa ne olur?',
    a: 'Katkı süresi dolunca tur ek süreye geçer. Ek süre de biterse herkes havuzu sonlandırabilir; yalnızca henüz ödenmemiş turun katkıları iade edilir, önceki turlar geri alınamaz. Bu akış Testnet’te canlı olarak denendi (işlem bağlantıları README’de).',
  },
  {
    q: 'Ayşe ilk turda alıp sonra bırakırsa ne olur?',
    a: 'Dört kişi 10’ar birim yatırırsa ilk tur 40 birim satıcıya gider ve havuzda o turun parası kalmaz. Ayşe sonraki turu ödemezse ikinci tur durur; yalnızca ikinci turda yatırılan katkılar iade edilir. Mehmet, Zeynep ve Can’ın ilk tur payları kontrattan geri alınamaz.',
  },
  {
    q: 'Sponsor, sigorta ya da teslimat garantisi var mı?',
    a: 'Hayır. Ayrı sponsor, başkası adına avans ve platform garantisi yok. Daha fazla kişi katılması da erken teslim alanın gelecekteki ödeme riskini ortadan kaldırmaz. Bu risk yalnızca tanıdık, kapalı bir grupta ve ayrı sözleşmelerle yönetilebilir.',
  },
  {
    q: 'Param kime gider?',
    a: 'Doğrulayıcı eşiği onayladıktan sonra yalnızca havuzda kayıtlı izinli demo satıcısına test varlığı gider. Gerçek satıcı, tapu veya ruhsat doğrulanmaz.',
  },
  {
    q: 'Faiz veya vade farkı var mı?',
    a: 'Hayır, örnek hesapta faiz veya vade farkı modellenmiyor. Kontratta ücret ya da organizasyon bedeli yok; gerçek bir ürünün ücretleri ayrıca belirlenir.',
  },
  {
    q: 'Gerçek para mı kullanılıyor?',
    a: 'Hayır. Bu, Stellar Testnet arayüzüdür; test varlığı kullanılır. Gerçek para, gerçek TL girişi/çıkışı, ev veya araç teslimi yoktur. Anchor bölümü test anchor’ı ile çalışır, Türk lirası sunmaz.',
  },
]
</script>

<template>
  <div class="home-page space-y-24">
    <HomeHero />

    <p
      v-if="!poolContractId"
      role="status"
      class="home-contract-notice flex items-start gap-3 rounded-3xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm leading-relaxed text-amber-950 sm:items-center sm:p-5"
    >
      <Illo name="bulb" :size="30" />
      <span>
        <strong>Durum:</strong> Bu derleme henüz bir havuz kontratına bağlı değil
        (<code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> boş). Sponsorsuz havuz kontratı Testnet’te yayında;
        adresi yapılandırılınca havuz açılır ve okunur. Gerçek para yoktur.
      </span>
    </p>

    <div class="home-intro-strip" v-reveal>
      <span class="intro-strip-label">BİRLİKTE BİRİKİMİN TEMELİ</span>
      <span><AppIcon name="users" /> Tanıdığın bir grup</span><span><AppIcon name="lock" /> Herkesin onayladığı kurallar</span><span><AppIcon name="eye" /> Görünür para akışı</span>
    </div>

    <HowItWorks />

    <!-- 2b · HİKÂYE: kendiliğinden oynayan örnek tur -->
    <section id="hikaye" class="home-story scroll-mt-28 space-y-8" aria-labelledby="hikaye-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">İzle</p>
        <h2 id="hikaye-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Bir tur böyle işler</h2>
        <p class="mt-3 text-stone-600">
          Dört arkadaşın katkısı tek bir hedefte buluşuyor. Paraları takip et, onayların toplanışını izle;
          ilk turun satıcıya ödemeyle tamamlanışına eşlik et.
        </p>
      </div>
      <div v-reveal><StorySim /></div>
    </section>

    <!-- 3 · BAŞLA (hesap hazırlama sihirbazı) -->
    <section id="basla" class="home-setup scroll-mt-28 space-y-8" aria-labelledby="basla-baslik">
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
                  <p v-if="poolAssetFromCircleFaucet" class="text-stone-600">
                    Deneme için gerçek olmayan test parası gerekir. Circle’ın sayfasında “Stellar Testnet”
                    ağını seç, adresini yapıştır.
                  </p>
                  <p v-else class="text-stone-600">
                    Deneme için gerçek olmayan test parası gerekir. Bu havuz varlığı ({{ token }}) bir faucet'ten değil,
                    anchor üzerinden yüklenir.
                  </p>
                </div>
              </div>
              <AnchorDemo v-if="!poolAssetFromCircleFaucet" compact @completed="refresh" />
              <div class="flex flex-wrap gap-3">
                <a v-if="poolAssetFromCircleFaucet" :href="circleFaucetUrl" target="_blank" rel="noopener noreferrer" class="btn-primary btn-lg">
                  Circle’ı aç <AppIcon name="external" class="!size-4" />
                </a>
                <button v-if="poolAssetFromCircleFaucet" type="button" class="btn-secondary btn-lg" @click="copyAddress">
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
    <section class="home-roles space-y-8" aria-labelledby="roller-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Roller</p>
        <h2 id="roller-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Havuzda kim ne yapar?</h2>
      </div>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <article v-for="(r, i) in ROLES" :key="r.title" v-reveal="i" v-tilt class="role-card bento h-full">
          <span class="role-number" aria-hidden="true">0{{ i + 1 }}</span>
          <span class="role-art grid size-16 place-items-center rounded-3xl" :class="r.tone">
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
    <section class="home-trust espresso relative overflow-hidden rounded-[2.25rem] px-5 py-12 text-white sm:px-10 sm:py-16" aria-labelledby="guven-baslik">
      <div class="trust-intro">
        <div v-reveal>
          <p class="eyebrow text-gold-300">Güven modeli</p>
          <h2 id="guven-baslik" class="mt-3 text-4xl font-extrabold sm:text-5xl">Kurallar kodda.<br /><span>Herkese açık.</span></h2>
          <p class="trust-description">Katkının nereye gittiğini gör. Sırayı, onayları ve ödeme koşullarını grubunla birlikte takip et.</p>
          <a href="#sss" class="trust-link">Kuralları ve sınırları incele <AppIcon name="arrow" /></a>
        </div>
        <div v-reveal><ContractFlow /></div>
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
        Kontrat Testnet’te yayında ve birim testlerinden geçti, ancak bağımsız güvenlik denetiminden geçmedi;
        gerçek para için kullanılmamalıdır.
      </p>
    </section>

    <!-- 7 · ANCHOR SİMÜLASYONU -->
    <div v-reveal><AnchorDemo /></div>

    <!-- 8 · SSS -->
    <section id="sss" class="home-faq scroll-mt-28" aria-labelledby="sss-baslik">
      <div v-reveal class="faq-heading">
        <p class="eyebrow text-brand-700">Aklındakiler</p>
        <h2 id="sss-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Sık sorulan sorular</h2>
        <p class="mt-3 text-sm text-stone-600">
          Cevaplar Testnet prototipini anlatır. Bağımsız denetimden geçmeden gerçek para için kullanılmamalıdır.
        </p>
      </div>
      <div class="faq-list">
        <details v-for="(item, i) in FAQ" :key="item.q" v-reveal="i % 2" class="faq-item group card cursor-pointer !p-0 open:shadow-[0_14px_32px_-16px_rgb(20_128_90/0.4)]">
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
    <section v-reveal class="home-closing" aria-labelledby="closing-title">
      <div><p class="eyebrow">BÜYÜK HEDEFLER, KÜÇÜK ADIMLAR</p><h2 id="closing-title">İlk adımı<br /><span>birlikte atalım.</span></h2><p>Grubunu düşün, planını oluştur.<br />Nasıl işlediğini Testnet’te keşfet.</p></div>
      <div class="closing-actions"><a href="#basla" class="closing-primary">Hesabını hazırla <AppIcon name="arrow" /></a><a href="#hesapla">Önce planımı hesaplayayım ↗</a><span>Gerçek para kullanılmaz.</span></div>
      <span class="closing-orbit" aria-hidden="true" /><span class="closing-star" aria-hidden="true">✳</span>
    </section>
  </div>
</template>

<style scoped>
.home-page :deep(.home-hero) { margin-bottom: 0; }
.home-contract-notice { margin-top: 24px; margin-bottom: 0; }
.home-intro-strip { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 20px; padding: 23px 12px; margin-top: 28px; border-bottom: 1px solid #dfe4d1; color: #788269; }
.home-intro-strip > span { display: inline-flex; align-items: center; gap: 8px; font-size: 11px; }
.home-intro-strip .intro-strip-label { font-size: 9px; letter-spacing: .13em; color: #9a9f88; }
.home-intro-strip :deep(svg) { width: 17px; height: 17px; color: #82996b; }
.home-page :deep(.eyebrow) { letter-spacing: .14em; font-size: 10px; }
.home-page :deep(h2) { color: #2e462f; letter-spacing: -.045em; line-height: 1.1; }
.home-story { position: relative; }
.home-story::before { content: ''; position: absolute; top: 80px; left: -30px; right: -30px; bottom: -30px; background: #f0f2e4; border-radius: 42px; z-index: -1; }
.home-setup { padding: 40px 0 0; }
.home-setup > .card { border-radius: 27px; border-color: #dde5d1; box-shadow: 0 16px 50px -28px #345c3a30; background: #fffff9; }
.role-card { position: relative; overflow: hidden; background: linear-gradient(140deg,#fffef6,#f3f5e8); border-color: #e0e5d5; padding: 28px 23px; border-radius: 24px; }
.role-number { position: absolute; top: 20px; right: 20px; color: #bbc4a9; font: 500 13px var(--font-display); }
.role-art { position: relative; margin-top: 12px; border: 1px solid #ffffffac; box-shadow: 0 6px 0 #cdd8bd80, 0 13px 22px -13px #4467443d; transform: rotate(-7deg); transition: transform .7s; }
.role-art :deep(img), .role-art :deep(svg) { transform: rotate(7deg) translateY(-4px); filter: drop-shadow(0 5px 3px #36592a1f); }
.role-card:hover .role-art { transform: rotate(0deg) translateY(-4px); }
.role-card h3 { margin-top: 27px; color: #365037; }
.role-card p { color: #7a826d; font-size: 12px; line-height: 1.85; }
.home-trust { background: radial-gradient(ellipse at 85% 0,#47603975,transparent 70%),#1c3c2b; }
.trust-intro { display: grid; grid-template-columns: .95fr 1.15fr; gap: 35px; align-items: center; }
.home-trust h2 { color: #f3f1d9; font-size: clamp(32px,3.8vw,46px); }
.home-trust h2 span { color: #b9c58b; }
.trust-description { margin-top: 19px; color: #bbc9a8; max-width: 340px; font-size: 13px; line-height: 1.85; }
.trust-link { display: inline-flex; align-items: center; gap: 8px; color: #e2d4a1; margin-top: 22px; font-size: 11px; min-height: 35px; }
.trust-link :deep(svg) { width: 15px; }
.home-trust article { background: #ffffff05; border-color: #ffffff14; border-radius: 20px; }
.home-trust article h3 { color: #edf0d9; font-size: 16px; }
.home-trust article p { color: #adbc9a; font-size: 12px; line-height: 1.85; }
.home-trust > p { color: #92a785; font-size: 10px; }
.home-faq { display: grid; grid-template-columns: .8fr 1.5fr; gap: 55px; align-items: start; }
.faq-heading { position: sticky; top: 125px; }
.home-faq h2 { font-size: 43px; }
.faq-heading > p:last-child { font-size: 12px; color: #828973; line-height: 1.9; max-width: 290px; }
.faq-list { display: grid; gap: 10px; }
.faq-item { background: #fffef7; border-color: #e0e5d4; border-radius: 17px; box-shadow: none; }
.faq-item[open] { background: #f1f5e7; border-color: #c5d7b3; }
.faq-item summary { font-size: 14px; min-height: 66px; color: #49603e; }
.faq-item > p { font-size: 12px; color: #7a836b; line-height: 1.9; }
.home-closing { position: relative; overflow: hidden; display: flex; align-items: center; justify-content: space-between; gap: 30px; padding: 48px 55px; border: 1px solid #dce5cb; border-radius: 32px; background: linear-gradient(110deg,#e9efda,#e0e8cb); isolation: isolate; }
.home-closing .eyebrow { color: #829363; }
.home-closing h2 { color: #3b5736; font-size: 48px; margin-top: 16px; }
.home-closing h2 span { color: #879b64; }
.home-closing p:last-child { color: #7d8b66; font-size: 12px; line-height: 1.8; margin-top: 20px; }
.closing-actions { display: flex; flex-direction: column; align-items: center; gap: 16px; z-index: 1; }
.closing-primary { display: flex; align-items: center; gap: 35px; padding: 16px 23px; border-radius: 15px; color: #fffef0; background: #365e3b; font-size: 13px; box-shadow: 0 5px 0 #21472c; transition: transform .3s; }
.closing-primary:hover { transform: translateY(-3px); }
.closing-primary :deep(svg) { width: 17px; }
.closing-actions > a:nth-child(2) { font-size: 11px; color: #698258; }
.closing-actions > span { font-size: 9px; color: #95a27b; }
.closing-orbit { position: absolute; width: 400px; height: 400px; right: -90px; top: -50px; border: 1px solid #b3c19550; border-radius: 50%; z-index: -1; }
.closing-orbit::after { content: ''; position: absolute; inset: 35px; border: 1px dashed #b3c19555; border-radius: 50%; }
.closing-star { position: absolute; top: 38px; right: 330px; font-size: 95px; line-height: 1; color: #acbd8452; transform: rotate(-15deg); z-index: -1; }
@media(max-width:1200px) { .home-story::before { left: -10px; right: -10px; } }
@media(max-width:1023px) { .home-faq { gap: 30px; grid-template-columns: .8fr 1.3fr; } .trust-intro { gap: 25px; grid-template-columns: 1fr; } .trust-description { max-width: 480px; } }
@media(max-width:767px) { .home-intro-strip { justify-content: center; gap: 15px 20px; padding-inline: 0; } .home-intro-strip .intro-strip-label { flex-basis: 100%; justify-content: center; } .home-intro-strip > span { font-size: 10px; } .home-page :deep(h2) { font-size: 34px; } .home-story::before { inset: 90px -10px -20px; border-radius: 28px; } .home-faq { grid-template-columns: 1fr; gap: 28px; } .faq-heading { position: static; text-align: center; } .faq-heading > p:last-child { margin-inline: auto; max-width: 330px; } .home-closing { padding: 32px 25px; flex-direction: column; align-items: flex-start; gap: 27px; border-radius: 25px; } .home-closing h2 { font-size: 41px; } .closing-actions { align-items: flex-start; } .closing-star { right: 20px; top: 40px; } .role-card { padding: 23px 20px; } }
@media(prefers-reduced-motion:reduce) { .role-art, .closing-primary { transition: none; } .role-card:hover .role-art { transform: rotate(-7deg); } .closing-primary:hover { transform: none; } }
</style>
