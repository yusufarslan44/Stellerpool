<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppIcon from './AppIcon.vue'
import StepScene3D from './StepScene3D.vue'
import type { IconName } from '@/types/icons'
interface HowStep {
  icon: IconName
  title: string
  short: string
  detail: string
  who: string
}
const steps: HowStep[] = [
  {
    icon: 'users',
    title: 'Join a pool',
    short: 'Tell us the amount and installment; you are routed to a matching pool.',
    detail:
      'You enter the total price, the down payment and the installment you can afford; the number of people and the term are worked out automatically. You join an open pool with the same plan, or a new pool is opened for you. Everyone pays the same amount and everyone receives once.',
    who: 'You',
  },
  {
    icon: 'users',
    title: 'The group fills',
    short: 'The down payment is paid on joining; the order is set when the group is full.',
    detail:
      'Members with the same plan join with their wallets and deposit their down payments into the contract. When the group is full, the join order becomes the delivery order (there is no order in a draw). If the group does not fill within the setup period, the pool is cancelled and the down payments are refunded.',
    who: 'Members',
  },
  {
    icon: 'lock',
    title: 'Rules lock in',
    short: 'The monthly installment and delivery method are approved by everyone.',
    detail:
      'In fixed order the join order is recorded automatically; in a draw the recipient is chosen every round. The pool does not start until members approve the same installment and schedule terms.',
    who: 'Everyone',
  },
  {
    icon: 'wallet',
    title: 'Everyone pays',
    short: 'The money sits in the contract, not with a person.',
    detail:
      'Every round everyone deposits the same contribution into the contract. Once all are in, the recipient is known; in a draw it is drawn by a transaction anyone can call. If someone does not pay, a grace period starts; when it ends, only that round’s contributions and unspent down payments are refunded, and earlier rounds cannot be recovered.',
    who: 'Members',
  },
  {
    icon: 'receipt',
    title: 'The seller is paid',
    short: 'The amount goes to the seller, not to the person whose turn it is.',
    detail:
      'The recipient registers the demo seller recorded in the pool and the purchase-document digest. The amount is the pool plus the recipient’s own down payment and goes straight to that seller; anyone can trigger the payment, and nobody can freely withdraw the funds. If nothing is registered within the purchase period, the round stops and refunds begin.',
    who: 'Recipient',
  },
]

const stageNotes = [
  ['Starting together', 'Equal contributions · shared goal'],
  ['Group complete', 'Down payment paid · order set'],
  ['Everyone approves', 'Same installment · monthly schedule'],
  ['Contributions combine', 'Every round · equal payment'],
  ['Goal reached', 'Purchase record · direct payment'],
]
const section = ref<HTMLElement | null>(null)
const active = ref(0)
// Çubuk her karede güncellenir; reaktif tutulursa tüm bileşen saniyede 60 kez yeniden çizilir. DOM'a doğrudan yazılır.
let progress = 0
const autoplay = ref(true)
const reduced = ref(false)
const visible = ref(false)
const pageVisible = ref(true)
const focused = ref(false)
const current = computed(() => steps[active.value]!)
// Fare üstünde durmaz (kaydırırken imleç bölümün üstünde kalınca anlatım hiç ilerlemiyordu);
// yalnızca klavye odağında, sekme/bölüm görünmezken ve duraklatılınca durur.
const playing = computed(() => autoplay.value && visible.value && pageVisible.value && !reduced.value && !focused.value)
// Her adım 16 sn; elle seçilen adım 2,5 sn daha bekler.
const duration = 16000
const HOLD = -2500 / duration
let raf = 0
let last = 0
let observer: IntersectionObserver | undefined
let motion: MediaQueryList | undefined

function choose(index: number) {
  active.value = (index + steps.length) % steps.length
  // Elle seçim anlatımı kapatmaz; adım biraz daha bekler, sonra otomatik devam eder.
  progress = HOLD
  paintProgress()
}
function togglePlay() {
  autoplay.value = !autoplay.value
  // Açık bir oynat eylemi klavye odağında bile hemen sürdürür.
  focused.value = false
}
function paintProgress() {
  const bar = section.value?.querySelector<HTMLElement>('.how-step-progress')
  if (bar) bar.style.transform = `scaleX(${Math.max(0, Math.min(1, progress))})`
}
function tick(now: number) {
  progress += Math.min(now - last, 100) / duration
  last = now
  if (progress >= 1) {
    progress = 0
    active.value = (active.value + 1) % steps.length
  }
  paintProgress()
  raf = requestAnimationFrame(tick)
}
// Adım değişince yeni çubuk DOM'a girer; ilk boyamayı güncel değerle yap.
watch(active, paintProgress, { flush: 'post' })
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
function updateVisibility() { pageVisible.value = !document.hidden }
function updateMotion() {
  reduced.value = motion?.matches ?? false
  if (reduced.value) autoplay.value = false
}
function leaveFocus(event: FocusEvent) {
  if (!section.value?.contains(event.relatedTarget as Node | null)) focused.value = false
}
onMounted(() => {
  motion = window.matchMedia('(prefers-reduced-motion: reduce)')
  updateMotion()
  updateVisibility()
  motion.addEventListener('change', updateMotion)
  document.addEventListener('visibilitychange', updateVisibility)
  observer = new IntersectionObserver(([entry]) => { visible.value = !!entry?.isIntersecting }, { threshold: 0.1 })
  if (section.value) observer.observe(section.value)
})
onBeforeUnmount(() => {
  cancelAnimationFrame(raf)
  observer?.disconnect()
  motion?.removeEventListener('change', updateMotion)
  document.removeEventListener('visibilitychange', updateVisibility)
})
</script>

<template>
  <section id="nasil" ref="section" class="how-section scroll-mt-28" aria-labelledby="nasil-baslik">
    <header v-reveal class="how-heading">
      <p class="eyebrow text-brand-700"><span class="how-eyebrow-dot" /> Step by step</p>
      <h2 id="nasil-baslik" class="mt-3 text-4xl font-extrabold sm:text-5xl">How it works in 5 steps</h2>
      <p class="mt-4 text-stone-600">From a shared goal to the first payment. See who does what at every step.</p>
    </header>

    <div v-reveal class="how-layout" @focusin="focused = ($event.target as HTMLElement).matches(':focus-visible')" @focusout="leaveFocus">
      <div class="how-steps">
        <div class="how-list-heading"><span>Together, step by step</span><span>01 — 05</span></div>
        <ol class="how-list" aria-label="Steps of how a pool works">
          <li v-for="(step, index) in steps" :key="step.title" :class="{ 'is-active': active === index, 'is-past': active > index }">
            <button type="button" class="how-step" :aria-current="active === index ? 'step' : undefined" aria-controls="how-detail" @click="choose(index)">
              <span class="how-step-number">{{ String(index + 1).padStart(2, '0') }}</span>
              <span class="how-step-content">
                <span class="how-step-role">{{ step.who }}</span>
                <span class="how-step-title">{{ step.title }}</span>
                <span class="how-step-short">{{ step.short }}</span>
              </span>
              <span class="how-step-icon"><AppIcon :name="step.icon" /></span>
              <span v-if="active === index" class="how-step-progress" />
            </button>
          </li>
        </ol>
        <p class="how-list-note"><AppIcon name="eye" /> Tap a step and explore at your own pace.</p>
      </div>

      <article id="how-detail" class="how-feature" aria-label="Details of the selected step">
        <div class="how-stage">
          <div class="how-stage-top"><span><i /> HOW-IT-WORKS GUIDE</span><span class="how-stage-counter">0{{ active + 1 }}<span> / 05</span></span></div>
          <span class="how-stage-watermark" aria-hidden="true">0{{ active + 1 }}</span>
          <StepScene3D :step="active" :reduced="reduced" :paused="!visible || !pageVisible || !autoplay" :preview-duration="duration" />
          <Transition name="how-note" mode="out-in">
            <div :key="active" class="how-scene-note"><span class="how-note-icon"><AppIcon :name="current.icon" /></span><span><strong>{{ stageNotes[active]![0] }}</strong><small>{{ stageNotes[active]![1] }}</small></span></div>
          </Transition>
          <span class="how-stage-caption">Visual explanation of the target design</span>
        </div>

        <div class="how-detail-body">
          <div class="how-detail-meta"><span>ADIM 0{{ active + 1 }}</span><span>{{ current.who }}</span></div>
          <div class="how-copy" :aria-live="autoplay ? 'off' : 'polite'" aria-atomic="true">
            <Transition name="how-copy" mode="out-in">
              <div :key="active"><h3>{{ current.title }}</h3><p>{{ current.detail }}</p></div>
            </Transition>
          </div>
          <div class="how-controls">
            <button type="button" class="how-play" :aria-label="autoplay ? 'Pause auto-play' : 'Start auto-play'" :disabled="reduced" @click="togglePlay">
              <svg viewBox="0 0 20 20" aria-hidden="true"><path v-if="autoplay" d="M6 4v12M14 4v12" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" /><path v-else d="m6 3 11 7-11 7Z" fill="currentColor" /></svg>
              <span>{{ reduced ? 'Move through the steps' : autoplay ? 'Auto-play' : 'Start auto-play' }}</span>
            </button>
            <div class="how-pagination"><button type="button" class="how-nav" aria-label="Previous step" @click="choose(active - 1)"><AppIcon name="back" /></button><span>{{ active + 1 }} / 5</span><button type="button" class="how-nav how-nav-next" aria-label="Next step" @click="choose(active + 1)"><AppIcon name="arrow" /></button></div>
          </div>
        </div>
      </article>
    </div>
    <div v-reveal class="how-footer"><span><AppIcon name="users" /> Equal contributions</span><i /><span><AppIcon name="lock" /> Shared rules</span><i /><span><AppIcon name="store" /> Straight to the seller</span></div>
  </section>
</template>

<style scoped>
.how-section { position: relative; }
.how-heading { max-width: 680px; margin: 0 auto 36px; text-align: center; }
.how-heading .eyebrow { display: flex; justify-content: center; align-items: center; gap: 9px; }
.how-eyebrow-dot { width: 7px; height: 7px; border-radius: 50%; background: #14805a; box-shadow: 0 0 0 5px #14805a0d; }
.how-layout { display: grid; grid-template-columns: 1fr 1.18fr; gap: 32px; align-items: stretch; }
.how-steps { padding: 12px 0; display: flex; flex-direction: column; justify-content: center; min-width: 0; }
.how-list-heading { display: flex; justify-content: space-between; padding: 0 8px 20px; font-size: 11px; font-weight: 600; letter-spacing: .13em; text-transform: uppercase; color: #786c5b; }
.how-list-heading span:last-child { font-variant-numeric: tabular-nums; }
.how-list { display: grid; gap: 12px; position: relative; }
.how-list::before { content: ''; position: absolute; left: 39px; top: 28px; bottom: 28px; width: 1px; background: #d6dece; }
.how-list li { position: relative; }
.how-step { position: relative; display: flex; width: 100%; align-items: center; gap: 16px; min-height: 103px; padding: 17px; border: 1px solid #e7e7da; border-radius: 21px; text-align: left; background: #fffcf4df; cursor: pointer; overflow: hidden; transition: transform .2s, background .2s, border-color .2s, box-shadow .2s; }
.how-step:hover { transform: translateX(4px); background: #fff; border-color: #b4cfbb; }
.is-active .how-step { background: #fff; border-color: #79b99a; box-shadow: 0 10px 30px -14px #0a523a38, inset 0 0 0 1px #79b99a24; transform: translateX(7px); }
.how-step-number { display: grid; place-items: center; width: 43px; height: 47px; flex-shrink: 0; border-radius: 14px; color: #807d67; background: linear-gradient(145deg, #f0f1e7, #e7eadd); border: 1px solid #fff; font: 650 18px var(--font-display); box-shadow: 0 3px 0 #dfe2d4; transition: background .18s, color .18s, box-shadow .18s; }
.is-active .how-step-number { background: linear-gradient(145deg, #24906b, #0b5a3e); color: #fff; box-shadow: 0 4px 0 #083d2e, 0 7px 14px #0a523a20; }
.is-past .how-step-number { color: #0f6748; background: #e4f0e3; }
.how-step-content { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
.how-step-role { color: #768069; font-size: 10px; font-weight: 600; letter-spacing: .07em; text-transform: uppercase; }
.is-active .how-step-role { color: #14805a; }
.how-step-title { color: #353c2e; font: 750 18px/1.2 var(--font-display); }
.how-step-short { font-size: 12px; line-height: 1.5; color: #776f60; }
.how-step-icon { display: grid; place-items: center; margin-left: auto; color: #a4ad96; flex-shrink: 0; }
.is-active .how-step-icon { color: #14805a; }
.how-step-progress { position: absolute; bottom: 0; left: 0; width: 100%; height: 3px; background: #1f9d6b; transform: scaleX(0); transform-origin: left; will-change: transform; }
.how-list-note { display: flex; align-items: center; gap: 7px; padding: 20px 8px 0; font-size: 12px; color: #7c7667; }
.how-list-note :deep(svg) { width: 16px; height: 16px; }
.how-feature { min-width: 0; overflow: hidden; border-radius: 30px; background: #fffefa; border: 1px solid #e3e4d5; box-shadow: 0 18px 60px -35px #3c563d45; }
.how-stage { position: relative; height: 335px; isolation: isolate; overflow: hidden; background: radial-gradient(ellipse at 50% 45%, #f2f4d8 0%, #e4ebd4 45%, #d3dfc5 100%); }
.how-stage::after { content: ''; pointer-events: none; position: absolute; inset: 0; z-index: -1; opacity: .25; background-image: linear-gradient(#92a98735 1px, transparent 1px), linear-gradient(90deg, #92a98735 1px, transparent 1px); background-size: 42px 42px; mask-image: linear-gradient(transparent 20%, black); }
.how-stage-top { position: absolute; top: 22px; left: 24px; right: 24px; display: flex; justify-content: space-between; align-items: center; z-index: 2; pointer-events: none; }
.how-stage-top > span:first-child { display: flex; align-items: center; gap: 7px; font-size: 9px; font-weight: 600; letter-spacing: .15em; color: #4d6854; }
.how-stage-top i { width: 5px; height: 5px; background: #14805a; border-radius: 50%; }
.how-stage-counter { font: 650 16px var(--font-display); color: #35533c; }
.how-stage-counter span { font-size: 11px; opacity: .55; }
.how-stage-watermark { position: absolute; top: 35px; right: 22px; color: #66866410; font: 800 180px/1 var(--font-display); pointer-events: none; }
.how-scene-note { position: absolute; left: 22px; bottom: 29px; display: flex; align-items: center; gap: 10px; padding: 11px 15px 11px 11px; border: 1px solid #ffffffda; border-radius: 16px; background: #fffff4f2; box-shadow: 0 6px 20px #394d3212; pointer-events: none; }
.how-note-icon { display: grid; place-items: center; width: 34px; height: 34px; border-radius: 10px; background: #dfedda; color: #176c48; }
.how-note-icon :deep(svg) { width: 19px; height: 19px; }
.how-scene-note strong, .how-scene-note small { display: block; }
.how-scene-note strong { color: #2c4933; font-size: 12px; font-weight: 600; }
.how-scene-note small { color: #6e7967; font-size: 10px; margin-top: 2px; }
.how-stage-caption { position: absolute; right: 20px; bottom: 9px; font-size: 9px; color: #617455; }
.how-detail-body { padding: 25px 28px 21px; }
.how-detail-meta { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 8px; font-size: 10px; color: #6e7865; }
.how-detail-meta > span:first-child { color: #14805a; font-weight: 700; letter-spacing: .12em; }
.how-detail-meta > span:last-child { padding: 4px 9px; border: 1px solid #e5e9dd; border-radius: 7px; background: #f4f6ed; }
.how-copy { min-height: 200px; padding-top: 13px; }
.how-copy h3 { color: #263e2d; font-size: 27px; font-weight: 750; line-height: 1.15; }
.how-copy p { margin-top: 10px; color: #726d61; font-size: 14px; line-height: 1.75; }
.how-controls { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 17px; border-top: 1px solid #ecece3; }
.how-play { display: flex; align-items: center; gap: 8px; min-height: 40px; padding: 0 4px; color: #64705c; font-size: 11px; cursor: pointer; border-radius: 8px; }
.how-play > svg { width: 13px; height: 13px; }
.how-play:hover { color: #0f6748; }
.how-play:disabled { cursor: default; }
.how-pagination { display: flex; gap: 12px; align-items: center; font-size: 11px; color: #828676; font-variant-numeric: tabular-nums; }
.how-nav { display: grid; place-items: center; width: 40px; height: 40px; border-radius: 50%; border: 1px solid #e1e5d7; color: #466042; cursor: pointer; transition: transform .2s, background .2s; }
.how-nav-next { background: #145b40; color: #fff; border-color: #145b40; box-shadow: 0 4px 10px #145b401c; }
.how-nav:hover { transform: translateY(-2px); background: #e4eddd; }
.how-nav-next:hover { background: #0a4230; }
.how-nav :deep(svg) { width: 17px; height: 17px; }
.how-footer { display: flex; justify-content: center; align-items: center; flex-wrap: wrap; gap: 22px; margin-top: 28px; color: #7d806c; font-size: 11px; }
.how-footer > span { display: flex; align-items: center; gap: 6px; }
.how-footer :deep(svg) { width: 15px; height: 15px; color: #5c805c; }
.how-footer > i { width: 3px; height: 3px; border-radius: 50%; background: #b8bea5; }
.how-copy-enter-active, .how-note-enter-active { transition: opacity .42s var(--ease-out-soft, ease-out), transform .5s var(--ease-out-soft, ease-out); }
.how-copy-leave-active, .how-note-leave-active { transition: opacity .18s ease-in, transform .2s ease-in; }
.how-copy-enter-from, .how-note-enter-from { opacity: 0; transform: translateY(12px); }
.how-copy-leave-to, .how-note-leave-to { opacity: 0; transform: translateY(-6px); }
@media (max-width: 1023px) {
  .how-layout { gap: 20px; grid-template-columns: 1fr 1.1fr; }
  .how-step { gap: 10px; padding: 13px; }
  .how-step-title { font-size: 16px; }
  .how-step-icon { display: none; }
  .how-detail-body { padding: 22px; }
  .how-copy { min-height: 235px; }
}
@media (max-width: 767px) {
  .how-layout { grid-template-columns: 1fr; gap: 22px; }
  .how-heading { margin-bottom: 23px; }
  .how-steps { padding: 0; }
  .how-list-heading { padding-bottom: 12px; }
  .how-list { gap: 8px; }
  .how-step { min-height: 78px; gap: 13px; padding: 12px; border-radius: 17px; }
  .how-step-role { display: none; }
  .how-step-number { width: 38px; height: 40px; font-size: 16px; }
  .how-step-icon { display: grid; }
  .is-active .how-step { transform: none; }
  .how-list-note { padding-top: 12px; }
  .how-stage { height: 310px; }
  .how-feature { border-radius: 24px; }
  .how-copy { min-height: 250px; }
  .how-copy h3 { font-size: 25px; }
  .how-footer { gap: 10px; font-size: 10px; }
}
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { transition: none !important; }
  .how-step:hover, .is-active .how-step, .how-nav:hover { transform: none; }
}
</style>
