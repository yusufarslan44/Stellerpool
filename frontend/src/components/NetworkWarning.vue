<script setup lang="ts">
import { useWalletStore } from '@/stores/wallet'

const wallet = useWalletStore()
</script>

<template>
  <p
    v-if="wallet.networkWarning"
    role="alert"
    class="mb-6 rounded-2xl border border-gold-300/60 bg-gold-100/70 px-4 py-3 text-center text-sm text-amber-900"
  >
    {{ wallet.networkWarning }}
  </p>
</template>
