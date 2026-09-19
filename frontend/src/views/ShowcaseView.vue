<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
import Illo from '@/components/Illo.vue'
import AnchorDemo from '@/components/AnchorDemo.vue'
import Scene3D from '@/components/Scene3D.vue'
import { useNetworkStatus } from '@/composables/useNetworkStatus'

const { ledger, online } = useNetworkStatus()
</script>

<template>
  <div class="space-y-8">
    <section class="sunrise relative isolate overflow-hidden rounded-[2.25rem] px-5 py-10 sm:px-10 sm:py-14">
      <div class="grid items-center gap-6 lg:grid-cols-[1.05fr_1fr]">
        <div>
          <div class="glass inline-flex items-center gap-2 rounded-full px-3.5 py-1.5 text-xs font-medium text-stone-700">
            <span
              class="size-2 rounded-full"
              :class="online === false ? 'bg-rose-500' : 'animate-pulse bg-sage-500'"
              aria-hidden="true"
            />
            <span v-if="online === false">Mainnet bağlantısı şu anda okunamıyor</span>
            <span v-else-if="ledger !== null">Stellar Mainnet · defter #{{ ledger.toLocaleString('tr-TR') }}</span>
            <span v-else>Stellar Mainnet bağlantısı kontrol ediliyor…</span>
          </div>

          <h1 class="mt-5 text-5xl leading-[1.02] font-extrabold sm:text-6xl">
            Birlikte biriktir.
            <span class="block bg-gradient-to-r from-brand-600 via-brand-500 to-gold-500 bg-clip-text text-transparent">
              Kuralları herkes görsün.
            </span>
          </h1>
          <p class="mt-5 max-w-xl text-lg leading-relaxed text-stone-700">
            Stellerpool, davetli altın günü gruplarında katkı ve sıranın doğrulanabilir olmasını
            hedefleyen bir prototip. Grup koşulları katılımcıların ortak onayıyla belirlenir.
          </p>
          <a
            href="https://github.com/yusufarslan44/Stellerpool"
            target="_blank"
            rel="noopener noreferrer"
            class="btn-primary btn-lg mt-7"
          >
            Projeyi GitHub'da incele <AppIcon name="arrow" class="!size-4" />
          </a>
        </div>
        <div class="relative mx-auto h-[300px] w-full max-w-xl sm:h-[380px]">
          <Scene3D :coins="4" label="Ortada büyük bir havuz parası, çevresinde dönen dört üye parası" />
        </div>
      </div>
    </section>

    <p role="status" class="rounded-3xl border border-gold-300/60 bg-gold-100/70 p-5 text-sm leading-relaxed text-amber-950">
      Bu, Mainnet'e bağlanan <strong>işlemsiz tanıtım sürümüdür</strong>. Cüzdan imzası, fon yatırma,
      havuz açma ve gerçek TL anchor işlemleri kapalıdır. Aşağıdaki anchor adımları yalnızca
      simülasyondur. Mainnet'te bir havuz kontratı yayınlanmadı;
      gerçek para, ev veya araç teslimi ve finansman garantisi sunulmaz.
    </p>

    <section aria-labelledby="akim" class="space-y-4">
      <div>
        <p class="eyebrow text-brand-700">Önerilen ürün akışı</p>
        <h2 id="akim" class="mt-1 text-2xl font-bold tracking-tight sm:text-3xl">
          Davetli grubun kendi kuralları
        </h2>
      </div>
      <div class="grid gap-4 md:grid-cols-3">
        <article class="card">
          <Illo name="handshake" :size="52" />
          <h3 class="mt-4 font-semibold">Sıra birlikte belirlenir</h3>
          <p class="mt-2 text-sm leading-relaxed text-stone-600">
            Üyeler katkıyı, takvimi ve teslim sırasını başlangıçta görür ve onaylar.
          </p>
        </article>
        <article class="card">
          <Illo name="lock" :size="52" />
          <h3 class="mt-4 font-semibold">Kurallar görünür olur</h3>
          <p class="mt-2 text-sm leading-relaxed text-stone-600">
            Prototip, üye onayı, eksik ödeme ve iade koşullarını Soroban üzerinde test etmeyi hedefler.
          </p>
        </article>
        <article class="card">
          <Illo name="shield" :size="52" />
          <h3 class="mt-4 font-semibold">Riskler incelenir</h3>
          <p class="mt-2 text-sm leading-relaxed text-stone-600">
            Planlanan AI denetçisi uyuşmazlıkları işaretler; fon yönetmez ve hukuki onay vermez.
          </p>
        </article>
      </div>
    </section>

    <AnchorDemo />

    <section class="card space-y-3">
      <h2 class="text-xl font-semibold">Ev ve araç hedefleri</h2>
      <p class="text-sm leading-relaxed text-stone-600">
        Grup üyeleri birikimlerini ev veya araç hedefi için kullanmayı isteyebilir. Belirli tarihte
        finansman ve mülk teslimi taahhüdü, ancak gerekli izinlere sahip bir şirketin kendi
        sözleşmesi ve fon düzeniyle ayrıca tasarlanabilir. Anchor demosu da gerçek TL hizmeti
        veya faaliyet izni yerine geçmez.
      </p>
    </section>
  </div>
</template>
