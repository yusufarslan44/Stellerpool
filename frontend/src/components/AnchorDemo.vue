<script setup lang="ts">
import { computed, ref } from 'vue'

type Direction = 'deposit' | 'withdraw'

const direction = ref<Direction>('deposit')
const step = ref(0)

const flows = {
  deposit: [
    { title: 'TRY yatırma talebi', text: 'Gerçek bir hizmette kullanıcı yetkili anchor üzerinden talep açar. Bu ekranda talep gönderilmez.' },
    { title: 'Banka hareketi doğrulaması', text: 'Gerçek hizmette ödemeyi sağlayıcı doğrular. Bu simülasyon banka hesabına bağlanmaz.' },
    { title: 'Stellar varlığının gönderimi', text: 'Gerçek hizmette ihraççı varlığı cüzdana aktarabilir. Burada token basılmaz veya transfer edilmez.' },
    { title: 'Örnek akış bitti', text: 'Adımlar yalnızca anlatım içindir. TRY veya Stellar bakiyesi oluşmadı ve değişmedi.' },
  ],
  withdraw: [
    { title: 'Varlığı çekme talebi', text: 'Gerçek bir hizmette kullanıcı yetkili anchor üzerinden çekim talebi açar. Bu ekranda talep gönderilmez.' },
    { title: 'Varlık ve kimlik kontrolü', text: 'Gerçek hizmette sağlayıcı bakiye ve uygunluğu denetler. Bu simülasyon cüzdana bağlanmaz.' },
    { title: 'Banka hesabına TRY ödemesi', text: 'Gerçek hizmette sağlayıcı uygun talebi banka kanalıyla sonuçlandırır. Burada ödeme emri verilmez.' },
    { title: 'Örnek akış bitti', text: 'Adımlar yalnızca anlatım içindir. TRY veya Stellar bakiyesi oluşmadı ve değişmedi.' },
  ],
} as const

const current = computed(() => flows[direction.value][step.value]!)

function select(next: Direction) {
  direction.value = next
  step.value = 0
}

function advance() {
  if (step.value < flows[direction.value].length - 1) step.value += 1
  else step.value = 0
}
</script>

<template>
  <section class="card space-y-4" aria-labelledby="anchor-demo-title">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <p class="eyebrow text-brand-700">Anchor akış simülasyonu</p>
        <h2 id="anchor-demo-title" class="mt-1 text-2xl font-extrabold">TRY ile Stellar bağlantısı nasıl kurulabilir?</h2>
      </div>
      <span class="badge bg-amber-100 text-amber-900">Yalnızca simülasyon</span>
    </div>

    <p class="text-sm leading-relaxed text-stone-600">
      Gerçek anchor, banka veya ihraççı entegrasyonu yoktur. Bu araç kişisel bilgi istemez,
      ödeme talimatı veya zincir işlemi üretmez. Kullanılabilir bakiye oluşturmaz.
    </p>

    <div class="flex flex-wrap gap-2" role="group" aria-label="Örnek akış yönü">
      <button type="button" class="btn-secondary" :aria-pressed="direction === 'deposit'" :class="direction === 'deposit' ? '!border-brand-600 !bg-brand-50 !text-brand-900' : ''" @click="select('deposit')">
        TRY → Stellar varlığı
      </button>
      <button type="button" class="btn-secondary" :aria-pressed="direction === 'withdraw'" :class="direction === 'withdraw' ? '!border-brand-600 !bg-brand-50 !text-brand-900' : ''" @click="select('withdraw')">
        Stellar varlığı → TRY
      </button>
    </div>

    <div :key="`${direction}-${step}`" class="pop rounded-2xl border border-gold-300/60 bg-gold-100/70 p-5" role="status" aria-live="polite">
      <div class="flex items-center gap-1.5" aria-hidden="true">
        <span
          v-for="n in 4"
          :key="n"
          class="h-1.5 flex-1 rounded-full transition-colors duration-300"
          :class="n <= step + 1 ? 'bg-brand-500' : 'bg-gold-300/50'"
        />
      </div>
      <p class="mt-3 text-xs font-semibold uppercase tracking-wide text-amber-900">Simülasyon · adım {{ step + 1 }} / 4</p>
      <h3 class="mt-2 font-semibold text-amber-950">{{ current.title }}</h3>
      <p class="mt-2 text-sm leading-relaxed text-amber-950">{{ current.text }}</p>
    </div>

    <button type="button" class="btn-primary" @click="advance">
      {{ step === 3 ? 'Örneği başa al' : 'Sonraki örnek adım' }}
    </button>
  </section>
</template>
