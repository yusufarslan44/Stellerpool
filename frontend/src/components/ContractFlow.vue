<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import Illo from './Illo.vue'
const host = ref<HTMLElement | null>(null)
const visible = ref(false)
const paused = ref(false)
let observer: IntersectionObserver | undefined
onMounted(() => {
  observer = new IntersectionObserver(([entry]) => { visible.value = !!entry?.isIntersecting })
  if (host.value) observer.observe(host.value)
})
onBeforeUnmount(() => observer?.disconnect())
</script>
<template>
  <div ref="host" class="contract-flow" :class="{ running: visible && !paused }" aria-label="Money flow: member contributions are collected in the contract and paid to the registered demo seller.">
    <div class="flow-top"><span>THE PATH OF THE MONEY</span><button type="button" :aria-label="paused ? 'Play flow animation' : 'Pause flow animation'" @click="paused = !paused">{{ paused ? '▷' : 'Ⅱ' }}</button></div>
    <div class="flow-track">
      <div class="flow-node"><div class="flow-pedestal"><Illo name="crowd" :size="63" /></div><strong>Members</strong><small>Equal contributions</small></div>
      <div class="flow-line" aria-hidden="true"><i /><i /><span>›</span></div>
      <div class="flow-node central"><div class="flow-pedestal"><Illo name="lock" :size="70" /><span class="flow-orbit" /></div><strong>Contract</strong><small>Rules + approvals</small></div>
      <div class="flow-line second" aria-hidden="true"><i /><i /><span>›</span></div>
      <div class="flow-node"><div class="flow-pedestal"><Illo name="store" :size="64" /></div><strong>Demo seller</strong><small>Direct payment</small></div>
    </div>
    <p>Sample flow · to the seller after the purchase record</p>
  </div>
</template>
<style scoped>
.contract-flow { padding: 18px 22px; border: 1px solid #ffffff1c; border-radius: 25px; background: radial-gradient(ellipse at 50% 50%,#406d4544,transparent 70%),#ffffff04; }
.flow-top { display: flex; align-items: center; justify-content: space-between; color: #b7c59f; font-size: 9px; letter-spacing: .16em; }
.flow-top button { display: grid; place-items: center; width: 27px; height: 27px; border: 1px solid #ffffff26; border-radius: 50%; font-size: 13px; cursor: pointer; color: #d6e1c2; }
.flow-track { display: flex; align-items: flex-start; justify-content: center; margin: 25px 0 17px; }
.flow-node { position: relative; width: 100px; flex-shrink: 0; text-align: center; }
.flow-pedestal { position: relative; isolation: isolate; height: 83px; display: grid; place-items: start center; }
.flow-pedestal::after { content: ''; position: absolute; width: 80px; height: 31px; bottom: 3px; background: linear-gradient(160deg,#657d55,#304a32); border: 1px solid #8f9c6c80; border-radius: 50%; box-shadow: 0 7px 0 #193b29,0 12px 15px #071c2050; z-index: -1; }
.flow-pedestal :deep(img), .flow-pedestal :deep(svg) { animation: flow-hover 8s ease-in-out infinite; animation-play-state: paused; }
.central .flow-pedestal::after { background: linear-gradient(140deg,#d9bd72,#8e803e); }
.flow-node strong { display: block; color: #e9efd5; font: 550 15px var(--font-display); margin-top: 14px; }
.flow-node small { display: block; color: #a4b28e; font-size: 9px; margin-top: 4px; }
.flow-line { position: relative; height: 1px; background: #a6bd6240; flex: 1; min-width: 20px; max-width: 100px; margin: 62px -5px 0; }
.flow-line i { position: absolute; top: -4px; width: 8px; height: 8px; border-radius: 50%; background: #e3c878; box-shadow: 0 0 10px #e3c87866; animation: flow-transfer 10s linear infinite; animation-play-state: paused; }
.flow-line i:nth-child(2) { animation-delay: -2.5s; }
.flow-line.second i { animation-delay: -5s; }
.flow-line.second i:nth-child(2) { animation-delay: -7.5s; }
.flow-line > span { position: absolute; right: 0; top: -13px; font-size: 20px; color: #a9ba7c; }
.flow-orbit { position: absolute; width: 110px; height: 87px; border: 1px dashed #d2cb7c35; border-radius: 50%; top: 0; z-index: -1; }
.contract-flow > p { color: #8fa67e; font-size: 9px; text-align: center; margin-top: 20px; }
.running .flow-line i, .running .flow-pedestal :deep(img), .running .flow-pedestal :deep(svg) { animation-play-state: running; }
@keyframes flow-transfer { 0% { left: 0; opacity: 0; } 10% { opacity: 1; } 80% { opacity: 1; } 100% { left: 100%; opacity: 0; } }
@keyframes flow-hover { 0%,100% { transform: translateY(0); } 50% { transform: translateY(-7px); } }
@media(max-width:480px) { .contract-flow { padding: 15px 10px; } .flow-node { width: 83px; flex-shrink: 1; min-width: 0; } .flow-node strong { font-size: 12px; } .flow-node small { font-size: 8px; } .flow-pedestal::after { width: 70px; } .flow-line { min-width: 8px; } .flow-orbit { width: 87px; } }
@media(prefers-reduced-motion:reduce) { *, :deep(*) { animation: none !important; } .flow-line i { display: none; } }
</style>
