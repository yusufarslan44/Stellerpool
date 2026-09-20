<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from './AppIcon.vue'
import Scene3D from './Scene3D.vue'
import { useNetworkStatus } from '@/composables/useNetworkStatus'
import { config } from '@/lib/stellar'
import type { SceneGoal } from '@/lib/goal-models'
import type { IconName } from '@/types/icons'

const { ledger, online } = useNetworkStatus()
const goal = ref<SceneGoal>('home')
const paused = ref(false)
const options: { id: SceneGoal; title: string; icon: IconName; caption: string }[] = [
  { id: 'home', title: 'Bir ev', icon: 'home', caption: 'Bir anahtar, ortak bir hayal.' },
  { id: 'car', title: 'Bir araç', icon: 'car', caption: 'Birlikte çıkılan yeni bir yol.' },
  { id: 'work', title: 'Bir iş yeri', icon: 'briefcase', caption: 'Yeni bir başlangıç için, birlikte.' },
]
const selected = computed(() => options.find(option => option.id === goal.value)!)
</script>

<template>
  <section class="home-hero" aria-labelledby="hero-title" :class="{ 'motion-paused': paused }">
    <div class="hero-grid">
      <div class="hero-copy">
        <div class="hero-kicker"><span class="hero-kicker-line" /> ORTAK HEDEFLER, AÇIK KURALLAR</div>
        <h1 id="hero-title">Birlikte <br />biriktir.<br /><span>Her adımı gör.</span></h1>
        <p class="hero-description">Bir ev, bir araç ya da yeni bir başlangıç. Tutarını ve taksidini söyle, sana uygun havuza katıl; katkıları, sırayı ve ödemeleri birlikte takip et.</p>
        <div class="hero-actions"><a href="#basla" class="hero-primary">Birlikte başlayalım <span><AppIcon name="arrow" /></span></a><a href="#hikaye" class="hero-secondary"><span class="hero-play-icon" aria-hidden="true">▷</span> Hikâyeyi izle</a></div>
        <div class="hero-proof"><span class="hero-avatars" aria-hidden="true"><i>A</i><i>M</i><i>Z</i><i>C</i></span><p><strong>Senin grubun. Ortak kurallarınız.</strong><span>Testnet’te keşfet · Gerçek para kullanılmaz</span></p></div>
      </div>
      <div class="hero-visual">
        <div class="hero-visual-top"><span><i /> BİRLİKTE NE İÇİN?</span><button type="button" :aria-label="paused ? '3D animasyonu oynat' : '3D animasyonu duraklat'" :aria-pressed="paused" @click="paused = !paused"><svg viewBox="0 0 16 16" aria-hidden="true"><path v-if="paused" d="m5 3 8 5-8 5Z" fill="currentColor"/><path v-else d="M5 3v10M11 3v10" stroke="currentColor" stroke-width="2"/></svg></button></div>
        <div class="hero-goals" role="group" aria-label="3D hedefini seç"><button v-for="option in options" :key="option.id" type="button" :aria-pressed="goal === option.id" @click="goal = option.id"><AppIcon :name="option.icon" />{{ option.title }}</button></div>
        <div class="hero-scene-wrap">
          <div class="hero-orbit hero-orbit-one" aria-hidden="true" /><div class="hero-orbit hero-orbit-two" aria-hidden="true" />
          <span class="hero-scene-word" aria-hidden="true">birlikte.</span>
          <Scene3D :goal="goal" :coins="4" :paused="paused" :label="`${selected.title} hedefinin çevresinde dört üyeyi temsil eden altın paralar`" />
          <div class="hero-float-card hero-member-card"><span class="hero-float-icon"><AppIcon name="users" /></span><span><strong>4 üye</strong><small>Ortak bir hedef</small></span><span class="hero-small-check">✓</span></div>
          <div class="hero-float-card hero-contract-card"><span class="hero-float-icon gold"><AppIcon name="lock" /></span><span><strong>Katkılar sözleşmede</strong><small>Her adım takip edilebilir</small></span></div>
        </div>
        <div class="hero-visual-bottom"><div aria-live="polite"><span>HAYALİNLE BAŞLAR</span><p>{{ selected.caption }}</p></div><span class="hero-visual-count">0{{ options.findIndex(o => o.id === goal) + 1 }}<small> / 03</small></span></div>
        <p class="hero-demo-note">Örnek görselleştirme · Gerçek teslimat hizmeti değildir.</p>
      </div>
    </div>
    <div class="hero-bottom">
      <ol aria-label="Birlikte birikime başlangıç"><li><span>01</span> Grubunu kur</li><li><span>02</span> Kuralları onayla</li><li><span>03</span> Birlikte ilerle</li></ol>
      <div class="hero-live"><i :class="{ offline: online === false }" /><span v-if="online === false">Ağa ulaşılamıyor</span><span v-else-if="ledger">Stellar {{ config.label }} <b>#{{ ledger.toLocaleString('tr-TR') }}</b></span><span v-else>Stellar ağına bağlanılıyor</span></div>
    </div>
  </section>
</template>

<style scoped>
.home-hero { position: relative; isolation: isolate; border-radius: 36px; border: 1px solid #e5e8d7; padding: 43px 38px 0; background: radial-gradient(ellipse at 95% 12%,#e4eccd 0%,transparent 46%),linear-gradient(135deg,#fbf9ef,#f2f2e3); overflow: hidden; }
.hero-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 18px; align-items: center; }
.hero-copy { z-index: 1; padding: 12px 0 30px; animation: hero-arrive 1.1s cubic-bezier(.2,.8,.2,1) both; }
.hero-kicker { display: flex; align-items: center; gap: 10px; color: #738162; font-size: 9px; letter-spacing: .15em; font-weight: 600; }
.hero-kicker-line { width: 22px; height: 1px; background: #809768; }
.hero-copy h1 { color: #253f2a; font: 760 clamp(48px,5.6vw,72px)/1.02 var(--font-display); letter-spacing: -.065em; margin-top: 24px; }
.hero-copy h1 > span { color: #779158; font-size: .87em; letter-spacing: -.055em; }
.hero-description { max-width: 390px; color: #787a66; font-size: 15px; line-height: 1.85; margin-top: 25px; }
.hero-actions { display: flex; align-items: center; flex-wrap: wrap; gap: 20px; margin-top: 29px; }
.hero-primary { display: inline-flex; align-items: center; gap: 19px; padding: 8px 8px 8px 19px; min-height: 49px; border-radius: 15px; background: #245b3c; color: #fff; font-size: 12px; font-weight: 600; box-shadow: 0 4px 0 #173e2b,0 12px 25px -12px #215b3c50; transition: transform .3s,background .3s; }
.hero-primary:hover { transform: translateY(-3px); background: #19472e; }
.hero-primary > span { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 10px; background: #ffffff14; }
.hero-primary :deep(svg) { width: 17px; height: 17px; }
.hero-secondary { display: inline-flex; align-items: center; gap: 8px; font-size: 12px; color: #5d6b4f; min-height: 44px; }
.hero-secondary:hover { color: #1e5837; }
.hero-play-icon { display: grid; place-items: center; width: 29px; height: 29px; border: 1px solid #cbd6bb; border-radius: 50%; font-size: 19px; }
.hero-proof { display: flex; align-items: center; gap: 12px; margin-top: 29px; }
.hero-avatars { display: flex; padding-left: 5px; }
.hero-avatars i { display: grid; place-items: center; width: 27px; height: 27px; margin-left: -5px; border: 2px solid #f7f6e9; border-radius: 50%; background: #dee8d1; color: #4b7751; font-style: normal; font-size: 8px; font-weight: 600; }
.hero-avatars i:nth-child(2) { background: #ecdcac; color: #957136; }
.hero-avatars i:nth-child(3) { background: #b6c9a4; }
.hero-avatars i:nth-child(4) { background: #506e4f; color: #fff6cf; }
.hero-proof p strong, .hero-proof p span { display: block; font-size: 10px; line-height: 1.7; }
.hero-proof p strong { color: #596a4d; font-weight: 500; }
.hero-proof p span { color: #8a8f7b; font-size: 9px; }
.hero-visual { position: relative; align-self: stretch; min-width: 0; display: flex; flex-direction: column; padding-top: 3px; animation: hero-arrive 1.4s .15s both; }
.hero-visual-top { display: flex; justify-content: space-between; align-items: center; padding: 0 0 17px; }
.hero-visual-top > span { display: flex; align-items: center; gap: 7px; font-size: 9px; font-weight: 600; letter-spacing: .12em; color: #7b8a66; }
.hero-visual-top i { width: 4px; height: 4px; background: #7f945c; border-radius: 50%; }
.hero-visual-top button { display: grid; place-items: center; width: 30px; height: 30px; border: 1px solid #cbd6ba; border-radius: 50%; color: #788866; cursor: pointer; }
.hero-visual-top svg { width: 12px; height: 12px; }
.hero-goals { display: flex; gap: 7px; justify-content: center; z-index: 2; }
.hero-goals button { display: flex; align-items: center; justify-content: center; gap: 6px; min-height: 38px; padding: 7px 14px; border-radius: 11px; border: 1px solid #dce3ce; background: #f7f8eb7a; font-size: 11px; color: #818c70; cursor: pointer; transition: background .3s,color .3s,box-shadow .3s; }
.hero-goals button[aria-pressed=true] { background: #fffef5; color: #3e6847; border-color: #b8cba7; box-shadow: 0 4px 12px #4354380c; }
.hero-goals :deep(svg) { width: 16px; height: 16px; }
.hero-scene-wrap { position: relative; height: 370px; margin: -3px -25px -8px; flex: 1; min-height: 320px; }
.hero-scene-word { position: absolute; top: 20px; left: 0; right: 0; text-align: center; font: 700 87px var(--font-display); letter-spacing: -.07em; color: #a8b68d1c; }
.hero-orbit { position: absolute; width: 83%; height: 71%; top: 16%; left: 9%; border: 1px solid #b8c99d4d; border-radius: 50%; transform: rotate(-20deg); pointer-events: none; }
.hero-orbit-two { transform: rotate(20deg); width: 64%; left: 18%; height: 82%; top: 9%; border-style: dashed; border-color: #b8c99d35; }
.hero-float-card { position: absolute; display: flex; align-items: center; gap: 9px; padding: 10px 13px 10px 9px; border: 1px solid #ffffffd9; border-radius: 14px; background: #fffff3dc; backdrop-filter: blur(12px); box-shadow: 0 10px 24px -10px #55674023; animation: hero-float 9s ease-in-out infinite; pointer-events: none; }
.hero-member-card { left: 4%; top: 25%; transform: rotate(-5deg); }
.hero-contract-card { right: 2%; bottom: 14%; animation-delay: -4s; }
.hero-float-icon { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 10px; background: #e1ecd5; color: #608354; }
.hero-float-icon.gold { background: #f6e9bf; color: #a58b47; }
.hero-float-icon :deep(svg) { width: 18px; height: 18px; }
.hero-float-card strong, .hero-float-card small { display: block; }
.hero-float-card strong { color: #4d6744; font-size: 11px; font-weight: 600; }
.hero-float-card small { color: #90947f; font-size: 9px; margin-top: 2px; }
.hero-small-check { align-self: flex-start; color: #628654; font-size: 10px; }
.hero-visual-bottom { display: flex; align-items: center; justify-content: space-between; padding: 0 5px; gap: 10px; }
.hero-visual-bottom div > span { color: #8a9776; font-size: 8px; letter-spacing: .15em; }
.hero-visual-bottom p { color: #5c704d; margin-top: 3px; font: 550 17px var(--font-display); }
.hero-visual-count { color: #597349; font: 550 25px var(--font-display); }
.hero-visual-count small { color: #a2ad90; font-size: 12px; }
.hero-demo-note { color: #929b80; font-size: 8px; margin: 13px 5px 21px; }
.hero-bottom { display: flex; justify-content: space-between; align-items: center; gap: 20px; padding: 20px 0; border-top: 1px solid #dfe5d1; }
.hero-bottom ol { display: flex; flex-wrap: wrap; gap: 24px; }
.hero-bottom li { display: flex; align-items: center; gap: 8px; color: #68775a; font-size: 10px; }
.hero-bottom li > span { color: #a6b192; font: 600 11px var(--font-display); }
.hero-live { display: flex; align-items: center; gap: 6px; color: #889576; font-size: 9px; }
.hero-live i { width: 5px; height: 5px; border-radius: 50%; background: #7a9c5e; box-shadow: 0 0 0 4px #7a9c5e0d; }
.hero-live i.offline { background: #c57662; }
.hero-live b { font-weight: 400; opacity: .75; margin-left: 5px; font-variant-numeric: tabular-nums; }
.motion-paused .hero-float-card { animation-play-state: paused; }
@keyframes hero-arrive { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: none; } }
@keyframes hero-float { 0%,100% { translate: 0 0; } 50% { translate: 0 -9px; } }
@media (max-width:1023px) { .home-hero { padding: 30px 26px 0; } .hero-copy h1 { font-size: 59px; } .hero-grid { gap: 10px; } .hero-actions { gap: 12px; } .hero-description { font-size: 14px; } .hero-float-card { padding: 8px; } .hero-member-card { left: 0; top: 30%; } .hero-contract-card { right: 0; bottom: 13%; } .hero-goals button { padding: 7px 10px; } .hero-live b { display: none; } }
@media (max-width:767px) { .hero-grid { grid-template-columns: 1fr; gap: 30px; } .home-hero { border-radius: 26px; padding: 28px 23px 0; } .hero-copy { padding: 0; } .hero-copy h1 { font-size: clamp(49px,10.5vw,72px); } .hero-copy h1 br:first-child { display: none; } .hero-description { max-width: 460px; } .hero-proof { margin-top: 23px; } .hero-visual { padding-top: 18px; border-top: 1px solid #e0e6d1; } .hero-scene-wrap { min-height: 305px; height: 305px; flex: none; margin-inline: -15px; } .hero-float-card { padding: 8px; } .hero-contract-card { bottom: 12%; } .hero-bottom { flex-direction: column; gap: 16px; align-items: flex-start; padding: 18px 0; } .hero-bottom ol { gap: 16px; } .hero-live b { display: inline; } .hero-visual-top { padding-bottom: 13px; } .hero-demo-note { margin-bottom: 20px; } }
@media (max-width:370px) { .home-hero { padding-inline: 18px; } .hero-goals { gap: 5px; } .hero-goals button { padding-inline: 8px; } .hero-float-card { gap: 6px; } .hero-float-card small { font-size: 8px; } .hero-bottom ol { gap: 10px; } }
@media (prefers-reduced-motion:reduce) { *,*::before,*::after { animation: none !important; transition: none !important; } }
</style>
