<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import Illo from '@/components/Illo.vue'
import {
  anchorDomain,
  authenticate,
  getTransaction,
  resolveAnchor,
  startInteractive,
  STATUS_LABELS,
  supportsPoolAsset,
  TERMINAL_STATUSES,
  usingTestAnchor,
} from '@/lib/anchor'
import type { AnchorInfo, AnchorTransaction, InteractiveSession } from '@/lib/anchor'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { explorerAccount, mainnetShowcase, poolAsset } from '@/lib/stellar'
import { useWalletStore } from '@/stores/wallet'

/**
 * Anchor flow. On Testnet it runs the real SEP-1 (stellar.toml) → SEP-10 (wallet-signed login) → SEP-24
 * (interactive deposit) protocol. The default provider is the SDF test anchor: it issues a test
 * asset and is NOT Turkish lira. The Mainnet showcase accepts no signatures and only shows an illustrative
 * simulation. Withdrawal (Stellar → TRY) is not in the live flow yet.
 */
const props = defineProps<{
  /** Inside the pool page: only depositing the pool asset, without the heading and simulation. */
  compact?: boolean
}>()
const emit = defineEmits<{ completed: [] }>()

const live = !mainnetShowcase
/** A page can hold several instances (compact + full on the home page); ids must be unique. */
const uid = useId()
const titleId = `anchor-title-${uid}`
const assetSelectId = `anchor-asset-${uid}`
const poolCode = poolAsset.getCode()
const wallet = useWalletStore()

// --- Live flow ------------------------------------------------------------------------
const anchor = ref<AnchorInfo | null>(null)
const anchorError = ref<string | null>(null)
const loadingAnchor = ref(false)
const asset = ref('')
const busy = ref<'auth' | null>(null)
const error = ref<string | null>(null)
const session = ref<InteractiveSession | null>(null)
const tx = ref<AnchorTransaction | null>(null)
/** The SEP-10 session key is kept only in memory; it is erased when the page is refreshed. */
let token: string | null = null
let poll: ReturnType<typeof setInterval> | undefined

const depositAssets = computed(() =>
  Object.entries(anchor.value?.deposit ?? {})
    .filter(([, v]) => v.enabled)
    // Inside a pool, only the pool's own asset (code + issuer) is offered.
    .filter(([code]) => !props.compact || (anchor.value && supportsPoolAsset(anchor.value, code, poolAsset.getIssuer() ?? '')))
    .map(([code, v]) => ({ code, ...v })),
)
const selected = computed(() => depositAssets.value.find((a) => a.code === asset.value) ?? null)
/** If the anchor offers a real TRY asset (TRY/TRYB); otherwise it is labeled as a test anchor. */
const realTryAnchor = computed(() => !usingTestAnchor && !!anchor.value?.supportsTry)
const finished = computed(() => (tx.value ? TERMINAL_STATUSES.has(tx.value.status) : false))
watch(
  () => tx.value?.status,
  (status) => {
    if (status === 'completed') emit('completed')
  },
)

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

/** The window opens on the user's click so popup blockers do not stop it. */
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

/** The anchor window reports completion via `postMessage`; only the origin of the window we opened is listened to. */
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

// --- Illustrative simulation (offline) -------------------------------------------------------
type Direction = 'deposit' | 'withdraw'

const direction = ref<Direction>('deposit')
const step = ref(0)

const flows = {
  deposit: [
    { title: 'TRY deposit request', text: 'In a real service the user opens a request through an authorized anchor. No request is sent on this screen.' },
    { title: 'Bank transfer verification', text: 'In a real service the provider verifies the payment. This simulation does not connect to a bank account.' },
    { title: 'Sending the Stellar asset', text: 'In a real service the issuer can transfer the asset to the wallet. No token is minted or transferred here.' },
    { title: 'Sample flow finished', text: 'The steps are for illustration only. No TRY or Stellar balance was created or changed.' },
  ],
  withdraw: [
    { title: 'Asset withdrawal request', text: 'In a real service the user opens a withdrawal request through an authorized anchor. No request is sent on this screen.' },
    { title: 'Asset and identity check', text: 'In a real service the provider checks the balance and eligibility. This simulation does not connect to a wallet.' },
    { title: 'TRY payment to a bank account', text: 'In a real service the provider settles an eligible request through the banking channel. No payment order is issued here.' },
    { title: 'Sample flow finished', text: 'The steps are for illustration only. No TRY or Stellar balance was created or changed.' },
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
  <section v-if="!compact || live" :class="compact ? 'space-y-4' : 'card space-y-5'" :aria-labelledby="titleId">
    <div v-if="!compact" class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <p class="eyebrow text-brand-700">Anchor · SEP-1 / SEP-10 / SEP-24</p>
        <h2 :id="titleId" class="mt-1 text-2xl font-extrabold">Connecting TRY and Stellar</h2>
      </div>
      <span v-if="live" class="badge" :class="realTryAnchor ? 'bg-brand-100 text-brand-800' : 'bg-gold-100 text-amber-900'">
        {{ usingTestAnchor ? 'Test anchor · not TRY' : realTryAnchor ? anchorDomain : 'Test anchor · not real TRY' }}
      </span>
      <span v-else class="badge bg-amber-100 text-amber-900">Simulation only</span>
    </div>

    <!-- LIVE FLOW -->
    <template v-if="live">
      <p v-if="compact" :id="titleId" class="text-sm leading-relaxed text-stone-600">
        Load your balance as {{ poolCode }} through <strong>{{ anchorDomain }}</strong>: you sign in with your wallet, the deposit happens in the anchor's
        own window, and the status is tracked here.
        <template v-if="usingTestAnchor || (anchor && !anchor.supportsTry)">
          <strong>This provider issues a test asset; it does not offer Turkish lira.</strong>
        </template>
      </p>
      <p v-else class="text-sm leading-relaxed text-stone-600">
        This section runs the real anchor protocol with <strong>{{ anchorDomain }}</strong>: the endpoints are
        read from <code class="font-mono">stellar.toml</code>, you sign in with your wallet, and the deposit window opens in the anchor's
        own interface.
        <template v-if="usingTestAnchor || (anchor && !anchor.supportsTry)">
          <strong>This provider issues a test asset ({{ anchor?.assetCodes.join(', ') || 'USDC' }}); it does not offer Turkish lira.</strong>
          The hackathon's real-TRY requirement will be met with the same flow once a verified provider offering TRY is found.
        </template>
      </p>

      <div v-if="loadingAnchor" class="skeleton h-24 rounded-2xl" aria-live="polite" />

      <div v-else-if="anchorError" role="alert" class="space-y-2 rounded-2xl bg-rose-50 p-4 text-sm text-rose-800">
        <p>The anchor could not be reached: {{ anchorError }}</p>
        <button type="button" class="btn-secondary !min-h-10" @click="loadAnchor">Try again</button>
      </div>

      <template v-else-if="anchor">
        <p v-if="compact && !depositAssets.length" role="status" class="rounded-2xl bg-gold-100/80 p-3 text-sm text-amber-950">
          This anchor does not offer the pool asset ({{ poolCode }}, same issuer) for deposits. To load a balance,
          use the <RouterLink to="/#basla" class="font-semibold underline">setup wizard</RouterLink>.
        </p>
        <div v-if="!compact || depositAssets.length" class="flex flex-wrap items-center gap-2 text-xs">
          <span class="badge bg-sage-100 text-sage-800"><AppIcon name="check" class="!size-3.5" /> stellar.toml okundu</span>
          <span class="badge bg-stone-100 text-stone-700">Assets: {{ anchor.assetCodes.join(' · ') }}</span>
          <span class="badge" :class="anchor.supportsTry ? 'bg-sage-100 text-sage-800' : 'bg-gold-100 text-amber-900'">
            {{ anchor.supportsTry ? 'TRY supported' : anchor.representsTry ? 'Test asset representing TRY · not real TRY' : 'No TRY' }}
          </span>
        </div>

        <ol v-if="!compact || depositAssets.length" class="space-y-2 text-sm" aria-label="Anchor steps">
          <li class="flex items-start gap-2.5">
            <span class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-full bg-sage-600 text-xs font-bold text-white">
              <AppIcon name="check" class="!size-3.5" />
            </span>
            <span><strong>The anchor is recognized.</strong> Endpoints and supported assets were read.</span>
          </li>
          <li class="flex items-start gap-2.5">
            <span class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-full text-xs font-bold text-white" :class="session ? 'bg-sage-600' : 'bg-brand-600'">
              <AppIcon v-if="session" name="check" class="!size-3.5" /><template v-else>2</template>
            </span>
            <span><strong>You sign in with your wallet.</strong> The verification transaction (challenge) the anchor sends is validated in the app and you sign it. No fee is paid.</span>
          </li>
          <li class="flex items-start gap-2.5">
            <span class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-full text-xs font-bold text-white" :class="finished && tx?.status === 'completed' ? 'bg-sage-600' : session ? 'bg-brand-600' : 'bg-stone-300'">3</span>
            <span><strong>You complete the deposit in the anchor window.</strong> The status is tracked live here.</span>
          </li>
        </ol>

        <div v-if="!session && (!compact || depositAssets.length)" class="space-y-3 rounded-2xl bg-sand/60 p-4">
          <div class="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
            <p v-if="compact && selected" class="text-sm">
              To deposit: <strong>{{ poolCode }}</strong>
              <span v-if="selected.minAmount || selected.maxAmount" class="text-stone-600"> · per transaction {{ selected.minAmount ?? '—' }}–{{ selected.maxAmount ?? '—' }}</span>
            </p>
            <div v-else>
              <label class="label" :for="assetSelectId">Asset to deposit</label>
              <select :id="assetSelectId" v-model="asset" class="input">
                <option v-for="a in depositAssets" :key="a.code" :value="a.code">
                  {{ a.code === 'native' ? 'XLM' : a.code }}{{ a.minAmount || a.maxAmount ? ` (${a.minAmount ?? '—'}–${a.maxAmount ?? '—'})` : '' }}
                </option>
              </select>
            </div>
            <button v-if="!wallet.isConnected" type="button" class="btn-primary" :disabled="wallet.busy" @click="wallet.connect()">
              <AppIcon name="wallet" class="!size-4" /> Connect a wallet first
            </button>
            <button v-else type="button" class="btn-primary" :disabled="busy !== null || !selected" @click="begin">
              {{ busy === 'auth' ? 'Confirm in wallet…' : 'Start with the anchor' }}
            </button>
          </div>
          <p v-if="!compact && asset && asset !== 'native'" class="text-xs text-stone-600">
            To receive {{ asset }}, your account needs a trustline for this asset. The “trust” step of the setup wizard above does that.
          </p>
        </div>

        <div v-else-if="session" class="pop space-y-3 rounded-2xl border-2 border-brand-200 bg-brand-50/60 p-4" aria-live="polite">
          <div class="flex items-center gap-2">
            <Illo :name="tx?.status === 'completed' ? 'party' : 'hourglass'" :size="30" />
            <p class="font-display font-bold">{{ tx ? (STATUS_LABELS[tx.status] ?? tx.status) : 'Reading status…' }}</p>
          </div>
          <p v-if="tx" class="text-xs text-stone-600">
            Transaction no.: <span class="font-mono">{{ tx.id.slice(0, 8) }}…</span>
            <template v-if="tx.amountIn"> · deposited {{ tx.amountIn }}</template>
            <template v-if="tx.amountOut"> · gelen {{ tx.amountOut }}</template>
          </p>
          <div v-if="!finished" class="flex flex-wrap items-center gap-3">
            <button type="button" class="btn-primary" @click="openWindow">
              Open the anchor window <AppIcon name="external" class="!size-4" />
            </button>
            <span class="text-xs text-stone-600">
              The window opens at <strong class="font-mono">{{ session.host }}</strong><template v-if="!session.sameDomain">
              (the anchor's own interface server; check the address)</template>.
            </span>
          </div>
          <div v-else class="flex flex-wrap items-center gap-3">
            <a v-if="wallet.address && tx?.status === 'completed'" :href="explorerAccount(wallet.address)" target="_blank" rel="noopener noreferrer" class="btn-secondary !min-h-10">
              View the balance on Stellar Expert <AppIcon name="external" class="!size-4" />
            </a>
            <button type="button" class="btn-secondary !min-h-10" @click="session = null; tx = null">New transaction</button>
          </div>
        </div>

        <p v-if="error" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ error }}</p>
        <p class="text-xs leading-relaxed text-stone-500">
          The session key is kept only in this page's memory. The app never asks for bank details, identity data or a secret key;
          if identity verification (KYC) is needed, it is requested in the anchor's own window. The withdrawal (Stellar → TRY) flow is not live in this version.
        </p>
      </template>
    </template>

    <!-- ILLUSTRATIVE SIMULATION -->
    <component :is="live ? 'details' : 'div'" v-if="!compact" class="group space-y-4 rounded-2xl" :class="live ? 'border border-stone-200 p-4' : ''">
      <summary v-if="live" class="flex min-h-11 cursor-pointer list-none items-center justify-between gap-3 font-display font-bold marker:hidden [&::-webkit-details-marker]:hidden">
        Illustrative simulation (offline)
        <AppIcon name="chevron" class="text-brand-600 transition-transform duration-300 group-open:rotate-180" />
      </summary>

      <p class="text-sm leading-relaxed text-stone-600">
        An example of how a real TRY anchor would work. It asks for no personal information and produces no payment instruction or chain transaction,
        and creates no usable balance.
      </p>

      <div class="flex flex-wrap gap-2" role="group" aria-label="Direction of the sample flow">
        <button type="button" class="btn-secondary" :aria-pressed="direction === 'deposit'" :class="direction === 'deposit' ? '!border-brand-600 !bg-brand-50 !text-brand-900' : ''" @click="select('deposit')">
          TRY → Stellar asset
        </button>
        <button type="button" class="btn-secondary" :aria-pressed="direction === 'withdraw'" :class="direction === 'withdraw' ? '!border-brand-600 !bg-brand-50 !text-brand-900' : ''" @click="select('withdraw')">
          Stellar asset → TRY
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
        <p class="mt-3 text-xs font-semibold uppercase tracking-wide text-amber-900">Simulation · step {{ step + 1 }} / 4</p>
        <h3 class="mt-2 font-semibold text-amber-950">{{ current.title }}</h3>
        <p class="mt-2 text-sm leading-relaxed text-amber-950">{{ current.text }}</p>
      </div>

      <button type="button" class="btn-primary" @click="advance">
        {{ step === 3 ? 'Restart the example' : 'Next example step' }}
      </button>
    </component>
  </section>
</template>
