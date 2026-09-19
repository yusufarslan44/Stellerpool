<script setup lang="ts">
import { computed } from 'vue'
import { UI_ICONS } from '@/lib/icon-data'
import type { IconName } from '@/types/icons'

/**
 * Arayüz simgeleri: Phosphor Icons (MIT). Denetim simgeleri kalın, içerik simgeleri çift tonlu;
 * renk currentColor'dan gelir. Kaynak: scripts/build-icons.mjs.
 */
const props = defineProps<{ name: IconName }>()
const icon = computed(() => UI_ICONS[props.name])
</script>

<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    :viewBox="`0 0 ${icon.width} ${icon.height}`"
    fill="currentColor"
    aria-hidden="true"
    class="size-5 shrink-0"
    v-html="icon.body"
  />
</template>
