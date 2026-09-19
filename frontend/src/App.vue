<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import { RouterLink, RouterView } from 'vue-router'
import { config, explorerContract, mainnetShowcase, poolContractId } from '@/lib/stellar'

const WalletButton = defineAsyncComponent(() => import('@/components/WalletButton.vue'))
const NetworkWarning = defineAsyncComponent(() => import('@/components/NetworkWarning.vue'))

const NAV = [
  { to: '/#nasil', label: 'Nasıl çalışır' },
  { to: '/#hikaye', label: 'Dene' },
  { to: '/#basla', label: 'Başla' },
  { to: '/#hesapla', label: 'Hesapla' },
  { to: '/#sss', label: 'Sorular' },
]
</script>

<template>
  <div class="relative flex min-h-dvh flex-col">
    <!-- Arka plan: yumuşak, sıcak ışık lekeleri -->
    <div class="pointer-events-none fixed inset-0 -z-10" aria-hidden="true">
      <div class="drift absolute -top-40 -left-32 size-[34rem] rounded-full bg-brand-200/40 blur-3xl" />
      <div class="drift absolute top-1/3 -right-40 size-[30rem] rounded-full bg-gold-300/25 blur-3xl [animation-delay:-6s]" />
    </div>

    <!-- Yüzen cam menü -->
    <header class="sticky top-3 z-30 mx-auto mt-3 w-[calc(100%-1.5rem)] max-w-6xl">
      <div
        class="flex items-center justify-between gap-3 rounded-full border border-white/70 bg-white/80 py-2 pr-2 pl-3 shadow-[0_10px_34px_-14px_rgb(120_53_15/0.35)] backdrop-blur-xl"
      >
        <div class="flex items-center gap-7">
          <RouterLink to="/" class="flex items-center gap-2.5 font-display text-lg font-bold tracking-tight text-ink">
            <svg viewBox="0 0 32 32" class="size-8 drop-shadow-sm" aria-hidden="true">
              <defs>
                <linearGradient id="logo-gold" x1="0" y1="0" x2="1" y2="1">
                  <stop offset="0" stop-color="#f7c85a" />
                  <stop offset="1" stop-color="#14805a" />
                </linearGradient>
              </defs>
              <circle cx="16" cy="16" r="15" fill="url(#logo-gold)" />
              <circle cx="16" cy="16" r="11.2" fill="none" stroke="#fff8ee" stroke-opacity="0.75" stroke-width="1.4" />
              <path d="m16 8.6 2.2 4.6 5 .7-3.6 3.5.9 5-4.5-2.4-4.5 2.4.9-5-3.6-3.5 5-.7Z" fill="#fff8ee" />
            </svg>
            Stellerpool
          </RouterLink>
          <nav
            v-if="!mainnetShowcase"
            class="hidden items-center gap-1 text-sm font-medium text-stone-600 lg:flex"
            aria-label="Ana menü"
          >
            <RouterLink
              v-for="n in NAV"
              :key="n.to"
              :to="n.to"
              class="rounded-full px-3 py-1.5 transition-colors duration-200 hover:bg-brand-50 hover:text-brand-800"
            >
              {{ n.label }}
            </RouterLink>
          </nav>
        </div>
        <div class="flex items-center gap-2">
          <span
            class="badge hidden bg-sage-100 text-sage-800 sm:inline-flex"
            :title="mainnetShowcase ? 'Mainnet bilgisi okunur; işlem ve fon kabul edilmez.' : 'Testnet prototipi, gerçek para kullanılmaz.'"
          >
            <span class="size-1.5 rounded-full bg-sage-500" aria-hidden="true" />
            {{ config.label }}
          </span>
          <RouterLink v-if="!mainnetShowcase" to="/create" class="btn-secondary hidden !min-h-9 !px-4 !py-1.5 md:inline-flex">
            Havuz oluştur
          </RouterLink>
          <WalletButton v-if="!mainnetShowcase" />
        </div>
      </div>
    </header>

    <main class="mx-auto w-full max-w-6xl flex-1 px-4 pt-6 pb-20">
      <NetworkWarning v-if="!mainnetShowcase" />
      <RouterView v-slot="{ Component, route }">
        <Transition name="page" mode="out-in">
          <component :is="Component" :key="route.path" />
        </Transition>
      </RouterView>
    </main>

    <footer class="border-t border-stone-200/80 bg-sand/60">
      <div class="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-2 px-4 py-5 text-xs text-stone-600">
        <span class="font-display text-sm font-semibold text-ink">Birlikte biriktir. Her şeyi doğrula.</span>
        <span>
          {{ mainnetShowcase ? 'Mainnet tanıtımı: fon ve cüzdan işlemleri kapalı.' : 'Testnet prototipi, gerçek para kullanılmaz.' }}
        </span>
        <span v-if="poolContractId">
          Kontrat:
          <a
            :href="explorerContract(poolContractId)"
            target="_blank"
            rel="noopener noreferrer"
            class="font-mono text-brand-700 underline"
          >
            {{ poolContractId.slice(0, 6) }}…{{ poolContractId.slice(-4) }}
          </a>
        </span>
        <span v-else-if="!mainnetShowcase">Kontrat henüz yapılandırılmadı</span>
        <span v-else>Mainnet havuz kontratı yayınlanmadı</span>
      </div>
    </footer>
  </div>
</template>
