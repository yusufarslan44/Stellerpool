<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import { RouterLink, RouterView } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import { config, explorerContract, mainnetShowcase, poolContractId } from '@/lib/stellar'

const WalletButton = defineAsyncComponent(() => import('@/components/WalletButton.vue'))
const NetworkWarning = defineAsyncComponent(() => import('@/components/NetworkWarning.vue'))
</script>

<template>
  <div class="flex min-h-dvh flex-col">
    <!-- Yüzen cam menü: kenarlardan boşluklu, içerik altında kaybolmaz (sticky) -->
    <header class="sticky top-3 z-30 mx-auto mt-3 w-[calc(100%-1.5rem)] max-w-5xl">
      <div
        class="flex items-center justify-between gap-3 rounded-full border border-slate-200/80 bg-white/85 py-2 pr-2 pl-4 shadow-[0_8px_30px_-12px_rgb(15_23_42/0.25)] backdrop-blur-xl"
      >
        <div class="flex items-center gap-6">
          <RouterLink to="/" class="flex items-center gap-2 font-bold tracking-tight text-ink">
            <span
              class="grid size-8 place-items-center rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 text-white"
            >
              <AppIcon name="shield" class="!size-4" />
            </span>
            Stellerpool
          </RouterLink>
          <nav class="hidden items-center gap-5 text-sm font-medium text-slate-600 md:flex" aria-label="Ana menü">
            <RouterLink to="/" class="transition-colors duration-200 hover:text-ink" active-class="!text-ink">
              Ana sayfa
            </RouterLink>
            <RouterLink v-if="!mainnetShowcase" to="/create" class="transition-colors duration-200 hover:text-ink" active-class="!text-ink">
              Havuz oluştur
            </RouterLink>
          </nav>
        </div>
        <div class="flex items-center gap-2">
          <span
            class="badge hidden bg-emerald-100 text-emerald-800 sm:inline-flex"
            :title="mainnetShowcase ? 'Mainnet bilgisi okunur; işlem ve fon kabul edilmez.' : 'Testnet prototipi, gerçek para kullanılmaz.'"
          >
            {{ config.label }}
          </span>
          <WalletButton v-if="!mainnetShowcase" />
        </div>
      </div>
    </header>

    <main class="mx-auto w-full max-w-5xl flex-1 px-4 pt-6 pb-16">
      <NetworkWarning v-if="!mainnetShowcase" />
      <RouterView />
    </main>

    <footer class="border-t border-slate-200 bg-white/70">
      <div
        class="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-2 px-4 py-5 text-xs text-slate-600"
      >
        <span>Birlikte biriktir. Her şeyi doğrula.</span>
        <span class="text-slate-500">
          {{ mainnetShowcase ? 'Mainnet tanıtımı: fon ve cüzdan işlemleri kapalı.' : 'Testnet prototipi, gerçek para kullanılmaz.' }}
        </span>
        <span v-if="poolContractId">
          Kontrat:
          <a
            :href="explorerContract(poolContractId)"
            target="_blank"
            rel="noopener noreferrer"
            class="font-mono text-indigo-700 underline"
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
