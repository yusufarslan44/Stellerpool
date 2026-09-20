<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import Illo from '@/components/Illo.vue'
import Scene3D from '@/components/Scene3D.vue'
import { formatStroops, parseAmount, toPlainAmount } from '@/lib/format'
import { contributionFor, goalMembers, GOALS } from '@/lib/goals'
import type { Goal } from '@/lib/goals'
import { poolAsset, poolContractId } from '@/lib/stellar'
import { MAX_MEMBERS, MIN_MEMBERS } from '@/types/pool'
import type { OrderMode } from '@/types/pool'

const token = poolAsset.getCode()
const goal = ref('demo')
const scenePaused = ref(false)
const pot = ref('40')
const members = ref(4)
const mode = ref<OrderMode>('Fixed')

function pick(g: Goal) {
  goal.value = g.id
  pot.value = g.pot
  members.value = goalMembers(g)
  mode.value = g.mode
}

const potStroops = computed(() => {
  try {
    const v = parseAmount(pot.value)
    return v > 0n ? v : null
  } catch {
    return null
  }
})

// Tur başına katkı: hedef tutar / üye sayısı, 0,01 birime aşağı yuvarlanır.
const contribution = computed(() =>
  potStroops.value === null ? null : contributionFor(potStroops.value, members.value),
)

const effectivePot = computed(() =>
  contribution.value === null ? null : contribution.value * BigInt(members.value),
)

const createLink = computed(() => ({
  path: '/join',
  query: {
    amount: contribution.value === null ? '' : toPlainAmount(contribution.value),
    members: String(members.value),
    mode: mode.value,
    goal: goal.value,
  },
}))
</script>

<template>
  <section class="plan-calculator card overflow-hidden !p-0" aria-labelledby="hesap-araci">
    <div class="grid lg:grid-cols-[1.1fr_1fr]">
      <div class="space-y-6 p-5 sm:p-8">
        <div>
          <h3 id="hesap-araci" class="text-2xl font-extrabold">Planını hesapla</h3>
          <p class="mt-1 text-sm text-stone-600">Testnet senaryosu için örnek katkı ve tur tutarı.</p>
        </div>

        <div>
          <p class="label" id="calc-goal">1. Ne için biriktiriyorsun?</p>
          <div class="flex flex-wrap gap-2" role="group" aria-labelledby="calc-goal">
            <button
              v-for="g in GOALS"
              :key="g.id"
              type="button"
              class="inline-flex min-h-11 cursor-pointer items-center gap-1.5 rounded-full border-2 px-4 text-sm font-semibold transition-[background-color,border-color,transform] duration-200 active:scale-95"
              :class="
                goal === g.id
                  ? 'border-brand-600 bg-brand-600 text-white'
                  : 'border-stone-200 bg-white text-stone-700 hover:border-brand-300'
              "
              :aria-pressed="goal === g.id"
              @click="pick(g)"
            >
              <Illo :name="g.icon" :size="22" />
              {{ g.label }}
            </button>
          </div>
        </div>

        <div class="grid gap-5 sm:grid-cols-2">
          <div>
            <label class="label" for="calc-pot">2. Her turda alınacak tutar ({{ token }})</label>
            <input
              id="calc-pot"
              v-model="pot"
              class="input text-lg font-bold"
              type="text"
              inputmode="decimal"
              autocomplete="off"
            />
          </div>
          <div>
            <label class="label" for="calc-members">3. Kaç kişi? <span class="text-brand-700">{{ members }}</span></label>
            <input
              id="calc-members"
              v-model.number="members"
              class="h-11 w-full cursor-pointer accent-brand-600"
              type="range"
              :min="MIN_MEMBERS"
              :max="MAX_MEMBERS"
              step="1"
            />
            <div class="flex justify-between text-xs text-stone-500" aria-hidden="true"><span>{{ MIN_MEMBERS }}</span><span>{{ MAX_MEMBERS }}</span></div>
          </div>
        </div>

        <dl v-if="contribution !== null && effectivePot !== null" class="grid grid-cols-2 gap-3">
          <div class="rounded-2xl bg-sand/70 p-3.5">
            <dt class="text-xs text-stone-600">Tur başına katkın</dt>
            <dd class="mt-0.5 font-display text-xl font-extrabold tabular-nums">{{ formatStroops(contribution) }}</dd>
          </div>
          <div class="rounded-2xl bg-sand/70 p-3.5">
            <dt class="text-xs text-stone-600">Toplam tur</dt>
            <dd class="mt-0.5 font-display text-xl font-extrabold tabular-nums">{{ members }}</dd>
          </div>
          <div class="rounded-2xl bg-brand-50 p-3.5 ring-1 ring-brand-100">
            <dt class="text-xs text-brand-800">Örnek tur tutarı</dt>
            <dd class="mt-0.5 font-display text-xl font-extrabold tabular-nums text-brand-900">
              {{ formatStroops(effectivePot) }}
            </dd>
          </div>
          <div class="rounded-2xl bg-sage-50 p-3.5 ring-1 ring-sage-100">
            <dt class="text-xs text-sage-800">Alıcı nasıl belirlenir?</dt>
            <dd class="mt-0.5 text-sm font-bold text-sage-900">
              {{ mode === 'Draw' ? 'Kura, tüm katkılar gelince' : 'Onaylanan sabit sıra' }}
            </dd>
          </div>
        </dl>
        <p v-else class="rounded-2xl bg-gold-100 p-3 text-sm text-amber-950" role="status">Geçerli bir tutar gir.</p>

        <div>
          <RouterLink
            :to="createLink"
            class="btn-primary btn-lg w-full sm:w-auto"
            :class="contribution === null ? 'pointer-events-none opacity-50' : ''"
            :aria-disabled="contribution === null"
          >
            {{ poolContractId ? 'Bu planla havuza katıl' : 'Planı incele' }}
            <AppIcon name="arrow" class="!size-4" />
          </RouterLink>
          <p class="mt-3 text-xs leading-relaxed text-stone-600">
            Örnek hesaptır, {{ token }} cinsindendir. Faiz ve vade farkı modellenmez. Bu bir Testnet
            prototipidir; gerçek para veya teslimat garantisi yoktur.
          </p>
        </div>
      </div>

      <!-- 3B: üye sayısı kadar para yörüngede döner -->
      <div class="calculator-scene sunrise relative min-h-[300px] border-t border-stone-100 lg:min-h-0 lg:border-t-0 lg:border-l">
        <div class="calculator-scene-top"><span>PLANININ GÖRSELİ</span><button type="button" :aria-label="scenePaused ? 'Plan animasyonunu oynat' : 'Plan animasyonunu duraklat'" @click="scenePaused = !scenePaused">{{ scenePaused ? '▷' : 'Ⅱ' }}</button></div>
        <Scene3D :paused="scenePaused" :coins="members" :label="`${members} üyeyi temsil eden ${members} altın para havuzun çevresinde dönüyor`" />
        <div class="glass absolute right-4 bottom-4 left-4 flex items-center justify-between gap-2 rounded-2xl px-4 py-2.5 text-sm font-semibold">
          <span>{{ members }} üye · {{ members }} tur</span>
          <span v-if="effectivePot !== null" class="tabular-nums text-brand-800">Tur: {{ formatStroops(effectivePot) }} {{ token }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.plan-calculator { border-radius: 30px; border-color: #dce4cf; background: #fffef8; box-shadow: 0 18px 45px -30px #3b61343d; }
.calculator-scene { background: radial-gradient(ellipse at 50% 45%,#f8f8e5,#dde8ca); }
.calculator-scene-top { position: absolute; z-index: 1; top: 22px; left: 24px; right: 24px; display: flex; align-items: center; justify-content: space-between; color: #859572; font-size: 9px; letter-spacing: .14em; }
.calculator-scene-top button { display: grid; place-items: center; width: 29px; height: 29px; border: 1px solid #b5c79c; border-radius: 50%; color: #6c8354; font-size: 12px; cursor: pointer; }
.plan-calculator :deep(input) { border-color: #d4dfc4; }
.plan-calculator [aria-pressed] { border-radius: 12px; font-size: 12px; }
</style>
