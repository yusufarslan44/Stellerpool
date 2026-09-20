import { Asset, Horizon, Networks, rpc } from '@stellar/stellar-sdk'

const network = import.meta.env.VITE_STELLAR_NETWORK?.trim().toLowerCase() || 'testnet'
if (network !== 'testnet' && network !== 'mainnet') {
  throw new Error('VITE_STELLAR_NETWORK must be either testnet or mainnet.')
}

/** Mainnet is for the showcase and network reads only; funds actions are specific to Testnet. */
export const mainnetShowcase = network === 'mainnet'

export const config = mainnetShowcase
  ? {
      label: 'Mainnet showcase',
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
    throw new Error('The Mainnet showcase build does not accept funds transactions or wallet signatures.')
  }
}

export const rpcServer = new rpc.Server(config.rpcUrl)
export const horizonServer = new Horizon.Server(config.horizonUrl)

/** The asset used in the Testnet pool demo. The Mainnet showcase uses no pool asset. */
export const poolAsset = new Asset(
  import.meta.env.VITE_POOL_ASSET_CODE?.trim() || 'USDC',
  import.meta.env.VITE_POOL_ASSET_ISSUER?.trim() ||
    'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5',
)

/** Circle's Testnet USDC issuer: test balances of this asset come from the Circle faucet. */
export const CIRCLE_TESTNET_USDC_ISSUER = 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5'
export const poolAssetFromCircleFaucet =
  poolAsset.getCode() === 'USDC' && poolAsset.getIssuer() === CIRCLE_TESTNET_USDC_ISSUER

/** The Stellar Asset Contract (SAC) address of the pool asset. The contract takes this as its token. */
export const poolTokenContractId = poolAsset.contractId(config.passphrase)

/** The deployed RotatingPool contract. If empty, the interface shows the "not configured" state. */
export const poolContractId: string | null =
  mainnetShowcase ? null : import.meta.env.VITE_ROTATING_POOL_CONTRACT_ID?.trim() || null

/**
 * A ready-made Testnet demo seller anyone can use: it only receives payments (TRYT and Circle USDC trustlines
 * are open), its secret key is not stored, and it is not a real seller. It comes pre-filled in the creation form; you can type your own
 * address. Can be changed with `VITE_DEMO_SELLER`.
 */
export const demoSellerAddress: string =
  import.meta.env.VITE_DEMO_SELLER?.trim() || 'GBSYVBANLBFLTUNHLTYNCYQUI4LPBP4YEIPOLONUT3OEATONSLP57QTG'

/** The anchor's home domain (stellar.toml is published there). To be settled at the workshop. */
export const anchorHomeDomain: string | null =
  import.meta.env.VITE_ANCHOR_HOME_DOMAIN?.trim() || null

export const explorerAccount = (address: string) => `${config.explorer}/account/${address}`
export const explorerContract = (id: string) => `${config.explorer}/contract/${id}`
export const explorerTx = (hash: string) => `${config.explorer}/tx/${hash}`
