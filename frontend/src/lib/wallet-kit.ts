import { Networks, StellarWalletsKit } from '@creit-tech/stellar-wallets-kit'
import { AlbedoModule } from '@creit-tech/stellar-wallets-kit/modules/albedo'
import { FreighterModule } from '@creit-tech/stellar-wallets-kit/modules/freighter'
import { HanaModule } from '@creit-tech/stellar-wallets-kit/modules/hana'
import { LobstrModule } from '@creit-tech/stellar-wallets-kit/modules/lobstr'
import { RabetModule } from '@creit-tech/stellar-wallets-kit/modules/rabet'
import { xBullModule } from '@creit-tech/stellar-wallets-kit/modules/xbull'

let initialized = false

/**
 * Initializes the Stellar Wallets Kit once. Instead of `defaultModules()` we import only
 * the Stellar wallets: extra modules such as NEAR/Solana are not bundled,
 * which shrinks both the bundle size and the dependency surface.
 */
export function initWalletKit() {
  if (initialized) return
  StellarWalletsKit.init({
    modules: [
      new FreighterModule(),
      new xBullModule(),
      new AlbedoModule(),
      new LobstrModule(),
      new HanaModule(),
      new RabetModule(),
    ],
    network: Networks.TESTNET,
  })
  initialized = true
}

export { StellarWalletsKit }
