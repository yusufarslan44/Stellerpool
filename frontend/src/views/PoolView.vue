<script setup lang="ts">
import { StrKey } from '@stellar/stellar-sdk'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import AnchorDemo from '@/components/AnchorDemo.vue'
import AppIcon from '@/components/AppIcon.vue'
import CoinSpinner from '@/components/CoinSpinner.vue'
import DrawStage from '@/components/DrawStage.vue'
import Illo from '@/components/Illo.vue'
import Scene3D from '@/components/Scene3D.vue'
import { useNow } from '@/composables/useNow'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatDuration, formatStroops, shortAddress } from '@/lib/format'
import { sha256 } from '@/lib/hash'
import { explorerContract, explorerTx, poolAsset, poolContractId, poolTokenContractId } from '@/lib/stellar'
import { getTokenBalance, getTokenSymbol } from '@/services/account'
import {
  abortPool,
  approveTerms,
  cancelUnstartedPool,
  claimRefund,
  curePayment,
  deposit,
  drawRecipient,
  executeRound,
  getContractCapabilities,
  getMemberStatus,
  getPool,
  getRound,
  joinPool,
  markOverdue,
  proposePurchase,
  startPool,
} from '@/services/pool'
import type { Signer } from '@/services/pool'
import { useWalletStore } from '@/stores/wallet'
import type { MemberStatus, PoolInfo, RoundInfo, TxResult } from '@/types/pool'

const route = useRoute()
const wallet = useWalletStore()
const now = useNow()
/** Havuzun kendi varlığının kodu. Havuzlar farklı varlıklarla kurulabildiğinden zincirden okunur. */
const token = ref(poolAsset.getCode())
/** Havuz, arayüzün varsayılan varlığıyla (USDC) mı kurulmuş? Anchor yüklemesi yalnızca o durumda anlamlı. */
const usesPoolAsset = computed(() => !pool.value || pool.value.token === poolTokenContractId)

async function resolveToken(tokenId: string) {
  if (tokenId === poolTokenContractId) {
    token.value = poolAsset.getCode()
    return
  }
  token.value = await getTokenSymbol(tokenId).catch(() => shortAddress(tokenId))
}

const poolId = computed(() => Number.parseInt(String(route.params.id), 10))

const pool = ref<PoolInfo | null>(null)
const round = ref<RoundInfo | null>(null)
const members = ref<MemberStatus[]>([])
const contractBalance = ref<bigint | null>(null)
/** Bağlı cüzdanın havuz varlığı bakiyesi; katkıya yetip yetmediğini göstermek için. */
const myBalance = ref<bigint | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)
const actionBusy = ref<string | null>(null)
const actionError = ref<string | null>(null)
const lastTx = ref<string | null>(null)
/** Önceki sürüme bağlanıldığında yeni akışın yazma adımları yalnız v12 ile yapılır. */
const simpleTerms = ref<boolean | null>(null)
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
    await resolveToken(p.token)

    if (p.status === 'Filling') {
      round.value = null
    } else if (p.status === 'Active') {
      round.value = await getRound(p.id, p.currentRound)
    } else {
      round.value = p.currentRound > 0 ? await getRound(p.id, p.currentRound).catch(() => null) : null
    }

    const [statuses, balance] = await Promise.all([
      Promise.all(p.members.map((m) => getMemberStatus(p.id, m))),
      getTokenBalance(poolContractId, p.token).catch(() => null),
    ])
    members.value = statuses
    contractBalance.value = balance
  } catch (e) {
    loadError.value = errorMessage(e)
  } finally {
    loading.value = false
  }
}

async function loadMyBalance() {
  myBalance.value = wallet.address
    ? await getTokenBalance(wallet.address, pool.value?.token).catch(() => null)
    : null
}
watch(() => [wallet.address, pool.value?.token], () => void loadMyBalance(), { immediate: true })

let poll: ReturnType<typeof setInterval> | undefined
onMounted(() => {
  void getContractCapabilities().then((caps) => { simpleTerms.value = caps.simpleTerms }).catch(() => { simpleTerms.value = false })
  void load()
  poll = setInterval(() => void load(true), 10_000)
})
onUnmounted(() => clearInterval(poll))
watch(poolId, () => void load())

// --- Roller ------------------------------------------------------------------------------
const me = computed(() => wallet.address)
const isMember = computed(() => !!me.value && !!pool.value?.members.includes(me.value))
const isCreator = computed(() => !!me.value && pool.value?.creator === me.value)
const isDraw = computed(() => pool.value?.orderMode === 'Draw')
const isRecipient = computed(() => !!me.value && !!round.value?.recipient && round.value.recipient === me.value)
const isFull = computed(() => !!pool.value && pool.value.members.length >= pool.value.memberLimit)

// --- Tur durumu --------------------------------------------------------------------------
const paid = computed(() => new Set(round.value?.paid ?? []))
const isFunded = (m: string) => paid.value.has(m)
const missingMembers = computed(() => (pool.value?.members ?? []).filter((m) => !isFunded(m)))
const allFunded = computed(() => !!pool.value && pool.value.members.length > 0 && missingMembers.value.length === 0)
const roundDeadline = computed(() => {
  if (!round.value) return 0
  switch (round.value.phase) {
    case 'Collecting': return round.value.collectDeadline
    case 'Grace': return round.value.graceDeadline
    case 'AwaitingDraw':
    case 'AwaitingPurchase': return round.value.purchaseDeadline
    default: return 0
  }
})
const remaining = computed(() => (roundDeadline.value ? roundDeadline.value - now.value : 0))
const deadlinePassed = computed(() => round.value?.phase === 'Collecting' && now.value >= round.value.collectDeadline)
const myFunded = computed(() => !!me.value && isFunded(me.value))
const lowBalance = computed(() => myBalance.value !== null && !!pool.value && myBalance.value < pool.value.contributionAmount)
const recipientPaid = computed(() => !!round.value?.recipient && paid.value.has(round.value.recipient))
const myOwnPaid = computed(() => !!me.value && paid.value.has(me.value))
/** Bütün üyeler kendi katkısını yatırmadan tahsisat açılmaz. */
const recipientBlock = computed<string | null>(() => {
  if (!pool.value || pool.value.status !== 'Active' || !round.value) return null
  const phase = round.value.phase
  if (phase === 'Settled') return null
  if (!round.value.recipient) {
    // Kura modu: alıcı henüz belli değil; herkesin katkısı tamamlanmadan kura çekilmez.
    if (!allFunded.value && (phase === 'Grace' || deadlinePassed.value)) {
      return 'A contribution is missing. When the grace period ends the round stops and no draw is held; only contributions paid in this round can be refunded.'
    }
    return null
  }
  if (!recipientPaid.value && (phase === 'Grace' || deadlinePassed.value)) {
    return 'The member whose turn it is has not paid their contribution. When the grace period ends the round stops; only contributions paid in this round can be refunded.'
  }
  return null
})

const canExecute = computed(() =>
  round.value?.phase === 'AwaitingPurchase' &&
  allFunded.value && recipientPaid.value &&
  !!round.value.seller && now.value < round.value.purchaseDeadline,
)

const canAbort = computed(
  () =>
    pool.value?.status === 'Active' && !!round.value &&
    ((round.value.phase === 'Grace' && round.value.graceDeadline > 0 && now.value >= round.value.graceDeadline) ||
      ((round.value.phase === 'AwaitingPurchase' || round.value.phase === 'AwaitingDraw') && round.value.purchaseDeadline > 0 && now.value >= round.value.purchaseDeadline)),
)

const memberByAddress = computed(() => new Map(members.value.map((m) => [m.address, m])))

// --- Kura ------------------------------------------------------------------------------------
const excludedFromDraw = computed(() => (pool.value?.members ?? []).filter((m) => memberByAddress.value.get(m)?.received))
const drawCandidates = computed(() => (pool.value?.members ?? []).filter((m) => !memberByAddress.value.get(m)?.received))
const drawReady = computed(
  () => isDraw.value && round.value?.phase === 'AwaitingDraw' && allFunded.value && remaining.value > 0,
)
const showDrawStage = computed(
  () => isDraw.value && pool.value?.status === 'Active' && !!round.value &&
    (round.value.phase === 'AwaitingDraw' || !!round.value.recipient),
)
/**
 * Kontratın `get_member_status.refundable` değeri havuz durumuna bakmadan "mevcut turda yatırdıysa
 * katkı" döndürür; tamamlanmış havuzun son turunda da (para satıcıya gitmiş olsa bile) dolu gelir
 * (canlı Testnet verisiyle görüldü). İade yalnızca ödenmemiş tur için anlamlıdır, o yüzden
 * tamamlanmış havuzda ve ödenmiş (Settled) turda sıfır gösterilir.
 */
const refundableOf = (address: string): bigint => {
  // İade yalnızca iptal edilmiş havuzda anlamlıdır (peşinat da dahil: kontrat, henüz almamış üyenin
  // harcanmamış peşinatını da bu değere ekler). Aktif ya da tamamlanmış havuzda gösterilmez.
  if (pool.value?.status !== 'Aborted') return 0n
  return memberByAddress.value.get(address)?.refundable ?? 0n
}
/** Bir turda satıcıya giden toplam: tur katkıları + alıcının kendi peşinatı. */
const purchaseAmount = computed(() =>
  pool.value ? pool.value.contributionAmount * BigInt(pool.value.memberLimit) + pool.value.downPayment : 0n,
)
// Alım öneri formu hazır gelsin: satıcı havuzda kayıtlı tek adrestir, belge için düzenlenebilir bir taslak doldurulur.
watch(
  () => [pool.value?.id, pool.value?.demoSeller, round.value?.round, round.value?.phase] as const,
  () => {
    const p = pool.value
    const r = round.value
    if (!p) return
    if (!sellerInput.value) sellerInput.value = p.demoSeller
    if (!docInput.value && r && r.phase === 'AwaitingPurchase') {
      const date = new Date().toISOString().slice(0, 10)
      docInput.value =
        `Demo purchase document · Pool #${p.id} · Round ${r.round} · Total amount ${formatStroops(purchaseAmount.value)} ${token.value}` +
        (p.downPayment > 0n ? ` (pool + ${formatStroops(p.downPayment)} down payment)` : '') +
        ` · Seller ${shortAddress(p.demoSeller)} · Date ${date}`
    }
  },
  { immediate: true },
)
const totalRefundable = computed(() => members.value.reduce((sum, m) => sum + refundableOf(m.address), 0n))
const myRefundable = computed(() => (me.value ? refundableOf(me.value) : 0n))

const termsReady = computed(() => {
  if (!pool.value || pool.value.termsVersion === 0) return false
  return pool.value.members.every((v) => pool.value!.termsApprovals.includes(v))
})
const canApproveTerms = computed(() =>
  !!pool.value && pool.value.status === 'Filling' && pool.value.termsVersion > 0 &&
  !!me.value && isMember.value && !pool.value.termsApprovals.includes(me.value),
)
const setupExpired = computed(() => !!pool.value && now.value >= pool.value.setupDeadline)

function memberState(address: string) {
  const p = pool.value
  if (!p || p.status === 'Filling') return { label: 'Joined', tone: 'slate' as const }
  if (p.status === 'Completed' || p.status === 'Aborted') return { label: '—', tone: 'slate' as const }
  if (paid.value.has(address)) return { label: 'Paid ✓', tone: 'green' as const }
  return { label: 'Bekliyor', tone: 'slate' as const }
}

const toneClass = {
  green: 'bg-sage-100 text-sage-800',
  amber: 'bg-gold-100 text-amber-900',
  slate: 'bg-stone-100 text-stone-700',
}

const statusLabel = computed(() => {
  switch (pool.value?.status) {
    case 'Filling':
      return { text: 'Waiting for members', cls: 'bg-stone-100 text-stone-700' }
    case 'Active':
      if (round.value?.phase === 'AwaitingDraw') return { text: 'Waiting for the draw', cls: 'bg-gold-100 text-amber-900' }
      return round.value?.phase === 'Grace'
        ? { text: 'In grace period', cls: 'bg-gold-100 text-amber-900' }
        : { text: 'Devam ediyor', cls: 'bg-brand-100 text-brand-800' }
    case 'Completed':
      return { text: 'Completed', cls: 'bg-sage-100 text-sage-800' }
    case 'Aborted':
      return { text: 'Cancelled', cls: 'bg-rose-100 text-rose-800' }
    default:
      return { text: '', cls: '' }
  }
})

// --- İşlemler ----------------------------------------------------------------------------
const signer = computed<Signer | null>(() =>
  me.value ? { address: me.value, signTransaction: wallet.signTransaction } : null,
)

async function run(name: string, fn: (s: Signer) => Promise<TxResult>) {
  if (!signer.value || simpleTerms.value !== true) return
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

const sellerValid = computed(() =>
  StrKey.isValidEd25519PublicKey(sellerInput.value.trim()) && sellerInput.value.trim() === pool.value?.demoSeller,
)

async function propose() {
  const seller = sellerInput.value.trim()
  const doc = docInput.value.trim()
  if (!pool.value || !round.value || !sellerValid.value || !doc) return
  const hash = await sha256(doc)
  await run('propose', (s) => proposePurchase(s, {
    poolId: pool.value!.id,
    seller,
    asset: pool.value!.token,
    amount: purchaseAmount.value,
    docHash: hash,
  }))
}

const copied = ref(false)
async function copyLink() {
  await navigator.clipboard?.writeText(window.location.href)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}

const listedMembers = computed(() =>
  isDraw.value ? (pool.value?.members ?? []) : (pool.value?.recipientOrder.length ? pool.value.recipientOrder : (pool.value?.members ?? [])),
)
/** Büyük gruplarda liste varsayılan olarak kısaltılır. */
const COMPACT_LIST = 12
const showAllMembers = ref(false)
const visibleMembers = computed(() =>
  showAllMembers.value || listedMembers.value.length <= COMPACT_LIST
    ? listedMembers.value
    : listedMembers.value.slice(0, COMPACT_LIST),
)

// --- Adım adım rehber ----------------------------------------------------------------------
interface GuideStep {
  key: string
  title: string
  who: string
  detail: string
  done: boolean
}

const fundedCount = computed(() => (pool.value?.members ?? []).filter(isFunded).length)

const setupSteps = computed<GuideStep[]>(() => {
  const p = pool.value
  if (!p) return []
  return [
    { key: 'join', title: 'Members join', who: 'Members', detail: `${p.members.length} / ${p.memberLimit} members`, done: isFull.value },
    {
      key: 'approve',
      title: 'Everyone approves the terms',
      who: 'Members',
      detail: `${p.termsApprovals.length} / ${p.members.length} onay`,
      done: termsReady.value,
    },
    { key: 'start', title: 'The pool starts', who: 'Anyone can start it', detail: 'One click once the terms are complete', done: false },
  ]
})

const roundSteps = computed<GuideStep[]>(() => {
  const p = pool.value
  const r = round.value
  if (!p || !r) return []
  const settled = r.phase === 'Settled'
  const draw: GuideStep[] = isDraw.value
    ? [
        {
          key: 'draw',
          title: 'The draw is held',
          who: 'Anyone can call',
          detail: r.recipient
            ? `Kazanan: ${shortAddress(r.recipient, 6)}${r.recipient === me.value ? ' (sen)' : ''}`
            : `${drawCandidates.value.length} members in the draw`,
          done: settled || !!r.recipient,
        },
      ]
    : []
  return [
    {
      key: 'collect',
      title: 'Contributions are collected',
      who: 'Members',
      detail: `${fundedCount.value} / ${p.members.length} tamam`,
      done: settled || allFunded.value,
    },
    ...draw,
    {
      key: 'propose',
      title: 'The seller and purchase document are proposed',
      who: isDraw.value ? 'Draw winner' : 'Next member in order',
      detail: r.seller ? 'Proposal recorded' : 'Pending',
      done: settled || !!r.seller,
    },
    { key: 'pay', title: 'The amount goes to the seller', who: 'Anyone can call', detail: `${formatStroops(purchaseAmount.value)} ${token.value}`, done: settled },
  ]
})

const guideSteps = computed<GuideStep[]>(() =>
  simpleTerms.value !== true ? [] : pool.value?.status === 'Filling' ? setupSteps.value : pool.value?.status === 'Active' ? roundSteps.value : [],
)
const currentStepKey = computed(() => guideSteps.value.find((s) => !s.done)?.key ?? null)

/** Bağlı cüzdanın şu an yapabileceği bir işlem olan adımlar. */
const stepMine = computed<Record<string, boolean | undefined>>(() => {
  const p = pool.value
  const r = round.value
  if (!p || !me.value) return {}
  const purchasing = r?.phase === 'AwaitingPurchase'
  return {
    join: !isMember.value && !isFull.value,
    approve: canApproveTerms.value && !setupExpired.value,
    start: isFull.value && termsReady.value && !setupExpired.value,
    draw: drawReady.value && isMember.value,
    collect:
      isMember.value && (r?.phase === 'Collecting' || r?.phase === 'Grace') && !myFunded.value,
    propose:
      isRecipient.value && purchasing && !r?.seller && recipientPaid.value && remaining.value > 0,
    pay: canExecute.value,
  }
})

interface Banner {
  tone: 'mine' | 'wait' | 'warn' | 'done'
  title: string
  text: string
}

const banner = computed<Banner | null>(() => {
  const p = pool.value
  if (!p) return null
  if (simpleTerms.value === false) return { tone: 'warn', title: 'Old contract version', text: 'This pool is on the v11 contract, which requires verifier approval. The new flow needs the v12 contract address and a new pool.' }
  if (p.status === 'Completed') return { tone: 'done', title: 'Pool completed', text: 'All rounds were paid to sellers.' }
  if (p.status === 'Aborted') {
    return {
      tone: 'warn',
      title: 'Pool cancelled',
      text:
        isMember.value && myRefundable.value > 0n
          ? `You have a refund: ${formatStroops(myRefundable.value)} ${token.value}. You can claim it below.`
          : 'Only your own contributions in the round that has not been paid out can be recovered. Payments of earlier rounds cannot be recovered.',
    }
  }
  if (!wallet.isConnected) {
    return { tone: 'wait', title: 'Connect your wallet first', text: 'Connect your wallet from the top right to see what you can do in this pool.' }
  }
  if (p.status === 'Filling' && setupExpired.value) {
    return { tone: 'warn', title: 'Setup period ended', text: 'The pool did not start in time. It can be cancelled below.' }
  }
  if (recipientBlock.value) return { tone: 'warn', title: 'This round cannot proceed right now', text: recipientBlock.value }
  if (canAbort.value) {
    return { tone: 'warn', title: 'Time is up', text: 'The pool can be ended. Only the contributions of the round not yet paid to the seller are refunded.' }
  }
  const open = guideSteps.value.filter((s) => !s.done)
  const mineStep = open.find((s) => stepMine.value[s.key])
  if (mineStep) return { tone: 'mine', title: 'It’s your turn', text: mineStep.title }
  const wait = open[0]
  if (wait) return { tone: 'wait', title: `Currently waiting for: ${wait.who}`, text: wait.title }
  return null
})

const bannerIllo = { mine: 'sparkles', warn: 'warning', done: 'party', wait: 'hourglass' } as const
const STEP_ILLO = {
  join: 'handshake',
  approve: 'check',
  start: 'rocket',
  collect: 'moneybag',
  draw: 'dice',
  propose: 'receipt',
  pay: 'store',
} as const

const bannerClass = {
  mine: 'border-brand-300 bg-brand-50 text-brand-900',
  wait: 'border-stone-200 bg-stone-50 text-stone-800',
  warn: 'border-gold-300 bg-gold-100/80 text-amber-950',
  done: 'border-sage-200 bg-sage-50 text-sage-900',
}

const roleChips = computed(() => {
  const chips: string[] = []
  if (isCreator.value) chips.push('Kurucu')
  if (isMember.value) chips.push('Member')
  if (isRecipient.value && pool.value?.status === 'Active') chips.push('This round’s recipient')
  return chips
})

/** 3B sahne en fazla 12 para gösterir; büyük gruplarda para sayısı ve dolu oranı ölçeklenir. */
const SCENE_MAX = 12
const sceneCoins = computed(() => Math.min(SCENE_MAX, pool.value?.memberLimit ?? 4))
/** 3B sahnede altın görünen para sayısı: katılan / bu tur ödeyen üye sayısı (ölçekli). */
const sceneFilled = computed(() => {
  const p = pool.value
  if (!p) return -1
  if (p.status === 'Completed') return -1
  if (p.status === 'Aborted') return 0
  const count = p.status === 'Filling' ? p.members.length : fundedCount.value
  return p.memberLimit <= SCENE_MAX ? count : Math.round((count / p.memberLimit) * sceneCoins.value)
})
const progressPercent = computed(() => {
  const p = pool.value
  if (!p) return 0
  if (p.status === 'Completed') return 100
  if (p.status === 'Filling') return 0
  return Math.min(100, Math.round(((p.currentRound - (round.value?.phase === 'Settled' ? 0 : 1)) / p.memberLimit) * 100))
})
const countdownLabel = computed(() =>
  round.value?.phase === 'Grace' ? 'Grace period'
    : round.value?.phase === 'AwaitingDraw' ? 'Draw and purchase period'
    : round.value?.phase === 'AwaitingPurchase' ? 'Purchase period' : 'Contribution period',
)

</script>

<template>
  <div class="space-y-6">
    <RouterLink to="/" class="inline-flex min-h-11 items-center gap-1 text-sm font-medium text-brand-700 hover:underline">
      <AppIcon name="back" class="!size-4" /> Ana sayfa
    </RouterLink>

    <p
      v-if="!poolContractId"
      role="status"
      class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950"
    >
      <Illo name="bulb" :size="28" />
      <span>
        The pool contract is not configured yet, so pool data cannot be read. Once the contract
        is published and its address is added as <code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code>,
        this page will show real data.
      </span>
    </p>

    <div v-else-if="loading" class="space-y-4" aria-live="polite">
      <div class="flex items-center gap-3 text-stone-600">
        <CoinSpinner :size="40" /> Reading the pool from the chain…
      </div>
      <div class="skeleton h-48 rounded-[2rem]" />
      <div class="grid gap-4 sm:grid-cols-3">
        <div class="skeleton h-24 rounded-3xl" />
        <div class="skeleton h-24 rounded-3xl" />
        <div class="skeleton h-24 rounded-3xl" />
      </div>
    </div>

    <div v-else-if="loadError" role="alert" class="card space-y-3">
      <h1 class="text-2xl font-extrabold">Pool #{{ poolId }} could not be read</h1>
      <p class="text-sm text-rose-700">{{ loadError }}</p>
      <p class="text-sm text-stone-600">In the new v12 contract, pool numbers start over. The old pool’s on-chain record stays in the <a class="font-semibold text-brand-700 underline" href="https://stellar.expert/explorer/testnet/contract/CD23I5ZNVHOUBJOHODHHEW6NH7NB3FDBQ2DE2C2TST5BZM4KTJLBPK33" target="_blank" rel="noopener noreferrer">v11 contract</a>.</p>
      <button type="button" class="btn-secondary" @click="load()">Try again</button>
    </div>

    <template v-else-if="pool">
      <!-- BAŞLIK + 3B SAHNE -->
      <header class="sunrise relative overflow-hidden rounded-[2rem] p-5 sm:p-8">
        <div class="grid items-center gap-4 md:grid-cols-[1.3fr_1fr]">
          <div>
            <div class="flex flex-wrap items-center gap-2">
              <span class="badge" :class="statusLabel.cls">{{ statusLabel.text }}</span>
              <span class="badge bg-gold-100 text-amber-900">{{ isDraw ? 'Draw' : 'Fixed order' }}</span>
              <span v-for="c in roleChips" :key="c" class="badge bg-ink text-cream">Sen: {{ c }}</span>
            </div>
            <h1 class="mt-3 text-4xl font-extrabold sm:text-5xl">Havuz #{{ pool.id }}</h1>
            <p class="mt-2 text-stone-700">
              {{ pool.members.length }} / {{ pool.memberLimit }} members · every round
              <strong>{{ formatStroops(pool.contributionAmount) }} {{ token }}</strong>
            </p>
            <p v-if="pool.downPayment > 0n" class="mt-1 text-sm text-stone-700">
              Down payment: <strong>{{ formatStroops(pool.downPayment) }} {{ token }}</strong> (paid into the contract when joining,
              added to your purchase when your turn comes)
            </p>
            <p class="mt-1 text-sm text-stone-600">
              Every round each member pays their own contribution. If a payment is missing, no allocation is made.
              {{ isDraw ? 'The recipient is chosen by draw once all contributions are in.' : '' }}
            </p>
            <div class="mt-5 flex flex-wrap gap-2">
              <button type="button" class="btn-secondary !min-h-10" @click="copyLink">
                <AppIcon :name="copied ? 'check' : 'copy'" class="!size-4" />
                {{ copied ? 'Copied' : 'Copy link' }}
              </button>
              <a
                v-if="poolContractId"
                :href="explorerContract(poolContractId)"
                target="_blank"
                rel="noopener noreferrer"
                class="btn-secondary !min-h-10"
              >
                View on-chain <AppIcon name="external" class="!size-4" />
              </a>
            </div>
          </div>
          <div class="relative h-44 sm:h-56">
            <Scene3D
              :coins="sceneCoins"
              :filled="sceneFilled"
              :label="`Pool of ${pool.memberLimit} members; ${sceneFilled < 0 ? 'all' : sceneFilled} coins gold`"
            />
          </div>
        </div>

        <div v-if="pool.status === 'Active'" class="mt-5">
          <div class="mb-1 flex justify-between text-xs font-semibold text-stone-700">
            <span>Round {{ pool.currentRound }} / {{ pool.memberLimit }}</span>
            <span class="tabular-nums">{{ progressPercent }}% complete</span>
          </div>
          <div class="h-2.5 overflow-hidden rounded-full bg-white/70" role="progressbar" :aria-valuenow="progressPercent" aria-valuemin="0" aria-valuemax="100" aria-label="Havuz ilerlemesi">
            <div class="h-full rounded-full bg-gradient-to-r from-brand-500 to-gold-400 transition-[width] duration-700" :style="{ width: `${progressPercent}%` }" />
          </div>
        </div>
      </header>

      <!-- ADIM ADIM REHBER -->
      <section class="card space-y-5 !p-5 sm:!p-7" aria-labelledby="rehber">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <h2 id="rehber" class="text-2xl font-extrabold">
            {{ pool.status === 'Filling' ? 'How does the pool start?' : pool.status === 'Active' ? `Round ${pool.currentRound}: step by step` : 'Status' }}
          </h2>
          <span v-if="pool.status === 'Active' && roundDeadline > 0" class="badge bg-brand-50 text-brand-800 ring-1 ring-brand-100">
            <AppIcon name="clock" class="!size-3.5" />
            {{ countdownLabel }}:
            <span class="tabular-nums" :class="remaining <= 0 ? 'text-rose-700' : ''">
              {{ remaining <= 0 ? 'time is up' : formatDuration(remaining) }}
            </span>
          </span>
        </div>

        <!-- Şimdi ne yapmalı? -->
        <div
          v-if="banner"
          class="pop flex items-start gap-3 rounded-2xl border-2 p-4"
          :class="bannerClass[banner.tone]"
          :role="banner.tone === 'warn' ? 'alert' : 'status'"
        >
          <Illo :name="bannerIllo[banner.tone]" :size="44" class="mt-0.5" />
          <div>
            <p class="font-display text-lg font-bold">{{ banner.title }}</p>
            <p class="text-sm leading-relaxed">{{ banner.text }}</p>
          </div>
        </div>

        <div v-if="!wallet.isConnected && (pool.status === 'Filling' || pool.status === 'Active')">
          <button type="button" class="btn-primary btn-lg" :disabled="wallet.busy" @click="wallet.connect()">
            <AppIcon name="wallet" class="!size-4" /> Connect wallet
          </button>
        </div>

        <!-- Adım listesi: her adımın kendi işlemi kendi içinde -->
        <ol v-if="guideSteps.length" class="space-y-0" aria-label="Steps">
          <li v-for="(s, i) in guideSteps" :key="s.key" class="grid grid-cols-[2.5rem_1fr] gap-x-3">
            <div class="flex flex-col items-center">
              <span
                class="relative grid size-10 shrink-0 place-items-center rounded-full text-sm font-bold transition-colors duration-300"
                :class="
                  s.done ? 'bg-sage-600 text-white'
                  : s.key === currentStepKey ? 'bg-brand-600 text-white'
                  : 'bg-stone-200 text-stone-600'
                "
              >
                <AppIcon v-if="s.done" name="check" class="!size-5" />
                <template v-else>{{ i + 1 }}</template>
                <span
                  v-if="!s.done && s.key === currentStepKey"
                  class="absolute inset-0 rounded-full border-2 border-brand-500"
                  style="animation: ring-ping 2s ease-out infinite"
                  aria-hidden="true"
                />
              </span>
              <span
                v-if="i < guideSteps.length - 1"
                class="my-1 w-0.5 flex-1 rounded-full transition-colors duration-500"
                :class="s.done ? 'bg-sage-500' : 'bg-stone-200'"
                aria-hidden="true"
              />
            </div>

            <div class="min-w-0 pb-6" :class="i === guideSteps.length - 1 ? '!pb-0' : ''">
              <div class="flex flex-wrap items-center gap-x-2 gap-y-1 pt-1.5">
                <Illo :name="STEP_ILLO[s.key as keyof typeof STEP_ILLO]" :size="30" :class="s.done ? 'opacity-50 grayscale' : ''" />
                <h3 class="font-display text-lg font-bold" :class="s.done ? 'text-stone-500' : ''">{{ s.title }}</h3>
                <span v-if="stepMine[s.key] && !s.done" class="badge bg-brand-600 text-white">Your turn</span>
              </div>
              <p class="text-sm text-stone-600">
                <span class="font-semibold text-stone-700">{{ s.who }}</span> · {{ s.detail }}
              </p>

              <div
                v-if="s.key === 'collect' && pool.status === 'Active' && pool.members.length > COMPACT_LIST"
                class="mt-2 max-w-md"
              >
                <div
                  class="h-2 overflow-hidden rounded-full bg-stone-200"
                  role="progressbar"
                  :aria-valuenow="fundedCount"
                  aria-valuemin="0"
                  :aria-valuemax="pool.members.length"
                  aria-label="Members who paid this round"
                >
                  <div class="h-full rounded-full bg-brand-500 transition-[width] duration-500" :style="{ width: `${(fundedCount / pool.members.length) * 100}%` }" />
                </div>
                <p class="mt-1 text-xs text-stone-600">{{ fundedCount }} / {{ pool.members.length }} members paid</p>
              </div>

              <!-- Kura sahnesi (izleyenler de görür) -->
              <div v-if="s.key === 'draw' && showDrawStage" class="mt-3 rounded-2xl bg-sand/60 p-4">
                <DrawStage
                  :candidates="drawCandidates"
                  :excluded="excludedFromDraw"
                  :winner="round?.recipient ?? null"
                  :spinning="actionBusy === 'draw'"
                  :me="me"
                />
              </div>

              <!-- Adıma özel işlemler -->
              <div v-if="wallet.isConnected" class="mt-3 space-y-3 empty:hidden">
                <!-- KURULUM -->
                <template v-if="pool.status === 'Filling'">
                  <button
                    v-if="s.key === 'join' && !isMember && !isFull"
                    type="button"
                    class="btn-primary"
                    :disabled="actionBusy !== null"
                    @click="run('join', (sg) => joinPool(sg, pool!.id))"
                  >
                    {{ actionBusy === 'join' ? 'Confirm in wallet…' : pool.downPayment > 0n ? `Join the pool (${formatStroops(pool.downPayment)} ${token} down payment is deposited)` : 'Join the pool' }}
                  </button>
                  <details
                    v-if="s.key === 'join' && !isMember && !isFull && pool.downPayment > 0n && usesPoolAsset"
                    class="group rounded-2xl border border-stone-200 bg-white p-4"
                    :open="myBalance !== null && myBalance < pool.downPayment"
                  >
                    <summary class="flex min-h-11 cursor-pointer list-none items-center justify-between gap-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
                      <span>
                        Not enough balance for the down payment? Load {{ token }} with the anchor
                        <span v-if="myBalance !== null" class="ml-1 text-xs font-normal text-stone-600">
                          (bakiyen {{ formatStroops(myBalance) }} {{ token }})
                        </span>
                      </span>
                      <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
                    </summary>
                    <div class="mt-3"><AnchorDemo compact @completed="loadMyBalance" /></div>
                  </details>
                  <p v-if="s.key === 'join' && isMember" class="text-sm font-medium text-sage-800">
                    ✓ You have joined.
                    <template v-if="!isFull">Waiting for {{ pool.memberLimit - pool.members.length }} more members.</template>
                  </p>

                  <button
                    v-if="s.key === 'approve' && canApproveTerms && !setupExpired"
                    type="button"
                    class="btn-primary"
                    :disabled="actionBusy !== null"
                    @click="run('approve-terms', (sg) => approveTerms(sg, pool!.id, pool!.termsVersion))"
                  >
                    {{ actionBusy === 'approve-terms' ? 'Confirm in wallet…' : `Approve terms version ${pool.termsVersion}` }}
                  </button>

                  <button
                    v-if="s.key === 'start' && isFull && termsReady && !setupExpired"
                    type="button"
                    class="btn-primary btn-lg"
                    :disabled="actionBusy !== null"
                    @click="run('start', (sg) => startPool(sg, pool!.id))"
                  >
                    {{ actionBusy === 'start' ? 'Confirm in wallet…' : 'Start the pool' }}
                  </button>
                </template>

                <!-- AKTİF TUR -->
                <template v-else-if="pool.status === 'Active' && round">
                  <template v-if="s.key === 'collect'">
                    <p v-if="round.phase === 'Grace'" role="status" class="rounded-2xl bg-gold-100/80 p-3 text-sm text-amber-950">
                      The round is in its grace period. Only the member concerned can pay the missing contribution from their own wallet.
                    </p>
                    <p v-if="recipientBlock" role="status" class="rounded-2xl bg-gold-100/80 p-3 text-sm text-amber-950">
                      {{ recipientBlock }}
                    </p>
                    <div class="flex flex-wrap items-center gap-3">
                      <button
                        v-if="isMember && !myFunded && (round.phase === 'Collecting' || round.phase === 'Grace')"
                        type="button"
                        class="btn-primary"
                        :disabled="actionBusy !== null"
                        @click="run('deposit', (sg) => round!.phase === 'Grace' ? curePayment(sg, pool!.id) : deposit(sg, pool!.id))"
                      >
                        {{ actionBusy === 'deposit' ? 'Confirm in wallet…' : `Pay this round’s contribution (${formatStroops(pool.contributionAmount)} ${token})` }}
                      </button>
                      <p v-if="isMember && myOwnPaid" class="text-sm font-medium text-sage-800">
                        ✓ You paid this round’s contribution yourself
                      </p>
                      <button
                        v-if="deadlinePassed && !allFunded"
                        type="button"
                        class="btn-danger"
                        :disabled="actionBusy !== null"
                        @click="run('overdue', (sg) => markOverdue(sg, pool!.id))"
                      >
                        {{ actionBusy === 'overdue' ? 'Confirm in wallet…' : 'Time is up, stop the round' }}
                      </button>
                    </div>
                    <p v-if="deadlinePassed && !allFunded" class="text-xs text-stone-600">
                      Anyone in the pool can call this; no administrator is needed.
                    </p>

                    <!-- Anchor ile bakiye yükleme: katkıdan önceki gerçek fiat-kapısı adımı -->
                    <details
                      v-if="usesPoolAsset && isMember && !myFunded && (round.phase === 'Collecting' || round.phase === 'Grace')"
                      class="group rounded-2xl border border-stone-200 bg-white p-4"
                      :open="lowBalance"
                    >
                      <summary class="flex min-h-11 cursor-pointer list-none items-center justify-between gap-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
                        <span>
                          Not enough balance? Load {{ token }} with the anchor
                          <span v-if="myBalance !== null" class="ml-1 text-xs font-normal text-stone-600">
                            (bakiyen {{ formatStroops(myBalance) }} {{ token }})
                          </span>
                        </span>
                        <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
                      </summary>
                      <div class="mt-3">
                        <AnchorDemo compact @completed="loadMyBalance" />
                        <p v-if="pool.contributionAmount > 100_000_000n" class="mt-2 text-xs text-stone-600">
                          The anchor may set a per-transaction limit; if your contribution is large you may need to load several times.
                        </p>
                      </div>
                    </details>
                  </template>

                  <template v-else-if="s.key === 'draw'">
                    <button
                      v-if="drawReady"
                      type="button"
                      class="btn-primary btn-lg"
                      :disabled="actionBusy !== null"
                      @click="run('draw', (sg) => drawRecipient(sg, pool!.id))"
                    >
                      {{ actionBusy === 'draw' ? 'Confirm in wallet…' : 'Hold the draw' }}
                    </button>
                    <p v-if="drawReady" class="text-xs text-stone-600">Anyone in the pool can call this; no administrator is needed.</p>
                    <p v-else-if="!round.recipient" class="text-sm text-stone-600">
                      The draw can be held once all contributions are in. The winner comes from among those in the draw.
                    </p>
                  </template>

                  <template v-else-if="s.key === 'propose'">
                    <form
                      v-if="isRecipient && round.phase === 'AwaitingPurchase' && remaining > 0 && recipientPaid"
                      class="space-y-3 rounded-2xl bg-sand/60 p-4"
                      @submit.prevent="propose"
                    >
                      <div>
                        <label class="label" for="seller">The seller's Stellar address</label>
                        <input id="seller" v-model="sellerInput" class="input font-mono" type="text" placeholder="G…" autocomplete="off" readonly required />
                        <p v-if="sellerInput && !sellerValid" class="mt-1 text-xs text-rose-700">
                          Enter the allowed demo seller address registered in the pool.
                        </p>
                        <p v-else class="mt-1 text-xs text-stone-600">
                          The allowed seller set when the pool was created is filled in. The contract will not pay any other address.
                        </p>
                      </div>
                      <div>
                        <label class="label" for="doc">Purchase document (invoice or contract summary)</label>
                        <p class="mb-1 text-xs text-stone-600">A sample draft was filled in; you can replace it with a real document summary.</p>
                        <textarea id="doc" v-model="docInput" class="input min-h-20" placeholder="E.g. car/home, seller, total price, down payment if any and who paid, date, document number" required />
                        <p class="mt-1 text-xs text-stone-600">
                          The document itself is not written on-chain; only its SHA-256 digest is recorded.
                          The member proposing the purchase is responsible for the document content.
                        </p>
                      </div>
                      <button type="submit" class="btn-primary" :disabled="actionBusy !== null || !sellerValid || !docInput.trim()">
                        {{ actionBusy === 'propose' ? 'Confirm in wallet…' : 'Record the purchase' }}
                      </button>
                    </form>
                    <p v-else-if="round.phase === 'AwaitingPurchase' && allFunded && !round.seller && recipientPaid" class="text-sm text-stone-600">
                      All contributions are in. Waiting for the next member to propose the seller and purchase record.
                    </p>
                  </template>

                  <template v-else-if="s.key === 'pay'">
                    <button
                      v-if="canExecute"
                      type="button"
                      class="btn-primary btn-lg"
                      :disabled="actionBusy !== null"
                      @click="run('execute', (sg) => executeRound(sg, pool!.id))"
                    >
                      {{ actionBusy === 'execute' ? 'Confirm in wallet…' : 'Send the amount to the seller' }}
                    </button>
                    <p v-if="canExecute" class="text-xs text-stone-600">Anyone in the pool can call this; no administrator is needed.</p>
                  </template>
                </template>
              </div>
            </div>
          </li>
        </ol>

        <!-- Kuruluş süresi dolduysa iptal -->
        <div v-if="wallet.isConnected && simpleTerms === true && pool.status === 'Filling' && setupExpired" class="flex flex-wrap items-center gap-3">
          <button type="button" class="btn-danger" :disabled="actionBusy !== null" @click="run('cancel', (sg) => cancelUnstartedPool(sg, pool!.id))">
            {{ actionBusy === 'cancel' ? 'Confirm in wallet…' : 'Setup period ended: cancel' }}
          </button>
        </div>
        <p v-if="pool.status === 'Filling'" class="text-xs text-stone-600">
          Setup deadline: {{ new Date(pool.setupDeadline * 1000).toLocaleString('en-US') }}
        </p>

        <!-- Süre sonunda iptal -->
        <div v-if="wallet.isConnected && simpleTerms === true && pool.status === 'Active' && canAbort" class="flex flex-wrap items-center gap-3">
          <button type="button" class="btn-danger" :disabled="actionBusy !== null" @click="run('abort', (sg) => abortPool(sg, pool!.id))">
            {{ actionBusy === 'abort' ? 'Confirm in wallet…' : 'End the pool and open this round’s refunds' }}
          </button>
          <p class="text-sm text-stone-600">The contract checks the cancellation conditions; a deadline passing does not by itself start any asset transfer.</p>
        </div>

        <!-- İptal / Tamamlanma -->
        <div v-if="wallet.isConnected && simpleTerms === true && (pool.status === 'Aborted' || pool.status === 'Completed')" class="flex flex-wrap items-center gap-3">
          <button
            v-if="isMember && pool.status === 'Aborted' && myRefundable > 0n"
            type="button"
            class="btn-primary btn-lg"
            :disabled="actionBusy !== null"
            @click="run('refund', (sg) => claimRefund(sg, pool!.id))"
          >
            {{ actionBusy === 'refund' ? 'Confirm in wallet…' : `Claim your refund (${formatStroops(myRefundable)} ${token})` }}
          </button>
        </div>

        <p v-if="actionError" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ actionError }}</p>
        <p v-if="lastTx" class="pop rounded-2xl bg-sage-50 p-3 text-sm text-sage-800">
          ✓ Transaction completed:
          <a :href="explorerTx(lastTx)" target="_blank" rel="noopener noreferrer" class="font-mono underline">{{ lastTx.slice(0, 8) }}…</a>
          (view on Stellar Expert)
        </p>
      </section>

      <!-- PARA NEREDE? -->
      <section class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4" aria-label="Financial summary">
        <div v-reveal class="bento">
          <p class="text-xs font-semibold text-stone-600">Kontrattaki toplam {{ token }}</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums">{{ contractBalance === null ? '—' : formatStroops(contractBalance) }}</p>
          <p class="mt-1 text-xs text-stone-500">All pools; read directly from the chain</p>
        </div>
        <div v-reveal="1" class="bento">
          <p class="text-xs font-semibold text-stone-600">This round’s contributions</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums">{{ fundedCount }} / {{ pool.members.length }}</p>
          <p class="mt-1 text-xs text-stone-500">No allocation is made until all are in</p>
        </div>
        <div v-reveal="2" class="bento">
          <p class="text-xs font-semibold text-stone-600">Bu turdaki toplam iade</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums">{{ formatStroops(totalRefundable) }}</p>
          <p class="mt-1 text-xs text-stone-500">Payments from earlier rounds are not included</p>
        </div>
        <div v-reveal="3" class="bento">
          <p class="text-xs font-semibold text-stone-600">{{ countdownLabel }}</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums" :class="round && remaining <= 0 && pool.status === 'Active' ? 'text-amber-700' : ''">
            <template v-if="round && pool.status === 'Active' && roundDeadline > 0">
              {{ remaining <= 0 ? 'Time is up' : formatDuration(remaining) }}
            </template>
            <template v-else>—</template>
          </p>
        </div>
      </section>

      <!-- BU TURUN AYRINTISI -->
      <section v-if="round && pool.status === 'Active'" class="card space-y-4" aria-labelledby="tur">
        <h2 id="tur" class="text-xl font-extrabold">Details of this round</h2>
        <dl class="grid gap-3 sm:grid-cols-3">
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">This round’s recipient</dt>
            <dd class="mt-0.5 font-mono text-sm font-semibold" :title="round.recipient ?? ''">
              <template v-if="round.recipient">{{ shortAddress(round.recipient, 6) }}</template>
              <span v-else class="font-sans">Kura bekleniyor</span>
              <span v-if="isRecipient" class="badge bg-brand-100 text-brand-800">Sen</span>
            </dd>
          </div>
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">Amount going to the seller</dt>
            <dd class="mt-0.5 text-sm font-semibold">
              {{ formatStroops(purchaseAmount) }} {{ token }}
              <span v-if="pool.downPayment > 0n" class="block text-xs font-normal text-stone-600">
                pool {{ formatStroops(purchaseAmount - pool.downPayment) }} + recipient’s down payment {{ formatStroops(pool.downPayment) }}
              </span>
            </dd>
          </div>
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">Seller</dt>
            <dd class="mt-0.5 font-mono text-sm font-semibold" :title="round.seller ?? ''">
              {{ round.seller ? shortAddress(round.seller, 6) : 'Not proposed yet' }}
            </dd>
          </div>
        </dl>
        <div v-if="round.docHash" class="text-xs text-stone-600">
          Purchase document digest (SHA-256): <span class="font-mono break-all">{{ round.docHash }}</span>
        </div>
      </section>

      <!-- ÜYELER -->
      <section class="card" aria-labelledby="uyeler">
        <h2 id="uyeler" class="text-xl font-extrabold">{{ isDraw ? 'Members' : 'Members and order' }}</h2>
        <p v-if="isDraw && pool.status === 'Active'" class="mt-1 text-sm text-stone-600">
          Received: <strong>{{ excludedFromDraw.length }}</strong> · In the draw: <strong>{{ drawCandidates.length }}</strong>
        </p>
        <ul class="mt-3 space-y-2">
          <li
            v-for="(addr, idx) in visibleMembers"
            :key="addr"
            class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-stone-200/80 bg-white p-3 transition-colors duration-200"
            :class="round && addr === round.recipient && pool.status === 'Active' ? '!border-brand-300 bg-brand-50/60' : ''"
          >
            <div class="flex flex-wrap items-center gap-2.5">
              <span class="grid size-9 place-items-center rounded-full bg-sand text-sm font-extrabold text-ink-soft">{{ idx + 1 }}</span>
              <span class="font-mono text-sm" :title="addr">{{ shortAddress(addr, 6) }}</span>
              <span v-if="addr === me" class="badge bg-brand-100 text-brand-800">Sen</span>
              <span v-if="addr === pool.creator" class="badge bg-stone-100 text-stone-700">Kurucu</span>
              <span v-if="round && addr === round.recipient && pool.status === 'Active'" class="badge bg-sage-100 text-sage-800">This round’s recipient</span>
              <span v-if="memberByAddress.get(addr)?.received" class="badge bg-stone-100 text-stone-700">Received their share</span>
              <span
                v-else-if="isDraw && pool.status === 'Active'"
                class="badge bg-gold-100 text-amber-900"
              >Kurada</span>
            </div>
            <div class="flex flex-wrap items-center gap-3">
              <span v-if="pool.status === 'Aborted'" class="text-xs text-stone-600">Refund entitlement: {{ formatStroops(refundableOf(addr)) }} {{ token }}</span>
              <span class="badge" :class="toneClass[memberState(addr).tone]">{{ memberState(addr).label }}</span>
            </div>
          </li>
        </ul>
        <button
          v-if="listedMembers.length > COMPACT_LIST"
          type="button"
          class="btn-secondary mt-3 !min-h-10"
          :aria-expanded="showAllMembers"
          @click="showAllMembers = !showAllMembers"
        >
          {{ showAllMembers ? 'Show fewer' : `Show all (${listedMembers.length})` }}
        </button>
        <p v-if="!listedMembers.length" class="mt-3 text-sm text-stone-600">No members have joined yet. You can join as the first member.</p>
        <p v-if="pool.status === 'Filling' && !isDraw" class="mt-3 text-xs text-stone-600">
          The fixed delivery order is the join order. When the pool is full, everyone approves the terms.
        </p>
      </section>

      <!-- KELİMELER -->
      <details class="card group cursor-pointer !p-0">
        <summary class="flex min-h-14 list-none items-center justify-between gap-3 px-5 py-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
          What do the terms mean?
          <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
        </summary>
        <dl class="grid gap-3 px-5 pb-5 text-sm sm:grid-cols-2">
          <div><dt class="font-semibold">Risk</dt><dd class="text-stone-600">If someone who received early stops contributing, the earlier-round payments of those still waiting cannot be recovered automatically.</dd></div>
          <div><dt class="font-semibold">Purchase record</dt><dd class="text-stone-600">The next recipient records the registered demo seller and the document digest on-chain.</dd></div>
          <div><dt class="font-semibold">Grace period</dt><dd class="text-stone-600">The last chance given to a late member after the contribution period has ended.</dd></div>
          <div><dt class="font-semibold">Refund entitlement</dt><dd class="text-stone-600">The contribution you personally paid in a round not yet paid out to the seller.</dd></div>
          <div v-if="isDraw"><dt class="font-semibold">Draw</dt><dd class="text-stone-600">Once all contributions are in, the contract picks the recipient among members who have not yet received. Randomness is hackathon-grade.</dd></div>
          <div><dt class="font-semibold">Terms version</dt><dd class="text-stone-600">The installment, schedule and delivery method recorded when the pool fills. All members approve before it starts.</dd></div>
        </dl>
      </details>

      <p class="px-2 text-center text-sm text-stone-600">
        Everything on this page is read directly from the contract on the Stellar network.
        <a v-if="poolContractId" :href="explorerContract(poolContractId)" target="_blank" rel="noopener noreferrer" class="text-brand-700 underline">
          View the contract on the blockchain
        </a>
      </p>
    </template>
  </div>
</template>
