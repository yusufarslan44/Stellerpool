<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
import Illo from '@/components/Illo.vue'
import AnchorDemo from '@/components/AnchorDemo.vue'
import Scene3D from '@/components/Scene3D.vue'
import { useNetworkStatus } from '@/composables/useNetworkStatus'

const { ledger, online } = useNetworkStatus()
</script>

<template>
  <div class="space-y-8">
    <section class="sunrise relative isolate overflow-hidden rounded-[2.25rem] px-5 py-10 sm:px-10 sm:py-14">
      <div class="grid items-center gap-6 lg:grid-cols-[1.05fr_1fr]">
        <div>
          <div class="glass inline-flex items-center gap-2 rounded-full px-3.5 py-1.5 text-xs font-medium text-stone-700">
            <span
              class="size-2 rounded-full"
              :class="online === false ? 'bg-rose-500' : 'animate-pulse bg-sage-500'"
              aria-hidden="true"
            />
            <span v-if="online === false">The Mainnet connection cannot be read right now</span>
            <span v-else-if="ledger !== null">Stellar Mainnet · ledger #{{ ledger.toLocaleString('en-US') }}</span>
            <span v-else>Checking the Stellar Mainnet connection…</span>
          </div>

          <h1 class="mt-5 text-5xl leading-[1.02] font-extrabold sm:text-6xl">
            Birlikte biriktir.
            <span class="block bg-gradient-to-r from-brand-600 via-brand-500 to-gold-500 bg-clip-text text-transparent">
              Let everyone see the rules.
            </span>
          </h1>
          <p class="mt-5 max-w-xl text-lg leading-relaxed text-stone-700">
            Stellarpool is a prototype that aims to make contributions and order verifiable
            in invite-only rotating savings groups. Group terms are set by the participants’ shared approval.
          </p>
          <a
            href="https://github.com/yusufarslan44/Stellerpool"
            target="_blank"
            rel="noopener noreferrer"
            class="btn-primary btn-lg mt-7"
          >
            Projeyi GitHub'da incele <AppIcon name="arrow" class="!size-4" />
          </a>
        </div>
        <div class="relative mx-auto h-[300px] w-full max-w-xl sm:h-[380px]">
          <Scene3D :coins="4" label="A large pool coin in the middle, with four member coins orbiting it" />
        </div>
      </div>
    </section>

    <p role="status" class="rounded-3xl border border-gold-300/60 bg-gold-100/70 p-5 text-sm leading-relaxed text-amber-950">
      This is the <strong>transaction-free showcase build</strong> connected to Mainnet. Wallet signing, deposits,
      pool creation and real TRY anchor actions are disabled. The anchor steps below are only a
      simulation. The pool contract is published only on Testnet and not on Mainnet;
      no real money, home or car delivery or financing guarantee is offered.
    </p>

    <section aria-labelledby="akim" class="space-y-4">
      <div>
        <p class="eyebrow text-brand-700">Proposed product flow</p>
        <h2 id="akim" class="mt-1 text-2xl font-bold tracking-tight sm:text-3xl">
          An invited group’s own rules
        </h2>
      </div>
      <div class="grid gap-4 md:grid-cols-3">
        <article class="card">
          <Illo name="handshake" :size="52" />
          <h3 class="mt-4 font-semibold">The order is set together</h3>
          <p class="mt-2 text-sm leading-relaxed text-stone-600">
            Members see and approve the contribution, schedule and delivery order at the start.
          </p>
        </article>
        <article class="card">
          <Illo name="lock" :size="52" />
          <h3 class="mt-4 font-semibold">Rules become visible</h3>
          <p class="mt-2 text-sm leading-relaxed text-stone-600">
            The prototype implements and tests member approval, missed payments and refund conditions on Soroban (Testnet).
          </p>
        </article>
        <article class="card">
          <Illo name="shield" :size="52" />
          <h3 class="mt-4 font-semibold">Riskler incelenir</h3>
          <p class="mt-2 text-sm leading-relaxed text-stone-600">
            The planned AI reviewer flags inconsistencies; it does not manage funds or give legal approval.
          </p>
        </article>
      </div>
    </section>

    <AnchorDemo />

    <section class="card space-y-3">
      <h2 class="text-xl font-semibold">Home and car goals</h2>
      <p class="text-sm leading-relaxed text-stone-600">
        Group members may want to use their savings for a home or car goal. A commitment to financing and
        delivery of property on a specific date can only be designed separately, under the own
        contract and fund structure of a company holding the required licences. The anchor demo is likewise
        not a substitute for a real TRY service or a licence to operate.
      </p>
    </section>
  </div>
</template>
