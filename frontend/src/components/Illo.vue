<script setup lang="ts">
import { computed } from 'vue'
import { illoUrl } from '@/lib/illo'
import type { IlloName } from '@/lib/icon-data'

/**
 * Renkli 3B tarzı çizim (Microsoft Fluent Emoji, MIT). Dosyalar src/assets/illo altında,
 * <img> olarak yalnızca kullanıldığında yüklenir. Süs amaçlıdır; anlam metinde verilir (alt="").
 */
const props = withDefaults(defineProps<{ name: IlloName; size?: number }>(), { size: 40 })

const src = computed(() => illoUrl(props.name))
</script>

<template>
  <img
    :src="src"
    alt=""
    aria-hidden="true"
    :width="size"
    :height="size"
    class="inline-block shrink-0 select-none drop-shadow-[0_6px_8px_rgb(120_53_15/0.22)]"
    :style="{ width: `${size}px`, height: `${size}px` }"
    draggable="false"
    decoding="async"
  />
</template>
