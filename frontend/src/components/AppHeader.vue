<script setup lang="ts">
import { defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { config, mainnetShowcase } from '@/lib/stellar'
import AppIcon from './AppIcon.vue'

const WalletButton = defineAsyncComponent(() => import('./WalletButton.vue'))
const route = useRoute()
const items = [
  { id: 'nasil', label: 'Nasıl çalışır', hint: 'Beş adımda keşfet' },
  { id: 'hikaye', label: 'Hikâyeyi izle', hint: 'Bir turun içinden' },
  { id: 'basla', label: 'Başla', hint: 'Hesabını hazırla' },
  { id: 'hesapla', label: 'Hesapla', hint: 'Planını oluştur' },
  { id: 'sss', label: 'Sorular', hint: 'Aklındakiler' },
]
const header = ref<HTMLElement | null>(null)
const toggle = ref<HTMLButtonElement | null>(null)
const menuOpen = ref(false)
const scrolled = ref(false)
const active = ref(-1)
const progress = ref(0)
let raf = 0
let desktop: MediaQueryList | undefined
function measure() {
  raf = 0
  scrolled.value = window.scrollY > 24
  const total = document.documentElement.scrollHeight - window.innerHeight
  progress.value = total > 0 ? Math.min(1, window.scrollY / total) : 0
  active.value = -1
  if (route.path !== '/') return
  items.forEach((item, index) => {
    const section = document.getElementById(item.id)
    if (section && section.getBoundingClientRect().top <= 180) active.value = index
  })
}
function onScroll() { if (!raf) raf = requestAnimationFrame(measure) }
function close(restoreFocus = false) {
  menuOpen.value = false
  if (restoreFocus) toggle.value?.focus()
}
function outside(event: PointerEvent) {
  if (menuOpen.value && !header.value?.contains(event.target as Node)) close()
}
function resizeMenu() { if (desktop?.matches) close() }
watch(() => route.fullPath, async () => { close(); await nextTick(); onScroll() })
onMounted(() => {
  measure()
  desktop = window.matchMedia('(min-width: 1024px)')
  desktop.addEventListener('change', resizeMenu)
  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('resize', onScroll, { passive: true })
  document.addEventListener('pointerdown', outside)
})
onBeforeUnmount(() => {
  cancelAnimationFrame(raf)
  desktop?.removeEventListener('change', resizeMenu)
  window.removeEventListener('scroll', onScroll)
  window.removeEventListener('resize', onScroll)
  document.removeEventListener('pointerdown', outside)
})
</script>

<template>
  <a href="#main-content" class="skip-link">İçeriğe geç</a>
  <header ref="header" class="site-header" :class="{ 'is-scrolled': scrolled, 'menu-open': menuOpen }" @keydown.esc="close(true)">
    <div class="nav-shell">
      <RouterLink to="/" class="brand" aria-label="Stellerpool ana sayfa" @click="close()">
        <span class="brand-symbol" aria-hidden="true"><svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="15.5" fill="none" stroke="currentColor" stroke-width="1.1" opacity=".65"/><path d="m20 9 3.3 7 7.7 1.1-5.5 5.4 1.3 7.6L20 26.5l-6.8 3.6 1.3-7.6L9 17.1l7.7-1.1Z" fill="currentColor" /></svg></span>
        <span>Stellerpool<span class="brand-dot">.</span></span>
      </RouterLink>
      <nav v-if="!mainnetShowcase" class="desktop-nav" aria-label="Ana menü" :style="{ '--active': active }">
        <span class="nav-indicator" :class="{ visible: active >= 0 }" aria-hidden="true" />
        <RouterLink v-for="(item, index) in items" :key="item.id" :to="`/#${item.id}`" :class="{ current: active === index }" :aria-current="active === index ? 'location' : undefined">{{ item.label }}</RouterLink>
      </nav>
      <div class="nav-actions">
        <span class="nav-network" :title="mainnetShowcase ? 'Salt okunur Mainnet tanıtımı' : 'Gerçek para kullanılmayan test ağı'"><i />{{ config.label }}</span>
        <div v-if="!mainnetShowcase" class="nav-wallet"><WalletButton /></div>
        <button v-if="!mainnetShowcase" ref="toggle" type="button" class="menu-toggle" :aria-expanded="menuOpen" aria-controls="mobile-navigation" :aria-label="menuOpen ? 'Menüyü kapat' : 'Menüyü aç'" @click="menuOpen = !menuOpen"><span /><span /></button>
      </div>
    </div>
    <div class="nav-progress" aria-hidden="true"><span :style="{ transform: `scaleX(${progress})` }" /></div>
    <Transition name="mobile-nav">
      <nav v-if="menuOpen && !mainnetShowcase" id="mobile-navigation" class="mobile-navigation" aria-label="Mobil menü">
        <div class="mobile-nav-heading">BİRLİKTE KEŞFEDELİM <span>01 — 05</span></div>
        <RouterLink v-for="(item, index) in items" :key="item.id" :to="`/#${item.id}`" :aria-current="active === index ? 'location' : undefined" @click="close()"><span class="mobile-nav-number">0{{ index + 1 }}</span><span><strong>{{ item.label }}</strong><small>{{ item.hint }}</small></span><AppIcon name="arrow" /></RouterLink>
        <div class="mobile-nav-bottom"><WalletButton /><RouterLink to="/create" class="mobile-create" @click="close()">Havuz oluştur <AppIcon name="arrow" /></RouterLink></div>
        <p>Stellar {{ config.label }} · Gerçek para kullanılmaz.</p>
      </nav>
    </Transition>
  </header>
</template>

<style scoped>
.site-header { position: sticky; top: 16px; z-index: 40; width: calc(100% - 40px); max-width: 1200px; margin: 16px auto 0; border: 1px solid #ffffffd9; border-radius: 23px; background: #fffdf7e8; box-shadow: 0 6px 30px -17px #3e4c2c35; backdrop-filter: blur(24px); transition: background .4s, box-shadow .4s, border-color .4s; }
.site-header.is-scrolled { background: #fffffff2; border-color: #dfe7d5; box-shadow: 0 12px 36px -18px #263c2b40; }
.nav-shell { display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 12px 15px; }
.brand { display: flex; align-items: center; gap: 10px; flex-shrink: 0; color: #203e2f; font: 750 23px var(--font-display); letter-spacing: -.8px; }
.brand-dot { color: #bd9444; }
.brand-symbol { width: 40px; height: 40px; color: #f6e2a7; border: 1px solid #41805c; border-radius: 14px; background: linear-gradient(145deg,#397955,#17462f); box-shadow: 0 3px 0 #103b29, inset 0 1px 1px #ffffff60; transition: transform .6s; }
.brand:hover .brand-symbol { transform: rotate(-9deg); }
.desktop-nav { position: relative; display: grid; grid-template-columns: repeat(5, 1fr); padding: 4px; border: 1px solid #eceee4; border-radius: 13px; background: #f4f5edb0; }
.desktop-nav > a { z-index: 1; padding: 9px 12px; white-space: nowrap; font-size: 12px; color: #707564; transition: color .3s; }
.desktop-nav > a:hover, .desktop-nav > a.current { color: #155739; }
.nav-indicator { position: absolute; top: 4px; bottom: 4px; left: calc(var(--active) * (100% - 8px) / 5 + 4px); width: calc((100% - 8px) / 5); border-radius: 9px; background: #fff; box-shadow: 0 2px 5px #1d422b0b; opacity: 0; transition: left .45s cubic-bezier(.2,.8,.2,1), opacity .3s; }
.nav-indicator.visible { opacity: 1; }
.nav-actions { display: flex; align-items: center; gap: 13px; }
.nav-network { display: flex; align-items: center; gap: 6px; font-size: 10px; color: #6c775f; }
.nav-network i { width: 5px; height: 5px; border-radius: 50%; background: #809b63; }
.nav-wallet :deep(button) { border-radius: 12px; min-height: 40px; font-size: 12px; }
.nav-progress { position: absolute; bottom: -1px; left: 22px; right: 22px; height: 2px; overflow: hidden; pointer-events: none; }
.nav-progress span { display: block; width: 100%; height: 100%; transform-origin: left; background: linear-gradient(90deg,#256a46,#d6b364); }
.menu-toggle { display: none; width: 42px; height: 42px; border: 1px solid #dbe3d3; border-radius: 13px; cursor: pointer; position: relative; background: #f2f5e9; }
.menu-toggle span { position: absolute; left: 12px; right: 12px; height: 1.5px; background: #28543a; transition: transform .3s, top .3s; }
.menu-toggle span:first-child { top: 16px; }
.menu-toggle span:last-child { top: 23px; }
.menu-open .menu-toggle span:first-child { top: 20px; transform: rotate(45deg); }
.menu-open .menu-toggle span:last-child { top: 20px; transform: rotate(-45deg); }
.mobile-navigation { max-height: calc(100dvh - 110px); overflow-y: auto; padding: 10px 18px 20px; border-top: 1px solid #e9eddf; }
.mobile-nav-heading { display: flex; justify-content: space-between; color: #869079; font-size: 9px; letter-spacing: .12em; margin: 12px 0; }
.mobile-navigation > a { display: flex; align-items: center; gap: 14px; padding: 13px 7px; border-bottom: 1px solid #ecefe3; border-radius: 8px; }
.mobile-navigation > a[aria-current] { background: #edf4e8; }
.mobile-nav-number { font: 600 13px var(--font-display); color: #869b75; }
.mobile-navigation strong, .mobile-navigation small { display: block; }
.mobile-navigation strong { color: #244531; font: 650 17px var(--font-display); }
.mobile-navigation small { color: #828673; font-size: 11px; margin-top: 2px; }
.mobile-navigation > a > :deep(svg) { margin-left: auto; width: 17px; color: #769265; }
.mobile-nav-bottom { margin-top: 20px; display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap; }
.mobile-create { display: flex; align-items: center; gap: 5px; color: #326644; font-size: 12px; }
.mobile-create :deep(svg) { width: 15px; }
.mobile-navigation > p { font-size: 10px; color: #898d7b; margin-top: 18px; }
.mobile-nav-enter-active, .mobile-nav-leave-active { transition: opacity .25s, transform .3s; transform-origin: top; }
.mobile-nav-enter-from, .mobile-nav-leave-to { opacity: 0; transform: translateY(-10px); }
.skip-link { position: fixed; z-index: 60; top: 8px; left: 20px; padding: 12px; border-radius: 10px; background: #fff; transform: translateY(-150%); }
.skip-link:focus { transform: none; }
@media (max-width: 1150px) { .nav-network { display: none; } .desktop-nav > a { padding-inline: 10px; } }
@media (max-width: 1023px) { .desktop-nav { display: none; } .menu-toggle { display: block; } .nav-network { display: flex; } }
@media (max-width: 639px) { .site-header { top: 10px; width: calc(100% - 24px); margin-top: 10px; border-radius: 19px; } .nav-shell { padding: 9px 10px; gap: 10px; } .brand { font-size: 21px; gap: 8px; } .brand-symbol { width: 35px; height: 35px; border-radius: 11px; } .nav-wallet { display: none; } .nav-actions { gap: 10px; } }
@media (prefers-reduced-motion: reduce) { *, *::before, *::after { transition: none !important; } }
</style>
