<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import { RouterLink, RouterView } from 'vue-router'
import AppHeader from '@/components/AppHeader.vue'
import { config, explorerContract, mainnetShowcase, poolContractId } from '@/lib/stellar'

const NetworkWarning = defineAsyncComponent(() => import('@/components/NetworkWarning.vue'))

</script>

<template>
  <div class="relative flex min-h-dvh flex-col">
    <!-- Arka plan: yumuşak, sıcak ışık lekeleri -->
    <div class="pointer-events-none fixed inset-0 -z-10 overflow-hidden" aria-hidden="true">
      <div class="drift absolute -top-40 -left-32 size-[34rem] rounded-full bg-brand-200/40 blur-3xl" />
      <div class="drift absolute top-1/3 -right-40 size-[30rem] rounded-full bg-gold-300/25 blur-3xl [animation-delay:-6s]" />
    </div>

    <AppHeader />

    <main id="main-content" tabindex="-1" class="mx-auto w-full max-w-6xl flex-1 px-4 pt-6 pb-20">
      <NetworkWarning v-if="!mainnetShowcase" />
      <RouterView v-slot="{ Component, route }">
        <Transition name="page" mode="out-in">
          <component :is="Component" :key="route.path" />
        </Transition>
      </RouterView>
    </main>

    <footer class="site-footer">
      <div class="footer-top">
        <div><RouterLink to="/" class="footer-brand">Stellarpool<span>.</span></RouterLink><p>Shared goals.<br />Public rules.</p></div>
        <nav v-if="!mainnetShowcase" aria-label="Footer menu"><span>EXPLORE</span><RouterLink to="/#nasil">How it works</RouterLink><RouterLink to="/#hikaye">Watch the story</RouterLink><RouterLink to="/#ucretler">Fees</RouterLink><RouterLink to="/#sss">FAQ</RouterLink></nav>
        <nav v-if="!mainnetShowcase" aria-label="Getting started"><span>FIRST STEP</span><RouterLink to="/#hesapla">Calculate your plan</RouterLink><RouterLink to="/#basla">Set up your wallet</RouterLink><RouterLink to="/join">Join a pool ↗</RouterLink></nav>
        <div class="footer-network"><span class="eyebrow">BUILT ON STELLAR</span><strong>{{ config.label }}</strong><p>{{ mainnetShowcase ? 'Read-only showcase.' : 'Try it together, explore the flow.' }}<br />No real money is used.</p></div>
      </div>
      <div class="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-2 px-4 py-5 text-xs text-stone-600">
        <span class="font-display text-sm font-semibold text-ink">Save together. Verify everything.</span>
        <span>
          {{ mainnetShowcase ? 'Mainnet showcase: funds and wallet actions are disabled.' : 'Testnet prototype, no real money is used.' }}
        </span>
        <span v-if="poolContractId">
          Contract:
          <a
            :href="explorerContract(poolContractId)"
            target="_blank"
            rel="noopener noreferrer"
            class="font-mono text-brand-700 underline"
          >
            {{ poolContractId.slice(0, 6) }}…{{ poolContractId.slice(-4) }}
          </a>
        </span>
        <span v-else-if="!mainnetShowcase">Contract not configured yet</span>
        <span v-else>Mainnet pool contract not published</span>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.site-footer { border-top: 1px solid #dfe5d3; background: #edf0e3; }
.footer-top { max-width: 1152px; margin: auto; padding: 52px 16px 36px; display: grid; grid-template-columns: 1.5fr 1fr 1fr 1.2fr; gap: 32px; }
.footer-brand { font: 750 29px var(--font-display); color: #264b34; letter-spacing: -1px; }
.footer-brand span { color: #b99a50; }
.footer-top p { color: #77816b; font-size: 12px; line-height: 1.8; margin-top: 13px; }
.footer-top nav { display: flex; flex-direction: column; align-items: flex-start; gap: 0; font-size: 12px; color: #4b5e43; }
.footer-top nav a { display: inline-flex; align-items: center; min-height: 44px; }
.footer-top nav > span, .footer-network > span { font-size: 9px; letter-spacing: .14em; color: #89937b; margin-bottom: 5px; }
.footer-top nav > span { margin-bottom: 0; }
.footer-top nav a:hover { color: #14805a; text-decoration: underline; text-underline-offset: 4px; }
.footer-network { border-left: 1px solid #d6deca; padding-left: 30px; }
.footer-network strong { display: block; margin-top: 8px; font: 600 23px var(--font-display); color: #3a5b3d; }
@media (max-width: 767px) { .footer-top { grid-template-columns: 1fr 1fr; gap: 32px 20px; padding-top: 35px; } .footer-network { padding-left: 0; border: 0; } }
</style>
