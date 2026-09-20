<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import Scene3D from '@/components/Scene3D.vue'
import Illo from '@/components/Illo.vue'
import { sha256, toHex } from '@/lib/hash'
import { illoUrl } from '@/lib/illo'
import type { IlloName } from '@/lib/icon-data'

/** A visual narration of the normal flow: four contributions, the purchase record and payment to the seller.
 * The people and amounts are examples; no transaction is sent to the chain.
 * Each scene carries its own state; forward/back and chapter selection give the same result.
 */
type Phase = 'intro' | 'collect' | 'funded' | 'purchase' | 'pay' | 'done'

interface Person {
  id: string
  name: string
  /** Possessive form: Ayşe’s, Mehmet’s, Zeynep’s, Can’s. */
  gen: string
  illo: IlloName
}
const PEOPLE: Person[] = [
  { id: 'ayse', name: 'Ayşe', gen: 'Ayşe’s', illo: 'ayse' },
  { id: 'mehmet', name: 'Mehmet', gen: 'Mehmet’s', illo: 'mehmet' },
  { id: 'zeynep', name: 'Zeynep', gen: 'Zeynep’s', illo: 'zeynep' },
  { id: 'can', name: 'Can', gen: 'Can’s', illo: 'can' },
]
const ALL = PEOPLE.map((p) => p.id)
const AMOUNT = 10
const POT = AMOUNT * PEOPLE.length
// Plan: the down payment goes into the contract when joining; when the turn comes it is added only to the member's own purchase (purchase = pool + recipient's down payment).
const DOWN = 2
const PAYOUT = POT + DOWN
const DOC = `Sample purchase document: car, ${PAYOUT} units, Sample Gallery`

const reduced =
  typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

// --- Senaryo -------------------------------------------------------------------------------
interface Step {
  phase: Phase
  paid?: string[]
  /** Coin animation played when entering this scene. */
  fly?: { kind: 'in' | 'send'; ids?: string[] }
  /** How long the scene stays on screen (ms), based on the reading time of the narration. */
  hold: number
}
const A_M_Z = ['ayse', 'mehmet', 'zeynep']

const STEPS: Step[] = [
  { phase: 'intro', hold: 3500 },
  { phase: 'collect', hold: 2200 },
  { phase: 'collect', paid: ['ayse'], fly: { kind: 'in', ids: ['ayse'] }, hold: 1500 },
  { phase: 'collect', paid: ['ayse', 'mehmet'], fly: { kind: 'in', ids: ['mehmet'] }, hold: 1500 },
  { phase: 'collect', paid: A_M_Z, fly: { kind: 'in', ids: ['zeynep'] }, hold: 1500 },
  { phase: 'funded', paid: ALL, fly: { kind: 'in', ids: ['can'] }, hold: 2800 },
  { phase: 'purchase', paid: ALL, hold: 3200 },
  { phase: 'pay', paid: ALL, hold: 2800 },
  { phase: 'done', paid: ALL, fly: { kind: 'send' }, hold: 5000 },
]

// --- Durum ---------------------------------------------------------------------------------
const index = ref(0)
/** Time elapsed in the current scene (ms). After a manual selection it starts negative: the scene waits a bit longer. */
const elapsed = ref(0)
const autoplay = ref(!reduced)
const visible = ref(false)
const pageVisible = ref(true)

const step = computed(() => STEPS[index.value]!)
const phase = computed(() => step.value.phase)
const paid = computed(() => step.value.paid ?? [])
const rulesAccepted = computed(() => index.value > 0)
const sent = computed(() => phase.value === 'done')
const docHash = ref<string | null>(null)
const showHash = computed(() => (['purchase', 'pay', 'done'] as Phase[]).includes(phase.value))

const stage = ref<HTMLElement | null>(null)
const jarEl = ref<HTMLElement | null>(null)
const storeEl = ref<HTMLElement | null>(null)
const personEls: Record<string, HTMLElement | null> = {}

const isFunded = (id: string) => paid.value.includes(id)
const funded = computed(() => PEOPLE.filter((p) => isFunded(p.id)).length)
const recipient = 'ayse'
const incoming = computed(() => step.value.fly?.kind === 'in' ? step.value.fly.ids?.[0] : undefined)
const balance = computed(() => sent.value ? 0 : funded.value * AMOUNT)

const CHAPTERS = ['Rules', 'Contribution', 'Purchase', 'Payment']
function chapterOf(p: Phase): number {
  switch (p) {
    case 'intro': return 0
    case 'collect': case 'funded': return 1
    case 'purchase': return 2
    case 'pay': case 'done': return 3
  }
}
const chapter = computed(() => chapterOf(phase.value))
/** Choosing from the chapter bar goes to the first scene of that chapter. */
const chapterStart = (c: number) => STEPS.findIndex((s) => chapterOf(s.phase) === c)

const flights = new Map<Animation, HTMLImageElement>()
function clearFlights() {
  for (const [animation, element] of flights) { animation.cancel(); element.remove() }
  flights.clear()
}

// --- Flying coin animation ------------------------------------------------------------------
function flyCoin(from: HTMLElement | null, to: HTMLElement | null, delay = 0) {
  if (reduced || !from || !to || !stage.value) return
  const s = stage.value.getBoundingClientRect()
  const a = from.getBoundingClientRect()
  const b = to.getBoundingClientRect()
  const size = 42
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
    { duration: 1000, delay, easing: 'cubic-bezier(0.3, 0.7, 0.3, 1)', fill: 'both' },
  )
  flights.set(anim, coin)
  anim.onfinish = () => { coin.remove(); flights.delete(anim) }
  anim.oncancel = () => { coin.remove(); flights.delete(anim) }
}

function playFx(s: Step) {
  const fly = s.fly
  if (!fly) return
  if (fly.kind === 'in') fly.ids?.forEach((id) => flyCoin(personEls[id] ?? null, jarEl.value))
  else for (let i = 0; i < PEOPLE.length; i++) flyCoin(jarEl.value, storeEl.value, i * 150)
}

// --- Player ------------------------------------------------------------------------------
const playing = computed(() => autoplay.value && visible.value && pageVisible.value && !reduced)
let raf = 0
let last = 0

function goTo(i: number, animate: boolean, hold = false) {
  clearFlights()
  index.value = (i + STEPS.length) % STEPS.length
  elapsed.value = hold ? -1000 : 0
  void nextTick(paintBar)
  const selectedIndex = index.value
  if (animate) void nextTick(() => { if (index.value === selectedIndex) playFx(STEPS[selectedIndex]!) })
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
  paintBar()
  raf = requestAnimationFrame(tick)
}
watch(
  playing,
  (play) => {
    cancelAnimationFrame(raf)
    for (const animation of flights.keys()) { if (play) animation.play(); else animation.pause() }
    if (play) {
      last = performance.now()
      raf = requestAnimationFrame(tick)
    }
  },
  { immediate: true },
)

const barEl = ref<HTMLElement | null>(null)
function fraction(): number {
  if (sent.value) return 1
  const within = Math.max(0, Math.min(1, elapsed.value / step.value.hold))
  return (index.value + within) / (STEPS.length - 1)
}
function paintBar() {
  if (barEl.value) barEl.value.style.transform = `scaleX(${Math.min(1, fraction())})`
}
const progress = computed(() => {
  if (sent.value) return 100
  const within = Math.max(0, Math.min(1, elapsed.value / step.value.hold))
  return Math.round(((index.value + within) / (STEPS.length - 1)) * 100)
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
  clearFlights()
  cancelAnimationFrame(raf)
  observer?.disconnect()
  document.removeEventListener('visibilitychange', updatePageVisible)
})

// --- Narration -------------------------------------------------------------------------------
const story = computed<{ title: string; text: string }>(() => {
  switch (phase.value) {
    case 'intro':
      return { title: 'Four friends. One shared goal.', text: `Ayşe, Mehmet, Zeynep and Can agree on a plan: ${AMOUNT} units each per round, ${PEOPLE.length} rounds. Round one is Ayşe’s.` }
    case 'collect': {
      const person = PEOPLE.find(p => p.id === incoming.value)
      return person
        ? { title: `${person.gen} contribution joined the pool`, text: `${person.name} deposits ${AMOUNT} units. The pool now holds ${balance.value}.` }
        : { title: 'Small contributions, one shared amount', text: `Each friend deposits ${AMOUNT} units into the shared contract.` }
    }
    case 'funded':
      return { title: '4 contributions combined. 40 units ready!', text: `Can pays too. The ${POT}-unit pool is complete, and it is Ayşe’s turn to register her purchase.` }
    case 'purchase':
      return { title: 'Ayşe proposes her purchase', text: `Ayşe registers Sample Gallery and the document digest on-chain. Purchase: ${POT} pool + ${DOWN} her down payment = ${PAYOUT} units.` }
    case 'pay':
      return { title: 'Purchase recorded. Ready to pay.', text: `${PAYOUT} units can go only to this seller. Anyone can trigger the payment; no administrator is needed.` }
    case 'done':
      return { title: 'Round one done. Mehmet is next!', text: `${PAYOUT} units were paid straight to Sample Gallery. Next round, everyone contributes again and it is Mehmet’s turn.` }
  }
})

const shortHash = computed(() =>
  showHash.value && docHash.value ? `${docHash.value.slice(0, 10)}…${docHash.value.slice(-4)}` : null,
)

function statusOf(id: string): { label: string; cls: string } {
  return paid.value.includes(id)
    ? { label: `+${AMOUNT} units ✓`, cls: 'bg-sage-100 text-sage-800' }
    : { label: 'Awaiting contribution', cls: 'bg-stone-100 text-stone-700' }
}

</script>

<template>
  <div class="story-board card overflow-hidden !p-0" :class="{ 'story-complete': sent, 'story-paused': !playing }">
    <!-- Header: chapter bar + honesty label -->
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-stone-100 px-5 py-4 sm:px-7">
      <ol class="flex flex-wrap items-center gap-1.5 text-xs font-semibold" aria-label="Story chapters">
        <li v-for="(c, i) in CHAPTERS" :key="c" class="flex items-center gap-1.5">
          <button
            type="button"
            class="cursor-pointer rounded-full px-2.5 py-1 transition-colors duration-300 hover:bg-brand-100"
            :class="i === chapter ? 'bg-brand-600 !text-white' : i < chapter ? 'bg-sage-100 text-sage-800' : 'bg-stone-100 text-stone-600'"
            :aria-current="i === chapter ? 'step' : undefined"
            @click="goTo(chapterStart(i), true, true)"
          >
            {{ i < chapter ? '✓ ' : '' }}{{ c }}
          </button>
          <span v-if="i < CHAPTERS.length - 1" class="text-stone-300" aria-hidden="true">›</span>
        </li>
      </ol>
      <div class="flex flex-wrap items-center gap-2">
        <span class="badge bg-brand-50 text-brand-800 ring-1 ring-brand-100">Sample round 1 / 4</span>
        <span class="badge bg-gold-100 text-amber-900">Sample story · not a real transaction</span>
      </div>
    </div>

    <!-- Narrator -->
    <div class="flex items-start gap-3 bg-sand/50 px-5 py-4 sm:gap-4 sm:px-7">
      <div class="story-chapter-token" aria-hidden="true"><span>0{{ chapter + 1 }}</span><small>{{ CHAPTERS[chapter] }}</small></div>
      <div
        :key="index"
        class="pop relative min-h-[7.5rem] min-w-0 flex-1 rounded-3xl rounded-tl-lg bg-white p-4 shadow-[0_10px_26px_-16px_rgb(120_53_15/0.4)]"
        role="status"
        :aria-live="playing ? 'off' : 'polite'"
      >
        <p class="font-display text-lg font-extrabold">{{ story.title }}</p>
        <p class="mt-1 text-sm leading-relaxed text-stone-700">{{ story.text }}</p>
      </div>
    </div>

    <div class="story-scoreboard" aria-label="Round summary">
      <div><span>SHARED GOAL</span><strong><AppIcon name="car" /> Ayşe’s purchase</strong></div>
      <div><span>CONTRIBUTIONS</span><strong :key="funded" class="pop">{{ funded }}<small> / {{ PEOPLE.length }} people</small></strong></div>
      <div><span>{{ sent ? 'PAID TO SELLER' : 'IN THE CONTRACT' }}</span><strong :key="`${balance}-${sent}`" class="pop">{{ sent ? PAYOUT : balance }}<small> units</small></strong></div>
      <div><span>PURCHASE RECORD</span><strong class="pop"><small>{{ chapter >= 2 ? 'Document digest recorded' : 'Pending' }}</small></strong></div>
    </div>

    <!-- Sahne -->
    <div ref="stage" class="story-stage relative grid gap-5 px-5 py-6 sm:px-7 md:grid-cols-[1.25fr_1fr_1fr]">
      <!-- Friends -->
      <div class="order-2 md:order-1">
        <p class="eyebrow mb-2 text-stone-600">Friends</p>
        <div class="grid grid-cols-2 gap-2.5">
          <div
            v-for="p in PEOPLE"
            :key="p.id"
            :ref="(el) => (personEls[p.id] = el as HTMLElement | null)"
            class="story-person relative flex min-h-[7.5rem] flex-col items-center justify-center gap-1 rounded-2xl border-2 bg-white p-2.5 text-center transition-[border-color,box-shadow] duration-300"
            :class="[
              isFunded(p.id) ? 'border-sage-300 is-funded' : 'border-stone-200',
              p.id === incoming ? 'is-contributing' : '',
              p.id === recipient && !sent ? 'ring-2 ring-gold-300 ring-offset-2' : '',
            ]"
            :aria-label="`${p.name}: ${statusOf(p.id).label}`"
          >
            <Illo :name="p.illo" :size="48" />
            <span class="font-display text-sm font-bold">{{ p.name }}</span>
            <span class="badge !px-2 !py-0.5 text-[11px]" :class="statusOf(p.id).cls">{{ statusOf(p.id).label }}</span>
            <span v-if="p.id === recipient" class="absolute -top-2.5 left-2 flex items-center gap-1 rounded-full bg-gold-400 px-2 py-0.5 text-[10px] font-bold text-ink">
              {{ sent ? 'Purchase paid' : 'This round’s turn' }}
              <Illo v-if="sent" name="car" :size="16" class="pop" />
            </span>
          </div>
        </div>
      </div>

      <!-- Rules + shared pool -->
      <div class="order-1 flex flex-row items-center justify-around gap-3 md:order-2 md:flex-col md:justify-between">
        <div class="flex flex-col items-center">
          <span class="relative grid size-20 place-items-center rounded-3xl bg-gold-100">
            <span
              v-if="phase === 'intro'"
              class="story-rule-pulse absolute inset-0 rounded-3xl border-2 border-brand-500"
              style="animation: ring-ping 1.8s ease-out infinite"
              aria-hidden="true"
            />
            <Illo name="memo" :size="52" :class="rulesAccepted ? 'pop' : ''" />
          </span>
          <span class="mt-1 text-xs font-bold">Group rules</span>
          <span class="badge mt-0.5 text-[11px]" :class="rulesAccepted ? 'bg-sage-100 text-sage-800' : 'bg-stone-100 text-stone-600'">
            {{ rulesAccepted ? 'Accepted' : 'Awaiting approval' }}
          </span>
        </div>

        <div class="flex flex-col items-center gap-2">
        <div :ref="(el) => (jarEl = el as HTMLElement | null)" class="relative">
          <div class="story-pool-scene" :class="{ 'pool-ready': funded === PEOPLE.length && !sent }">
            <Scene3D :coins="PEOPLE.length" :filled="sent ? 0 : funded" :paused="!playing" :label="`Pool: ${sent ? 0 : funded * AMOUNT} / ${POT} units; ${sent ? 0 : funded} members’ contributions are in the contract`" />
          </div>
          <Illo v-if="funded === PEOPLE.length && !sent" name="lock" :size="34" class="pop absolute right-4 bottom-3" />
        </div>
        <p class="text-center text-sm font-bold tabular-nums">
          {{ sent ? 0 : funded * AMOUNT }} / {{ POT }} units
          <span class="block text-xs font-normal text-stone-600">{{ sent ? `${PAYOUT} units went to the seller` : 'in the contract' }}</span>
        </p>
        </div>
      </div>

      <!-- Purchase record and the registered demo seller -->
      <div class="order-3 flex flex-col gap-4">
        <div class="rounded-2xl border border-stone-200 bg-white p-3">
          <p class="eyebrow mb-2 text-stone-600">Purchase record</p>
          <p class="text-sm text-stone-700">The recipient registers the demo seller and the document digest. Purchase = pool {{ POT }} + recipient’s down payment {{ DOWN }} = {{ PAYOUT }} units.</p>
          <p v-if="shortHash" class="pop mt-2 break-all rounded-xl bg-sand/70 p-2 font-mono text-[11px] text-stone-700">
            Document digest (SHA-256): {{ shortHash }}
          </p>
        </div>

        <div :ref="(el) => (storeEl = el as HTMLElement | null)" class="story-store mt-auto flex items-center gap-3 rounded-2xl bg-sand/60 p-3" :class="{ 'is-paid': sent, 'is-proposed': chapter >= 2 }">
          <Illo name="store" :size="48" :class="sent ? 'pop' : ''" />
          <div class="min-w-0">
            <p class="text-sm font-bold">Sample Gallery</p>
            <p class="text-xs text-stone-600">{{ sent ? `Received ${PAYOUT} units ✓` : chapter >= 2 ? 'The seller Ayşe proposed' : 'Seller (to be proposed)' }}</p>
          </div>
        </div>
      </div>
    </div>

    <Transition name="story-finale">
      <div v-if="sent" class="story-finale" :aria-live="playing ? 'off' : 'polite'">
        <div class="finale-art" aria-hidden="true"><span /><Illo name="car" :size="78" /></div>
        <div class="finale-copy"><span>COMPLETED TOGETHER</span><strong>{{ PAYOUT }} units, straight to the seller.</strong><p>Ayşe’s sample purchase is paid. In the new round it is Mehmet’s turn.</p><div class="finale-order"><span v-for="(person, i) in PEOPLE" :key="person.id" :class="{ completed: i === 0, next: i === 1 }">{{ person.name }} <small>{{ i === 0 ? '✓' : i === 1 ? 'Next' : `0${i + 1}` }}</small></span></div></div>
      </div>
    </Transition>

    <!-- Player -->
    <div class="space-y-3 border-t border-stone-100 bg-white px-5 py-4 sm:px-7">
      <div
        class="h-1.5 overflow-hidden rounded-full bg-stone-200"
        role="progressbar"
        aria-label="Story progress"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-valuenow="progress"
      >
        <div ref="barEl" class="h-full w-full origin-left rounded-full bg-gradient-to-r from-brand-500 to-gold-400 will-change-transform" style="transform: scaleX(0)" />
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          class="btn-primary !min-h-10"
          :aria-label="autoplay ? 'Pause the story' : 'Play the story'"
          @click="togglePlay"
        >
          <svg v-if="autoplay" viewBox="0 0 24 24" class="size-4" fill="currentColor" aria-hidden="true"><path d="M6 5h4v14H6zM14 5h4v14h-4z" /></svg>
          <svg v-else viewBox="0 0 24 24" class="size-4" fill="currentColor" aria-hidden="true"><path d="M8 5v14l11-7z" /></svg>
          {{ autoplay ? 'Pause' : 'Play' }}
        </button>
        <button type="button" class="btn-secondary !min-h-10" aria-label="Previous scene" @click="prev">
          <AppIcon name="back" class="!size-4" />
        </button>
        <button type="button" class="btn-secondary !min-h-10" aria-label="Next scene" @click="next">
          <AppIcon name="arrow" class="!size-4" />
        </button>
        <button type="button" class="btn-secondary !min-h-10" @click="restart">
          <AppIcon name="refresh" class="!size-4" /> Restart
        </button>
        <span class="text-xs tabular-nums text-stone-500">Scene {{ index + 1 }} / {{ STEPS.length }}</span>
        <RouterLink to="/join" class="btn-secondary !min-h-10 ml-auto">
          Join a pool <AppIcon name="arrow" class="!size-4" />
        </RouterLink>
      </div>
    </div>

    <p class="border-t border-stone-100 bg-sand/40 px-5 py-3 text-xs leading-relaxed text-stone-600 sm:px-7">
      This is an illustrative example: the people, amounts and seller are fictional, and no transaction reaches the chain. The rules
      reflect those of the pool contract published on Testnet.
    </p>
  </div>
</template>

<style scoped>
.story-chapter-token { display: flex; width: 64px; min-height: 72px; flex-shrink: 0; flex-direction: column; align-items: center; justify-content: center; border: 1px solid #c9d9b9; border-radius: 20px; background: linear-gradient(145deg,#edf3dd,#d9e7c5); color: #577544; box-shadow: 0 4px 0 #c3d3ae; }
.story-chapter-token > span { font: 650 28px var(--font-display); }
.story-chapter-token small { font-size: 9px; }
.story-scoreboard { display: grid; grid-template-columns: 1.4fr 1fr 1fr 1fr; gap: 14px; padding: 20px 28px; border-block: 1px solid #e6ecda; background: #f7f9ee; }
.story-scoreboard > div > span { display: block; font-size: 9px; letter-spacing: .12em; color: #8a967b; }
.story-scoreboard strong { display: flex; align-items: center; gap: 5px; margin-top: 6px; font: 650 23px var(--font-display); color: #386141; font-variant-numeric: tabular-nums; }
.story-scoreboard > div:first-child strong { font-size: 17px; }
.story-scoreboard small { font: 400 11px var(--font-sans); color: #859273; }
.story-scoreboard :deep(svg) { width: 20px; }
.story-board .is-funded { background: #f4f8ec; border-color: #bed2a5; }
.story-board .is-contributing { animation: contribution-highlight 3s ease-out both; }
.pool-ready::after { content: ''; position: absolute; inset: 14%; border: 1px solid #bf9a4380; border-radius: 50%; animation: pool-halo 4s ease-in-out infinite; pointer-events: none; }
.story-pool-scene { position: relative; }
.story-store { transition: background .5s, box-shadow .5s; border: 1px solid transparent; }
.story-store.is-proposed { border-color: #d7dfc1; }
.story-store.is-paid { background: #dfedcc; border-color: #a9c58c; box-shadow: 0 8px 25px -15px #40713666; }
.story-finale { display: flex; gap: 25px; align-items: center; padding: 25px 28px; background: linear-gradient(110deg,#edf3db,#dceacb); border-top: 1px solid #cdddba; }
.finale-art { width: 100px; height: 95px; position: relative; display: grid; place-items: center; flex-shrink: 0; }
.finale-art::after { content: ''; position: absolute; bottom: 4px; width: 94px; height: 24px; border-radius: 50%; background: #b6cc924f; }
.finale-art :deep(img) { z-index: 1; animation: finale-car 1.5s cubic-bezier(.2,.8,.2,1) both; }
.finale-art > span { position: absolute; inset: 5px; border: 1px dashed #a0b87a; border-radius: 50%; }
.finale-copy > span { font-size: 9px; letter-spacing: .13em; color: #7a915e; }
.finale-copy > strong { display: block; margin-top: 6px; font: 650 24px var(--font-display); color: #365c37; }
.finale-copy > p { font-size: 12px; margin-top: 5px; color: #6c8158; }
.finale-order { display: flex; flex-wrap: wrap; gap: 7px; margin-top: 15px; }
.finale-order > span { padding: 5px 9px; border-radius: 8px; font-size: 10px; color: #839274; background: #f8fbed85; }
.finale-order small { margin-left: 4px; font-size: 9px; }
.finale-order .completed { color: #568049; }
.finale-order .next { color: #fffdeb; background: #4f7443; }
.story-finale-enter-active, .story-finale-leave-active { transition: opacity .6s, transform .6s; }
.story-finale-enter-from, .story-finale-leave-to { opacity: 0; transform: translateY(12px); }
.story-paused .pool-ready::after, .story-paused .story-rule-pulse { animation-play-state: paused !important; }
@keyframes contribution-highlight { 0% { box-shadow: 0 0 0 0 #a4bd6860; } 35% { box-shadow: 0 0 0 7px #a4bd6830; } 100% { box-shadow: 0 0 0 0 transparent; } }
@keyframes pool-halo { 0%,100% { transform: scale(.95); opacity: .35; } 50% { transform: scale(1.13); opacity: .85; } }
@keyframes finale-car { from { opacity: 0; transform: translateX(-35px) rotate(-8deg); } to { opacity: 1; transform: none; } }
@media(max-width:767px) { .story-scoreboard { grid-template-columns: 1fr 1fr; padding: 18px 20px; gap: 18px; } .story-finale { align-items: flex-start; gap: 15px; padding: 22px 20px; } .finale-art { width: 70px; height: 85px; } .finale-art::after { width: 70px; } .finale-art :deep(img) { width: 66px !important; height: 66px !important; } .finale-copy > strong { font-size: 21px; } .story-chapter-token { width: 48px; min-height: 60px; border-radius: 15px; } .story-chapter-token > span { font-size: 24px; } }
@media(prefers-reduced-motion:reduce) { .story-board :deep(*) { animation: none !important; transition: none !important; } }
.story-board { border-color: #dde4d1; border-radius: 30px; box-shadow: 0 18px 50px -35px #2d53353b; }
.story-stage { background: radial-gradient(ellipse at 50% 50%,#e7eed970,transparent 65%),#fffef8; }
.story-pool-scene { width: 215px; height: 190px; border-radius: 50%; background: radial-gradient(ellipse,#ecf1dc88,transparent 65%); }
.story-person { box-shadow: 0 4px 0 #eef0e5; transition: transform .5s, border-color .5s, box-shadow .5s; }
.story-person:hover { transform: translateY(-3px); box-shadow: 0 7px 0 #e9eddf; }
@media(max-width:767px) { .story-pool-scene { width: 185px; height: 175px; } }
@media(max-width:370px) { .story-pool-scene { width: 153px; height: 155px; } }
@media(prefers-reduced-motion:reduce) { .story-person { transition: none; } .story-person:hover { transform: none; } }
</style>
