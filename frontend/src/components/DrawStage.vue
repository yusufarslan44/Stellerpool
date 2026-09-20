<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import Illo from '@/components/Illo.vue'
import { shortAddress } from '@/lib/format'

/**
 * Kura sahnesi: her üye bir çip. `spinning` iken vurgu adaylar arasında döner (yalnızca görsel),
 * kazanan belli olunca çip büyür. Sonucu her zaman kontrat belirler; bu bileşen sonucu üretmez.
 */
const props = defineProps<{
  /** Kurada olan (henüz teslim almamış) üyeler. */
  candidates: string[]
  /** Daha önce teslim almış, kura dışı üyeler. */
  excluded: string[]
  /** Kontratın seçtiği kazanan; henüz çekilmediyse null. */
  winner: string | null
  spinning: boolean
  me: string | null
}>()

const all = computed(() => [...props.candidates, ...props.excluded])
const compact = computed(() => all.value.length > 12)
const excludedSet = computed(() => new Set(props.excluded))

const TONES = [
  'bg-brand-100 text-brand-800',
  'bg-gold-100 text-amber-900',
  'bg-sage-100 text-sage-800',
  'bg-sand text-ink-soft',
]
function tone(address: string): string {
  let h = 0
  for (const ch of address) h = (h * 31 + ch.charCodeAt(0)) >>> 0
  return TONES[h % TONES.length]!
}
const initials = (address: string) => address.slice(1, 3)

// Dönen vurgu: yalnızca kura çekilirken ve hareket azaltma kapalıyken.
const reduced =
  typeof window !== 'undefined' && window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
const cursor = ref(-1)
let timer: ReturnType<typeof setInterval> | undefined

function stop() {
  if (timer) clearInterval(timer)
  timer = undefined
  cursor.value = -1
}
watch(
  () => props.spinning && props.winner === null,
  (on) => {
    stop()
    if (!on || reduced || props.candidates.length === 0) return
    timer = setInterval(() => {
      cursor.value = (cursor.value + 1) % props.candidates.length
    }, 90)
  },
  { immediate: true },
)
onBeforeUnmount(stop)

const highlighted = computed(() => (cursor.value >= 0 ? props.candidates[cursor.value] : null))

const live = computed(() => {
  if (props.winner) return `Draw result: ${shortAddress(props.winner, 6)}${props.winner === props.me ? ' (you)' : ''}`
  if (props.spinning) return 'Drawing…'
  return `${props.candidates.length} members in the draw`
})
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <Illo :name="winner ? 'trophy' : 'dice'" :size="30" :class="spinning && !winner ? 'float' : ''" />
      <p class="font-display font-bold" role="status" aria-live="polite">{{ live }}</p>
    </div>

    <ul class="flex flex-wrap gap-1.5" aria-label="Members in the draw">
      <li
        v-for="a in all"
        :key="a"
        class="grid place-items-center rounded-full font-mono font-bold transition-[transform,box-shadow,opacity] duration-150"
        :class="[
          compact ? 'size-8 text-[0.65rem]' : 'size-11 text-xs',
          excludedSet.has(a) ? 'bg-stone-100 text-stone-400 opacity-60' : tone(a),
          winner === a ? 'pop z-10 scale-125 ring-4 ring-gold-400 shadow-[0_10px_24px_-8px_rgb(217_162_27/0.8)]' : '',
          highlighted === a ? 'scale-110 ring-2 ring-brand-500' : '',
          a === me && winner !== a ? 'outline outline-2 outline-offset-1 outline-ink/50' : '',
        ]"
        :title="`${a}${excludedSet.has(a) ? ' · already received' : ''}${a === me ? ' · you' : ''}`"
      >
        {{ initials(a) }}
      </li>
    </ul>

    <p class="text-xs leading-relaxed text-stone-600">
      The contract decides the result; this animation is visual only. Members who already received (faded chips) are not in the draw.
    </p>
  </div>
</template>
