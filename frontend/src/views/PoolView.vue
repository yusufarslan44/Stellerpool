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
  approvePurchase,
  approveTerms,
  cancelUnstartedPool,
  claimRefund,
  curePayment,
  deposit,
  drawRecipient,
  executeRound,
  getMemberStatus,
  getPool,
  getRound,
  joinPool,
  markOverdue,
  proposePurchase,
  proposeTerms,
  startPool,
} from '@/services/pool'
import type { Signer } from '@/services/pool'
import { useWalletStore } from '@/stores/wallet'
import { MAX_VERIFIERS, MIN_VERIFIERS } from '@/types/pool'
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
/** Havuz başlamadan önce kurucunun belirleyeceği tahsisat sırası. */
const order = ref<string[]>([])
const verifiersInput = ref('')
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
      // Başlamadan önce sıra = katılım sırası; kurucu değiştirebilir.
      if (order.value.length !== p.members.length || !order.value.every((m) => p.members.includes(m))) {
        order.value = p.recipientOrder.length === p.members.length ? [...p.recipientOrder] : [...p.members]
      }
      if (!verifiersInput.value && p.verifiers.length) verifiersInput.value = p.verifiers.join('\n')
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
  void load()
  poll = setInterval(() => void load(true), 10_000)
})
onUnmounted(() => clearInterval(poll))
watch(poolId, () => void load())

// --- Roller ------------------------------------------------------------------------------
const me = computed(() => wallet.address)
const isMember = computed(() => !!me.value && !!pool.value?.members.includes(me.value))
const isCreator = computed(() => !!me.value && pool.value?.creator === me.value)
const isVerifier = computed(() => !!me.value && !!pool.value?.verifiers.includes(me.value))
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
      return 'Eksik katkı var. Ek süre sonunda tur durur ve kura çekilmez; yalnızca bu turda yatırılmış katkılar iade edilebilir.'
    }
    return null
  }
  if (!recipientPaid.value && (phase === 'Grace' || deadlinePassed.value)) {
    return 'Sıradaki üye kendi katkısını yatırmadı. Ek süre sonunda tur durur; yalnızca bu turda yatırılmış katkılar iade edilebilir.'
  }
  return null
})

const approvalsCount = computed(() => round.value?.approvals.length ?? 0)
const approvedByMe = computed(() => !!me.value && !!round.value?.approvals.includes(me.value))
const approvalsOk = computed(
  () => !!pool.value && approvalsCount.value >= pool.value.approvalThreshold,
)
const canExecute = computed(() =>
  round.value?.phase === 'AwaitingPurchase' &&
  allFunded.value && recipientPaid.value &&
  !!round.value.seller && approvalsOk.value && now.value < round.value.purchaseDeadline,
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
  if (pool.value?.status === 'Completed' || round.value?.phase === 'Settled') return 0n
  return memberByAddress.value.get(address)?.refundable ?? 0n
}
const totalRefundable = computed(() => members.value.reduce((sum, m) => sum + refundableOf(m.address), 0n))
const myRefundable = computed(() => (me.value ? refundableOf(me.value) : 0n))

const proposedVerifiers = computed(() => verifiersInput.value.split(/[\s,;]+/).map((v) => v.trim()).filter(Boolean))
const verifiersValid = computed(() => {
  if (!pool.value || proposedVerifiers.value.length < MIN_VERIFIERS || proposedVerifiers.value.length > MAX_VERIFIERS) return false
  const excluded = new Set([pool.value.creator, ...pool.value.members])
  return new Set(proposedVerifiers.value).size === proposedVerifiers.value.length &&
    proposedVerifiers.value.every((v) => StrKey.isValidEd25519PublicKey(v) && !excluded.has(v))
})
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
  if (!p || p.status === 'Filling') return { label: 'Katıldı', tone: 'slate' as const }
  if (p.status === 'Completed' || p.status === 'Aborted') return { label: '—', tone: 'slate' as const }
  if (paid.value.has(address)) return { label: 'Ödedi ✓', tone: 'green' as const }
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
      return { text: 'Üyeler bekleniyor', cls: 'bg-stone-100 text-stone-700' }
    case 'Active':
      if (round.value?.phase === 'AwaitingDraw') return { text: 'Kura bekleniyor', cls: 'bg-gold-100 text-amber-900' }
      return round.value?.phase === 'Grace'
        ? { text: 'Ek sürede', cls: 'bg-gold-100 text-amber-900' }
        : { text: 'Devam ediyor', cls: 'bg-brand-100 text-brand-800' }
    case 'Completed':
      return { text: 'Tamamlandı', cls: 'bg-sage-100 text-sage-800' }
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
    amount: round.value!.pot,
    docHash: hash,
  }))
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

const listedMembers = computed(() => {
  if (isDraw.value) return pool.value?.members ?? []
  return pool.value?.status === 'Filling' && isCreator.value
    ? order.value
    : (pool.value?.recipientOrder.length ? pool.value.recipientOrder : (pool.value?.members ?? []))
})
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
  const partyCount = p.members.length
  return [
    { key: 'join', title: 'Üyeler katılır', who: 'Üyeler', detail: `${p.members.length} / ${p.memberLimit} üye`, done: isFull.value },
    {
      key: 'terms',
      title: isDraw.value ? 'Doğrulayıcılar önerilir' : 'Sıra ve doğrulayıcılar önerilir',
      who: 'Kurucu',
      detail: p.termsVersion ? `Koşul sürümü ${p.termsVersion}` : 'Henüz önerilmedi',
      done: isFull.value && p.termsVersion > 0,
    },
    {
      key: 'approve',
      title: 'Herkes koşulları onaylar',
      who: 'Üyeler',
      detail: `${p.termsApprovals.length} / ${partyCount} onay`,
      done: termsReady.value,
    },
    { key: 'start', title: 'Havuz başlar', who: 'Herkes başlatabilir', detail: 'Koşullar tamamsa tek tıkla', done: false },
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
          title: 'Kura çekilir',
          who: 'Herkes çağırabilir',
          detail: r.recipient
            ? `Kazanan: ${shortAddress(r.recipient, 6)}${r.recipient === me.value ? ' (sen)' : ''}`
            : `${drawCandidates.value.length} üye kurada`,
          done: settled || !!r.recipient,
        },
      ]
    : []
  return [
    {
      key: 'collect',
      title: 'Katkılar toplanır',
      who: 'Üyeler',
      detail: `${fundedCount.value} / ${p.members.length} tamam`,
      done: settled || allFunded.value,
    },
    ...draw,
    {
      key: 'propose',
      title: 'Satıcı ve alım belgesi önerilir',
      who: isDraw.value ? 'Kura kazananı' : 'Sıradaki üye',
      detail: r.seller ? 'Öneri kaydedildi' : 'Bekleniyor',
      done: settled || !!r.seller,
    },
    {
      key: 'verify',
      title: 'Doğrulayıcılar onaylar',
      who: 'Doğrulayıcılar',
      detail: `${approvalsCount.value} / ${p.approvalThreshold} onay`,
      done: settled || approvalsOk.value,
    },
    { key: 'pay', title: 'Tutar satıcıya gider', who: 'Herkes çağırabilir', detail: `${formatStroops(r.pot)} ${token.value}`, done: settled },
  ]
})

const guideSteps = computed<GuideStep[]>(() =>
  pool.value?.status === 'Filling' ? setupSteps.value : pool.value?.status === 'Active' ? roundSteps.value : [],
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
    terms: isCreator.value && isFull.value && !setupExpired.value,
    approve: canApproveTerms.value && !setupExpired.value,
    start: isFull.value && termsReady.value && !setupExpired.value,
    draw: drawReady.value && isMember.value,
    collect:
      isMember.value && (r?.phase === 'Collecting' || r?.phase === 'Grace') && !myFunded.value,
    propose:
      isRecipient.value && purchasing && !r?.seller && recipientPaid.value && remaining.value > 0,
    verify: isVerifier.value && purchasing && !!r?.seller && !approvedByMe.value && remaining.value > 0,
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
  if (p.status === 'Completed') return { tone: 'done', title: 'Havuz tamamlandı', text: 'Tüm turlar satıcılara ödendi.' }
  if (p.status === 'Aborted') {
    return {
      tone: 'warn',
      title: 'Havuz iptal edildi',
      text:
        isMember.value && myRefundable.value > 0n
          ? `İade hakkın var: ${formatStroops(myRefundable.value)} ${token.value}. Aşağıdan iadeni alabilirsin.`
          : 'Yalnızca ödenmemiş turdaki kendi katkıları geri alınabilir. Önceki turların ödemesi geri alınamaz.',
    }
  }
  if (!wallet.isConnected) {
    return { tone: 'wait', title: 'Önce cüzdanını bağla', text: 'Bu havuzda ne yapabileceğini görmek için sağ üstten cüzdanını bağla.' }
  }
  if (p.status === 'Filling' && setupExpired.value) {
    return { tone: 'warn', title: 'Kuruluş süresi doldu', text: 'Havuz zamanında başlamadı. Aşağıdan iptal edilebilir.' }
  }
  if (recipientBlock.value) return { tone: 'warn', title: 'Bu tur şu an ilerleyemez', text: recipientBlock.value }
  if (canAbort.value) {
    return { tone: 'warn', title: 'Süre doldu', text: 'Havuz sonlandırılabilir. Yalnızca henüz satıcıya ödenmemiş turun katkıları iade edilir.' }
  }
  const open = guideSteps.value.filter((s) => !s.done)
  const mineStep = open.find((s) => stepMine.value[s.key])
  if (mineStep) return { tone: 'mine', title: 'Sıra sende', text: mineStep.title }
  const wait = open[0]
  if (wait) return { tone: 'wait', title: `Şu an beklenen: ${wait.who}`, text: wait.title }
  return null
})

const bannerIllo = { mine: 'sparkles', warn: 'warning', done: 'party', wait: 'hourglass' } as const
const STEP_ILLO = {
  join: 'handshake',
  terms: 'memo',
  approve: 'check',
  start: 'rocket',
  collect: 'moneybag',
  draw: 'dice',
  propose: 'receipt',
  verify: 'magnifier',
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
  if (isMember.value) chips.push('Üye')
  if (isVerifier.value) chips.push('Doğrulayıcı')
  if (isRecipient.value && pool.value?.status === 'Active') chips.push('Bu turun alıcısı')
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
  round.value?.phase === 'Grace' ? 'Ek süre'
    : round.value?.phase === 'AwaitingDraw' ? 'Kura ve alım süresi'
    : round.value?.phase === 'AwaitingPurchase' ? 'Alım onayı süresi' : 'Katkı süresi',
)

</script>

<template>
  <div class="space-y-6">
    <RouterLink to="/" class="inline-flex items-center gap-1 text-sm font-medium text-brand-700 hover:underline">
      <AppIcon name="back" class="!size-4" /> Ana sayfa
    </RouterLink>

    <p
      v-if="!poolContractId"
      role="status"
      class="flex items-start gap-3 rounded-2xl border border-gold-300/60 bg-gold-100/70 p-4 text-sm text-amber-950"
    >
      <Illo name="bulb" :size="28" />
      <span>
        Havuz sözleşmesi henüz yapılandırılmadı, bu yüzden havuz verisi okunamıyor. Sözleşme
        yayınlanıp adresi <code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> olarak
        eklenince bu sayfa gerçek veriyi gösterecek.
      </span>
    </p>

    <div v-else-if="loading" class="space-y-4" aria-live="polite">
      <div class="flex items-center gap-3 text-stone-600">
        <CoinSpinner :size="40" /> Havuz zincirden okunuyor…
      </div>
      <div class="skeleton h-48 rounded-[2rem]" />
      <div class="grid gap-4 sm:grid-cols-3">
        <div class="skeleton h-24 rounded-3xl" />
        <div class="skeleton h-24 rounded-3xl" />
        <div class="skeleton h-24 rounded-3xl" />
      </div>
    </div>

    <div v-else-if="loadError" role="alert" class="card space-y-3">
      <h1 class="text-2xl font-extrabold">Havuz #{{ poolId }} okunamadı</h1>
      <p class="text-sm text-rose-700">{{ loadError }}</p>
      <button type="button" class="btn-secondary" @click="load()">Tekrar dene</button>
    </div>

    <template v-else-if="pool">
      <!-- BAŞLIK + 3B SAHNE -->
      <header class="sunrise relative overflow-hidden rounded-[2rem] p-5 sm:p-8">
        <div class="grid items-center gap-4 md:grid-cols-[1.3fr_1fr]">
          <div>
            <div class="flex flex-wrap items-center gap-2">
              <span class="badge" :class="statusLabel.cls">{{ statusLabel.text }}</span>
              <span class="badge bg-gold-100 text-amber-900">{{ isDraw ? 'Kura' : 'Sabit sıra' }}</span>
              <span v-for="c in roleChips" :key="c" class="badge bg-ink text-cream">Sen: {{ c }}</span>
            </div>
            <h1 class="mt-3 text-4xl font-extrabold sm:text-5xl">Havuz #{{ pool.id }}</h1>
            <p class="mt-2 text-stone-700">
              {{ pool.members.length }} / {{ pool.memberLimit }} üye · her tur
              <strong>{{ formatStroops(pool.contributionAmount) }} {{ token }}</strong>
            </p>
            <p class="mt-1 text-sm text-stone-600">
              Her tur herkes kendi katkısını yatırır. Ödeme eksikse tahsisat yapılmaz.
              {{ isDraw ? 'Alıcı, tüm katkılar gelince kura ile belirlenir.' : '' }}
            </p>
            <div class="mt-5 flex flex-wrap gap-2">
              <button type="button" class="btn-secondary !min-h-10" @click="copyLink">
                <AppIcon :name="copied ? 'check' : 'copy'" class="!size-4" />
                {{ copied ? 'Kopyalandı' : 'Bağlantıyı kopyala' }}
              </button>
              <a
                v-if="poolContractId"
                :href="explorerContract(poolContractId)"
                target="_blank"
                rel="noopener noreferrer"
                class="btn-secondary !min-h-10"
              >
                Zincirde gör <AppIcon name="external" class="!size-4" />
              </a>
            </div>
          </div>
          <div class="relative h-44 sm:h-56">
            <Scene3D
              :coins="sceneCoins"
              :filled="sceneFilled"
              :label="`${pool.memberLimit} üyeli havuz; ${sceneFilled < 0 ? 'hepsi' : sceneFilled} para altın`"
            />
          </div>
        </div>

        <div v-if="pool.status === 'Active'" class="mt-5">
          <div class="mb-1 flex justify-between text-xs font-semibold text-stone-700">
            <span>Tur {{ pool.currentRound }} / {{ pool.memberLimit }}</span>
            <span class="tabular-nums">%{{ progressPercent }} tamamlandı</span>
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
            {{ pool.status === 'Filling' ? 'Havuz nasıl başlar?' : pool.status === 'Active' ? `Tur ${pool.currentRound}: adım adım` : 'Durum' }}
          </h2>
          <span v-if="pool.status === 'Active' && roundDeadline > 0" class="badge bg-brand-50 text-brand-800 ring-1 ring-brand-100">
            <AppIcon name="clock" class="!size-3.5" />
            {{ countdownLabel }}:
            <span class="tabular-nums" :class="remaining <= 0 ? 'text-rose-700' : ''">
              {{ remaining <= 0 ? 'süre doldu' : formatDuration(remaining) }}
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
            <AppIcon name="wallet" class="!size-4" /> Cüzdan bağla
          </button>
        </div>

        <!-- Adım listesi: her adımın kendi işlemi kendi içinde -->
        <ol v-if="guideSteps.length" class="space-y-0" aria-label="Adımlar">
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
                <span v-if="stepMine[s.key] && !s.done" class="badge bg-brand-600 text-white">Sıra sende</span>
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
                  aria-label="Bu turda ödeyen üyeler"
                >
                  <div class="h-full rounded-full bg-brand-500 transition-[width] duration-500" :style="{ width: `${(fundedCount / pool.members.length) * 100}%` }" />
                </div>
                <p class="mt-1 text-xs text-stone-600">{{ fundedCount }} / {{ pool.members.length }} üye ödedi</p>
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
                    {{ actionBusy === 'join' ? 'Cüzdanı onayla…' : 'Havuza katıl' }}
                  </button>
                  <p v-if="s.key === 'join' && isMember" class="text-sm font-medium text-sage-800">
                    ✓ Katıldın.
                    <template v-if="!isFull">Kalan {{ pool.memberLimit - pool.members.length }} üye bekleniyor.</template>
                  </p>

                  <div v-if="s.key === 'terms' && isCreator && isFull && !setupExpired" class="space-y-2 rounded-2xl bg-sand/60 p-4">
                    <p class="text-sm text-stone-700">
                      {{ isDraw ? 'Bu havuzda sıra yok: alıcı kura ile belirlenir. Doğrulayıcıları yaz.' : 'Sırayı aşağıdaki “Üyeler ve sıra” listesinden ↑ ↓ ile düzenle, sonra doğrulayıcıları yaz.' }}
                    </p>
                    <label class="label" for="term-verifiers">Demo doğrulayıcıları (her satıra bir adres)</label>
                    <textarea id="term-verifiers" v-model="verifiersInput" class="input min-h-24 font-mono" placeholder="G…&#10;G…&#10;G…" />
                    <p class="text-xs text-stone-600">{{ MIN_VERIFIERS }}–{{ MAX_VERIFIERS }} farklı adres; kurucu veya üye olamaz. Yeni öneri önceki onayları sıfırlar.</p>
                    <p v-if="proposedVerifiers.length && !verifiersValid" class="text-xs text-rose-700">Doğrulayıcı adreslerini kontrol et.</p>
                    <button
                      type="button"
                      class="btn-primary"
                      :disabled="actionBusy !== null || !verifiersValid"
                      @click="run('terms', (sg) => proposeTerms(sg, pool!.id, isDraw ? [] : order, proposedVerifiers))"
                    >
                      {{ actionBusy === 'terms' ? 'Cüzdanı onayla…' : isDraw ? 'Doğrulayıcıları öner' : 'Sıra ve doğrulayıcıları öner' }}
                    </button>
                  </div>

                  <button
                    v-if="s.key === 'approve' && canApproveTerms && !setupExpired"
                    type="button"
                    class="btn-primary"
                    :disabled="actionBusy !== null"
                    @click="run('approve-terms', (sg) => approveTerms(sg, pool!.id, pool!.termsVersion))"
                  >
                    {{ actionBusy === 'approve-terms' ? 'Cüzdanı onayla…' : `Koşul sürümü ${pool.termsVersion} onayla` }}
                  </button>

                  <button
                    v-if="s.key === 'start' && isFull && termsReady && !setupExpired"
                    type="button"
                    class="btn-primary btn-lg"
                    :disabled="actionBusy !== null"
                    @click="run('start', (sg) => startPool(sg, pool!.id))"
                  >
                    {{ actionBusy === 'start' ? 'Cüzdanı onayla…' : 'Havuzu başlat' }}
                  </button>
                </template>

                <!-- AKTİF TUR -->
                <template v-else-if="pool.status === 'Active' && round">
                  <template v-if="s.key === 'collect'">
                    <p v-if="round.phase === 'Grace'" role="status" class="rounded-2xl bg-gold-100/80 p-3 text-sm text-amber-950">
                      Tur ek sürede. Eksik katkıyı yalnızca ilgili üye kendi cüzdanından yatırabilir.
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
                        {{ actionBusy === 'deposit' ? 'Cüzdanı onayla…' : `Bu turun katkısını öde (${formatStroops(pool.contributionAmount)} ${token})` }}
                      </button>
                      <p v-if="isMember && myOwnPaid" class="text-sm font-medium text-sage-800">
                        ✓ Bu turun katkısını kendin ödedin
                      </p>
                      <button
                        v-if="deadlinePassed && !allFunded"
                        type="button"
                        class="btn-danger"
                        :disabled="actionBusy !== null"
                        @click="run('overdue', (sg) => markOverdue(sg, pool!.id))"
                      >
                        {{ actionBusy === 'overdue' ? 'Cüzdanı onayla…' : 'Süre doldu, turu durdur' }}
                      </button>
                    </div>
                    <p v-if="deadlinePassed && !allFunded" class="text-xs text-stone-600">
                      Bu işlemi havuzdaki herkes çağırabilir, bir yöneticiye gerek yok.
                    </p>

                    <!-- Anchor ile bakiye yükleme: katkıdan önceki gerçek fiat-kapısı adımı -->
                    <details
                      v-if="usesPoolAsset && isMember && !myFunded && (round.phase === 'Collecting' || round.phase === 'Grace')"
                      class="group rounded-2xl border border-stone-200 bg-white p-4"
                      :open="lowBalance"
                    >
                      <summary class="flex min-h-11 cursor-pointer list-none items-center justify-between gap-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
                        <span>
                          Bakiyen yetmiyor mu? Anchor ile {{ token }} yükle
                          <span v-if="myBalance !== null" class="ml-1 text-xs font-normal text-stone-600">
                            (bakiyen {{ formatStroops(myBalance) }} {{ token }})
                          </span>
                        </span>
                        <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
                      </summary>
                      <div class="mt-3">
                        <AnchorDemo compact @completed="loadMyBalance" />
                        <p v-if="pool.contributionAmount > 100_000_000n" class="mt-2 text-xs text-stone-600">
                          Anchor işlem başına sınır koyabilir; katkın büyükse birkaç kez yüklemen gerekebilir.
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
                      {{ actionBusy === 'draw' ? 'Cüzdanı onayla…' : 'Kurayı çek' }}
                    </button>
                    <p v-if="drawReady" class="text-xs text-stone-600">Bu işlemi havuzdaki herkes çağırabilir, bir yöneticiye gerek yok.</p>
                    <p v-else-if="!round.recipient" class="text-sm text-stone-600">
                      Tüm katkılar gelince kura çekilebilir. Kazanan, kuraya girenler arasından çıkar.
                    </p>
                  </template>

                  <template v-else-if="s.key === 'propose'">
                    <form
                      v-if="isRecipient && round.phase === 'AwaitingPurchase' && remaining > 0 && recipientPaid"
                      class="space-y-3 rounded-2xl bg-sand/60 p-4"
                      @submit.prevent="propose"
                    >
                      <div>
                        <label class="label" for="seller">Satıcının Stellar adresi</label>
                        <input id="seller" v-model="sellerInput" class="input font-mono" type="text" placeholder="G…" autocomplete="off" required />
                        <p v-if="sellerInput && !sellerValid" class="mt-1 text-xs text-rose-700">
                          Havuzda kayıtlı izinli demo satıcısı adresini gir.
                        </p>
                      </div>
                      <div>
                        <label class="label" for="doc">Alım belgesi (fatura veya sözleşme özeti)</label>
                        <textarea id="doc" v-model="docInput" class="input min-h-20" placeholder="Örn. araç/ev, satıcı, tutar, tarih, belge numarası" required />
                        <p class="mt-1 text-xs text-stone-600">
                          Belgenin kendisi zincire yazılmaz, yalnızca SHA-256 özeti kaydedilir. Doğrulayıcılar belgeyi
                          zincir dışında kontrol eder.
                        </p>
                      </div>
                      <button type="submit" class="btn-primary" :disabled="actionBusy !== null || !sellerValid || !docInput.trim()">
                        {{ actionBusy === 'propose' ? 'Cüzdanı onayla…' : 'Öneriyi kaydet' }}
                      </button>
                    </form>
                    <p v-else-if="round.phase === 'AwaitingPurchase' && allFunded && !round.seller && recipientPaid" class="text-sm text-stone-600">
                      Katkılar tam. Sıradaki üyenin satıcıyı ve alım kaydını önermesi bekleniyor.
                    </p>
                  </template>

                  <template v-else-if="s.key === 'verify'">
                    <button
                      v-if="isVerifier && round.phase === 'AwaitingPurchase' && round.seller && !approvedByMe && remaining > 0"
                      type="button"
                      class="btn-primary"
                      :disabled="actionBusy !== null"
                      @click="run('approve', (sg) => approvePurchase(sg, pool!.id, round!.round, round!.purchaseVersion))"
                    >
                      {{ actionBusy === 'approve' ? 'Cüzdanı onayla…' : 'Alım kaydını onayla' }}
                    </button>
                  </template>

                  <template v-else-if="s.key === 'pay'">
                    <button
                      v-if="canExecute"
                      type="button"
                      class="btn-primary btn-lg"
                      :disabled="actionBusy !== null"
                      @click="run('execute', (sg) => executeRound(sg, pool!.id))"
                    >
                      {{ actionBusy === 'execute' ? 'Cüzdanı onayla…' : 'Tutarı satıcıya gönder' }}
                    </button>
                    <p v-if="canExecute" class="text-xs text-stone-600">Bu işlemi havuzdaki herkes çağırabilir, bir yöneticiye gerek yok.</p>
                  </template>
                </template>
              </div>
            </div>
          </li>
        </ol>

        <!-- Kuruluş süresi dolduysa iptal -->
        <div v-if="wallet.isConnected && pool.status === 'Filling' && setupExpired" class="flex flex-wrap items-center gap-3">
          <button type="button" class="btn-danger" :disabled="actionBusy !== null" @click="run('cancel', (sg) => cancelUnstartedPool(sg, pool!.id))">
            {{ actionBusy === 'cancel' ? 'Cüzdanı onayla…' : 'Kuruluş süresi doldu: iptal et' }}
          </button>
        </div>
        <p v-if="pool.status === 'Filling'" class="text-xs text-stone-600">
          Kuruluş son tarihi: {{ new Date(pool.setupDeadline * 1000).toLocaleString('tr-TR') }}
        </p>

        <!-- Süre sonunda iptal -->
        <div v-if="wallet.isConnected && pool.status === 'Active' && canAbort" class="flex flex-wrap items-center gap-3">
          <button type="button" class="btn-danger" :disabled="actionBusy !== null" @click="run('abort', (sg) => abortPool(sg, pool!.id))">
            {{ actionBusy === 'abort' ? 'Cüzdanı onayla…' : 'Havuzu sonlandır ve bu turun iadelerini aç' }}
          </button>
          <p class="text-sm text-stone-600">İptal şartlarını kontrat denetler; süre dolması tek başına varlık transferi başlatmaz.</p>
        </div>

        <!-- İptal / Tamamlanma -->
        <div v-if="wallet.isConnected && (pool.status === 'Aborted' || pool.status === 'Completed')" class="flex flex-wrap items-center gap-3">
          <button
            v-if="isMember && pool.status === 'Aborted' && myRefundable > 0n"
            type="button"
            class="btn-primary btn-lg"
            :disabled="actionBusy !== null"
            @click="run('refund', (sg) => claimRefund(sg, pool!.id))"
          >
            {{ actionBusy === 'refund' ? 'Cüzdanı onayla…' : `İadeni al (${formatStroops(myRefundable)} ${token})` }}
          </button>
        </div>

        <p v-if="actionError" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ actionError }}</p>
        <p v-if="lastTx" class="pop rounded-2xl bg-sage-50 p-3 text-sm text-sage-800">
          ✓ İşlem tamamlandı:
          <a :href="explorerTx(lastTx)" target="_blank" rel="noopener noreferrer" class="font-mono underline">{{ lastTx.slice(0, 8) }}…</a>
          (Stellar Expert’te görüntüle)
        </p>
      </section>

      <!-- PARA NEREDE? -->
      <section class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4" aria-label="Finansal özet">
        <div v-reveal class="bento">
          <p class="text-xs font-semibold text-stone-600">Kontrattaki toplam {{ token }}</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums">{{ contractBalance === null ? '—' : formatStroops(contractBalance) }}</p>
          <p class="mt-1 text-xs text-stone-500">Tüm havuzlar; doğrudan zincirden okunur</p>
        </div>
        <div v-reveal="1" class="bento">
          <p class="text-xs font-semibold text-stone-600">Bu turun katkıları</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums">{{ fundedCount }} / {{ pool.members.length }}</p>
          <p class="mt-1 text-xs text-stone-500">Tümü gelmeden tahsisat yapılmaz</p>
        </div>
        <div v-reveal="2" class="bento">
          <p class="text-xs font-semibold text-stone-600">Bu turdaki toplam iade</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums">{{ formatStroops(totalRefundable) }}</p>
          <p class="mt-1 text-xs text-stone-500">Önceki turlardaki ödemeler dahil değil</p>
        </div>
        <div v-reveal="3" class="bento">
          <p class="text-xs font-semibold text-stone-600">{{ countdownLabel }}</p>
          <p class="mt-1 font-display text-2xl font-extrabold tabular-nums" :class="round && remaining <= 0 && pool.status === 'Active' ? 'text-amber-700' : ''">
            <template v-if="round && pool.status === 'Active' && roundDeadline > 0">
              {{ remaining <= 0 ? 'Süre doldu' : formatDuration(remaining) }}
            </template>
            <template v-else>—</template>
          </p>
        </div>
      </section>

      <!-- BU TURUN AYRINTISI -->
      <section v-if="round && pool.status === 'Active'" class="card space-y-4" aria-labelledby="tur">
        <h2 id="tur" class="text-xl font-extrabold">Bu turun ayrıntısı</h2>
        <dl class="grid gap-3 sm:grid-cols-3">
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">Bu turun alıcısı</dt>
            <dd class="mt-0.5 font-mono text-sm font-semibold" :title="round.recipient ?? ''">
              <template v-if="round.recipient">{{ shortAddress(round.recipient, 6) }}</template>
              <span v-else class="font-sans">Kura bekleniyor</span>
              <span v-if="isRecipient" class="badge bg-brand-100 text-brand-800">Sen</span>
            </dd>
          </div>
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">Satıcıya gidecek tutar</dt>
            <dd class="mt-0.5 text-sm font-semibold">{{ formatStroops(round.pot) }} {{ token }}</dd>
          </div>
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">Satıcı</dt>
            <dd class="mt-0.5 font-mono text-sm font-semibold" :title="round.seller ?? ''">
              {{ round.seller ? shortAddress(round.seller, 6) : 'Henüz önerilmedi' }}
            </dd>
          </div>
        </dl>
        <div v-if="round.docHash" class="text-xs text-stone-600">
          Alım belgesi özeti (SHA-256): <span class="font-mono break-all">{{ round.docHash }}</span>
        </div>
        <div v-if="round.seller" class="text-sm">
          <p class="flex flex-wrap items-center gap-2 font-semibold">
            Doğrulayıcı onayı: {{ approvalsCount }} / {{ pool.approvalThreshold }}
            <span v-if="approvalsOk" class="badge bg-sage-100 text-sage-800">Onaylandı ✓</span>
          </p>
          <ul class="mt-2 space-y-1.5">
            <li v-for="v in pool.verifiers" :key="v" class="flex flex-wrap items-center gap-2 text-xs">
              <span class="font-mono" :title="v">{{ shortAddress(v, 6) }}</span>
              <span v-if="v === me" class="badge bg-brand-100 text-brand-800">Sen</span>
              <span class="badge" :class="round.approvals.includes(v) ? toneClass.green : toneClass.slate">
                {{ round.approvals.includes(v) ? 'Onayladı ✓' : 'Bekliyor' }}
              </span>
            </li>
          </ul>
        </div>
      </section>

      <!-- ÜYELER -->
      <section class="card" aria-labelledby="uyeler">
        <h2 id="uyeler" class="text-xl font-extrabold">{{ isDraw ? 'Üyeler' : 'Üyeler ve sıra' }}</h2>
        <p v-if="isDraw && pool.status === 'Active'" class="mt-1 text-sm text-stone-600">
          Teslim aldı: <strong>{{ excludedFromDraw.length }}</strong> · Kurada: <strong>{{ drawCandidates.length }}</strong>
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
              <span v-if="round && addr === round.recipient && pool.status === 'Active'" class="badge bg-sage-100 text-sage-800">Bu turun alıcısı</span>
              <span v-if="memberByAddress.get(addr)?.received" class="badge bg-stone-100 text-stone-700">Payını aldı</span>
              <span
                v-else-if="isDraw && pool.status === 'Active'"
                class="badge bg-gold-100 text-amber-900"
              >Kurada</span>
            </div>
            <div class="flex flex-wrap items-center gap-3">
              <span class="text-xs text-stone-600">Bu turdaki iade: {{ formatStroops(refundableOf(addr)) }} {{ token }}</span>
              <span class="badge" :class="toneClass[memberState(addr).tone]">{{ memberState(addr).label }}</span>
              <span v-if="pool.status === 'Filling' && isCreator && !isDraw" class="flex gap-1">
                <button type="button" class="btn-secondary !min-h-9 !min-w-9 !px-2 !py-1" :disabled="idx === 0" :aria-label="`${shortAddress(addr)} adresini yukarı taşı`" @click="move(idx, -1)">↑</button>
                <button type="button" class="btn-secondary !min-h-9 !min-w-9 !px-2 !py-1" :disabled="idx === order.length - 1" :aria-label="`${shortAddress(addr)} adresini aşağı taşı`" @click="move(idx, 1)">↓</button>
              </span>
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
          {{ showAllMembers ? 'Listeyi kısalt' : `Tümünü göster (${listedMembers.length})` }}
        </button>
        <p v-if="!listedMembers.length" class="mt-3 text-sm text-stone-600">Henüz katılan üye yok. İlk üye olarak katılabilirsin.</p>
        <p v-if="pool.status === 'Filling' && isCreator && !isDraw" class="mt-3 text-xs text-stone-600">
          Sırayı düzenledikten sonra koşul sürümünü öner. Tüm üyeler aynı sürümü onaylayınca havuz başlatılabilir.
        </p>
      </section>

      <!-- KELİMELER -->
      <details class="card group cursor-pointer !p-0">
        <summary class="flex min-h-14 list-none items-center justify-between gap-3 px-5 py-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
          Kelimeler ne demek?
          <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
        </summary>
        <dl class="grid gap-3 px-5 pb-5 text-sm sm:grid-cols-2">
          <div><dt class="font-semibold">Risk</dt><dd class="text-stone-600">Erken teslim alan sonraki katkıyı bırakırsa bekleyenlerin geçmiş tur ödemeleri otomatik geri alınamaz.</dd></div>
          <div><dt class="font-semibold">Doğrulayıcı</dt><dd class="text-stone-600">Alım belgesini kontrol edip onaylayan kişi. Yeterli onay olmadan tutar çıkmaz.</dd></div>
          <div><dt class="font-semibold">Ek süre</dt><dd class="text-stone-600">Katkı süresi dolduktan sonra geciken üyeye tanınan son şans.</dd></div>
          <div><dt class="font-semibold">İade hakkı</dt><dd class="text-stone-600">Henüz satıcıya ödenmemiş turda bizzat yatırdığın katkı.</dd></div>
          <div v-if="isDraw"><dt class="font-semibold">Kura</dt><dd class="text-stone-600">Tüm katkılar gelince, henüz teslim almamış üyeler arasından alıcıyı kontrat seçer. Rastgelelik hackathon düzeyindedir.</dd></div>
          <div><dt class="font-semibold">Koşul sürümü</dt><dd class="text-stone-600">Sıra ve doğrulayıcıların onaylanan hali. Değişirse herkes yeniden onaylar.</dd></div>
        </dl>
      </details>

      <p class="px-2 text-center text-sm text-stone-600">
        Bu sayfadaki her bilgi doğrudan Stellar ağındaki sözleşmeden okunuyor.
        <a v-if="poolContractId" :href="explorerContract(poolContractId)" target="_blank" rel="noopener noreferrer" class="text-brand-700 underline">
          Sözleşmeyi blockchain üzerinde görüntüle
        </a>
      </p>
    </template>
  </div>
</template>
