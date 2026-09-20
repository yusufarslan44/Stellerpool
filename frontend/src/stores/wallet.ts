import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { errorMessage, isUserRejection } from '@/lib/errors'
import { config, requireTestnetDemo } from '@/lib/stellar'
import { StellarWalletsKit } from '@/lib/wallet-kit'

export interface SignOptions {
  networkPassphrase?: string
  address?: string
}

export const useWalletStore = defineStore('wallet', () => {
  const address = ref<string | null>(null)
  const busy = ref(false)
  const error = ref<string | null>(null)
  /** To warn the user if the wallet is on another network (e.g. mainnet). */
  const networkWarning = ref<string | null>(null)

  const isConnected = computed(() => address.value !== null)

  async function checkNetwork() {
    networkWarning.value = null
    try {
      const { networkPassphrase } = await StellarWalletsKit.getNetwork()
      if (networkPassphrase && networkPassphrase !== config.passphrase) {
        networkWarning.value = `Your wallet is not on ${config.label}. Switch your wallet to ${config.label} before transacting.`
      }
    } catch {
      // Some wallets do not provide network information; we already send the network passphrase when signing.
    }
  }

  /** After a page refresh, restores the previous connection (the kit keeps the address in localStorage). */
  async function restore() {
    try {
      const { address: saved } = await StellarWalletsKit.getAddress()
      address.value = saved
      await checkNetwork()
    } catch {
      address.value = null
    }
  }

  async function connect() {
    busy.value = true
    error.value = null
    try {
      const { address: connected } = await StellarWalletsKit.authModal()
      address.value = connected
      await checkNetwork()
    } catch (e) {
      // Closing the window is not treated as an error.
      if (!isUserRejection(e)) error.value = errorMessage(e)
    } finally {
      busy.value = false
    }
  }

  async function disconnect() {
    await StellarWalletsKit.disconnect()
    address.value = null
    networkWarning.value = null
  }

  /** Signing function for `contract.Client` and trustline transactions. */
  async function signTransaction(xdr: string, opts?: SignOptions) {
    requireTestnetDemo()
    if (!address.value) throw new Error('Connect your wallet first.')
    return StellarWalletsKit.signTransaction(xdr, {
      networkPassphrase: opts?.networkPassphrase ?? config.passphrase,
      address: opts?.address ?? address.value,
    })
  }

  return {
    address,
    busy,
    error,
    networkWarning,
    isConnected,
    restore,
    connect,
    disconnect,
    signTransaction,
  }
})
