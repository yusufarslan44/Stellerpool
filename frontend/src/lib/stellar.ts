import { Asset, Horizon, Networks, rpc } from '@stellar/stellar-sdk'

// Hackathon yalnızca Stellar Testnet'i hedefliyor.
export const config = {
  label: 'Testnet',
  passphrase: Networks.TESTNET,
  horizonUrl: 'https://horizon-testnet.stellar.org',
  rpcUrl: import.meta.env.VITE_STELLAR_RPC_URL?.trim() || 'https://soroban-testnet.stellar.org',
  friendbotUrl: 'https://friendbot.stellar.org',
  explorer: 'https://stellar.expert/explorer/testnet',
  circleFaucetUrl: 'https://faucet.circle.com',
} as const

export const rpcServer = new rpc.Server(config.rpcUrl)
export const horizonServer = new Horizon.Server(config.horizonUrl)

/** Havuzda kullanılan Stellar asset (varsayılan: Circle testnet USDC). */
export const poolAsset = new Asset(
  import.meta.env.VITE_POOL_ASSET_CODE?.trim() || 'USDC',
  import.meta.env.VITE_POOL_ASSET_ISSUER?.trim() ||
    'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5',
)

/** Havuz asset'inin Stellar Asset Contract (SAC) adresi. Kontrat token olarak bunu alır. */
export const poolTokenContractId = poolAsset.contractId(config.passphrase)

/** Deploy edilmiş RotatingPool kontratı. Boşsa arayüz "yapılandırılmadı" durumunu gösterir. */
export const poolContractId: string | null =
  import.meta.env.VITE_ROTATING_POOL_CONTRACT_ID?.trim() || null

/** Anchor'ın home domain'i (stellar.toml burada yayınlanır). Workshop'ta netleşecek. */
export const anchorHomeDomain: string | null =
  import.meta.env.VITE_ANCHOR_HOME_DOMAIN?.trim() || null

export const explorerAccount = (address: string) => `${config.explorer}/account/${address}`
export const explorerContract = (id: string) => `${config.explorer}/contract/${id}`
export const explorerTx = (hash: string) => `${config.explorer}/tx/${hash}`
