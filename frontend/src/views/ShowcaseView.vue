<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
import AnchorDemo from '@/components/AnchorDemo.vue'
import { useNetworkStatus } from '@/composables/useNetworkStatus'

const { ledger, online } = useNetworkStatus()
</script>

<template>
  <div class="space-y-8">
    <section class="aurora relative overflow-hidden rounded-[2rem] px-6 py-12 text-white sm:px-12 sm:py-16">
      <div class="relative z-10 max-w-2xl">
        <div class="glass-dark inline-flex items-center gap-2 rounded-full px-3.5 py-1.5 text-xs font-medium">
          <span
            class="size-2 rounded-full"
            :class="online === false ? 'bg-rose-400' : 'bg-emerald-400'"
            aria-hidden="true"
          />
          <span v-if="online === false">Mainnet bağlantısı şu anda okunamıyor</span>
          <span v-else-if="ledger !== null">Stellar Mainnet · defter #{{ ledger.toLocaleString('tr-TR') }}</span>
          <span v-else>Stellar Mainnet bağlantısı kontrol ediliyor…</span>
        </div>

        <h1 class="mt-6 text-4xl leading-tight font-bold tracking-tight sm:text-5xl">
          Birlikte biriktir.
          <span class="block text-emerald-300">Kuralları herkes görsün.</span>
        </h1>
        <p class="mt-5 max-w-xl text-lg leading-relaxed text-slate-200">
          Stellerpool, davetli altın günü gruplarında katkı ve sıranın doğrulanabilir olmasını
          hedefleyen bir prototip. Grup koşulları katılımcıların ortak onayıyla belirlenir.
        </p>
        <a
          href="https://github.com/yusufarslan44/Stellerpool"
          target="_blank"
          rel="noopener noreferrer"
          class="btn-light mt-7 inline-flex items-center gap-2"
        >
          Projeyi GitHub'da incele <AppIcon name="arrow" class="!size-4" />
        </a>
      </div>
    </section>

    <p role="status" class="rounded-2xl border border-amber-200 bg-amber-50 p-5 text-sm leading-relaxed text-amber-950">
      Bu, Mainnet'e bağlanan <strong>işlemsiz tanıtım sürümüdür</strong>. Cüzdan imzası, fon yatırma,
      havuz açma ve gerçek TL anchor işlemleri kapalıdır. Aşağıdaki anchor adımları yalnızca
      simülasyondur. Mainnet'te bir havuz kontratı yayınlanmadı;
      gerçek para, ev veya araç teslimi ve finansman garantisi sunulmaz.
    </p>

    <section aria-labelledby="akim" class="space-y-4">
      <div>
        <p class="eyebrow text-indigo-700">Önerilen ürün akışı</p>
        <h2 id="akim" class="mt-1 text-2xl font-bold tracking-tight sm:text-3xl">
          Davetli grubun kendi kuralları
        </h2>
      </div>
      <div class="grid gap-4 md:grid-cols-3">
        <article class="card">
          <AppIcon name="users" class="text-indigo-700" />
          <h3 class="mt-4 font-semibold">Sıra birlikte belirlenir</h3>
          <p class="mt-2 text-sm leading-relaxed text-slate-600">
            Üyeler katkıyı, takvimi ve teslim sırasını başlangıçta görür ve onaylar.
          </p>
        </article>
        <article class="card">
          <AppIcon name="lock" class="text-indigo-700" />
          <h3 class="mt-4 font-semibold">Kurallar görünür olur</h3>
          <p class="mt-2 text-sm leading-relaxed text-slate-600">
            Prototip, üye onayı, eksik ödeme ve iade koşullarını Soroban üzerinde test etmeyi hedefler.
          </p>
        </article>
        <article class="card">
          <AppIcon name="shield" class="text-indigo-700" />
          <h3 class="mt-4 font-semibold">Riskler incelenir</h3>
          <p class="mt-2 text-sm leading-relaxed text-slate-600">
            Planlanan AI denetçisi uyuşmazlıkları işaretler; fon yönetmez ve hukuki onay vermez.
          </p>
        </article>
      </div>
    </section>

    <AnchorDemo />

    <section class="card space-y-3">
      <h2 class="text-xl font-semibold">Ev ve araç hedefleri</h2>
      <p class="text-sm leading-relaxed text-slate-600">
        Grup üyeleri birikimlerini ev veya araç hedefi için kullanmayı isteyebilir. Belirli tarihte
        finansman ve mülk teslimi taahhüdü, ancak gerekli izinlere sahip bir şirketin kendi
        sözleşmesi ve fon düzeniyle ayrıca tasarlanabilir. Anchor demosu da gerçek TL hizmeti
        veya faaliyet izni yerine geçmez.
      </p>
    </section>
  </div>
</template>
