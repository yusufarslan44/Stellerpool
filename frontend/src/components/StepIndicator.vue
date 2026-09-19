<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'

/** Sihirbaz adım göstergesi. Tamamlanan adımlara geri dönülebilir, ilerisine atlanamaz. */
defineProps<{ steps: { label: string }[]; current: number }>()
const emit = defineEmits<{ goto: [index: number] }>()
</script>

<template>
  <nav aria-label="Adımlar">
    <ol class="flex items-center">
      <li
        v-for="(s, i) in steps"
        :key="s.label"
        class="flex items-center"
        :class="i < steps.length - 1 ? 'flex-1' : ''"
      >
        <button
          type="button"
          class="flex min-h-11 items-center gap-2 rounded-full pr-1 text-left disabled:cursor-default"
          :disabled="i >= current"
          :aria-current="i === current ? 'step' : undefined"
          :aria-label="`Adım ${i + 1}: ${s.label}${i < current ? ' (tamamlandı, geri dön)' : ''}`"
          @click="emit('goto', i)"
        >
          <span
            class="grid size-9 shrink-0 place-items-center rounded-full text-sm font-bold transition-[background-color,color,box-shadow,transform] duration-300"
            :class="
              i < current
                ? 'bg-sage-600 text-white'
                : i === current
                  ? 'scale-110 bg-brand-600 text-white shadow-[0_0_0_5px_rgb(120_210_166/0.5)]'
                  : 'bg-stone-200 text-stone-600'
            "
          >
            <AppIcon v-if="i < current" name="check" class="!size-4" />
            <template v-else>{{ i + 1 }}</template>
          </span>
          <span
            class="text-sm font-semibold"
            :class="[i === current ? 'text-ink' : 'hidden text-stone-600 lg:inline', i < current ? '!text-sage-800' : '']"
          >
            {{ s.label }}
          </span>
        </button>
        <span
          v-if="i < steps.length - 1"
          class="relative mx-2 h-1 flex-1 overflow-hidden rounded-full bg-stone-200"
          aria-hidden="true"
        >
          <span
            class="absolute inset-y-0 left-0 rounded-full bg-sage-500 transition-[width] duration-500 ease-out"
            :style="{ width: i < current ? '100%' : '0%' }"
          />
        </span>
      </li>
    </ol>
  </nav>
</template>
