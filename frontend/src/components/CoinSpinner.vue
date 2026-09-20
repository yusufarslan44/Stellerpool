<script setup lang="ts">
/**
 * A three-dimensional spinning gold coin in pure CSS (loading and success indicator).
 * The thickness is built from layers at closely spaced z positions.
 */
withDefaults(defineProps<{ size?: number }>(), { size: 56 })
const LAYERS = 8
</script>

<template>
  <span
    class="inline-block"
    :style="{ width: `${size}px`, height: `${size}px`, perspective: `${size * 12}px` }"
    role="status"
    aria-label="Loading"
  >
    <span
      class="relative block size-full"
      style="transform-style: preserve-3d; animation: spin-y 2.6s linear infinite"
    >
      <span
        v-for="n in LAYERS"
        :key="n"
        class="absolute inset-0 rounded-full bg-gold-500"
        :style="{ transform: `translateZ(${(n - LAYERS / 2 - 0.5) * (size / 22)}px)` }"
      />
      <span
        v-for="side in [1, -1]"
        :key="side"
        class="absolute inset-0 grid place-items-center rounded-full border-[3px] border-gold-100/70 bg-gold-400"
        :style="{ transform: `rotateY(${side === 1 ? 0 : 180}deg) translateZ(${size / 5.2}px)`, backfaceVisibility: 'hidden' }"
      >
        <svg viewBox="0 0 24 24" class="size-3/5" aria-hidden="true">
          <path
            d="m12 3 2.7 5.6 6.1.8-4.5 4.3 1.1 6.1L12 17l-5.4 2.8 1.1-6.1-4.5-4.3 6.1-.8Z"
            fill="#fdeec3"
          />
        </svg>
      </span>
    </span>
  </span>
</template>
