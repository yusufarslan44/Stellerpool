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
  /** Cüzdan başka bir ağdaysa (örn. mainnet) kullanıcıyı uyarmak için. */
  const networkWarning = ref<string | null>(null)

  const isConnected = computed(() => address.value !== null)

  async function checkNetwork() {
    networkWarning.value = null
    try {
      const { networkPassphrase } = await StellarWalletsKit.getNetwork()
      if (networkPassphrase && networkPassphrase !== config.passphrase) {
        networkWarning.value = `Cüzdanınız ${config.label} ağında değil. İşlem yapmadan önce cüzdanınızı ${config.label} ağına geçirin.`
      }
    } catch {
      // Bazı cüzdanlar ağ bilgisini vermez; imza sırasında zaten ağ parolası gönderiyoruz.
    }
  }

  /** Sayfa yenilenince önceki bağlantıyı geri yükler (kit adresi localStorage'da tutar). */
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
      // Pencereyi kapatmak hata sayılmaz.
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

  /** `contract.Client` ve trustline işlemleri için imza fonksiyonu. */
  async function signTransaction(xdr: string, opts?: SignOptions) {
    requireTestnetDemo()
    if (!address.value) throw new Error('Önce cüzdanınızı bağlayın.')
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
