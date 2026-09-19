<script setup lang="ts">
import { StrKey } from '@stellar/stellar-sdk'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { useNow } from '@/composables/useNow'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatDuration, formatStroops, shortAddress } from '@/lib/format'
import { sha256 } from '@/lib/hash'
import { explorerContract, explorerTx, poolAsset, poolContractId } from '@/lib/stellar'
import { getTokenBalance } from '@/services/account'
import {
  abortPool,
  approvePurchase,
  claimRefund,
  claimSponsorRemainder,
  deposit,
  executeRound,
  fundGuarantee,
  getMemberStatus,
  getPool,
  getRound,
  joinPool,
  markOverdue,
  proposePurchase,
  startPool,
  topUp,
} from '@/services/pool'
import type { Signer } from '@/services/pool'
import { useWalletStore } from '@/stores/wallet'
import type { MemberStatus, PoolInfo, RoundInfo, TxResult } from '@/types/pool'

const route = useRoute()
const wallet = useWalletStore()
const now = useNow()
const token = poolAsset.getCode()

const poolId = computed(() => Number.parseInt(String(route.params.id), 10))

const pool = ref<PoolInfo | null>(null)
const round = ref<RoundInfo | null>(null)
const members = ref<MemberStatus[]>([])
const contractBalance = ref<bigint | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)
const actionBusy = ref<string | null>(null)
const actionError = ref<string | null>(null)
const lastTx = ref<string | null>(null)
/** Havuz başlamadan önce kurucunun belirleyeceği tahsisat sırası. */
const order = ref<string[]>([])
const sellerInput = ref('')
const docInput = ref('')

async function load(silent = false) {
  if (!poolContractId) {
    loading.value = false
    return
  }
  if (!silent) loading.value = true
  loadError.value = null
  try {
    const p = await getPool(poolId.value)
    pool.value = p

    if (p.status === 'Filling') {
      // Başlamadan önce sıra = katılım sırası; kurucu değiştirebilir.
      if (order.value.length !== p.members.length || !order.value.every((m) => p.members.includes(m))) {
        order.value = [...p.members]
      }
      round.value = null
    } else {
      round.value = await getRound(p.id, p.currentRound).catch(() => null)
    }

    const [statuses, balance] = await Promise.all([
      Promise.all(
        p.members.map((m) =>
          getMemberStatus(p.id, m).catch(
            (): MemberStatus => ({ address: m, contributed: 0n, refundable: 0n, received: false }),
          ),
        ),
      ),
      getTokenBalance(poolContractId).catch(() => null),
    ])
    members.value = statuses
    contractBalance.value = balance
  } catch (e) {
    loadError.value = errorMessage(e)
  } finally {
    loading.value = false
  }
}

let poll: ReturnType<typeof setInterval> | undefined
onMounted(() => {
  void load()
  poll = setInterval(() => void load(true), 10_000)
})
onUnmounted(() => clearInterval(poll))
watch(poolId, () => void load())

// --- Roller ------------------------------------------------------------------------------
const me = computed(() => wallet.address)
const isMember = computed(() => !!me.value && !!pool.value?.members.includes(me.value))
const isCreator = computed(() => !!me.value && pool.value?.creator === me.value)
const isSponsor = computed(() => !!me.value && pool.value?.sponsor === me.value)
const isVerifier = computed(() => !!me.value && !!pool.value?.verifiers.includes(me.value))
const isRecipient = computed(() => !!me.value && round.value?.recipient === me.value)
const isFull = computed(() => !!pool.value && pool.value.members.length >= pool.value.memberLimit)

// --- Güvence -----------------------------------------------------------------------------
const guaranteeMissing = computed(() => {
  if (!pool.value) return 0n
  const gap = pool.value.requiredGuarantee - pool.value.guaranteeDeposited
  return gap > 0n ? gap : 0n
})
const guaranteeOk = computed(() => !!pool.value && guaranteeMissing.value === 0n)
const guaranteePercent = computed(() => {
  const p = pool.value
  if (!p || p.requiredGuarantee === 0n) return 0
  return Math.min(100, Number((p.guaranteeDeposited * 100n) / p.requiredGuarantee))
})

// --- Tur durumu --------------------------------------------------------------------------
const paid = computed(() => new Set(round.value?.paid ?? []))
const covered = computed(() => new Set(round.value?.sponsorCovered ?? []))
const isFunded = (m: string) => paid.value.has(m) || covered.value.has(m)
const missingMembers = computed(() => (pool.value?.members ?? []).filter((m) => !isFunded(m)))
const allFunded = computed(() => !!pool.value && pool.value.members.length > 0 && missingMembers.value.length === 0)
const remaining = computed(() => (round.value ? round.value.deadline - now.value : 0))
const deadlinePassed = computed(() => !!round.value && remaining.value <= 0)
const myFunded = computed(() => !!me.value && isFunded(me.value))

const approvalsCount = computed(() => round.value?.approvals.length ?? 0)
const approvedByMe = computed(() => !!me.value && !!round.value?.approvals.includes(me.value))
const approvalsOk = computed(
  () => !!pool.value && approvalsCount.value >= pool.value.approvalThreshold,
)
const canExecute = computed(() => allFunded.value && !!round.value?.seller && approvalsOk.value)

const canAbort = computed(
  () =>
    pool.value?.status === 'Paused' &&
    !!round.value &&
    (round.value.abortAfter === 0 || now.value >= round.value.abortAfter),
)
const abortRemaining = computed(() => (round.value ? round.value.abortAfter - now.value : 0))

const memberByAddress = computed(() => new Map(members.value.map((m) => [m.address, m])))
const totalRefundable = computed(() => members.value.reduce((sum, m) => sum + m.refundable, 0n))
const myRefundable = computed(() => (me.value ? (memberByAddress.value.get(me.value)?.refundable ?? 0n) : 0n))

function memberState(address: string) {
  const p = pool.value
  if (!p || p.status === 'Filling') return { label: 'Katıldı', tone: 'slate' as const }
  if (p.status === 'Completed' || p.status === 'Aborted') return { label: '—', tone: 'slate' as const }
  if (paid.value.has(address)) return { label: 'Ödedi ✓', tone: 'green' as const }
  if (covered.value.has(address)) return { label: 'Sponsor tamamladı ⚠', tone: 'amber' as const }
  return { label: 'Bekliyor', tone: 'slate' as const }
}

const toneClass = {
  green: 'bg-emerald-100 text-emerald-800',
  amber: 'bg-amber-100 text-amber-900',
  slate: 'bg-slate-100 text-slate-700',
}

const statusLabel = computed(() => {
  switch (pool.value?.status) {
    case 'Filling':
      return { text: 'Üye ve güvence bekleniyor', cls: 'bg-slate-100 text-slate-700' }
    case 'Active':
      return { text: 'Devam ediyor', cls: 'bg-indigo-100 text-indigo-800' }
    case 'Paused':
      return { text: 'Durduruldu: eksik katkı', cls: 'bg-amber-100 text-amber-900' }
    case 'Completed':
      return { text: 'Tamamlandı', cls: 'bg-emerald-100 text-emerald-800' }
    case 'Aborted':
      return { text: 'İptal edildi', cls: 'bg-rose-100 text-rose-800' }
    default:
      return { text: '', cls: '' }
  }
})

// --- İşlemler ----------------------------------------------------------------------------
const signer = computed<Signer | null>(() =>
  me.value ? { address: me.value, signTransaction: wallet.signTransaction } : null,
)

async function run(name: string, fn: (s: Signer) => Promise<TxResult>) {
  if (!signer.value) return
  actionBusy.value = name
  actionError.value = null
  try {
    const result = await fn(signer.value)
    lastTx.value = result.hash
    await load(true)
  } catch (e) {
    if (!isUserRejection(e)) actionError.value = errorMessage(e)
  } finally {
    actionBusy.value = null
  }
}

const sellerValid = computed(() => StrKey.isValidEd25519PublicKey(sellerInput.value.trim()))

async function propose() {
  const seller = sellerInput.value.trim()
  const doc = docInput.value.trim()
  if (!pool.value || !sellerValid.value || !doc) return
  const hash = await sha256(doc)
  await run('propose', (s) => proposePurchase(s, pool.value!.id, seller, hash))
}

function move(index: number, delta: -1 | 1) {
  const target = index + delta
  if (target < 0 || target >= order.value.length) return
  const next = [...order.value]
  const [item] = next.splice(index, 1)
  next.splice(target, 0, item!)
  order.value = next
}

const copied = ref(false)
async function copyLink() {
  await navigator.clipboard?.writeText(window.location.href)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}

const listedMembers = computed(() =>
  pool.value?.status === 'Filling' && isCreator.value ? order.value : (pool.value?.members ?? []),
)
</script>

<template>
  <div class="space-y-6">
    <RouterLink to="/" class="text-sm text-indigo-700 underline">← Ana sayfa</RouterLink>

    <p
      v-if="!poolContractId"
      role="status"
      class="rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900"
    >
      Havuz sözleşmesi henüz yapılandırılmadı, bu yüzden havuz verisi okunamıyor. Sözleşme
      yayınlanıp adresi <code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> olarak
      eklenince bu sayfa gerçek veriyi gösterecek.
    </p>

    <p v-else-if="loading" class="text-slate-500">Havuz zincirden okunuyor…</p>

    <div v-else-if="loadError" role="alert" class="card space-y-3">
      <h1 class="text-lg font-semibold">Havuz #{{ poolId }} okunamadı</h1>
      <p class="text-sm text-rose-700">{{ loadError }}</p>
      <button type="button" class="btn-secondary" @click="load()">Tekrar dene</button>
    </div>

    <template v-else-if="pool">
      <header class="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 class="text-2xl font-bold tracking-tight">Tasarruf havuzu #{{ pool.id }}</h1>
          <p class="mt-1 text-sm text-slate-600">
            {{ pool.members.length }} / {{ pool.memberLimit }} üye · her tur
            {{ formatStroops(pool.contributionAmount) }} {{ token }} ·
            <span class="font-mono" :title="pool.sponsor">sponsor {{ shortAddress(pool.sponsor) }}</span>
            <span v-if="pool.sponsor === me" class="badge bg-indigo-100 text-indigo-800">Sen</span>
          </p>
        </div>
        <div class="flex items-center gap-2">
          <span class="badge" :class="statusLabel.cls">{{ statusLabel.text }}</span>
          <button type="button" class="btn-secondary !min-h-9 !px-3 !py-1.5" @click="copyLink">
            {{ copied ? 'Kopyalandı' : 'Bağlantıyı kopyala' }}
          </button>
        </div>
      </header>

      <p
        v-if="pool.status === 'Paused'"
        role="status"
        class="rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900"
      >
        Tur durdu: bazı üyelerin katkısı eksik. Sponsor eksik katkıyı ilave fonla tamamlayabilir.
        <template v-if="canAbort">Bekleme süresi doldu, iptal tetiklenebilir.</template>
        <template v-else-if="round && round.abortAfter > 0">
          İptal {{ formatDuration(abortRemaining) }} sonra tetiklenebilir.
        </template>
      </p>
      <p
        v-if="pool.status === 'Aborted'"
        role="status"
        class="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-900"
      >
        Havuz iptal edildi. Üyeler hak ettikleri iadeyi kontrattan geri alabilir.
      </p>

      <!-- Finansal özet: para nerede? -->
      <section class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4" aria-label="Finansal özet">
        <div class="card">
          <p class="text-xs text-slate-500">Kontrattaki toplam {{ token }}</p>
          <p class="mt-1 text-xl font-bold">
            {{ contractBalance === null ? '—' : formatStroops(contractBalance) }}
          </p>
          <p class="mt-1 text-xs text-slate-500">Doğrudan zincirden okunur</p>
        </div>
        <div class="card">
          <p class="text-xs text-slate-500">Sponsor güvencesi</p>
          <p class="mt-1 text-xl font-bold">
            {{ formatStroops(pool.guaranteeDeposited) }} / {{ formatStroops(pool.requiredGuarantee) }}
          </p>
          <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-slate-100" aria-hidden="true">
            <div class="h-full rounded-full bg-emerald-500" :style="{ width: `${guaranteePercent}%` }" />
          </div>
        </div>
        <div class="card">
          <p class="text-xs text-slate-500">Üyelerin toplam iade hakkı</p>
          <p class="mt-1 text-xl font-bold">{{ formatStroops(totalRefundable) }}</p>
          <p class="mt-1 text-xs text-slate-500">İptal olursa geri alınabilecek</p>
        </div>
        <div class="card">
          <p class="text-xs text-slate-500">Tur son ödeme</p>
          <p
            class="mt-1 text-xl font-bold tabular-nums"
            :class="round && deadlinePassed && pool.status === 'Active' ? 'text-amber-700' : ''"
          >
            <template v-if="round && (pool.status === 'Active' || pool.status === 'Paused')">
              {{ deadlinePassed ? 'Süre doldu' : formatDuration(remaining) }}
            </template>
            <template v-else>—</template>
          </p>
        </div>
      </section>

      <!-- Tur ve alım kaydı -->
      <section
        v-if="round && (pool.status === 'Active' || pool.status === 'Paused')"
        class="card space-y-4"
        aria-labelledby="tur"
      >
        <h2 id="tur" class="text-lg font-semibold">
          Tur {{ pool.currentRound }} / {{ pool.memberLimit }}
        </h2>
        <dl class="grid gap-4 sm:grid-cols-3">
          <div>
            <dt class="text-xs text-slate-500">Bu turun alıcısı</dt>
            <dd class="mt-0.5 font-mono text-sm font-semibold" :title="round.recipient">
              {{ shortAddress(round.recipient, 6) }}
              <span v-if="isRecipient" class="badge bg-indigo-100 text-indigo-800">Sen</span>
            </dd>
          </div>
          <div>
            <dt class="text-xs text-slate-500">Satıcıya gidecek tutar</dt>
            <dd class="mt-0.5 text-sm font-semibold">{{ formatStroops(round.pot) }} {{ token }}</dd>
          </div>
          <div>
            <dt class="text-xs text-slate-500">Satıcı</dt>
            <dd class="mt-0.5 font-mono text-sm font-semibold" :title="round.seller ?? ''">
              {{ round.seller ? shortAddress(round.seller, 6) : 'Henüz önerilmedi' }}
            </dd>
          </div>
        </dl>
        <div v-if="round.docHash" class="text-xs text-slate-500">
          Alım belgesi özeti (SHA-256):
          <span class="break-all font-mono">{{ round.docHash }}</span>
        </div>
        <div v-if="round.seller" class="text-sm">
          <p class="font-medium">
            Doğrulayıcı onayı: {{ approvalsCount }} / {{ pool.approvalThreshold }}
            <span v-if="approvalsOk" class="badge bg-emerald-100 text-emerald-800">Onaylandı ✓</span>
          </p>
          <ul class="mt-2 space-y-1">
            <li v-for="v in pool.verifiers" :key="v" class="flex items-center gap-2 text-xs">
              <span class="font-mono" :title="v">{{ shortAddress(v, 6) }}</span>
              <span v-if="v === me" class="badge bg-indigo-100 text-indigo-800">Sen</span>
              <span class="badge" :class="round.approvals.includes(v) ? toneClass.green : toneClass.slate">
                {{ round.approvals.includes(v) ? 'Onayladı ✓' : 'Bekliyor' }}
              </span>
            </li>
          </ul>
        </div>
      </section>

      <!-- Üyeler -->
      <section class="card" aria-labelledby="uyeler">
        <h2 id="uyeler" class="text-lg font-semibold">Üyeler ve sıra</h2>
        <ul class="mt-3 divide-y divide-slate-100">
          <li
            v-for="(addr, idx) in listedMembers"
            :key="addr"
            class="flex flex-wrap items-center justify-between gap-2 py-3"
          >
            <div class="flex flex-wrap items-center gap-2">
              <span class="w-5 text-sm text-slate-400">{{ idx + 1 }}</span>
              <span class="font-mono text-sm" :title="addr">{{ shortAddress(addr, 6) }}</span>
              <span v-if="addr === me" class="badge bg-indigo-100 text-indigo-800">Sen</span>
              <span v-if="addr === pool.creator" class="badge bg-slate-100 text-slate-700">Kurucu</span>
              <span
                v-if="round && addr === round.recipient && (pool.status === 'Active' || pool.status === 'Paused')"
                class="badge bg-emerald-100 text-emerald-800"
              >
                Bu turun alıcısı
              </span>
              <span v-if="memberByAddress.get(addr)?.received" class="badge bg-slate-100 text-slate-700">
                Payını aldı
              </span>
            </div>
            <div class="flex flex-wrap items-center gap-3">
              <span class="text-xs text-slate-500">
                İade hakkı: {{ formatStroops(memberByAddress.get(addr)?.refundable ?? 0n) }} {{ token }}
              </span>
              <span class="badge" :class="toneClass[memberState(addr).tone]">
                {{ memberState(addr).label }}
              </span>
              <button
                v-if="isSponsor && pool.status === 'Paused' && !isFunded(addr)"
                type="button"
                class="btn-danger !min-h-8 !px-3 !py-1"
                :disabled="actionBusy !== null"
                @click="run(`topup-${addr}`, (s) => topUp(s, pool!.id, addr))"
              >
                {{ actionBusy === `topup-${addr}` ? 'Onayla…' : `Sponsor olarak tamamla (${formatStroops(pool.contributionAmount)})` }}
              </button>
              <span v-if="pool.status === 'Filling' && isCreator" class="flex gap-1">
                <button
                  type="button"
                  class="btn-secondary !min-h-8 !px-2 !py-1"
                  :disabled="idx === 0"
                  :aria-label="`${shortAddress(addr)} adresini yukarı taşı`"
                  @click="move(idx, -1)"
                >
                  ↑
                </button>
                <button
                  type="button"
                  class="btn-secondary !min-h-8 !px-2 !py-1"
                  :disabled="idx === order.length - 1"
                  :aria-label="`${shortAddress(addr)} adresini aşağı taşı`"
                  @click="move(idx, 1)"
                >
                  ↓
                </button>
              </span>
            </div>
          </li>
        </ul>
        <p v-if="pool.status === 'Filling' && isCreator" class="mt-3 text-xs text-slate-500">
          Başlatınca sıra kilitlenir ve kimse değiştiremez. Yukarıdan aşağıya her tur bir kişi alır.
        </p>
      </section>

      <!-- İşlemler -->
      <section class="card space-y-4" aria-labelledby="islemler">
        <h2 id="islemler" class="text-lg font-semibold">İşlemler</h2>

        <div v-if="!wallet.isConnected">
          <button type="button" class="btn-primary" :disabled="wallet.busy" @click="wallet.connect()">
            İşlem yapmak için cüzdan bağla
          </button>
        </div>

        <div v-else class="space-y-4">
          <!-- Kurulum -->
          <div v-if="pool.status === 'Filling'" class="flex flex-wrap items-center gap-3">
            <button
              v-if="isSponsor && !guaranteeOk"
              type="button"
              class="btn-primary"
              :disabled="actionBusy !== null"
              @click="run('fund', (s) => fundGuarantee(s, pool!.id, guaranteeMissing))"
            >
              {{ actionBusy === 'fund' ? 'Cüzdanı onayla…' : `Sponsor güvencesini yatır (${formatStroops(guaranteeMissing)} ${token})` }}
            </button>
            <button
              v-if="!isMember && !isFull"
              type="button"
              class="btn-primary"
              :disabled="actionBusy !== null"
              @click="run('join', (s) => joinPool(s, pool!.id))"
            >
              {{ actionBusy === 'join' ? 'Cüzdanı onayla…' : 'Havuza katıl' }}
            </button>
            <button
              v-if="isCreator && isFull && guaranteeOk"
              type="button"
              class="btn-primary"
              :disabled="actionBusy !== null"
              @click="run('start', (s) => startPool(s, pool!.id, order))"
            >
              {{ actionBusy === 'start' ? 'Cüzdanı onayla…' : 'Sırayı kilitle ve başlat' }}
            </button>
            <p v-if="isMember" class="text-sm text-slate-600">
              Katıldın.
              <template v-if="!isFull">Kalan {{ pool.memberLimit - pool.members.length }} üye bekleniyor.</template>
            </p>
            <p v-if="!guaranteeOk" class="text-sm text-slate-600">
              Havuz, sponsor güvencesi tamamlanmadan başlayamaz.
            </p>
            <p v-else-if="isFull && !isCreator" class="text-sm text-slate-600">
              Havuz hazır, kurucunun başlatması bekleniyor.
            </p>
          </div>

          <!-- Aktif tur -->
          <div v-if="pool.status === 'Active' && round" class="space-y-4">
            <div class="flex flex-wrap items-center gap-3">
              <button
                v-if="isMember && !myFunded"
                type="button"
                class="btn-primary"
                :disabled="actionBusy !== null"
                @click="run('deposit', (s) => deposit(s, pool!.id))"
              >
                {{ actionBusy === 'deposit' ? 'Cüzdanı onayla…' : `Bu turun katkısını öde (${formatStroops(pool.contributionAmount)} ${token})` }}
              </button>
              <p v-if="isMember && myFunded" class="text-sm text-emerald-800">Bu turun katkısı tamam ✓</p>

              <button
                v-if="isVerifier && round.seller && !approvedByMe"
                type="button"
                class="btn-secondary"
                :disabled="actionBusy !== null"
                @click="run('approve', (s) => approvePurchase(s, pool!.id, round!.round))"
              >
                {{ actionBusy === 'approve' ? 'Cüzdanı onayla…' : 'Alım kaydını onayla' }}
              </button>

              <button
                v-if="canExecute"
                type="button"
                class="btn-primary"
                :disabled="actionBusy !== null"
                @click="run('execute', (s) => executeRound(s, pool!.id))"
              >
                {{ actionBusy === 'execute' ? 'Cüzdanı onayla…' : 'Tutarı satıcıya gönder' }}
              </button>

              <button
                v-if="deadlinePassed && !allFunded"
                type="button"
                class="btn-danger"
                :disabled="actionBusy !== null"
                @click="run('overdue', (s) => markOverdue(s, pool!.id))"
              >
                {{ actionBusy === 'overdue' ? 'Cüzdanı onayla…' : 'Süre doldu, turu durdur' }}
              </button>
            </div>
            <p v-if="canExecute || (deadlinePassed && !allFunded)" class="text-xs text-slate-500">
              Bu işlemi havuzdaki herkes çağırabilir, bir yöneticiye gerek yok.
            </p>
            <p v-if="allFunded && !round.seller" class="text-sm text-slate-600">
              Katkılar tam. Sıradaki üyenin satıcıyı ve alım kaydını önermesi bekleniyor.
            </p>

            <!-- Satıcı önerisi (sıradaki üye) -->
            <form
              v-if="isRecipient && !round.seller"
              class="space-y-3 rounded-xl border border-slate-200 p-4"
              @submit.prevent="propose"
            >
              <h3 class="font-semibold">Satıcıyı ve alım kaydını öner</h3>
              <div>
                <label class="label" for="seller">Satıcının Stellar adresi</label>
                <input
                  id="seller"
                  v-model="sellerInput"
                  class="input font-mono"
                  type="text"
                  placeholder="G…"
                  autocomplete="off"
                  required
                />
                <p v-if="sellerInput && !sellerValid" class="mt-1 text-xs text-rose-700">
                  Geçerli bir Stellar adresi gir.
                </p>
              </div>
              <div>
                <label class="label" for="doc">Alım belgesi (fatura veya sözleşme özeti)</label>
                <textarea
                  id="doc"
                  v-model="docInput"
                  class="input min-h-20"
                  placeholder="Örn. araç/ev, satıcı, tutar, tarih, belge numarası"
                  required
                />
                <p class="mt-1 text-xs text-slate-500">
                  Belgenin kendisi zincire yazılmaz, yalnızca SHA-256 özeti kaydedilir. Doğrulayıcılar belgeyi
                  zincir dışında kontrol eder.
                </p>
              </div>
              <button
                type="submit"
                class="btn-primary"
                :disabled="actionBusy !== null || !sellerValid || !docInput.trim()"
              >
                {{ actionBusy === 'propose' ? 'Cüzdanı onayla…' : 'Öneriyi kaydet' }}
              </button>
            </form>
          </div>

          <!-- Duraklatılmış -->
          <div v-if="pool.status === 'Paused'" class="flex flex-wrap items-center gap-3">
            <button
              v-if="canAbort"
              type="button"
              class="btn-danger"
              :disabled="actionBusy !== null"
              @click="run('abort', (s) => abortPool(s, pool!.id))"
            >
              {{ actionBusy === 'abort' ? 'Cüzdanı onayla…' : 'Havuzu iptal et ve iadeleri aç' }}
            </button>
            <p v-if="!isSponsor" class="text-sm text-slate-600">
              Sponsor eksik katkıyı tamamlarsa tur devam eder, tamamlamazsa iptal edilebilir.
            </p>
          </div>

          <!-- İptal / Tamamlanma -->
          <div
            v-if="pool.status === 'Aborted' || pool.status === 'Completed'"
            class="flex flex-wrap items-center gap-3"
          >
            <button
              v-if="isMember && pool.status === 'Aborted' && myRefundable > 0n"
              type="button"
              class="btn-primary"
              :disabled="actionBusy !== null"
              @click="run('refund', (s) => claimRefund(s, pool!.id))"
            >
              {{ actionBusy === 'refund' ? 'Cüzdanı onayla…' : `İadeni al (${formatStroops(myRefundable)} ${token})` }}
            </button>
            <button
              v-if="isSponsor"
              type="button"
              class="btn-secondary"
              :disabled="actionBusy !== null"
              @click="run('remainder', (s) => claimSponsorRemainder(s, pool!.id))"
            >
              {{ actionBusy === 'remainder' ? 'Cüzdanı onayla…' : 'Sponsor kalanını geri al' }}
            </button>
            <p v-if="pool.status === 'Completed'" class="text-sm text-slate-600">
              Havuz tamamlandı. Tüm turlar satıcılara ödendi.
            </p>
          </div>

          <p v-if="actionError" role="alert" class="text-sm text-rose-700">{{ actionError }}</p>
          <p v-if="lastTx" class="text-sm text-emerald-800">
            İşlem tamamlandı:
            <a :href="explorerTx(lastTx)" target="_blank" rel="noopener noreferrer" class="font-mono underline">
              {{ lastTx.slice(0, 8) }}…
            </a>
            (Stellar Expert'te görüntüle)
          </p>
        </div>
      </section>

      <section class="card text-sm text-slate-600" aria-label="Doğrulama">
        <p>
          Bu sayfadaki her bilgi doğrudan Stellar ağındaki sözleşmeden okunuyor.
          <a
            v-if="poolContractId"
            :href="explorerContract(poolContractId)"
            target="_blank"
            rel="noopener noreferrer"
            class="text-indigo-700 underline"
          >
            Sözleşmeyi blockchain üzerinde görüntüle
          </a>
        </p>
      </section>
    </template>
  </div>
</template>
