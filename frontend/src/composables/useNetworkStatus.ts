import { onMounted, onUnmounted, ref } from 'vue'
import { config, rpcServer } from '@/lib/stellar'

/**
 * Seçilen Stellar ağının son defter numarası. RPC'nin ağ kimliği ilk okumada doğrulanır;
 * yanlış ağa yönlendiren bir URL, Mainnet verisi gibi gösterilmez.
 */
export function useNetworkStatus() {
  const ledger = ref<number | null>(null)
  const online = ref<boolean | null>(null)
  let timer: ReturnType<typeof setInterval> | undefined
  let networkVerified = false

  async function refresh() {
    try {
      if (!networkVerified) {
        const network = await rpcServer.getNetwork()
        if (network.passphrase !== config.passphrase) throw new Error('RPC network does not match.')
        networkVerified = true
      }
      const latest = await rpcServer.getLatestLedger()
      ledger.value = latest.sequence
      online.value = true
    } catch {
      online.value = false
    }
  }

  onMounted(() => {
    void refresh()
    timer = setInterval(refresh, 10_000)
  })
  onUnmounted(() => clearInterval(timer))

  return { ledger, online }
}
