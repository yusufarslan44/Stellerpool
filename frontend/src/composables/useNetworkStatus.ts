import { onMounted, onUnmounted, ref } from 'vue'
import { config, rpcServer } from '@/lib/stellar'

/**
 * The latest ledger number of the selected Stellar network. The RPC's network identity is verified on the first read;
 * a URL that points to the wrong network is not shown as if it were Mainnet data.
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
