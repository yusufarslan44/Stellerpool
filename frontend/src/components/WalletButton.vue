<script setup lang="ts">
import AppIcon from './AppIcon.vue'
import { shortAddress } from '@/lib/format'
import { useWalletStore } from '@/stores/wallet'

const wallet = useWalletStore()
</script>

<template>
  <div class="flex items-center gap-2">
    <template v-if="wallet.isConnected && wallet.address">
      <span
        class="wallet-address rounded-full bg-stone-100 px-3 py-1.5 font-mono text-xs text-stone-700"
        :title="wallet.address"
      >
        {{ shortAddress(wallet.address) }}
      </span>
      <button type="button" class="btn-secondary !min-h-9 !px-3 !py-1.5" @click="wallet.disconnect()">
        Çıkış
      </button>
    </template>
    <button
      v-else
      type="button"
      class="btn-primary !min-h-9 !px-4 !py-1.5"
      :disabled="wallet.busy"
      @click="wallet.connect()"
    >
      <AppIcon name="wallet" class="wallet-symbol" />
      {{ wallet.busy ? 'Bağlanıyor…' : 'Cüzdan bağla' }}
    </button>
  </div>
</template>
