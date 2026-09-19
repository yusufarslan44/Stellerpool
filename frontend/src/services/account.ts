import { BASE_FEE, contract, Operation, TransactionBuilder } from '@stellar/stellar-sdk'
import { config, horizonServer, poolAsset, poolTokenContractId } from '@/lib/stellar'
import type { SignOptions } from '@/stores/wallet'

export interface AccountInfo {
  /** Hesap Testnet'te aktif mi? (Friendbot ile fonlanmamışsa false) */
  exists: boolean
  /** XLM bakiyesi, Horizon'un verdiği ondalıklı string ("9999.9999900"). */
  xlm: string
  /** Havuz asset'inin (USDC) bakiyesi; trustline yoksa null. */
  asset: string | null
  hasTrustline: boolean
}

type Signer = (xdr: string, opts?: SignOptions) => Promise<{ signedTxXdr: string }>

function isNotFound(e: unknown): boolean {
  const err = e as { name?: string; response?: { status?: number }; status?: number }
  return err?.name === 'NotFoundError' || err?.response?.status === 404 || err?.status === 404
}

/** Hesabı gerçek Horizon Testnet'ten okur. */
export async function loadAccount(address: string): Promise<AccountInfo> {
  try {
    const account = await horizonServer.loadAccount(address)
    let xlm = '0'
    let asset: string | null = null
    for (const b of account.balances) {
      if (b.asset_type === 'native') {
        xlm = b.balance
      } else if (
        'asset_code' in b &&
        b.asset_code === poolAsset.getCode() &&
        b.asset_issuer === poolAsset.getIssuer()
      ) {
        asset = b.balance
      }
    }
    return { exists: true, xlm, asset, hasTrustline: asset !== null }
  } catch (e) {
    if (isNotFound(e)) return { exists: false, xlm: '0', asset: null, hasTrustline: false }
    throw e
  }
}

/**
 * Bir adresin havuz asset'i bakiyesini SAC (Stellar Asset Contract) üzerinden okurken
 * hem G-hesaplarını hem de kontrat adreslerini destekler. Kontratın toplam bakiyesini
 * göstermek için kullanılır. Stroop cinsinden bigint döner.
 */
export async function getTokenBalance(holder: string): Promise<bigint> {
  const client = await contract.Client.from({
    contractId: poolTokenContractId,
    rpcUrl: config.rpcUrl,
    networkPassphrase: config.passphrase,
  })
  const tx = await (
    client as unknown as {
      balance: (args: { id: string }) => Promise<contract.AssembledTransaction<bigint>>
    }
  ).balance({ id: holder })
  return BigInt(tx.result)
}

/** Friendbot: Testnet hesabını oluşturur ve XLM yükler. */
export async function fundWithFriendbot(address: string): Promise<void> {
  const res = await fetch(`${config.friendbotUrl}?addr=${encodeURIComponent(address)}`)
  if (!res.ok) {
    const detail = await res.text().catch(() => '')
    throw new Error(`Friendbot hesabı fonlayamadı (${res.status}). ${detail.slice(0, 120)}`)
  }
}

/** Havuz asset'i (USDC) için trustline ekler. Yoksa asset alınamaz, payout başarısız olur. */
export async function addTrustline(address: string, sign: Signer): Promise<string> {
  const account = await horizonServer.loadAccount(address)
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: config.passphrase,
  })
    .addOperation(Operation.changeTrust({ asset: poolAsset }))
    .setTimeout(60)
    .build()

  const { signedTxXdr } = await sign(tx.toXDR(), { address })
  const signed = TransactionBuilder.fromXDR(signedTxXdr, config.passphrase)
  const result = await horizonServer.submitTransaction(signed)
  return result.hash
}
