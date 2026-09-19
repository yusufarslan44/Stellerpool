<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import WalletButton from '@/components/WalletButton.vue'
import { config, explorerContract, poolContractId } from '@/lib/stellar'
import { useWalletStore } from '@/stores/wallet'

const wallet = useWalletStore()
</script>

<template>
  <div class="flex min-h-dvh flex-col">
    <header class="sticky top-0 z-10 border-b border-slate-200 bg-white/90 backdrop-blur">
      <div class="mx-auto flex max-w-5xl items-center justify-between gap-3 px-4 py-3">
        <div class="flex items-center gap-5">
          <RouterLink to="/" class="text-lg font-bold tracking-tight text-indigo-700">
            Stellerpool
          </RouterLink>
          <nav class="hidden items-center gap-4 text-sm font-medium text-slate-600 sm:flex">
            <RouterLink to="/" class="hover:text-slate-900" active-class="text-slate-900">
              Ana sayfa
            </RouterLink>
            <RouterLink to="/create" class="hover:text-slate-900" active-class="text-slate-900">
              Havuz oluştur
            </RouterLink>
          </nav>
        </div>
        <div class="flex items-center gap-3">
          <span
            class="badge hidden bg-emerald-100 text-emerald-800 sm:inline-flex"
            title="Uygulama Stellar Testnet üzerinde çalışır, gerçek para kullanılmaz."
          >
            {{ config.label }}
          </span>
          <WalletButton />
        </div>
      </div>
      <p
        v-if="wallet.networkWarning"
        role="alert"
        class="border-t border-amber-200 bg-amber-50 px-4 py-2 text-center text-sm text-amber-900"
      >
        {{ wallet.networkWarning }}
      </p>
    </header>

    <main class="mx-auto w-full max-w-5xl flex-1 px-4 py-8">
      <RouterView />
    </main>

    <footer class="border-t border-slate-200 bg-white">
      <div
        class="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-2 px-4 py-4 text-xs text-slate-500"
      >
        <span>Birlikte biriktir. Her şeyi doğrula.</span>
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
        <span v-else>Kontrat henüz yapılandırılmadı</span>
      </div>
    </footer>
  </div>
</template>
