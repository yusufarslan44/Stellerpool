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
import FeesTable from '@/components/FeesTable.vue'
import HowItWorks from '@/components/HowItWorks.vue'
import StorySim from '@/components/StorySim.vue'
import StepIndicator from '@/components/StepIndicator.vue'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatDecimalString, parseAmount, shortAddress } from '@/lib/format'
import {
  circleFaucetUrl,
  config,
  explorerAccount,
  explorerContract,
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

// --- Getting-started wizard: account state -----------------------------------------------------
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
  { key: 'wallet', label: 'Wallet', done: wallet.isConnected },
  { key: 'activate', label: 'Account', done: account.value?.exists === true },
  { key: 'trust', label: `Accept ${token}`, done: account.value?.hasTrustline === true },
  { key: 'faucet', label: `Test ${token}`, done: hasBalance.value },
])
const setupIndex = computed(() => {
  const i = SETUP.value.findIndex((s) => !s.done)
  return i === -1 ? SETUP.value.length : i
})
const setupKey = computed(() => SETUP.value[setupIndex.value]?.key ?? 'ready')

// On the test-asset step the balance is checked automatically (so it shows up when the user returns from the Circle page).
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
    title: 'Founder',
    text: 'Opens the pool and joins it. In fixed order, the join order is used; the founder cannot withdraw funds.',
    tone: 'bg-brand-100 text-brand-800',
  },
  {
    icon: 'lock',
    title: 'Contract',
    text: 'Holds contributions per round and, once conditions are met, pays only the allowed demo seller.',
    tone: 'bg-gold-100 text-amber-900',
  },
  {
    icon: 'handshake',
    title: 'Member',
    text: 'Joins and pays their contribution every round. When their turn comes, they register the seller and the purchase document.',
    tone: 'bg-sage-100 text-sage-800',
  },
]

// Decentralization: every sentence was verified against the contract source (no admin, upgrade, pause or fee function;
// execute_round / mark_overdue / abort_pool / cancel_unstarted_pool do not require a privileged caller).
const TRUST: { icon: IlloName; title: string; text: string }[] = [
  {
    icon: 'lock',
    title: 'No admin key',
    text: 'The contract has no admin, upgrade, pause or fee function. The founder cannot withdraw the money, and neither can we. A new version means a new address.',
  },
  {
    icon: 'key',
    title: 'Your keys, your funds',
    text: 'Your wallet signs every transaction. We never see or store your keys; only you and the contract’s rules can touch your money.',
  },
  {
    icon: 'handshake',
    title: 'Anyone can call',
    text: 'Marking a deadline as missed, ending the pool and triggering payment for a round whose conditions are met are open to everyone. Even if this site goes down, the contract keeps working.',
  },
  {
    icon: 'eyes',
    title: 'Anyone can verify',
    text: 'The contract address, transactions and source code are public. You do not have to trust us; you can check for yourself.',
  },
]

// Honesty panel: the places we do not count as decentralized. The claim "fully decentralized" is deliberately not made.
const OFFCHAIN: { title: string; text: string }[] = [
  { title: 'Anchor and TRYT', text: 'We operate the test anchor and the TRYT asset; it is not real Turkish lira.' },
  { title: 'Purchase document', text: 'The document itself stays off-chain; only its digest is recorded on-chain. No real purchase verification is performed.' },
  { title: 'Draw randomness', text: 'Hackathon-grade; not sufficient for high amounts.' },
  { title: 'Audit', text: 'No independent security audit has been done; there are only unit tests.' },
  { title: 'This site and network endpoints', text: 'The site and the RPC/Horizon endpoints are centralized servers; the contract itself can also be called from any other client.' },
  { title: 'Seller', text: 'Payment goes only to the allowed demo address registered in the pool; no real seller is verified.' },
]

const FAQ = [
  {
    q: 'Can the organizer take the money and run?',
    a: 'No. The contract has no free-withdrawal function for the founder: the round amount can only go to the allowed demo seller registered in the pool, and refunds go only to the contributor. The fixed delivery order is created automatically from the join order. Once the pool starts, members and order are locked. No real seller, title deed or registration is verified. The contract passed its unit tests but has not had an independent security audit.',
  },
  {
    q: 'How does the draw work, and when is it drawn?',
    a: 'In draw mode there is no predefined order. Every round each member pays their own contribution; once all are in, anyone can call a transaction and the contract picks the recipient among members who have not yet received. The winner has already paid their share. Unlike companies, there is no fixed draw day, notary or live broadcast: the draw can happen as soon as everyone has paid their installment. If someone does not pay, a grace period starts; if that also ends, the round stops and refunds begin. On-chain randomness is hackathon-grade and not enough for real high-value use.',
  },
  {
    q: 'What is the down payment, and are there fees?',
    a: 'The down payment is not a fee: when the pool is created, an amount per member is set, everyone pays it when joining, and it is held in the contract. When your turn comes, it is added to your purchase and goes to the seller; if the pool is cancelled and you have not received yet, it is refunded. The higher the down payment, the smaller the target, so your installment gives fewer people and a shorter term. There is no organization fee, commission or interest; details are in the “Fees” section.',
  },
  {
    q: 'How large can a group be?',
    a: 'The published Testnet contract supports 2 to 30 members. In a large group the amount sent to the seller in one round also grows; if someone who received early stops contributing, the gap grows too. Making the group bigger does not remove this risk.',
  },
  {
    q: 'What happens if someone stops paying?',
    a: 'When the contribution deadline passes, the round enters a grace period. If that ends too, anyone can end the pool; only the contributions of the round that has not yet been paid out are refunded, earlier rounds cannot be recovered. Example: if four people pay 10 units each, 40 units go to the seller in round one. If Ayşe does not pay round two, that round stops and only the round-two contributions are refunded; Mehmet, Zeynep and Can’s round-one shares cannot be recovered. This flow was tried live on Testnet (transaction links are in the README).',
  },
  {
    q: 'Is there a sponsor, insurance or delivery guarantee?',
    a: 'No. There is no separate sponsor, no advance on someone else’s behalf and no platform guarantee. More people joining does not remove the risk that someone who received early stops paying later. That risk can only be managed in a familiar, closed group and with separate agreements.',
  },
  {
    q: 'Is real money used?',
    a: 'No. This is a Stellar Testnet interface; a test asset is used. There is no real money, no real TRY deposit/withdrawal and no home or car delivery. The anchor section works with a test anchor and does not offer Turkish lira.',
  },
  {
    q: 'Is it fully decentralized?',
    a: 'No, only partly. The money rules are on-chain and there is no admin key; however the test anchor, the purchase-document check, the draw randomness and this site remain centralized or limited. We list exactly which ones in the “Still centralized” box in the Trust section.',
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
        <strong>Status:</strong> This build is not connected to a pool contract yet
        (<code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> is empty). The sponsor-free pool contract is live on Testnet;
        once its address is configured, pools can be opened and read. No real money is involved.
      </span>
    </p>

    <div class="home-intro-strip" v-reveal>
      <span class="intro-strip-label">THE FOUNDATION OF SAVING TOGETHER</span>
      <span><AppIcon name="users" /> A group you know</span><span><AppIcon name="lock" /> Rules everyone approves</span><span><AppIcon name="eye" /> Visible money flow</span>
    </div>

    <HowItWorks />

    <!-- 2b · STORY: a sample round that plays by itself -->
    <section id="hikaye" class="home-story scroll-mt-28 space-y-8" aria-labelledby="hikaye-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Watch</p>
        <h2 id="hikaye-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">This is how a round works</h2>
        <p class="mt-3 text-stone-600">
          Four friends pool their contributions, record a purchase and pay the seller, in under half a minute.
        </p>
      </div>
      <div v-reveal><StorySim /></div>
    </section>

    <!-- 3 · GET STARTED (account setup wizard) -->
    <section id="basla" class="home-setup scroll-mt-28 space-y-8" aria-labelledby="basla-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Get started</p>
        <h2 id="basla-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Set up your account in four steps</h2>
        <p class="mt-3 text-stone-600">All on {{ config.label }}, no real money. Go step by step.</p>
      </div>

      <div v-reveal class="card mx-auto max-w-3xl space-y-6 !p-5 sm:!p-8">
        <StepIndicator :steps="SETUP" :current="setupIndex" />

        <Transition name="step-next" mode="out-in">
          <div :key="setupKey" class="min-h-56">
            <!-- 1 · Wallet -->
            <div v-if="setupKey === 'wallet'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-brand-100 text-brand-700">
                  <Illo name="purse" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">1. Connect your wallet</h3>
                  <p class="text-stone-600">Your wallet is your account and your signature. We never share your secrets.</p>
                </div>
              </div>
              <button type="button" class="btn-primary btn-lg" :disabled="wallet.busy" @click="wallet.connect()">
                {{ wallet.busy ? 'Connecting…' : 'Connect wallet' }}
              </button>
              <p v-if="wallet.error" role="alert" class="text-sm text-rose-700">{{ wallet.error }}</p>
              <p class="text-sm text-stone-600">
                You can use Freighter, xBull, Albedo, LOBSTR, Hana or Rabet. If you do not want to
                install an extension, Albedo works in the browser.
              </p>
            </div>

            <!-- Hesap okunuyor -->
            <div v-else-if="loading && !account" class="grid min-h-56 place-items-center">
              <div class="flex flex-col items-center gap-3 text-stone-600">
                <CoinSpinner :size="52" />
                Reading your account…
              </div>
            </div>

            <!-- 2 · Activate the account -->
            <div v-else-if="setupKey === 'activate'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-gold-100 text-amber-800">
                  <Illo name="sparkles" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">2. Activate your account</h3>
                  <p class="text-stone-600">
                    A Stellar account needs a small amount of transaction fee (XLM) when first created. On Testnet we get it
                    from a free faucet (Friendbot).
                  </p>
                </div>
              </div>
              <button type="button" class="btn-primary btn-lg" :disabled="busy !== null" @click="fund">
                {{ busy === 'fund' ? 'Activating…' : `Activate your ${config.label} account` }}
              </button>
            </div>

            <!-- 3 · Enable acceptance -->
            <div v-else-if="setupKey === 'trust'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-sage-100 text-sage-700">
                  <Illo name="coin" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">3. Enable accepting {{ token }}</h3>
                  <p class="text-stone-600">
                    To let your account hold {{ token }}, you need to allow it once (a trustline). Your wallet
                    will ask for a signature.
                  </p>
                </div>
              </div>
              <button type="button" class="btn-primary btn-lg" :disabled="busy !== null" @click="trust">
                {{ busy === 'trust' ? 'Confirm in wallet…' : `Enable accepting ${token}` }}
              </button>
            </div>

            <!-- 4 · Get the test asset -->
            <div v-else-if="setupKey === 'faucet'" class="space-y-4">
              <div class="flex items-center gap-4">
                <span class="grid size-14 shrink-0 place-items-center rounded-2xl bg-brand-100 text-brand-700">
                  <Illo name="moneybag" :size="40" />
                </span>
                <div>
                  <h3 class="text-2xl font-extrabold">4. Get test {{ token }}</h3>
                  <p v-if="poolAssetFromCircleFaucet" class="text-stone-600">
                    You need non-real test money to try things out. On Circle’s page, choose the “Stellar Testnet”
                    network and paste your address.
                  </p>
                  <p v-else class="text-stone-600">
                    You need non-real test money to try things out. This pool asset ({{ token }}) does not come from a faucet;
                    it is loaded through the anchor.
                  </p>
                </div>
              </div>
              <AnchorDemo v-if="!poolAssetFromCircleFaucet" compact @completed="refresh" />
              <div class="flex flex-wrap gap-3">
                <a v-if="poolAssetFromCircleFaucet" :href="circleFaucetUrl" target="_blank" rel="noopener noreferrer" class="btn-primary btn-lg">
                  Open Circle <AppIcon name="external" class="!size-4" />
                </a>
                <button v-if="poolAssetFromCircleFaucet" type="button" class="btn-secondary btn-lg" @click="copyAddress">
                  <AppIcon name="copy" class="!size-4" />
                  {{ copied ? 'Copied ✓' : 'Copy my address' }}
                </button>
                <button type="button" class="btn-secondary btn-lg" :disabled="loading" @click="refresh">
                  <AppIcon name="refresh" class="!size-4" />
                  Check my balance
                </button>
              </div>
              <p class="text-sm text-stone-600">This screen moves to the next step automatically once the balance arrives.</p>
            </div>

            <!-- Ready -->
            <div v-else class="flex flex-col items-center gap-4 py-4 text-center">
              <Illo name="party" :size="84" class="pop" />
              <h3 class="text-3xl font-extrabold">You're all set!</h3>
              <p class="max-w-md text-stone-600">
                Your wallet is connected, your account is active and you hold test {{ token }}. You can join a pool now.
              </p>
              <RouterLink to="/join" class="btn-primary btn-lg">
                Join a pool <AppIcon name="arrow" class="!size-4" />
              </RouterLink>
            </div>
          </div>
        </Transition>

        <!-- Account summary -->
        <dl v-if="account && wallet.address" class="grid gap-3 border-t border-stone-100 pt-5 sm:grid-cols-3">
          <div class="rounded-2xl bg-sand/60 p-3.5">
            <dt class="text-xs text-stone-600">Address</dt>
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
            <dt class="text-xs text-stone-600">XLM (transaction fee)</dt>
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
          Transaction sent:
          <a :href="explorerTx(lastTx)" target="_blank" rel="noopener noreferrer" class="font-mono underline">
            {{ lastTx.slice(0, 8) }}…
          </a>
        </p>
        <p v-if="error" role="alert" class="rounded-2xl bg-rose-50 p-3 text-sm text-rose-800">{{ error }}</p>
      </div>

      <form v-reveal class="mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-3 px-2" @submit.prevent="openPool">
        <div>
          <h3 class="text-lg font-bold">Have a pool number?</h3>
          <p class="text-sm text-stone-600">Enter the number you were given and go straight to the pool.</p>
        </div>
        <div class="flex gap-2">
          <label class="sr-only" for="pool-id">Pool number</label>
          <input
            id="pool-id"
            v-model="poolIdInput"
            class="input w-36"
            type="number"
            min="0"
            step="1"
            inputmode="numeric"
            placeholder="Pool no."
            required
          />
          <button type="submit" class="btn-primary">Open</button>
        </div>
      </form>
    </section>

    <!-- 4 · WHO DOES WHAT -->
    <section class="home-roles space-y-8" aria-labelledby="roller-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Roles</p>
        <h2 id="roller-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Who does what in a pool?</h2>
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
        <p class="eyebrow text-brand-700">Calculate</p>
        <h2 id="hesapla-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Let’s sketch your plan together</h2>
        <p class="mt-3 text-stone-600">Pick your goal and adjust the amount and number of people. The coins on the right show the member count.</p>
      </div>
      <div v-reveal>
        <PoolCalculator />
      </div>
    </section>

    <!-- 6 · TRUST -->
    <section class="home-trust espresso relative overflow-hidden rounded-[2.25rem] px-5 py-12 text-white sm:px-10 sm:py-16" aria-labelledby="guven-baslik">
      <div class="trust-intro">
        <div v-reveal>
          <p class="eyebrow text-gold-300">Trust model</p>
          <h2 id="guven-baslik" class="mt-3 text-4xl font-extrabold sm:text-5xl">Rules in code.<br /><span>Open to everyone.</span></h2>
          <p class="trust-description">See where your contribution goes. Follow the order, approvals and payment conditions together with your group.</p>
          <a href="#sss" class="trust-link">Review the rules and limits <AppIcon name="arrow" /></a>
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
      <div class="trust-verify" v-reveal>
        <span>Check for yourself:</span>
        <a
          v-if="poolContractId"
          :href="explorerContract(poolContractId)"
          target="_blank"
          rel="noopener noreferrer"
        >
          Open the contract on Stellar Expert <AppIcon name="external" />
        </a>
        <a href="https://github.com/yusufarslan44/Stellerpool" target="_blank" rel="noopener noreferrer">
          Source code (GitHub) <AppIcon name="external" />
        </a>
      </div>
      <div class="trust-offchain" v-reveal aria-labelledby="merkezi-baslik">
        <h3 id="merkezi-baslik">Still centralized</h3>
        <p>We do not claim to be fully decentralized. For the following items you still rely on someone, or on a server:</p>
        <ul>
          <li v-for="o in OFFCHAIN" :key="o.title"><strong>{{ o.title }}</strong><span>{{ o.text }}</span></li>
        </ul>
      </div>
      <p class="mt-8 text-center text-xs text-stone-300">
        The contract is live on Testnet and passed its unit tests, but has not had an independent security audit;
        it must not be used with real money.
      </p>
    </section>

    <!-- 6b · FEES -->
    <section id="ucretler" class="home-fees scroll-mt-28 space-y-8" aria-labelledby="ucret-baslik">
      <div v-reveal class="mx-auto max-w-2xl text-center">
        <p class="eyebrow text-brand-700">Fees</p>
        <h2 id="ucret-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">What do you pay, and to whom?</h2>
        <p class="mt-3 text-stone-600">
          No organization fee, commission or interest. The only thing you pay is the network’s very small transaction fee.
        </p>
      </div>
      <div v-reveal><FeesTable /></div>
    </section>

    <!-- 7 · ANCHOR SIMULATION -->
    <div v-reveal><AnchorDemo /></div>

    <!-- 8 · SSS -->
    <section id="sss" class="home-faq scroll-mt-28" aria-labelledby="sss-baslik">
      <div v-reveal class="faq-heading">
        <p class="eyebrow text-brand-700">Your questions</p>
        <h2 id="sss-baslik" class="mt-2 text-4xl font-extrabold sm:text-5xl">Frequently asked questions</h2>
        <p class="mt-3 text-sm text-stone-600">
          The answers describe the Testnet prototype. It must not be used with real money before an independent audit.
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
      <div><p class="eyebrow">BIG GOALS, SMALL STEPS</p><h2 id="closing-title">Let’s take the<br /><span>first step together.</span></h2><p>Think of your group, build your plan.<br />Discover how it works on Testnet.</p></div>
      <div class="closing-actions"><a href="#basla" class="closing-primary">Set up your account <AppIcon name="arrow" /></a><a href="#hesapla">Let me calculate my plan first ↗</a><span>No real money is used.</span></div>
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
.trust-verify { display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: 8px 22px; margin-top: 26px; color: #92a785; font-size: 11px; }
.trust-verify a { display: inline-flex; align-items: center; gap: 6px; min-height: 40px; color: #e2d4a1; text-decoration: underline; text-underline-offset: 3px; }
.trust-verify :deep(svg) { width: 13px; height: 13px; }
.trust-offchain { margin-top: 26px; padding: 26px 28px; border: 1px dashed #d9c98a66; border-radius: 22px; background: #ffffff05; }
.trust-offchain h3 { color: #e9dfb2; font: 600 18px var(--font-display); }
.trust-offchain > p { margin-top: 6px; color: #adbc9a; font-size: 12px; line-height: 1.8; }
.trust-offchain ul { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px 24px; margin-top: 18px; }
.trust-offchain li { display: grid; gap: 3px; align-content: start; }
.trust-offchain li strong { color: #edf0d9; font-size: 13px; font-weight: 600; }
.trust-offchain li span { color: #adbc9a; font-size: 12px; line-height: 1.75; }
.home-fees { padding-top: 8px; }
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
@media(max-width:1023px) { .trust-offchain ul { grid-template-columns: repeat(2, 1fr); } .home-faq { gap: 30px; grid-template-columns: .8fr 1.3fr; } .trust-intro { gap: 25px; grid-template-columns: 1fr; } .trust-description { max-width: 480px; } }
@media(max-width:767px) { .trust-offchain { padding: 22px 18px; } .trust-offchain ul { grid-template-columns: 1fr; } .home-intro-strip { justify-content: center; gap: 15px 20px; padding-inline: 0; } .home-intro-strip .intro-strip-label { flex-basis: 100%; justify-content: center; } .home-intro-strip > span { font-size: 10px; } .home-page :deep(h2) { font-size: 34px; } .home-story::before { inset: 90px -10px -20px; border-radius: 28px; } .home-faq { grid-template-columns: 1fr; gap: 28px; } .faq-heading { position: static; text-align: center; } .faq-heading > p:last-child { margin-inline: auto; max-width: 330px; } .home-closing { padding: 32px 25px; flex-direction: column; align-items: flex-start; gap: 27px; border-radius: 25px; } .home-closing h2 { font-size: 41px; } .closing-actions { align-items: flex-start; } .closing-star { right: 20px; top: 40px; } .role-card { padding: 23px 20px; } }
@media(prefers-reduced-motion:reduce) { .role-art, .closing-primary { transition: none; } .role-card:hover .role-art { transform: rotate(-7deg); } .closing-primary:hover { transform: none; } }
</style>
