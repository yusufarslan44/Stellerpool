import { BASE_FEE, contract, Operation, TransactionBuilder } from '@stellar/stellar-sdk'
import { config, friendbotUrl, horizonServer, poolAsset, poolTokenContractId, requireTestnetDemo } from '@/lib/stellar'
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
 * Bir adresin bir token'daki bakiyesini SAC (Stellar Asset Contract) üzerinden okur; hem G-hesaplarını
 * hem de kontrat adreslerini destekler. `tokenId` verilmezse arayüzün varsayılan havuz varlığı
 * kullanılır; bir havuzun bakiyesi için o havuzun kendi `token` adresi verilmelidir (havuzlar farklı
 * varlıklarla kurulabilir). Stroop cinsinden bigint döner.
 */
export async function getTokenBalance(holder: string, tokenId: string = poolTokenContractId): Promise<bigint> {
  const client = await contract.Client.from({
    contractId: tokenId,
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
  requireTestnetDemo()
  const res = await fetch(`${friendbotUrl}?addr=${encodeURIComponent(address)}`)
  if (!res.ok) {
    const detail = await res.text().catch(() => '')
    throw new Error(`Friendbot hesabı fonlayamadı (${res.status}). ${detail.slice(0, 120)}`)
  }
}

/** Havuz asset'i (USDC) için trustline ekler. Yoksa asset alınamaz, payout başarısız olur. */
export async function addTrustline(address: string, sign: Signer): Promise<string> {
  requireTestnetDemo()
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

const symbolCache = new Map<string, string>()

/** Bir SAC token'ının sembolünü (varlık kodu) zincirden okur; sonuç önbelleğe alınır. */
export async function getTokenSymbol(tokenId: string): Promise<string> {
  const cached = symbolCache.get(tokenId)
  if (cached) return cached
  const client = await contract.Client.from({
    contractId: tokenId,
    rpcUrl: config.rpcUrl,
    networkPassphrase: config.passphrase,
  })
  const tx = await (
    client as unknown as { symbol: () => Promise<contract.AssembledTransaction<string>> }
  ).symbol()
  const symbol = String(tx.result)
  symbolCache.set(tokenId, symbol)
  return symbol
}
