<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import Scene3D from '@/components/Scene3D.vue'
import Illo from '@/components/Illo.vue'
import { sha256, toHex } from '@/lib/hash'
import { illoUrl } from '@/lib/illo'
import type { IlloName } from '@/lib/icon-data'

/**
 * Anlatıcı, kendiliğinden oynayan örnek hikâye: dört arkadaş, iki tur.
 * Tur 1: Can geç öder, doğrulayıcılar onaylar, tutar satıcıya gider.
 * Tur 2: ilk turda alan Ayşe ödemez, ek süre biter, yalnızca o turun katkıları iade edilir.
 * Tamamen kurgusaldır (kişiler ve tutarlar örnek); zincire işlem gitmez. Kurallar Testnet
 * kontratının kurallarıdır. Tek gerçek hesap: alım belgesinin SHA-256 özeti tarayıcıda hesaplanır.
 *
 * Her sahne bir "anlık görüntü"dür (durum, tarihten bağımsız hesaplanır); bu yüzden ileri/geri
 * ve bölüm seçimi de çalışır.
 */
type Phase =
  | 'intro' | 'collect' | 'grace' | 'cured' | 'purchase' | 'verify' | 'pay' | 'done'
  | 'r2' | 'r2grace' | 'r2aborted'

interface Person {
  id: string
  name: string
  illo: IlloName
}
const PEOPLE: Person[] = [
  { id: 'ayse', name: 'Ayşe', illo: 'ayse' },
  { id: 'mehmet', name: 'Mehmet', illo: 'mehmet' },
  { id: 'zeynep', name: 'Zeynep', illo: 'zeynep' },
  { id: 'can', name: 'Can', illo: 'can' },
]
const ALL = PEOPLE.map((p) => p.id)
const VERIFIERS = ['A', 'B', 'C']
const AMOUNT = 10
const POT = AMOUNT * PEOPLE.length
const NEEDED = 2
const DOC = 'Örnek alım belgesi: araç, 40 birim, Örnek Galeri'

const reduced =
  typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

// --- Senaryo -------------------------------------------------------------------------------
interface Step {
  phase: Phase
  paid?: string[]
  approvals?: string[]
  refunded?: string[]
  /** Bu sahneye girilirken oynatılan para animasyonu. */
  fly?: { kind: 'in' | 'out' | 'send'; ids?: string[] }
  /** Sahnenin ekranda kalma süresi (ms); anlatı metninin okunma süresine göre. */
  hold: number
}
const A_M_Z = ['ayse', 'mehmet', 'zeynep']
const M_Z_C = ['mehmet', 'zeynep', 'can']

const STEPS: Step[] = [
  { phase: 'intro', hold: 6500 },
  { phase: 'collect', hold: 4200 },
  { phase: 'collect', paid: ['ayse'], fly: { kind: 'in', ids: ['ayse'] }, hold: 2000 },
  { phase: 'collect', paid: ['ayse', 'mehmet'], fly: { kind: 'in', ids: ['mehmet'] }, hold: 2000 },
  { phase: 'collect', paid: A_M_Z, fly: { kind: 'in', ids: ['zeynep'] }, hold: 2300 },
  { phase: 'grace', paid: A_M_Z, hold: 6500 },
  { phase: 'cured', paid: ALL, fly: { kind: 'in', ids: ['can'] }, hold: 5000 },
  { phase: 'purchase', paid: ALL, hold: 6000 },
  { phase: 'verify', paid: ALL, hold: 5200 },
  { phase: 'verify', paid: ALL, approvals: ['A'], hold: 2500 },
  { phase: 'pay', paid: ALL, approvals: ['A', 'B'], hold: 5500 },
  { phase: 'done', paid: ALL, approvals: ['A', 'B'], fly: { kind: 'send' }, hold: 6500 },
  { phase: 'r2', paid: [], hold: 6500 },
  { phase: 'r2', paid: ['mehmet'], fly: { kind: 'in', ids: ['mehmet'] }, hold: 2000 },
  { phase: 'r2', paid: ['mehmet', 'zeynep'], fly: { kind: 'in', ids: ['zeynep'] }, hold: 2000 },
  { phase: 'r2', paid: M_Z_C, fly: { kind: 'in', ids: ['can'] }, hold: 2200 },
  { phase: 'r2grace', paid: M_Z_C, hold: 6500 },
  { phase: 'r2aborted', paid: [], refunded: M_Z_C, fly: { kind: 'out', ids: M_Z_C }, hold: 11000 },
]

// --- Durum ---------------------------------------------------------------------------------
const index = ref(0)
/** Geçerli sahnede geçen süre (ms). Elle seçimden sonra negatif başlar: sahne biraz daha bekler. */
const elapsed = ref(0)
const autoplay = ref(!reduced)
const visible = ref(false)
const pageVisible = ref(true)

const step = computed(() => STEPS[index.value]!)
const phase = computed(() => step.value.phase)
const paid = computed(() => step.value.paid ?? [])
const approvals = computed(() => step.value.approvals ?? [])
const refunded = computed(() => step.value.refunded ?? [])
const rulesAccepted = computed(() => index.value > 0)
const round2 = computed(() => phase.value.startsWith('r2'))
const sent = computed(() => phase.value === 'done')
const docHash = ref<string | null>(null)
const showHash = computed(() => (['verify', 'pay', 'done'] as Phase[]).includes(phase.value))

const stage = ref<HTMLElement | null>(null)
const jarEl = ref<HTMLElement | null>(null)
const storeEl = ref<HTMLElement | null>(null)
const personEls: Record<string, HTMLElement | null> = {}

const isFunded = (id: string) => paid.value.includes(id)
const funded = computed(() => PEOPLE.filter((p) => isFunded(p.id)).length)
const recipient = computed(() => (round2.value ? 'mehmet' : 'ayse'))

const CHAPTERS = ['Kurallar', 'Katkı', 'Alım', 'Onay', 'Gönderim']
function chapterOf(p: Phase): number {
  switch (p) {
    case 'intro': return 0
    case 'collect': case 'grace': case 'cured': case 'r2': case 'r2grace': case 'r2aborted': return 1
    case 'purchase': return 2
    case 'verify': return 3
    default: return 4
  }
}
const chapter = computed(() => chapterOf(phase.value))
/** Bölüm çubuğundan seçilince ilk tur içindeki ilk sahneye gidilir. */
const chapterStart = (c: number) => STEPS.findIndex((s) => chapterOf(s.phase) === c)

// --- Uçan para animasyonu ------------------------------------------------------------------
function flyCoin(from: HTMLElement | null, to: HTMLElement | null, delay = 0) {
  if (reduced || !from || !to || !stage.value) return
  const s = stage.value.getBoundingClientRect()
  const a = from.getBoundingClientRect()
  const b = to.getBoundingClientRect()
  const size = 34
  const x0 = a.left - s.left + a.width / 2 - size / 2
  const y0 = a.top - s.top + a.height / 2 - size / 2
  const x1 = b.left - s.left + b.width / 2 - size / 2
  const y1 = b.top - s.top + b.height / 2 - size / 2
  const coin = document.createElement('img')
  coin.src = illoUrl('coin')
  coin.alt = ''
  Object.assign(coin.style, {
    position: 'absolute', left: '0', top: '0', width: `${size}px`, height: `${size}px`,
    pointerEvents: 'none', zIndex: '30', opacity: '0',
  })
  stage.value.appendChild(coin)
  const anim = coin.animate(
    [
      { transform: `translate(${x0}px, ${y0}px) scale(0.6)`, opacity: 0 },
      { transform: `translate(${(x0 + x1) / 2}px, ${Math.min(y0, y1) - 46}px) scale(1.15)`, opacity: 1, offset: 0.5 },
      { transform: `translate(${x1}px, ${y1}px) scale(0.7)`, opacity: 0.9 },
    ],
    { duration: 1100, delay, easing: 'cubic-bezier(0.3, 0.7, 0.3, 1)', fill: 'both' },
  )
  anim.onfinish = () => coin.remove()
}

function playFx(s: Step) {
  const fly = s.fly
  if (!fly) return
  if (fly.kind === 'in') fly.ids?.forEach((id) => flyCoin(personEls[id] ?? null, jarEl.value))
  else if (fly.kind === 'out') fly.ids?.forEach((id, i) => flyCoin(jarEl.value, personEls[id] ?? null, i * 160))
  else for (let i = 0; i < PEOPLE.length; i++) flyCoin(jarEl.value, storeEl.value, i * 140)
}

// --- Oynatıcı ------------------------------------------------------------------------------
const playing = computed(() => autoplay.value && visible.value && pageVisible.value && !reduced)
let raf = 0
let last = 0

function goTo(i: number, animate: boolean, hold = false) {
  index.value = (i + STEPS.length) % STEPS.length
  elapsed.value = hold ? -1500 : 0
  if (animate) void nextTick(() => playFx(step.value))
}
const next = () => goTo(index.value + 1, true, true)
const prev = () => goTo(index.value - 1, false, true)
const restart = () => goTo(0, false)
function togglePlay() {
  autoplay.value = !autoplay.value
}

function tick(now: number) {
  elapsed.value += Math.min(now - last, 100)
  last = now
  if (elapsed.value >= step.value.hold) goTo(index.value + 1, true)
  raf = requestAnimationFrame(tick)
}
watch(
  playing,
  (play) => {
    cancelAnimationFrame(raf)
    if (play) {
      last = performance.now()
      raf = requestAnimationFrame(tick)
    }
  },
  { immediate: true },
)

const progress = computed(() => {
  const within = Math.max(0, Math.min(1, elapsed.value / step.value.hold))
  return Math.round(((index.value + within) / STEPS.length) * 100)
})

let observer: IntersectionObserver | undefined
const updatePageVisible = () => (pageVisible.value = !document.hidden)
onMounted(async () => {
  updatePageVisible()
  document.addEventListener('visibilitychange', updatePageVisible)
  if (stage.value) {
    observer = new IntersectionObserver(([e]) => (visible.value = !!e?.isIntersecting), { threshold: 0.12 })
    observer.observe(stage.value)
  }
  docHash.value = toHex(await sha256(DOC))
})
onBeforeUnmount(() => {
  cancelAnimationFrame(raf)
  observer?.disconnect()
  document.removeEventListener('visibilitychange', updatePageVisible)
})

// --- Anlatım -------------------------------------------------------------------------------
const story = computed<{ title: string; text: string }>(() => {
  switch (phase.value) {
    case 'intro':
      return {
        title: 'Dört arkadaş, bir hedef',
        text: `Ayşe, Mehmet, Zeynep ve Can birlikte araç biriktiriyor. Her tur herkes ${AMOUNT} birim yatırır; tamamı gelince sıradaki kişinin ${POT} birimlik alımı yapılır. Önce herkes grup kurallarını kabul eder.`,
      }
    case 'collect':
      return {
        title: 'Herkes payını yatırır',
        text: `Kurallar kabul edildi. Şimdi herkes kendi ${AMOUNT} birimini yatırır; para bir kişinin cüzdanına değil, sözleşmeye gider. Kimse başkası adına ödeyemez.`,
      }
    case 'grace':
      return {
        title: 'Can yatırmadı: ek süre başladı',
        text: 'Süre doldu ama tur hemen durmadı: Can hâlâ kendi payını yatırabilir. Ek süre biterse tur durur ve bu turda yatırılan katkılar geri verilir.',
      }
    case 'cured':
      return {
        title: 'Can ek sürede ödedi',
        text: 'Bütün katkılar tamamlandı; tur alım adımına geçiyor. Ödeme geciktiğinde yalnızca ek süre devreye girdi, kimse zarar görmedi.',
      }
    case 'purchase':
      return {
        title: 'Alım önerisi',
        text: `Katkılar tamam: ${POT} birim sözleşmede. Şimdi Ayşe satıcıyı (Örnek Galeri) ve alım belgesini önerir. Belgenin kendisi değil, yalnızca parmak izi (SHA-256 özeti) kaydedilir.`,
      }
    case 'verify':
      return {
        title: 'Doğrulayıcılar belgeye bakar',
        text: `Üç doğrulayıcı belgeyi zincir dışında kontrol eder. Tutarın çıkması için en az ${NEEDED} onay gerekir (${approvals.value.length} / ${NEEDED}).`,
      }
    case 'pay':
      return {
        title: 'Onaylar tamam',
        text: 'Artık kim isterse “gönder” diyebilir; bir yöneticiye gerek yok. Para Ayşe’ye değil, doğrudan satıcıya gider.',
      }
    case 'done':
      return {
        title: 'Tur 1 / 4 tamamlandı',
        text: `${POT} birim satıcıya gitti, Ayşe’nin aracı yolda. Kimse parayı elle çekemedi. Sıradaki tur Mehmet’in. Peki Ayşe artık ödemezse?`,
      }
    case 'r2':
      return {
        title: 'Tur 2: Ayşe artık ödemiyor',
        text: `Sıra Mehmet’te. Ayşe ilk turda ${POT} birimlik alımını yaptırdı; şimdi bu turun payını yatırmıyor. Diğer üçü kendi ${AMOUNT}’ar birimini yatırıyor. Herkes ödemeden satıcıya hiçbir şey gitmez.`,
      }
    case 'r2grace':
      return {
        title: 'Ek süre başladı',
        text: 'Ayşe hâlâ kendi payını yatırabilir; yatırmazsa ek süre bitince tur durur. Kimse onun yerine ödeyemez.',
      }
    default:
      return {
        title: 'Tur durdu, ama ilk tur geri gelmiyor',
        text: `Bu turda yatırılan ${refunded.value.length * AMOUNT} birim üç arkadaşa iade edildi. Ancak 1. turdaki ${POT} birim çoktan satıcıya gitti; Mehmet, Zeynep ve Can’ın ilk tur payları kontrattan geri alınamaz. Ayşe’nin kalan borcunu kontrat tahsil edemez; bu, grubun ayrıca çözmesi gereken bir alacaktır.`,
      }
  }
})

const shortHash = computed(() =>
  showHash.value && docHash.value ? `${docHash.value.slice(0, 10)}…${docHash.value.slice(-4)}` : null,
)

function statusOf(id: string): { label: string; cls: string } {
  if (paid.value.includes(id)) return { label: 'Ödedi ✓', cls: 'bg-sage-100 text-sage-800' }
  if (refunded.value.includes(id)) return { label: 'İade aldı ✓', cls: 'bg-sage-100 text-sage-800' }
  if (phase.value === 'grace' && id === 'can') return { label: 'Geciktirdi', cls: 'bg-rose-100 text-rose-800' }
  if (round2.value && id === 'ayse') {
    return { label: phase.value === 'r2' ? 'Ödemiyor' : 'Ödemedi', cls: 'bg-rose-100 text-rose-800' }
  }
  return { label: 'Bekliyor', cls: 'bg-stone-100 text-stone-700' }
}
</script>

<template>
  <div class="story-board card overflow-hidden !p-0">
    <!-- Başlık: bölüm çubuğu + dürüstlük etiketi -->
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-stone-100 px-5 py-4 sm:px-7">
      <ol class="flex flex-wrap items-center gap-1.5 text-xs font-semibold" aria-label="Hikâye bölümleri">
        <li v-for="(c, i) in CHAPTERS" :key="c" class="flex items-center gap-1.5">
          <button
            type="button"
            class="cursor-pointer rounded-full px-2.5 py-1 transition-colors duration-300 hover:bg-brand-100"
            :class="i === chapter ? 'bg-brand-600 !text-white' : i < chapter ? 'bg-sage-100 text-sage-800' : 'bg-stone-100 text-stone-600'"
            :aria-current="i === chapter ? 'step' : undefined"
            @click="goTo(chapterStart(i), false, true)"
          >
            {{ i < chapter ? '✓ ' : '' }}{{ c }}
          </button>
          <span v-if="i < CHAPTERS.length - 1" class="text-stone-300" aria-hidden="true">›</span>
        </li>
      </ol>
      <div class="flex flex-wrap items-center gap-2">
        <span class="badge bg-brand-50 text-brand-800 ring-1 ring-brand-100">{{ round2 ? 'Tur 2' : 'Tur 1' }}</span>
        <span class="badge bg-gold-100 text-amber-900">Örnek hikâye · gerçek işlem değil</span>
      </div>
    </div>

    <!-- Anlatıcı -->
    <div class="flex items-start gap-3 bg-sand/50 px-5 py-4 sm:gap-4 sm:px-7">
      <Illo name="fox" :size="64" class="float" />
      <div
        :key="phase"
        class="pop relative min-h-[7.5rem] min-w-0 flex-1 rounded-3xl rounded-tl-lg bg-white p-4 shadow-[0_10px_26px_-16px_rgb(120_53_15/0.4)]"
        role="status"
        :aria-live="playing ? 'off' : 'polite'"
      >
        <p class="font-display text-lg font-extrabold">{{ story.title }}</p>
        <p class="mt-1 text-sm leading-relaxed text-stone-700">{{ story.text }}</p>
      </div>
    </div>

    <!-- Sahne -->
    <div ref="stage" class="story-stage relative grid gap-5 px-5 py-6 sm:px-7 md:grid-cols-[1.25fr_1fr_1fr]">
      <!-- Arkadaşlar -->
      <div class="order-2 md:order-1">
        <p class="eyebrow mb-2 text-stone-600">Arkadaşlar</p>
        <div class="grid grid-cols-2 gap-2.5">
          <div
            v-for="p in PEOPLE"
            :key="p.id"
            :ref="(el) => (personEls[p.id] = el as HTMLElement | null)"
            class="story-person relative flex min-h-[7.5rem] flex-col items-center justify-center gap-1 rounded-2xl border-2 bg-white p-2.5 text-center transition-[border-color,box-shadow] duration-300"
            :class="[
              isFunded(p.id) ? 'border-sage-300' : 'border-stone-200',
              p.id === recipient && !sent ? 'ring-2 ring-gold-300 ring-offset-2' : '',
            ]"
            :aria-label="`${p.name}: ${statusOf(p.id).label}`"
          >
            <Illo :name="p.illo" :size="48" />
            <span class="font-display text-sm font-bold">{{ p.name }}</span>
            <span class="badge !px-2 !py-0.5 text-[11px]" :class="statusOf(p.id).cls">{{ statusOf(p.id).label }}</span>
            <span v-if="p.id === recipient" class="absolute -top-2.5 left-2 flex items-center gap-1 rounded-full bg-gold-400 px-2 py-0.5 text-[10px] font-bold text-ink">
              {{ sent ? 'Aracı yolda' : 'Bu tur sırası' }}
              <Illo v-if="sent" name="car" :size="16" class="pop" />
            </span>
          </div>
        </div>
      </div>

      <!-- Kurallar + kavanoz -->
      <div class="order-1 flex flex-row items-center justify-around gap-3 md:order-2 md:flex-col md:justify-between">
        <div class="flex flex-col items-center">
          <span class="relative grid size-20 place-items-center rounded-3xl bg-gold-100">
            <span
              v-if="phase === 'intro'"
              class="absolute inset-0 rounded-3xl border-2 border-brand-500"
              style="animation: ring-ping 1.8s ease-out infinite"
              aria-hidden="true"
            />
            <Illo name="memo" :size="52" :class="rulesAccepted ? 'pop' : ''" />
          </span>
          <span class="mt-1 text-xs font-bold">Grup kuralları</span>
          <span class="badge mt-0.5 text-[11px]" :class="rulesAccepted ? 'bg-sage-100 text-sage-800' : 'bg-stone-100 text-stone-600'">
            {{ rulesAccepted ? 'Kabul edildi' : 'Onay bekliyor' }}
          </span>
        </div>

        <div class="flex flex-col items-center gap-2">
        <div :ref="(el) => (jarEl = el as HTMLElement | null)" class="relative">
          <div class="story-pool-scene">
            <Scene3D :coins="PEOPLE.length" :filled="sent ? 0 : funded" :paused="!playing" :label="`Havuz: ${sent ? 0 : funded * AMOUNT} / ${POT} birim; ${sent ? 0 : funded} üyenin katkısı sözleşmede`" />
          </div>
          <Illo v-if="funded === PEOPLE.length && !sent" name="lock" :size="34" class="pop absolute right-4 bottom-3" />
        </div>
        <p class="text-center text-sm font-bold tabular-nums">
          {{ sent ? 0 : funded * AMOUNT }} / {{ POT }} birim
          <span class="block text-xs font-normal text-stone-600">{{ sent ? `${POT} birim satıcıya gitti` : 'sözleşmede' }}</span>
        </p>
        <ul v-if="round2" class="w-full space-y-1.5 text-left text-[11px] leading-snug" aria-label="Tur geçmişi">
          <li class="flex items-start gap-1.5 rounded-xl bg-stone-100 p-2 text-stone-700">
            <Illo name="lock" :size="16" class="mt-0.5" />
            <span><b>Tur 1:</b> {{ POT }} birim Örnek Galeri’ye gitti; geri alınamaz.</span>
          </li>
          <li v-if="phase === 'r2aborted'" class="pop flex items-start gap-1.5 rounded-xl bg-sage-50 p-2 text-sage-800">
            <Illo name="check" :size="16" class="mt-0.5" />
            <span><b>Tur 2:</b> {{ refunded.length * AMOUNT }} birim sahiplerine iade edildi.</span>
          </li>
        </ul>
        </div>
      </div>

      <!-- Doğrulayıcılar + satıcı -->
      <div class="order-3 flex flex-col gap-4">
        <div>
          <p class="eyebrow mb-2 text-stone-600">Doğrulayıcılar</p>
          <div class="grid grid-cols-3 gap-2">
            <div
              v-for="v in VERIFIERS"
              :key="v"
              class="flex flex-col items-center gap-1 rounded-2xl border-2 bg-white p-2 text-center transition-[border-color] duration-300"
              :class="approvals.includes(v) ? 'border-sage-500 bg-sage-50' : 'border-stone-200 opacity-80'"
              :aria-label="`Doğrulayıcı ${v}: ${approvals.includes(v) ? 'onayladı' : 'bekliyor'}`"
            >
              <Illo name="magnifier" :size="34" />
              <span class="text-[11px] font-bold">{{ v }}</span>
              <span class="text-[10px]" :class="approvals.includes(v) ? 'font-bold text-sage-800' : 'text-stone-500'">
                {{ approvals.includes(v) ? 'Onayladı ✓' : 'Bekliyor' }}
              </span>
            </div>
          </div>
          <p v-if="shortHash" class="pop mt-2 break-all rounded-xl bg-sand/70 p-2 font-mono text-[11px] text-stone-700">
            Belge özeti (SHA-256): {{ shortHash }}
          </p>
        </div>

        <div :ref="(el) => (storeEl = el as HTMLElement | null)" class="mt-auto flex items-center gap-3 rounded-2xl bg-sand/60 p-3">
          <Illo name="store" :size="48" :class="sent ? 'pop' : ''" />
          <div class="min-w-0">
            <p class="text-sm font-bold">Örnek Galeri</p>
            <p class="text-xs text-stone-600">{{ sent ? `${POT} birim aldı` : 'Satıcı (önerilecek)' }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Oynatıcı -->
    <div class="space-y-3 border-t border-stone-100 bg-white px-5 py-4 sm:px-7">
      <div
        class="h-1.5 overflow-hidden rounded-full bg-stone-200"
        role="progressbar"
        aria-label="Hikâye ilerlemesi"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-valuenow="progress"
      >
        <div class="h-full rounded-full bg-gradient-to-r from-brand-500 to-gold-400 transition-[width] duration-200" :style="{ width: `${progress}%` }" />
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          class="btn-primary !min-h-10"
          :aria-label="autoplay ? 'Hikâyeyi duraklat' : 'Hikâyeyi oynat'"
          @click="togglePlay"
        >
          <svg v-if="autoplay" viewBox="0 0 24 24" class="size-4" fill="currentColor" aria-hidden="true"><path d="M6 5h4v14H6zM14 5h4v14h-4z" /></svg>
          <svg v-else viewBox="0 0 24 24" class="size-4" fill="currentColor" aria-hidden="true"><path d="M8 5v14l11-7z" /></svg>
          {{ autoplay ? 'Duraklat' : 'Oynat' }}
        </button>
        <button type="button" class="btn-secondary !min-h-10" aria-label="Önceki sahne" @click="prev">
          <AppIcon name="back" class="!size-4" />
        </button>
        <button type="button" class="btn-secondary !min-h-10" aria-label="Sonraki sahne" @click="next">
          <AppIcon name="arrow" class="!size-4" />
        </button>
        <button type="button" class="btn-secondary !min-h-10" @click="restart">
          <AppIcon name="refresh" class="!size-4" /> Baştan
        </button>
        <span class="text-xs tabular-nums text-stone-500">Sahne {{ index + 1 }} / {{ STEPS.length }}</span>
        <RouterLink to="/create" class="btn-secondary !min-h-10 ml-auto">
          Kendi havuzunu kur <AppIcon name="arrow" class="!size-4" />
        </RouterLink>
      </div>
    </div>

    <p class="border-t border-stone-100 bg-sand/40 px-5 py-3 text-xs leading-relaxed text-stone-600 sm:px-7">
      Bu bir anlatım örneğidir: kişiler, tutarlar ve satıcı kurgusaldır, zincire hiçbir işlem gitmez. Kurallar,
      Testnet’te yayındaki havuz kontratının kurallarını yansıtır.
    </p>
  </div>
</template>



<style scoped>
.story-board { border-color: #dde4d1; border-radius: 30px; box-shadow: 0 18px 50px -35px #2d53353b; }
.story-stage { background: radial-gradient(ellipse at 50% 50%,#e7eed970,transparent 65%),#fffef8; }
.story-pool-scene { width: 215px; height: 190px; border-radius: 50%; background: radial-gradient(ellipse,#ecf1dc88,transparent 65%); }
.story-person { box-shadow: 0 4px 0 #eef0e5; transition: transform .5s, border-color .5s, box-shadow .5s; }
.story-person:hover { transform: translateY(-3px); box-shadow: 0 7px 0 #e9eddf; }
@media(max-width:767px) { .story-pool-scene { width: 185px; height: 175px; } }
@media(max-width:370px) { .story-pool-scene { width: 153px; height: 155px; } }
@media(prefers-reduced-motion:reduce) { .story-person { transition: none; } .story-person:hover { transform: none; } }
</style>
