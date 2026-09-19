<script setup lang="ts">
import { StrKey } from '@stellar/stellar-sdk'
import { computed, ref, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
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

const amount = ref('10')
const memberLimit = ref(4)
const duration = ref(DURATIONS[0]!.seconds)
const sponsorIsMe = ref(false)
const sponsorInput = ref('')
const verifiersInput = ref('')
const threshold = ref(1)
const busy = ref(false)
const error = ref<string | null>(null)
const txHash = ref<string | null>(null)

const sponsor = computed(() =>
  sponsorIsMe.value ? (wallet.address ?? '') : sponsorInput.value.trim(),
)

const verifiers = computed(() =>
  verifiersInput.value
    .split(/[\s,;]+/)
    .map((v) => v.trim())
    .filter(Boolean),
)

// Onay eşiği doğrulayıcı sayısını aşmasın.
watch(verifiers, (list) => {
  if (list.length > 0 && threshold.value > list.length) threshold.value = list.length
})

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
const badVerifier = computed(() => verifiers.value.find((v) => !StrKey.isValidEd25519PublicKey(v)))
const verifiersValid = computed(() => verifiers.value.length > 0 && badVerifier.value === undefined)

const valid = computed(
  () =>
    contribution.value !== null &&
    memberLimit.value >= 2 &&
    memberLimit.value <= 12 &&
    sponsorValid.value &&
    verifiersValid.value &&
    threshold.value >= 1 &&
    threshold.value <= verifiers.value.length,
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
        verifiers: verifiers.value,
        approvalThreshold: threshold.value,
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
        Her tur herkes aynı tutarı yatırır, tur tutarı doğrulanmış satıcıya gider. Para sözleşmede
        durur, kurucunun serbest çekim yetkisi yoktur.
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

      <fieldset class="space-y-3">
        <legend class="label">Sponsor</legend>
        <p class="text-xs text-slate-500">
          Havuz başlamadan önce güvence yatıran taraf. Bir üye ödemeyi bırakırsa bekleyenlerin
          iadesi bu güvenceyle korunur.
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
        <label class="label" for="verifiers">Doğrulayıcılar (her satıra bir adres)</label>
        <textarea
          id="verifiers"
          v-model="verifiersInput"
          class="input min-h-24 font-mono"
          placeholder="G…&#10;G…"
          autocomplete="off"
        />
        <p v-if="badVerifier" class="mt-1 text-xs text-rose-700">
          Geçersiz adres: {{ badVerifier.slice(0, 12) }}…
        </p>
        <p class="mt-1 text-xs text-slate-500">
          Satıcı ve alım kaydını onaylayan bağımsız taraflar. Kurucunun tek başına onayı yeterli değildir.
        </p>
      </div>

      <div>
        <label class="label" for="threshold">Gereken onay sayısı</label>
        <input
          id="threshold"
          v-model.number="threshold"
          class="input max-w-32"
          type="number"
          min="1"
          :max="Math.max(1, verifiers.length)"
          step="1"
          required
        />
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
