<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import { formatStroops, parseAmount, toPlainAmount } from '@/lib/format'
import { requiredGuarantee } from '@/lib/guarantee'
import { poolAsset, poolContractId } from '@/lib/stellar'
import type { IconName } from '@/types/icons'

interface Goal {
  id: string
  label: string
  icon: IconName
  /** Örnek hedef tutar (bir turda satıcıya giden). Yalnızca başlangıç değeridir. */
  pot: string
  members: number
}

// Kategoriler yalnızca arayüz etiketidir; kontrat için hepsi aynıdır.
const GOALS: Goal[] = [
  { id: 'home', label: 'Ev', icon: 'home', pot: '60000', members: 12 },
  { id: 'car', label: 'Araç', icon: 'car', pot: '20000', members: 6 },
  { id: 'work', label: 'İş yeri', icon: 'briefcase', pot: '30000', members: 8 },
  { id: 'other', label: 'Diğer', icon: 'layers', pot: '6000', members: 6 },
  { id: 'demo', label: 'Demo', icon: 'shield', pot: '40', members: 4 },
]

const token = poolAsset.getCode()
const goal = ref('demo')
const pot = ref('40')
const members = ref(4)

function pick(g: Goal) {
  goal.value = g.id
  pot.value = g.pot
  members.value = g.members
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
const contribution = computed(() => {
  if (potStroops.value === null) return null
  const c = (potStroops.value / BigInt(members.value) / 100_000n) * 100_000n
  return c > 0n ? c : null
})

const effectivePot = computed(() =>
  contribution.value === null ? null : contribution.value * BigInt(members.value),
)

const guarantee = computed(() =>
  contribution.value === null ? null : requiredGuarantee(members.value, contribution.value),
)

const createLink = computed(() => ({
  path: '/create',
  query: {
    amount: contribution.value === null ? '' : toPlainAmount(contribution.value),
    members: String(members.value),
  },
}))
</script>

<template>
  <section class="glass rounded-bento p-5 text-slate-900 sm:p-6" aria-labelledby="hesap-araci">
    <h2 id="hesap-araci" class="text-lg font-semibold">Planını hesapla</h2>
    <p class="mt-0.5 text-sm text-slate-600">Testnet senaryosu için örnek katkı ve sponsor güvencesini hesapla.</p>

    <div class="mt-4 flex flex-wrap gap-2" role="group" aria-label="Amaç">
      <button
        v-for="g in GOALS"
        :key="g.id"
        type="button"
        class="inline-flex min-h-10 cursor-pointer items-center gap-1.5 rounded-full border px-3.5 text-sm font-medium transition-colors duration-200"
        :class="
          goal === g.id
            ? 'border-indigo-600 bg-indigo-600 text-white'
            : 'border-slate-300 bg-white text-slate-700 hover:border-slate-400'
        "
        :aria-pressed="goal === g.id"
        @click="pick(g)"
      >
        <AppIcon :name="g.icon" class="!size-4" />
        {{ g.label }}
      </button>
    </div>

    <div class="mt-4 grid gap-4 sm:grid-cols-2">
      <div>
        <label class="label" for="calc-pot">Her turda alınacak tutar ({{ token }})</label>
        <input
          id="calc-pot"
          v-model="pot"
          class="input"
          type="text"
          inputmode="decimal"
          autocomplete="off"
        />
      </div>
      <div>
        <label class="label" for="calc-members">Üye sayısı: {{ members }}</label>
        <input
          id="calc-members"
          v-model.number="members"
          class="h-11 w-full cursor-pointer accent-indigo-600"
          type="range"
          min="2"
          max="12"
          step="1"
        />
      </div>
    </div>

    <dl v-if="contribution !== null && effectivePot !== null && guarantee !== null" class="mt-4 grid grid-cols-2 gap-3">
      <div class="rounded-2xl bg-slate-100/80 p-3">
        <dt class="text-xs text-slate-600">Tur başına katkın</dt>
        <dd class="mt-0.5 text-lg font-bold tabular-nums">{{ formatStroops(contribution) }}</dd>
      </div>
      <div class="rounded-2xl bg-slate-100/80 p-3">
        <dt class="text-xs text-slate-600">Toplam tur</dt>
        <dd class="mt-0.5 text-lg font-bold tabular-nums">{{ members }}</dd>
      </div>
      <div class="rounded-2xl bg-indigo-50 p-3">
        <dt class="text-xs text-indigo-800">Örnek tur tutarı</dt>
        <dd class="mt-0.5 text-lg font-bold tabular-nums text-indigo-900">
          {{ formatStroops(effectivePot) }}
        </dd>
      </div>
      <div class="rounded-2xl bg-emerald-50 p-3">
        <dt class="text-xs text-emerald-800">Sponsor güvencesi</dt>
        <dd class="mt-0.5 text-lg font-bold tabular-nums text-emerald-900">
          {{ formatStroops(guarantee) }}
        </dd>
      </div>
    </dl>
    <p v-else class="mt-4 rounded-2xl bg-amber-50 p-3 text-sm text-amber-900" role="status">
      Geçerli bir tutar gir.
    </p>

    <RouterLink
      :to="createLink"
      class="btn-primary mt-4 w-full"
      :class="contribution === null ? 'pointer-events-none opacity-50' : ''"
      :aria-disabled="contribution === null"
    >
      {{ poolContractId ? 'Bu planla havuz oluştur' : 'Planı incele (kontrat bekleniyor)' }}
      <AppIcon name="arrow" class="!size-4" />
    </RouterLink>
    <p class="mt-3 text-xs leading-relaxed text-slate-600">
      Örnek hesaptır, {{ token }} cinsindendir. Faiz ve vade farkı modellenmez. Havuz kontratı henüz
      yayınlanmadı; gerçek para veya teslimat garantisi yoktur.
    </p>
  </section>
</template>
