<script setup lang="ts">
import { StrKey } from '@stellar/stellar-sdk'
import { computed, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { formatStroops, parseAmount } from '@/lib/format'
import { requiredGuarantee } from '@/lib/guarantee'
import { explorerTx, poolAsset, poolContractId, poolTokenContractId } from '@/lib/stellar'
import { createPool } from '@/services/pool'
import { useWalletStore } from '@/stores/wallet'

const wallet = useWalletStore()
const router = useRouter()

const DURATIONS = [
  { label: '3 dakika (demo)', seconds: 3 * 60 },
  { label: '1 saat', seconds: 60 * 60 },
  { label: '1 gün', seconds: 24 * 60 * 60 },
  { label: '7 gün', seconds: 7 * 24 * 60 * 60 },
  { label: '30 gün (aylık)', seconds: 30 * 24 * 60 * 60 },
]

// Ana sayfadaki hesaplama aracından gelen değerler (?amount=…&members=…).
const route = useRoute()
const queryAmount = typeof route.query.amount === 'string' ? route.query.amount : ''
const queryMembers = Number.parseInt(String(route.query.members ?? ''), 10)

const amount = ref(queryAmount || '10')
const memberLimit = ref(
  Number.isInteger(queryMembers) && queryMembers >= 2 && queryMembers <= 12 ? queryMembers : 4,
)
const duration = ref(DURATIONS[0]!.seconds)
const graceDuration = ref(10 * 60)
const purchaseDuration = ref(30 * 60)
const setupDuration = ref(24 * 60 * 60)
const sponsorIsMe = ref(false)
const sponsorInput = ref('')
const demoSellerInput = ref('')
const busy = ref(false)
const error = ref<string | null>(null)
const txHash = ref<string | null>(null)

const sponsor = computed(() =>
  sponsorIsMe.value ? (wallet.address ?? '') : sponsorInput.value.trim(),
)

const contribution = computed(() => {
  try {
    const v = parseAmount(amount.value)
    return v > 0n ? v : null
  } catch {
    return null
  }
})

const pot = computed(() =>
  contribution.value === null ? null : contribution.value * BigInt(memberLimit.value),
)

const guarantee = computed(() =>
  contribution.value === null ? null : requiredGuarantee(memberLimit.value, contribution.value),
)

const sponsorValid = computed(() => StrKey.isValidEd25519PublicKey(sponsor.value))
const demoSeller = computed(() => demoSellerInput.value.trim())
const demoSellerValid = computed(() =>
  StrKey.isValidEd25519PublicKey(demoSeller.value) &&
  demoSeller.value !== sponsor.value &&
  demoSeller.value !== wallet.address,
)

const valid = computed(
  () =>
    contribution.value !== null &&
    memberLimit.value >= 2 &&
    memberLimit.value <= 12 &&
    sponsorValid.value &&
    demoSellerValid.value &&
    duration.value > 0 &&
    graceDuration.value > 0 &&
    purchaseDuration.value > 0 &&
    setupDuration.value > 0,
)

async function submit() {
  if (!wallet.address || contribution.value === null) return
  busy.value = true
  error.value = null
  txHash.value = null
  try {
    const result = await createPool(
      { address: wallet.address, signTransaction: wallet.signTransaction },
      {
        sponsor: sponsor.value,
        token: poolTokenContractId,
        contributionAmount: contribution.value,
        memberLimit: memberLimit.value,
        roundDuration: duration.value,
        graceDuration: graceDuration.value,
        purchaseDuration: purchaseDuration.value,
        setupDeadline: Math.floor(Date.now() / 1000) + setupDuration.value,
        demoSeller: demoSeller.value,
      },
    )
    txHash.value = result.hash
    if (result.poolId !== null) router.push(`/pool/${result.poolId}`)
  } catch (e) {
    if (!isUserRejection(e)) error.value = errorMessage(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="mx-auto max-w-xl space-y-6">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Yeni tasarruf havuzu</h1>
      <p class="mt-1 text-sm text-slate-600">
        Testnet için hedeflenen akışta herkes aynı tutarı yatırır; demo satıcıya ödeme ve kurucunun
        serbest çekimini engelleyen kurallar henüz kontratta uygulanmadı.
      </p>
    </div>

    <p
      v-if="!poolContractId"
      role="status"
      class="rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900"
    >
      Havuz sözleşmesi henüz yapılandırılmadı. Sözleşme yayınlanıp adresi
      <code class="font-mono">VITE_ROTATING_POOL_CONTRACT_ID</code> olarak eklenince havuz
      oluşturabilirsin.
    </p>

    <form class="card space-y-5" @submit.prevent="submit">
      <div>
        <label class="label" for="amount">Her turdaki katkı ({{ poolAsset.getCode() }})</label>
        <input
          id="amount"
          v-model="amount"
          class="input"
          type="text"
          inputmode="decimal"
          autocomplete="off"
          required
        />
        <p v-if="contribution === null" class="mt-1 text-xs text-rose-700">
          Geçerli bir tutar gir (en fazla 7 ondalık).
        </p>
      </div>

      <div>
        <label class="label" for="members">Üye sayısı</label>
        <input
          id="members"
          v-model.number="memberLimit"
          class="input max-w-32"
          type="number"
          min="2"
          max="12"
          step="1"
          required
        />
        <p class="mt-1 text-xs text-slate-500">Üye sayısı kadar tur olur, herkes bir kez alır.</p>
      </div>

      <div>
        <label class="label" for="duration">Tur başına son ödeme süresi</label>
        <select id="duration" v-model.number="duration" class="input">
          <option v-for="d in DURATIONS" :key="d.seconds" :value="d.seconds">{{ d.label }}</option>
        </select>
      </div>

      <div class="grid gap-4 sm:grid-cols-3">
        <div>
          <label class="label" for="grace-duration">Ek süre</label>
          <select id="grace-duration" v-model.number="graceDuration" class="input">
            <option :value="600">10 dakika (demo)</option>
            <option :value="86400">1 gün</option>
            <option :value="604800">7 gün</option>
          </select>
        </div>
        <div>
          <label class="label" for="purchase-duration">Alım onayı süresi</label>
          <select id="purchase-duration" v-model.number="purchaseDuration" class="input">
            <option :value="1800">30 dakika (demo)</option>
            <option :value="86400">1 gün</option>
            <option :value="604800">7 gün</option>
          </select>
        </div>
        <div>
          <label class="label" for="setup-duration">Kuruluş süresi</label>
          <select id="setup-duration" v-model.number="setupDuration" class="input">
            <option :value="3600">1 saat (demo)</option>
            <option :value="86400">1 gün</option>
            <option :value="604800">7 gün</option>
          </select>
        </div>
      </div>

      <fieldset class="space-y-3">
        <legend class="label">Sponsor</legend>
        <p class="text-xs text-slate-500">
          Planlanan Testnet senaryosunda güvence yatıran taraf. Bu hesap gerçek para veya teslimat
          garantisi değildir.
        </p>
        <label class="flex items-center gap-2 text-sm">
          <input v-model="sponsorIsMe" type="checkbox" class="size-4" :disabled="!wallet.address" />
          Sponsor benim (bağlı cüzdan)
        </label>
        <input
          v-if="!sponsorIsMe"
          v-model="sponsorInput"
          class="input font-mono"
          type="text"
          placeholder="G… sponsor adresi"
          autocomplete="off"
          aria-label="Sponsor adresi"
        />
        <p v-if="sponsor && !sponsorValid" class="text-xs text-rose-700">Geçerli bir Stellar adresi gir.</p>
      </fieldset>

      <div>
        <label class="label" for="demo-seller">İzinli demo satıcısı (Stellar Testnet adresi)</label>
        <input
          id="demo-seller"
          v-model="demoSellerInput"
          class="input font-mono"
          type="text"
          placeholder="G… demo satıcısı"
          autocomplete="off"
          required
        />
        <p v-if="demoSeller && !demoSellerValid" class="mt-1 text-xs text-rose-700">
          Satıcı geçerli bir adres olmalı; kurucu veya sponsor adresiyle aynı olamaz.
        </p>
        <p class="mt-1 text-xs text-slate-500">
          Sıra ve doğrulayıcılar üyeler katıldıktan sonra ayrı bir koşul sürümü olarak önerilir ve onaylanır.
        </p>
      </div>

      <dl v-if="valid && contribution !== null && pot !== null && guarantee !== null" class="rounded-xl bg-slate-50 p-4 text-sm">
        <div class="flex justify-between gap-3">
          <dt class="text-slate-600">Her tur satıcıya giden tutar</dt>
          <dd class="font-semibold">{{ formatStroops(pot) }} {{ poolAsset.getCode() }}</dd>
        </div>
        <div class="mt-2 flex justify-between gap-3">
          <dt class="text-slate-600">Gereken sponsor güvencesi</dt>
          <dd class="font-semibold">{{ formatStroops(guarantee) }} {{ poolAsset.getCode() }}</dd>
        </div>
        <div class="mt-2 flex justify-between gap-3">
          <dt class="text-slate-600">Toplam tur sayısı</dt>
          <dd class="font-semibold">{{ memberLimit }}</dd>
        </div>
        <p class="mt-3 text-xs text-slate-500">
          Sponsor güvencesi hesaplanan alt sınırdır: N üye için ⌊N²/4⌋ × katkı.
        </p>
      </dl>

      <div v-if="!wallet.isConnected">
        <button type="button" class="btn-primary w-full" :disabled="wallet.busy" @click="wallet.connect()">
          Önce cüzdan bağla
        </button>
      </div>
      <button
        v-else
        type="submit"
        class="btn-primary w-full"
        :disabled="busy || !valid || !poolContractId"
      >
        {{ busy ? 'Cüzdanı onayla…' : 'Havuzu oluştur' }}
      </button>

      <p v-if="error" role="alert" class="text-sm text-rose-700">{{ error }}</p>
      <p v-if="txHash" class="text-sm text-emerald-800">
        İşlem gönderildi:
        <a :href="explorerTx(txHash)" target="_blank" rel="noopener noreferrer" class="font-mono underline">
          {{ txHash.slice(0, 8) }}…
        </a>
        <span> Havuz numarası okunamadıysa ana sayfadan aç.</span>
      </p>
    </form>

    <RouterLink to="/" class="text-sm text-indigo-700 underline">← Ana sayfa</RouterLink>
  </div>
</template>
