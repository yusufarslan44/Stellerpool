<script setup lang="ts">
/**
 * Small animated illustrations for the "How it works" steps (pure SVG + CSS).
 * Hareket azaltma tercihinde animasyonlar tek kareye iner (style.css).
 */
defineProps<{ kind: 1 | 2 | 3 | 4 | 5 }>()
</script>

<template>
  <svg viewBox="0 0 240 170" class="h-auto w-full" role="img" aria-hidden="true">
    <rect width="240" height="170" rx="22" fill="#f4f3e2" />
    <circle cx="200" cy="30" r="46" fill="#c9edd8" opacity="0.55" />

    <!-- 1: set up the pool -->
    <g v-if="kind === 1">
      <g v-for="(x, i) in [52, 100, 148, 196]" :key="x">
        <circle
          :cx="x"
          cy="92"
          r="19"
          fill="#fffaf2"
          stroke="#14805a"
          stroke-width="2.5"
          stroke-dasharray="5 5"
          class="pop"
          :style="{ animationDelay: `${i * 0.18}s`, transformOrigin: `${x}px 92px` }"
        />
        <path
          :d="`M${x - 6} 92h12M${x} 86v12`"
          stroke="#14805a"
          stroke-width="2.5"
          stroke-linecap="round"
          class="pop"
          :style="{ animationDelay: `${i * 0.18 + 0.1}s` }"
        />
      </g>
      <path d="M36 44h44" stroke="#2b1a12" stroke-width="5" stroke-linecap="round" opacity="0.8" />
      <path d="M36 58h70" stroke="#2b1a12" stroke-width="3" stroke-linecap="round" opacity="0.25" />
    </g>

    <!-- 2: shared approval of the group rules -->
    <g v-else-if="kind === 2">
      <circle cx="120" cy="92" r="38" fill="none" stroke="#14805a" stroke-width="2" style="transform-origin:120px 92px;animation:ring-ping 2.4s ease-out infinite" />
      <path
        d="M120 48c14 9 28 12 40 12v34c0 24-18 38-40 47-22-9-40-23-40-47V60c12 0 26-3 40-12Z"
        fill="#d5f3e1"
        stroke="#14805a"
        stroke-width="3.5"
        stroke-linejoin="round"
      />
      <path d="m104 96 12 12 22-24" fill="none" stroke="#0f6748" stroke-width="6" stroke-linecap="round" stroke-linejoin="round" pathLength="1" stroke-dasharray="1" style="animation:draw 1.2s ease-out 0.5s both" />
    </g>

    <!-- 3: kurallar kilitlenir -->
    <g v-else-if="kind === 3">
      <g v-for="(x, i) in [46, 88, 130, 172]" :key="x">
        <circle :cx="x" cy="52" r="15" fill="#d5f3e1" stroke="#14805a" stroke-width="2.5" />
        <path
          :d="`M${x - 6} 52l4 4 8-9`"
          fill="none"
          stroke="#0f6748"
          stroke-width="4"
          stroke-linecap="round"
          stroke-linejoin="round"
          pathLength="1"
          stroke-dasharray="1"
          :style="{ animation: `draw 0.6s ease-out ${0.3 + i * 0.35}s both` }"
        />
      </g>
      <g transform="translate(96 84)">
        <rect x="0" y="26" width="48" height="40" rx="9" fill="#14805a" />
        <circle cx="24" cy="44" r="5" fill="#f4f3e2" />
        <rect x="22" y="46" width="4" height="10" rx="2" fill="#f4f3e2" />
        <path
          d="M10 26v-9a14 14 0 0 1 28 0v9"
          fill="none"
          stroke="#2b1a12"
          stroke-width="6"
          stroke-linecap="round"
          class="pop"
          style="animation-delay: 1.6s"
        />
      </g>
    </g>

    <!-- 4: everyone pays -->
    <g v-else-if="kind === 4">
      <path d="M84 84h72l-6 66a8 8 0 0 1-8 7H98a8 8 0 0 1-8-7Z" fill="#fffaf2" stroke="#2b1a12" stroke-width="3" stroke-linejoin="round" />
      <path d="M92 118h56l-3 32a6 6 0 0 1-6 5H101a6 6 0 0 1-6-5Z" fill="#f2b134" opacity="0.85" />
      <path d="M78 84h84" stroke="#2b1a12" stroke-width="4" stroke-linecap="round" />
      <g v-for="(x, i) in [96, 112, 128, 144]" :key="x">
        <g :style="{ animation: `coin-fall 2.4s ease-in ${i * 0.5}s infinite` }">
          <circle :cx="x" cy="52" r="9" fill="#f2b134" stroke="#c98a10" stroke-width="2.5" />
          <path :d="`M${x - 3} 52h6`" stroke="#fdeec3" stroke-width="2.5" stroke-linecap="round" />
        </g>
      </g>
    </g>

    <!-- 5: the seller is paid -->
    <g v-else>
      <path d="M30 84h56l-5 60a7 7 0 0 1-7 6H42a7 7 0 0 1-7-6Z" fill="#fffaf2" stroke="#2b1a12" stroke-width="3" stroke-linejoin="round" />
      <path d="M40 112h40l-2 30a5 5 0 0 1-5 4H47a5 5 0 0 1-5-4Z" fill="#f2b134" opacity="0.85" />
      <path d="M24 84h68" stroke="#2b1a12" stroke-width="4" stroke-linecap="round" />
      <path d="M100 116h60" stroke="#14805a" stroke-width="3.5" stroke-linecap="round" stroke-dasharray="2 8" />
      <path d="m150 106 12 10-12 10" fill="none" stroke="#14805a" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" />
      <g style="animation: slide-x 2.2s ease-in-out infinite">
        <circle cx="124" cy="116" r="9" fill="#f2b134" stroke="#c98a10" stroke-width="2.5" />
      </g>
      <g transform="translate(168 92)">
        <path d="M4 14 12 2h32l8 12" fill="#d5f3e1" stroke="#2b1a12" stroke-width="3" stroke-linejoin="round" />
        <path d="M4 14h48v10a8 8 0 0 1-16 0 8 8 0 0 1-16 0 8 8 0 0 1-16 0Z" fill="#14805a" stroke="#2b1a12" stroke-width="3" stroke-linejoin="round" />
        <rect x="8" y="30" width="40" height="34" rx="3" fill="#fffaf2" stroke="#2b1a12" stroke-width="3" />
        <rect x="22" y="42" width="12" height="22" rx="2" fill="#2b1a12" opacity="0.8" />
      </g>
    </g>
  </svg>
</template>
