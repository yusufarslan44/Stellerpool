import { Asset, BASE_FEE, Horizon, Operation, TransactionBuilder } from '@stellar/stellar-sdk'
import { config } from '../config.js'

export const horizon = new Horizon.Server(config.horizonUrl)

/** TRY'yi temsil eden test varlığı. Gerçek TRY değildir — bkz. backend/README.md. */
export const tryAsset = new Asset(config.assetCode, config.issuerKeypair.publicKey())

export class NoTrustlineError extends Error {
  constructor(account: string) {
    super(`${account} hesabı ${config.assetCode} için trustline açmamış.`)
    this.name = 'NoTrustlineError'
  }
}

async function hasTrustline(account: string): Promise<boolean> {
  const acc = await horizon.loadAccount(account)
  return acc.balances.some(
    (b) =>
      (b.asset_type === 'credit_alphanum4' || b.asset_type === 'credit_alphanum12') &&
      b.asset_code === tryAsset.getCode() &&
      b.asset_issuer === tryAsset.getIssuer(),
  )
}

/**
 * Dağıtım hesabından kullanıcıya `amount` kadar TRYT gönderir. Kullanıcının hesabında
 * henüz trustline yoksa `NoTrustlineError` fırlatır (çağıran bunu SEP-24 `pending_trust`
 * durumuna çevirmeli — trustline'ı yalnızca hesap sahibi kendi açabilir).
 */
export async function sendTry(destination: string, amount: string): Promise<string> {
  if (!(await hasTrustline(destination))) {
    throw new NoTrustlineError(destination)
  }
  const source = await horizon.loadAccount(config.distributionKeypair.publicKey())
  const tx = new TransactionBuilder(source, {
    fee: BASE_FEE,
    networkPassphrase: config.networkPassphrase,
  })
    .addOperation(Operation.payment({ destination, asset: tryAsset, amount }))
    .setTimeout(60)
    .build()
  tx.sign(config.distributionKeypair)
  const result = await horizon.submitTransaction(tx)
  return result.hash
}
