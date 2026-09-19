<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import Illo from '@/components/Illo.vue'
import { sha256, toHex } from '@/lib/hash'
import { illoUrl } from '@/lib/illo'
import type { IlloName } from '@/lib/icon-data'

/**
 * Anlatıcı, dokunarak oynanan örnek hikâye: dört arkadaş, bir tur.
 * Tamamen kurgusaldır (kişiler ve tutarlar örnek); zincire işlem gitmez. Kurallar hedef tasarımdır.
 * Tek gerçek hesap: alım belgesinin SHA-256 özeti tarayıcıda gerçekten hesaplanır.
 */
type Phase = 'intro' | 'collect' | 'grace' | 'blocked' | 'aborted' | 'purchase' | 'verify' | 'pay' | 'done'

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
const VERIFIERS = ['A', 'B', 'C']
const RECIPIENT = 'ayse'
const AMOUNT = 10
const POT = AMOUNT * PEOPLE.length
const NEEDED = 2
const DOC = 'Örnek alım belgesi: araç, 40 birim, Örnek Galeri'

const reduced =
  typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

const phase = ref<Phase>('intro')
const resumePhase = ref<Phase>('collect')
const rulesAccepted = ref(false)
const paid = ref<string[]>([])
const docHash = ref<string | null>(null)
const approvals = ref<string[]>([])
const sent = ref(false)
const locked = ref(false)

const stage = ref<HTMLElement | null>(null)
const jarEl = ref<HTMLElement | null>(null)
const storeEl = ref<HTMLElement | null>(null)
const personEls: Record<string, HTMLElement | null> = {}

const timers: number[] = []
function later(fn: () => void, ms: number) {
  if (reduced) return fn()
  timers.push(window.setTimeout(fn, ms))
}
onBeforeUnmount(() => timers.forEach((t) => clearTimeout(t)))

const isFunded = (id: string) => paid.value.includes(id)
const funded = computed(() => PEOPLE.filter((p) => isFunded(p.id)).length)
const fill = computed(() => (sent.value ? 0 : funded.value / PEOPLE.length))
const recipientPaid = computed(() => paid.value.includes(RECIPIENT))
const canTap = computed(() => (phase.value === 'collect' || phase.value === 'grace') && !locked.value)

const CHAPTERS = ['Kurallar', 'Katkı', 'Alım', 'Onay', 'Gönderim']
const chapter = computed(() => {
  switch (phase.value) {
    case 'intro': return 0
    case 'collect': case 'grace': case 'blocked': case 'aborted': return 1
    case 'purchase': return 2
    case 'verify': return 3
    default: return 4
  }
})

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
    { duration: 800, delay, easing: 'cubic-bezier(0.3, 0.7, 0.3, 1)', fill: 'both' },
  )
  anim.onfinish = () => coin.remove()
}

// --- Eylemler ------------------------------------------------------------------------------
function acceptRules() {
  rulesAccepted.value = true
  phase.value = 'collect'
}

function pay(id: string) {
  if (!canTap.value || isFunded(id)) return
  flyCoin(personEls[id] ?? null, jarEl.value)
  paid.value = [...paid.value, id]
  afterFunding()
}

function afterFunding() {
  if (funded.value < PEOPLE.length) return
  locked.value = true
  later(() => {
    phase.value = 'purchase'
    locked.value = false
  }, 1100)
}

function delayCan() {
  if (phase.value === 'collect' && !isFunded('can')) phase.value = 'grace'
}
function askAyse() {
  if (recipientPaid.value) return
  resumePhase.value = phase.value === 'grace' ? 'grace' : 'collect'
  phase.value = 'blocked'
}
function abortRound() {
  if (phase.value !== 'grace' || locked.value) return
  paid.value = []
  phase.value = 'aborted'
}

async function propose() {
  if (locked.value) return
  locked.value = true
  docHash.value = toHex(await sha256(DOC))
  locked.value = false
  phase.value = 'verify'
}

function approve(v: string) {
  if ((phase.value !== 'verify' && phase.value !== 'pay') || approvals.value.includes(v)) return
  approvals.value = [...approvals.value, v]
  if (phase.value === 'verify' && approvals.value.length >= NEEDED) {
    locked.value = true
    later(() => {
      phase.value = 'pay'
      locked.value = false
    }, 700)
  }
}

function send() {
  if (phase.value !== 'pay' || locked.value) return
  locked.value = true
  for (let i = 0; i < PEOPLE.length; i++) flyCoin(jarEl.value, storeEl.value, i * 140)
  later(() => {
    sent.value = true
    phase.value = 'done'
    locked.value = false
  }, 1100)
}

function reset() {
  timers.forEach((t) => clearTimeout(t))
  timers.length = 0
  phase.value = 'intro'
  rulesAccepted.value = false
  paid.value = []
  docHash.value = null
  approvals.value = []
  sent.value = false
  locked.value = false
}

// --- Anlatım -------------------------------------------------------------------------------
const story = computed<{ title: string; text: string }>(() => {
  switch (phase.value) {
    case 'intro':
      return {
        title: 'Dört arkadaş, bir hedef',
        text: `Ayşe, Mehmet, Zeynep ve Can birlikte araç biriktiriyor. Her tur herkes ${AMOUNT} birim yatırır; tamamı gelince sıradaki kişinin ${POT} birimlik alımı yapılır. Erken teslim alan biri sonraki ödemeyi bırakırsa diğerlerinin geçmiş katkıları otomatik geri gelmez.`,
      }
    case 'collect':
      return {
        title: 'Herkes payını yatırır',
        text: `Kurallar kabul edildi. Şimdi herkes kendi ${AMOUNT} birimini yatırır; para bir kişinin cüzdanına değil, sözleşmeye gider. Örnek için arkadaşlara dokun.`,
      }
    case 'grace':
      return {
        title: 'Can yatırmadı: ek süre başladı',
        text: 'Can hâlâ kendi payını yatırabilir. Ek süre biterse tur durur ve bu turda yatırılan katkılar geri verilir. Önceki tamamlanmış turların ödemeleri iade havuzunda değildir.',
      }
    case 'blocked':
      return {
        title: 'Sıradaki kişi ödemezse?',
        text: 'Ayşe kendi payını yatırmazsa tur ilerlemez. Ek süre bitince bu tur durur; yalnızca bu turda yatırılmış katkılar geri alınabilir.',
      }
    case 'aborted':
      return {
        title: 'Tur durdu',
        text: 'Bu örnekte satıcıya ödeme yapılmadı; bu turda yatırılan katkılar sahiplerine geri döner. Daha önce tamamlanan bir tur olsaydı, o turdaki para satıcıya gitmiş olurdu.',
      }
    case 'purchase':
      return {
        title: 'Alım önerisi',
        text: `Katkılar tamam: ${POT} birim sözleşmede. Şimdi Ayşe satıcıyı (Örnek Galeri) ve alım belgesini önerir. Belgenin kendisi değil, yalnızca parmak izi (SHA-256 özeti) kaydedilir.`,
      }
    case 'verify':
      return {
        title: 'Doğrulayıcılar belgeye bakar',
        text: `Üç doğrulayıcı belgeyi zincir dışında kontrol eder. Tutarın çıkması için en az ${NEEDED} onay gerekir. Onaylamaları için dokun (${approvals.value.length} / ${NEEDED}).`,
      }
    case 'pay':
      return {
        title: 'Onaylar tamam',
        text: 'Artık kim isterse “gönder” diyebilir; bir yöneticiye gerek yok. Para Ayşe’ye değil, doğrudan satıcıya gider.',
      }
    default:
      return {
        title: 'Tur 1 / 4 tamamlandı',
        text: `${POT} birim satıcıya gitti, Ayşe’nin aracı yolda. Kimse parayı elle çekemedi. Sıradaki tur Mehmet’in; aynı adımlar yeniden başlar.`,
      }
  }
})

const shortHash = computed(() => (docHash.value ? `${docHash.value.slice(0, 10)}…${docHash.value.slice(-4)}` : null))

function statusOf(id: string): { label: string; cls: string } {
  if (paid.value.includes(id)) return { label: 'Ödedi ✓', cls: 'bg-sage-100 text-sage-800' }
  if (phase.value === 'grace' && id === 'can') return { label: 'Geciktirdi', cls: 'bg-rose-100 text-rose-800' }
  return { label: 'Bekliyor', cls: 'bg-stone-100 text-stone-700' }
}
</script>

<template>
  <div class="card overflow-hidden !p-0">
    <!-- Başlık: bölüm çubuğu + dürüstlük etiketi -->
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-stone-100 px-5 py-4 sm:px-7">
      <ol class="flex flex-wrap items-center gap-1.5 text-xs font-semibold" aria-label="Hikâye bölümleri">
        <li v-for="(c, i) in CHAPTERS" :key="c" class="flex items-center gap-1.5">
          <span
            class="rounded-full px-2.5 py-1 transition-colors duration-300"
            :class="i === chapter ? 'bg-brand-600 text-white' : i < chapter ? 'bg-sage-100 text-sage-800' : 'bg-stone-100 text-stone-600'"
            :aria-current="i === chapter ? 'step' : undefined"
          >
            {{ i < chapter ? '✓ ' : '' }}{{ c }}
          </span>
          <span v-if="i < CHAPTERS.length - 1" class="text-stone-300" aria-hidden="true">›</span>
        </li>
      </ol>
      <span class="badge bg-gold-100 text-amber-900">Örnek hikâye · gerçek işlem değil</span>
    </div>

    <!-- Anlatıcı -->
    <div class="flex items-start gap-3 bg-sand/50 px-5 py-4 sm:gap-4 sm:px-7">
      <Illo name="fox" :size="64" class="float" />
      <div :key="phase" class="pop relative min-w-0 flex-1 rounded-3xl rounded-tl-lg bg-white p-4 shadow-[0_10px_26px_-16px_rgb(120_53_15/0.4)]" role="status" aria-live="polite">
        <p class="font-display text-lg font-extrabold">{{ story.title }}</p>
        <p class="mt-1 text-sm leading-relaxed text-stone-700">{{ story.text }}</p>
      </div>
    </div>

    <!-- Sahne -->
    <div ref="stage" class="relative grid gap-5 px-5 py-6 sm:px-7 md:grid-cols-[1.25fr_1fr_1fr]">
      <!-- Arkadaşlar -->
      <div class="order-2 md:order-1">
        <p class="eyebrow mb-2 text-stone-600">Arkadaşlar</p>
        <div class="grid grid-cols-2 gap-2.5">
          <button
            v-for="p in PEOPLE"
            :key="p.id"
            :ref="(el) => (personEls[p.id] = el as HTMLElement | null)"
            type="button"
            class="relative flex min-h-[7.5rem] flex-col items-center justify-center gap-1 rounded-2xl border-2 bg-white p-2.5 text-center transition-[border-color,box-shadow,transform] duration-200 disabled:cursor-default"
            :class="[
              canTap && !isFunded(p.id) ? 'cursor-pointer border-brand-300 hover:-translate-y-0.5 hover:shadow-[0_12px_24px_-14px_rgb(20_128_90/0.6)]' : 'border-stone-200',
              p.id === RECIPIENT && !sent ? 'ring-2 ring-gold-300 ring-offset-2' : '',
            ]"
            :disabled="!canTap || isFunded(p.id)"
            :aria-label="`${p.name}: ${statusOf(p.id).label}${canTap && !isFunded(p.id) ? '. Dokun ve öde' : ''}`"
            @click="pay(p.id)"
          >
            <span
              v-if="canTap && !isFunded(p.id)"
              class="absolute top-2 right-2 size-2.5 rounded-full bg-brand-500"
              style="animation: ring-ping 1.8s ease-out infinite"
              aria-hidden="true"
            />
            <Illo :name="p.illo" :size="48" />
            <span class="font-display text-sm font-bold">{{ p.name }}</span>
            <span class="badge !px-2 !py-0.5 text-[11px]" :class="statusOf(p.id).cls">{{ statusOf(p.id).label }}</span>
            <span v-if="p.id === RECIPIENT" class="absolute -top-2.5 left-2 flex items-center gap-1 rounded-full bg-gold-400 px-2 py-0.5 text-[10px] font-bold text-ink">
              {{ sent ? 'Aracı yolda' : 'Bu tur sırası' }}
              <Illo v-if="sent" name="car" :size="16" class="pop" />
            </span>
          </button>
        </div>
      </div>

      <!-- Kurallar + kavanoz -->
      <div class="order-1 flex flex-row items-center justify-around gap-3 md:order-2 md:flex-col md:justify-between">
        <div class="flex flex-col items-center">
          <button
            v-if="phase === 'intro'"
            type="button"
            class="group relative grid size-20 cursor-pointer place-items-center rounded-3xl bg-gold-100 transition-transform duration-200 hover:scale-105 active:scale-95"
            aria-label="Grup kurallarını kabul et"
            @click="acceptRules"
          >
            <span class="absolute inset-0 rounded-3xl border-2 border-brand-500" style="animation: ring-ping 1.8s ease-out infinite" aria-hidden="true" />
            <Illo name="memo" :size="52" />
          </button>
          <span v-else class="grid size-20 place-items-center rounded-3xl bg-gold-100">
            <Illo name="memo" :size="52" :class="rulesAccepted ? 'pop' : ''" />
          </span>
          <span class="mt-1 text-xs font-bold">Grup kuralları</span>
          <span class="badge mt-0.5 text-[11px]" :class="rulesAccepted ? 'bg-sage-100 text-sage-800' : 'bg-stone-100 text-stone-600'">
            {{ rulesAccepted ? 'Kabul edildi' : 'Onay bekliyor' }}
          </span>
        </div>

        <div class="flex flex-col items-center gap-2">
        <div :ref="(el) => (jarEl = el as HTMLElement | null)" class="relative">
          <svg viewBox="0 0 120 140" class="w-28 md:w-32" role="img" :aria-label="`Havuz: ${sent ? 0 : funded * AMOUNT} / ${POT} birim`">
            <defs>
              <clipPath id="story-jar"><path d="M22 34h76l-6 88a10 10 0 0 1-10 9H38a10 10 0 0 1-10-9Z" /></clipPath>
            </defs>
            <path d="M22 34h76l-6 88a10 10 0 0 1-10 9H38a10 10 0 0 1-10-9Z" fill="#fffaf2" />
            <g clip-path="url(#story-jar)">
              <rect class="jar-fill" x="0" y="34" width="120" height="100" fill="#f2b134" :style="{ transform: `scaleY(${fill})` }" />
            </g>
            <path d="M22 34h76l-6 88a10 10 0 0 1-10 9H38a10 10 0 0 1-10-9Z" fill="none" stroke="#2b1a12" stroke-width="3.5" stroke-linejoin="round" />
            <path d="M14 34h92" stroke="#2b1a12" stroke-width="5" stroke-linecap="round" />
          </svg>
          <Illo v-if="funded === PEOPLE.length && !sent" name="lock" :size="34" class="pop absolute top-14 left-1/2 -translate-x-1/2" />
        </div>
        <p class="text-center text-sm font-bold tabular-nums">
          {{ sent ? 0 : funded * AMOUNT }} / {{ POT }} birim
          <span class="block text-xs font-normal text-stone-600">{{ sent ? `${POT} birim satıcıya gitti` : 'sözleşmede' }}</span>
        </p>
        </div>
      </div>

      <!-- Doğrulayıcılar + satıcı -->
      <div class="order-3 flex flex-col gap-4">
        <div>
          <p class="eyebrow mb-2 text-stone-600">Doğrulayıcılar</p>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="v in VERIFIERS"
              :key="v"
              type="button"
              class="flex flex-col items-center gap-1 rounded-2xl border-2 bg-white p-2 text-center transition-[border-color,transform] duration-200 disabled:cursor-default"
              :class="
                approvals.includes(v)
                  ? 'border-sage-500 bg-sage-50'
                  : (phase === 'verify' || phase === 'pay') && !locked
                    ? 'cursor-pointer border-brand-300 hover:-translate-y-0.5'
                    : 'border-stone-200 opacity-70'
              "
              :disabled="approvals.includes(v) || !(phase === 'verify' || phase === 'pay')"
              :aria-label="`Doğrulayıcı ${v}: ${approvals.includes(v) ? 'onayladı' : 'bekliyor'}`"
              @click="approve(v)"
            >
              <Illo name="magnifier" :size="34" />
              <span class="text-[11px] font-bold">{{ v }}</span>
              <span class="text-[10px]" :class="approvals.includes(v) ? 'font-bold text-sage-800' : 'text-stone-500'">
                {{ approvals.includes(v) ? 'Onayladı ✓' : 'Bekliyor' }}
              </span>
            </button>
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

    <!-- Eylem çubuğu -->
    <div class="flex flex-wrap items-center gap-3 border-t border-stone-100 bg-white px-5 py-4 sm:px-7">
      <template v-if="phase === 'intro'">
        <button type="button" class="btn-primary" @click="acceptRules">
          <Illo name="memo" :size="20" /> Kuralları kabul et
        </button>
      </template>

      <template v-else-if="phase === 'collect'">
        <span class="text-sm text-stone-600">Ne olurdu?</span>
        <button v-if="!isFunded('can')" type="button" class="btn-secondary !min-h-10" :disabled="locked" @click="delayCan">Can unutursa?</button>
        <button v-if="!recipientPaid" type="button" class="btn-secondary !min-h-10" :disabled="locked" @click="askAyse">Ayşe unutursa?</button>
      </template>

      <template v-else-if="phase === 'grace'">
        <span class="text-sm text-stone-600">Geç ödemesi için Can’a dokun</span>
        <button type="button" class="btn-secondary !min-h-10" :disabled="locked" @click="abortRound">Ek süre bitti: turu durdur</button>
        <button v-if="!recipientPaid" type="button" class="btn-secondary !min-h-10" :disabled="locked" @click="askAyse">Ayşe unutursa?</button>
      </template>

      <template v-else-if="phase === 'blocked'">
        <button type="button" class="btn-primary" @click="phase = resumePhase">
          <AppIcon name="back" class="!size-4" /> Geri dön
        </button>
      </template>

      <template v-else-if="phase === 'purchase'">
        <button type="button" class="btn-primary" :disabled="locked" @click="propose">
          <Illo name="receipt" :size="20" /> Alım belgesini gönder
        </button>
      </template>

      <template v-else-if="phase === 'verify'">
        <span class="text-sm font-semibold text-stone-700">Doğrulayıcılara dokun: {{ approvals.length }} / {{ NEEDED }} onay</span>
      </template>

      <template v-else-if="phase === 'pay'">
        <button type="button" class="btn-primary btn-lg" :disabled="locked" @click="send">
          <Illo name="store" :size="24" /> Tutarı satıcıya gönder
        </button>
      </template>

      <template v-else-if="phase === 'done' || phase === 'aborted'">
        <Illo name="party" :size="40" class="pop" />
        <button type="button" class="btn-secondary" @click="reset"><AppIcon name="refresh" class="!size-4" /> Baştan oyna</button>
        <RouterLink to="/create" class="btn-primary">Kendi havuzunu kur <AppIcon name="arrow" class="!size-4" /></RouterLink>
      </template>

      <button v-if="phase !== 'intro' && phase !== 'done'" type="button" class="ml-auto text-xs font-medium text-stone-500 underline hover:text-stone-700" @click="reset">
        Baştan başla
      </button>
    </div>

    <p class="border-t border-stone-100 bg-sand/40 px-5 py-3 text-xs leading-relaxed text-stone-600 sm:px-7">
      Bu bir anlatım örneğidir: kişiler, tutarlar ve satıcı kurgusaldır, zincire hiçbir işlem gitmez. Kurallar
      hedef tasarımdır; havuz kontratı henüz yayınlanmadı.
    </p>
  </div>
</template>

<style scoped>
.jar-fill {
  transform-box: fill-box;
  transform-origin: bottom;
  transition: transform 0.9s var(--ease-out-soft);
}
</style>
