<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatDecimalString, shortAddress } from '@/lib/format'
import {
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
  const connected = wallet.isConnected
  const active = account.value?.exists === true
  const trusted = account.value?.hasTrustline === true
  return [
    { label: 'Cüzdanını bağla', done: connected },
    { label: `${config.label} hesabını etkinleştir`, done: active },
    { label: `${poolAsset.getCode()} kabul etmeyi aç (trustline)`, done: trusted },
  ]
})

const ready = computed(() => steps.value.every((s) => s.done))
</script>

<template>
  <div class="space-y-8">
    <section class="space-y-4 py-4 text-center sm:py-8">
      <h1 class="text-3xl font-bold tracking-tight text-slate-900 sm:text-5xl">
        Birlikte biriktir.<br />
        <span class="text-indigo-700">Her şeyi doğrula.</span>
      </h1>
      <p class="mx-auto max-w-2xl text-base text-slate-600 sm:text-lg">
        Ev, araç veya başka büyük hedefler için birlikte düzenli birikim yapın. Paranız bir şirketin
        hesabında değil, herkesin doğrulayabildiği bir akıllı sözleşmede durur.
      </p>
      <div class="flex flex-wrap items-center justify-center gap-3 pt-2">
        <RouterLink to="/create" class="btn-primary">Havuz oluştur</RouterLink>
        <a href="#havuz-ara" class="btn-secondary">Havuza git</a>
      </div>
    </section>

    <section class="grid gap-4 sm:grid-cols-3" aria-label="Neden güvenli">
      <div class="card">
        <h2 class="font-semibold">Para sözleşmede</h2>
        <p class="mt-1 text-sm text-slate-600">
          Yönetici para çekemez. Transferi yalnızca sözleşmenin kuralları belirler.
        </p>
      </div>
      <div class="card">
        <h2 class="font-semibold">Ödeme satıcıya</h2>
        <p class="mt-1 text-sm text-slate-600">
          Sıra ve üyeler kilitlidir. Tur tutarı, doğrulayıcıların onayladığı satıcıya gider.
        </p>
      </div>
      <div class="card">
        <h2 class="font-semibold">Sponsor güvencesi</h2>
        <p class="mt-1 text-sm text-slate-600">
          Başlamadan önce güvence kilitlenir. Eksik ödemede tur durur, gerekirse bekleyenlere iade edilir.
        </p>
      </div>
    </section>

    <section class="card" aria-labelledby="hesap-baslik">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 id="hesap-baslik" class="text-lg font-semibold">Hesabın</h2>
          <p class="text-sm text-slate-500">
            Başlamak için üç adımı tamamla. Hepsi {{ config.label }} üzerinde, gerçek para yok.
          </p>
        </div>
        <span v-if="ready" class="badge bg-emerald-100 text-emerald-800">Hazırsın</span>
      </div>

      <ol class="mt-4 space-y-2">
        <li v-for="(step, i) in steps" :key="step.label" class="flex items-center gap-3 text-sm">
          <span
            class="flex size-6 shrink-0 items-center justify-center rounded-full text-xs font-bold"
            :class="step.done ? 'bg-emerald-600 text-white' : 'bg-slate-200 text-slate-600'"
            aria-hidden="true"
          >
            {{ step.done ? '✓' : i + 1 }}
          </span>
          <span :class="step.done ? 'text-slate-500 line-through' : 'font-medium'">
            {{ step.label }}
          </span>
        </li>
      </ol>

      <div v-if="!wallet.isConnected" class="mt-5">
        <button
          type="button"
          class="btn-primary"
          :disabled="wallet.busy"
          @click="wallet.connect()"
        >
          {{ wallet.busy ? 'Bağlanıyor…' : 'Cüzdan bağla' }}
        </button>
        <p v-if="wallet.error" role="alert" class="mt-3 text-sm text-rose-700">
          {{ wallet.error }}
        </p>
        <p class="mt-3 text-xs text-slate-500">
          Freighter, xBull, Albedo, LOBSTR, Hana veya Rabet kullanabilirsin.
        </p>
      </div>

      <div v-else class="mt-5 space-y-4">
        <p v-if="loading && !account" class="text-sm text-slate-500">Hesap okunuyor…</p>

        <template v-if="account">
          <dl class="grid gap-3 sm:grid-cols-3">
            <div class="rounded-xl bg-slate-50 p-3">
              <dt class="text-xs text-slate-500">Adres</dt>
              <dd class="mt-0.5 font-mono text-sm">
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
            <div class="rounded-xl bg-slate-50 p-3">
              <dt class="text-xs text-slate-500">XLM (işlem ücreti)</dt>
              <dd class="mt-0.5 text-sm font-semibold">
                {{ account.exists ? formatDecimalString(account.xlm) : '—' }}
              </dd>
            </div>
            <div class="rounded-xl bg-slate-50 p-3">
              <dt class="text-xs text-slate-500">{{ poolAsset.getCode() }}</dt>
              <dd class="mt-0.5 text-sm font-semibold">
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
              :href="config.circleFaucetUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="btn-secondary"
            >
              Testnet {{ poolAsset.getCode() }} al (Circle)
            </a>
            <button type="button" class="btn-secondary" :disabled="loading" @click="refresh">
              Yenile
            </button>
          </div>
          <p v-if="account.hasTrustline" class="text-xs text-slate-500">
            Circle sayfasında adresini ve “Stellar Testnet” ağını seç. Adresin panoya kopyalanır:
            <button
              type="button"
              class="font-mono text-indigo-700 underline"
              @click="copyAddress"
            >
              kopyala
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

    <section id="havuz-ara" class="card scroll-mt-20" aria-labelledby="ara-baslik">
      <h2 id="ara-baslik" class="text-lg font-semibold">Havuza git</h2>
      <p class="text-sm text-slate-500">Sana verilen havuz numarasını gir.</p>
      <form class="mt-4 flex gap-3" @submit.prevent="openPool">
        <label class="sr-only" for="pool-id">Havuz numarası</label>
        <input
          id="pool-id"
          v-model="poolIdInput"
          class="input max-w-48"
          type="number"
          min="0"
          step="1"
          inputmode="numeric"
          placeholder="Havuz no"
          required
        />
        <button type="submit" class="btn-primary">Aç</button>
      </form>
      <p v-if="!poolContractId" class="mt-3 text-sm text-amber-800">
        Havuz sözleşmesi henüz yapılandırılmadı, havuz işlemleri sözleşme yayınlanınca açılacak.
      </p>
    </section>
  </div>
</template>
