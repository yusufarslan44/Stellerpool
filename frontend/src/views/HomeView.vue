<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import AnchorDemo from '@/components/AnchorDemo.vue'
import PoolCalculator from '@/components/PoolCalculator.vue'
import { useNetworkStatus } from '@/composables/useNetworkStatus'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatDecimalString, shortAddress } from '@/lib/format'
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

const wallet = useWalletStore()
const router = useRouter()
const { ledger, online } = useNetworkStatus()

const account = ref<AccountInfo | null>(null)
const loading = ref(false)
const busy = ref<'fund' | 'trust' | null>(null)
const error = ref<string | null>(null)
const lastTx = ref<string | null>(null)
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
  if (wallet.address) await navigator.clipboard?.writeText(wallet.address)
}

function openPool() {
  const id = Number.parseInt(poolIdInput.value, 10)
  if (Number.isInteger(id) && id >= 0) router.push(`/pool/${id}`)
}

const steps = computed(() => {
  const active = account.value?.exists === true
  const trusted = account.value?.hasTrustline === true
  return [
    { label: 'Cüzdanını bağla', done: wallet.isConnected },
    { label: `${config.label} hesabını etkinleştir`, done: active },
    { label: `${poolAsset.getCode()} kabul etmeyi aç`, done: trusted },
  ]
})
const ready = computed(() => steps.value.every((s) => s.done))

const HOW = [
  { icon: 'users', title: 'Havuzu kur', text: 'Katkıyı, üye sayısını ve son ödeme süresini belirle.' },
  { icon: 'shield', title: 'Güvence kilitlenir', text: 'Sponsor güvencesini yatırır, üyeler cüzdanlarıyla katılır.' },
  { icon: 'lock', title: 'Sıra kilitlenir', text: 'Havuz başlayınca üyeler ve sıra kimse tarafından değiştirilemez.' },
  { icon: 'wallet', title: 'Herkes öder', text: 'Her tur aynı katkı sözleşmeye yatırılır, para orada durur.' },
  { icon: 'receipt', title: 'Satıcıya ödenir', text: 'Doğrulayıcılar onaylar, tur tutarı doğrudan satıcıya gider.' },
] as const

const FAQ = [
  {
    q: 'Organizatör parayı alıp kaçabilir mi?',
    a: 'Hedef kontratta kurucunun serbest çekim yetkisi olmayacak. Kontrat henüz yazılıp yayınlanmadı; bu koruma şu anda çalışır durumda değil.',
  },
  {
    q: 'Sıram sonradan değiştirilebilir mi?',
    a: 'Hedef kurala göre havuz başladıktan sonra üyeler ve sıra kilitlenecek. Bu kuralın kontrat ve testlerle doğrulanması gerekiyor.',
  },
  {
    q: 'Biri ödemeyi bırakırsa ne olur?',
    a: 'Planlanan Testnet senaryosunda ek süre, sponsor tamamlama ve iptal/iade var. Bu akış henüz çalışan kontrat özelliği veya gerçek para garantisi değildir.',
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
] as const
</script>

<template>
  <div class="space-y-6">
    <!-- HERO: koyu aurora zemin, solda mesaj, sağda hesaplama aracı -->
    <section class="aurora relative isolate overflow-hidden rounded-[2rem] px-5 py-10 sm:px-10 sm:py-14">
      <div
        class="drift pointer-events-none absolute -top-24 -right-16 -z-10 size-96 rounded-full bg-indigo-500/25 blur-3xl"
        aria-hidden="true"
      />
      <div class="grid items-center gap-8 lg:grid-cols-[1.05fr_1fr]">
        <div class="text-white">
          <div class="glass-dark inline-flex items-center gap-2 rounded-full px-3.5 py-1.5 text-xs font-medium">
            <span
              class="size-2 rounded-full"
              :class="online === false ? 'bg-rose-400' : 'bg-emerald-400'"
              aria-hidden="true"
            />
            <span v-if="online === false">Ağa ulaşılamıyor</span>
            <span v-else-if="ledger !== null" class="tabular-nums">
              Stellar {{ config.label }} canlı · defter #{{ ledger.toLocaleString('tr-TR') }}
            </span>
            <span v-else>Ağa bağlanılıyor…</span>
          </div>

          <h1 class="mt-5 text-4xl leading-[1.08] font-bold tracking-tight sm:text-5xl lg:text-6xl">
            Birlikte biriktir.
            <span class="block bg-gradient-to-r from-indigo-300 via-violet-300 to-emerald-300 bg-clip-text text-transparent">
              Her şeyi doğrula.
            </span>
          </h1>
          <p class="mt-5 max-w-xl text-base leading-relaxed text-slate-300 sm:text-lg">
            Davetli altın günü grupları için ortak kurallı birikim fikri. Soroban havuzu tamamlandığında
            test varlığının hareketi herkesin görebildiği kurallara bağlanacak.
          </p>

          <div class="mt-7 flex flex-wrap gap-3">
            <RouterLink to="/create" class="btn-light">
              Havuz oluştur
              <AppIcon name="arrow" class="!size-4" />
            </RouterLink>
            <a href="#nasil" class="btn-ghost-dark">Nasıl çalışır?</a>
          </div>

          <ul class="mt-8 flex flex-wrap gap-x-6 gap-y-2 text-sm text-slate-300">
            <li class="flex items-center gap-1.5">
              <AppIcon name="check" class="!size-4 text-emerald-400" /> Hedef: yöneticiye serbest çekim yok
            </li>
            <li class="flex items-center gap-1.5">
              <AppIcon name="check" class="!size-4 text-emerald-400" /> Hedef: sabit sıra
            </li>
            <li class="flex items-center gap-1.5">
              <AppIcon name="check" class="!size-4 text-emerald-400" /> Hedef: açık işlem kaydı
            </li>
          </ul>
        </div>

        <PoolCalculator />
      </div>
    </section>

    <p v-if="!poolContractId" role="status" class="rounded-2xl border border-amber-200 bg-amber-50 p-5 text-sm leading-relaxed text-amber-950">
      Bu ekrandaki havuz, sponsor ve ödeme akışı hedef tasarımı anlatır. Soroban havuz kontratı henüz
      yayınlanmadı; şu anda test varlığıyla havuz açılamaz veya tahsisat yapılamaz.
    </p>

    <!-- BENTO: neden güvenli -->
    <section aria-labelledby="guvenli" class="space-y-4">
      <div class="px-1">
        <p class="eyebrow text-indigo-700">Güven modeli</p>
        <h2 id="guvenli" class="mt-1 text-2xl font-bold tracking-tight sm:text-3xl">
          Testnet için hedeflenen güven modeli
        </h2>
      </div>

      <div class="grid gap-4 md:grid-cols-4">
        <article class="bento bg-gradient-to-br from-indigo-600 to-violet-700 !border-transparent text-white md:col-span-2 md:row-span-2">
          <div class="grid size-11 place-items-center rounded-2xl bg-white/15">
            <AppIcon name="lock" />
          </div>
          <h3 class="mt-4 text-xl font-semibold">Hedef: varlık sözleşmede</h3>
          <p class="mt-2 max-w-md text-sm leading-relaxed text-indigo-100">
            Planlanan Testnet akışında katkılar Soroban kontratına gider ve kurucuya serbest çekim
            yetkisi verilmez. Kontrat henüz yayınlanmadığı için bu mekanizma kullanımda değildir.
          </p>
          <div class="mt-6 flex flex-wrap items-center gap-2 text-xs font-medium" aria-label="Para akışı">
            <span class="glass-dark rounded-full px-3 py-1.5">Üyeler</span>
            <AppIcon name="arrow" class="!size-4 text-indigo-200" />
            <span class="rounded-full bg-white px-3 py-1.5 text-indigo-800">Sözleşme</span>
            <AppIcon name="arrow" class="!size-4 text-indigo-200" />
            <span class="glass-dark rounded-full px-3 py-1.5">Doğrulanmış satıcı</span>
          </div>
        </article>

        <article class="bento">
          <div class="grid size-10 place-items-center rounded-xl bg-indigo-50 text-indigo-700">
            <AppIcon name="users" />
          </div>
          <h3 class="mt-3 font-semibold">Sabit sıra hedefi</h3>
          <p class="mt-1 text-sm leading-relaxed text-slate-600">
            Planda havuz başlayınca üyeler ve sıra değişmez.
          </p>
        </article>

        <article class="bento">
          <div class="grid size-10 place-items-center rounded-xl bg-emerald-50 text-emerald-700">
            <AppIcon name="shield" />
          </div>
          <h3 class="mt-3 font-semibold">Demo sponsor güvencesi</h3>
          <p class="mt-1 text-sm leading-relaxed text-slate-600">
            Yalnızca Testnet senaryosunda modellenir; gerçek teslim garantisi değildir.
          </p>
        </article>

        <article class="bento">
          <div class="grid size-10 place-items-center rounded-xl bg-violet-50 text-violet-700">
            <AppIcon name="receipt" />
          </div>
          <h3 class="mt-3 font-semibold">Demo satıcı ödemesi</h3>
          <p class="mt-1 text-sm leading-relaxed text-slate-600">
            Planlanan test ödemesi doğrulayıcı onayına bağlıdır.
          </p>
        </article>

        <article class="bento">
          <div class="grid size-10 place-items-center rounded-xl bg-sky-50 text-sky-700">
            <AppIcon name="eye" />
          </div>
          <h3 class="mt-3 font-semibold">Herkes doğrular</h3>
          <p class="mt-1 text-sm leading-relaxed text-slate-600">
            Zincire yazılacak demo işlemler Stellar Expert'te görülebilir.
          </p>
        </article>
      </div>
    </section>

    <!-- NASIL ÇALIŞIR -->
    <section id="nasil" class="scroll-mt-24 space-y-4" aria-labelledby="nasil-baslik">
      <div class="px-1">
        <p class="eyebrow text-indigo-700">Adım adım</p>
        <h2 id="nasil-baslik" class="mt-1 text-2xl font-bold tracking-tight sm:text-3xl">
          Planlanan akış nasıl çalışır?
        </h2>
      </div>
      <ol class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        <li v-for="(s, i) in HOW" :key="s.title" class="bento relative">
          <span class="absolute top-4 right-5 text-3xl font-bold text-slate-200 tabular-nums" aria-hidden="true">
            {{ i + 1 }}
          </span>
          <div class="grid size-10 place-items-center rounded-xl bg-indigo-50 text-indigo-700">
            <AppIcon :name="s.icon" />
          </div>
          <h3 class="mt-3 font-semibold">{{ s.title }}</h3>
          <p class="mt-1 text-sm leading-relaxed text-slate-600">{{ s.text }}</p>
        </li>
      </ol>
    </section>

    <AnchorDemo />

    <!-- HESABIN: başlangıç adımları -->
    <section id="basla" class="card scroll-mt-24 bg-gradient-to-br from-white to-indigo-50/60" aria-labelledby="hesap-baslik">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p class="eyebrow text-indigo-700">Başla</p>
          <h2 id="hesap-baslik" class="mt-1 text-2xl font-bold tracking-tight">Hesabını hazırla</h2>
          <p class="mt-1 text-sm text-slate-600">
            Üç adım. Hepsi {{ config.label }} üzerinde, gerçek para yok.
          </p>
        </div>
        <span v-if="ready" class="badge bg-emerald-100 text-emerald-800">Hazırsın ✓</span>
      </div>

      <ol class="mt-5 grid gap-3 sm:grid-cols-3">
        <li
          v-for="(step, i) in steps"
          :key="step.label"
          class="flex items-center gap-3 rounded-2xl border p-3 text-sm"
          :class="step.done ? 'border-emerald-200 bg-emerald-50' : 'border-slate-200 bg-white'"
        >
          <span
            class="grid size-7 shrink-0 place-items-center rounded-full text-xs font-bold"
            :class="step.done ? 'bg-emerald-600 text-white' : 'bg-slate-200 text-slate-700'"
          >
            <AppIcon v-if="step.done" name="check" class="!size-4" />
            <template v-else>{{ i + 1 }}</template>
          </span>
          <span :class="step.done ? 'text-emerald-900' : 'font-medium'">{{ step.label }}</span>
        </li>
      </ol>

      <div v-if="!wallet.isConnected" class="mt-5">
        <button type="button" class="btn-primary" :disabled="wallet.busy" @click="wallet.connect()">
          <AppIcon name="wallet" class="!size-4" />
          {{ wallet.busy ? 'Bağlanıyor…' : 'Cüzdan bağla' }}
        </button>
        <p v-if="wallet.error" role="alert" class="mt-3 text-sm text-rose-700">{{ wallet.error }}</p>
        <p class="mt-3 text-xs text-slate-600">
          Freighter, xBull, Albedo, LOBSTR, Hana veya Rabet kullanabilirsin. Uzantı yüklemek
          istemezsen Albedo web üzerinden çalışır.
        </p>
      </div>

      <div v-else class="mt-5 space-y-4">
        <p v-if="loading && !account" class="text-sm text-slate-600">Hesap okunuyor…</p>

        <template v-if="account">
          <dl class="grid gap-3 sm:grid-cols-3">
            <div class="rounded-2xl bg-white p-4 ring-1 ring-slate-200">
              <dt class="text-xs text-slate-600">Adres</dt>
              <dd class="mt-1 font-mono text-sm">
                <a
                  v-if="wallet.address"
                  :href="explorerAccount(wallet.address)"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-indigo-700 underline"
                >
                  {{ shortAddress(wallet.address, 6) }}
                </a>
              </dd>
            </div>
            <div class="rounded-2xl bg-white p-4 ring-1 ring-slate-200">
              <dt class="text-xs text-slate-600">XLM (işlem ücreti)</dt>
              <dd class="mt-1 text-lg font-bold tabular-nums">
                {{ account.exists ? formatDecimalString(account.xlm) : '—' }}
              </dd>
            </div>
            <div class="rounded-2xl bg-white p-4 ring-1 ring-slate-200">
              <dt class="text-xs text-slate-600">{{ poolAsset.getCode() }}</dt>
              <dd class="mt-1 text-lg font-bold tabular-nums">
                {{ account.asset !== null ? formatDecimalString(account.asset) : '—' }}
              </dd>
            </div>
          </dl>

          <div class="flex flex-wrap gap-3">
            <button
              v-if="!account.exists"
              type="button"
              class="btn-primary"
              :disabled="busy !== null"
              @click="fund"
            >
              {{ busy === 'fund' ? 'Etkinleştiriliyor…' : `${config.label} hesabını etkinleştir` }}
            </button>
            <button
              v-else-if="!account.hasTrustline"
              type="button"
              class="btn-primary"
              :disabled="busy !== null"
              @click="trust"
            >
              {{ busy === 'trust' ? 'Cüzdanı onayla…' : `${poolAsset.getCode()} kabul etmeyi aç` }}
            </button>
            <a
              v-if="account.hasTrustline"
              :href="circleFaucetUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="btn-secondary"
            >
              Testnet {{ poolAsset.getCode() }} al (Circle)
            </a>
            <button type="button" class="btn-secondary" :disabled="loading" @click="refresh">Yenile</button>
          </div>
          <p v-if="account.hasTrustline" class="text-xs text-slate-600">
            Circle sayfasında “Stellar Testnet” ağını seçip adresini yapıştır:
            <button type="button" class="font-medium text-indigo-700 underline" @click="copyAddress">
              adresi kopyala
            </button>
          </p>
        </template>

        <p v-if="lastTx" class="text-sm text-emerald-800">
          İşlem gönderildi:
          <a
            :href="explorerTx(lastTx)"
            target="_blank"
            rel="noopener noreferrer"
            class="font-mono underline"
          >
            {{ lastTx.slice(0, 8) }}…
          </a>
        </p>
        <p v-if="error" role="alert" class="text-sm text-rose-700">{{ error }}</p>
      </div>
    </section>

    <!-- SIK SORULANLAR -->
    <section id="sss" class="scroll-mt-24 space-y-4" aria-labelledby="sss-baslik">
      <div class="px-1">
        <p class="eyebrow text-indigo-700">Aklındakiler</p>
        <h2 id="sss-baslik" class="mt-1 text-2xl font-bold tracking-tight sm:text-3xl">
          Sık sorulan sorular
        </h2>
        <p class="mt-1 text-sm text-slate-600">
          Cevaplar hedef tasarımı anlatır. Sözleşme yayınlanıp denetlenmeden gerçek para için
          kullanılmamalıdır.
        </p>
      </div>
      <div class="grid gap-3 md:grid-cols-2">
        <details v-for="item in FAQ" :key="item.q" class="group card cursor-pointer !p-0">
          <summary
            class="flex min-h-14 list-none items-center justify-between gap-3 px-5 py-3 font-semibold marker:hidden [&::-webkit-details-marker]:hidden"
          >
            {{ item.q }}
            <AppIcon
              name="chevron"
              class="text-slate-500 transition-transform duration-200 group-open:rotate-180"
            />
          </summary>
          <p class="px-5 pb-5 text-sm leading-relaxed text-slate-600">{{ item.a }}</p>
        </details>
      </div>
    </section>

    <!-- HAVUZA GİT -->
    <section class="card flex flex-wrap items-center justify-between gap-4" aria-labelledby="ara-baslik">
      <div>
        <h2 id="ara-baslik" class="text-lg font-semibold">Havuz numaran var mı?</h2>
        <p class="text-sm text-slate-600">Sana verilen numarayı girip havuza git.</p>
        <p v-if="!poolContractId" class="mt-2 text-sm text-amber-800">
          Havuz sözleşmesi henüz yapılandırılmadı, havuz işlemleri sözleşme yayınlanınca açılacak.
        </p>
      </div>
      <form class="flex gap-3" @submit.prevent="openPool">
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
      </form>
    </section>
  </div>
</template>
