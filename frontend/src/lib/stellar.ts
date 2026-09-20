import { Asset, Horizon, Networks, rpc } from '@stellar/stellar-sdk'

const network = import.meta.env.VITE_STELLAR_NETWORK?.trim().toLowerCase() || 'testnet'
if (network !== 'testnet' && network !== 'mainnet') {
  throw new Error('VITE_STELLAR_NETWORK yalnızca testnet veya mainnet olabilir.')
}

/** Mainnet yalnızca tanıtım ve ağ okuması içindir; fon işlemleri Testnet'e özgüdür. */
export const mainnetShowcase = network === 'mainnet'

export const config = mainnetShowcase
  ? {
      label: 'Mainnet tanıtım',
      passphrase: Networks.PUBLIC,
      horizonUrl: 'https://horizon.stellar.org',
      rpcUrl: import.meta.env.VITE_STELLAR_RPC_URL?.trim() || 'https://mainnet.sorobanrpc.com',
      explorer: 'https://stellar.expert/explorer/public',
    }
  : {
      label: 'Testnet',
      passphrase: Networks.TESTNET,
      horizonUrl: 'https://horizon-testnet.stellar.org',
      rpcUrl: import.meta.env.VITE_STELLAR_RPC_URL?.trim() || 'https://soroban-testnet.stellar.org',
      explorer: 'https://stellar.expert/explorer/testnet',
    }

export const friendbotUrl = 'https://friendbot.stellar.org'
export const circleFaucetUrl = 'https://faucet.circle.com'

export function requireTestnetDemo(): void {
  if (mainnetShowcase) {
    throw new Error('Mainnet tanıtım sürümü fon işlemi veya cüzdan imzası kabul etmez.')
  }
}

export const rpcServer = new rpc.Server(config.rpcUrl)
export const horizonServer = new Horizon.Server(config.horizonUrl)

/** Testnet havuz demosunda kullanılan varlık. Mainnet tanıtımı havuz varlığı kullanmaz. */
export const poolAsset = new Asset(
  import.meta.env.VITE_POOL_ASSET_CODE?.trim() || 'USDC',
  import.meta.env.VITE_POOL_ASSET_ISSUER?.trim() ||
    'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5',
)

/** Circle'ın Testnet USDC ihraççısı: bu varlığın test bakiyesi Circle faucet'inden alınır. */
export const CIRCLE_TESTNET_USDC_ISSUER = 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5'
export const poolAssetFromCircleFaucet =
  poolAsset.getCode() === 'USDC' && poolAsset.getIssuer() === CIRCLE_TESTNET_USDC_ISSUER

/** Havuz asset'inin Stellar Asset Contract (SAC) adresi. Kontrat token olarak bunu alır. */
export const poolTokenContractId = poolAsset.contractId(config.passphrase)

/** Deploy edilmiş RotatingPool kontratı. Boşsa arayüz "yapılandırılmadı" durumunu gösterir. */
export const poolContractId: string | null =
  mainnetShowcase ? null : import.meta.env.VITE_ROTATING_POOL_CONTRACT_ID?.trim() || null

/** Anchor'ın home domain'i (stellar.toml burada yayınlanır). Workshop'ta netleşecek. */
export const anchorHomeDomain: string | null =
  import.meta.env.VITE_ANCHOR_HOME_DOMAIN?.trim() || null

export const explorerAccount = (address: string) => `${config.explorer}/account/${address}`
export const explorerContract = (id: string) => `${config.explorer}/contract/${id}`
export const explorerTx = (hash: string) => `${config.explorer}/tx/${hash}`
