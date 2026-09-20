import { Router } from 'express'
import { config } from '../config.js'
import { requireBearer, type AuthedRequest } from '../lib/authMiddleware.js'
import { createDepositTx, getTx } from '../lib/store.js'
import { NoTrustlineError, sendTry } from '../lib/stellar.js'

export const sep24Router = Router()

const MIN_AMOUNT = 10
const MAX_AMOUNT = 100000

/**
 * SEP-24 `/info`: `frontend/src/lib/anchor.ts` `resolveAnchor()` bunu okuyup hangi varlıkların
 * desteklendiğini (ve `supportsTry`'ı TRY* kod eşleşmesiyle) belirler.
 */
sep24Router.get('/sep24/info', (_req, res) => {
  res.json({
    deposit: {
      [config.assetCode]: { enabled: true, min_amount: MIN_AMOUNT, max_amount: MAX_AMOUNT },
    },
    withdraw: {},
    fee: { enabled: false },
  })
})

/**
 * SEP-24 `/transactions/deposit/interactive`: bir işlem kaydı açar, kullanıcının tarayıcıda
 * göreceği interaktif form adresini döner. Gerçek bir anchor burada KYC/tutar formunu sunar;
 * bu demo sunucusu aynı sunucuda basit bir HTML sayfası sunar (bkz. interactive.ts).
 */
sep24Router.post('/sep24/transactions/deposit/interactive', requireBearer, (req: AuthedRequest, res) => {
  const assetCode = String(req.body?.asset_code ?? '')
  const account = String(req.body?.account ?? req.account ?? '')
  if (assetCode !== config.assetCode) {
    res.status(400).json({ error: `Only ${config.assetCode} is supported.`, type: 'unsupported_asset' })
    return
  }
  if (account !== req.account) {
    res.status(403).json({ error: 'account does not match the session owner.' })
    return
  }
  const tx = createDepositTx(account, assetCode, '', '')
  tx.moreInfoUrl = `${config.publicBaseUrl}/sep24/interactive/${tx.id}`
  res.json({
    type: 'interactive_customer_info_needed',
    url: `${config.publicBaseUrl}/sep24/interactive/${tx.id}`,
    id: tx.id,
  })
})

/**
 * SEP-24 `/transaction`: `frontend/src/lib/anchor.ts` `getTransaction()` durumu buradan poll'lar.
 * `pending_trust` iken, kullanıcı trustline açtıysa ödemeyi burada sessizce yeniden dener —
 * ayrı bir "retry" ucu gerektirmeden kendi kendini iyileştirir.
 */
sep24Router.get('/sep24/transaction', requireBearer, async (req: AuthedRequest, res) => {
  const id = String(req.query.id ?? '')
  const tx = getTx(id)
  if (!tx || tx.account !== req.account) {
    res.status(404).json({ error: 'Transaction not found.' })
    return
  }
  if (tx.status === 'pending_trust') {
    try {
      const hash = await sendTry(tx.account, tx.amount)
      tx.status = 'completed'
      tx.stellarTxHash = hash
    } catch (err) {
      if (!(err instanceof NoTrustlineError)) {
        tx.status = 'error'
        tx.message = err instanceof Error ? err.message : 'Payment failed.'
      }
    }
  }
  res.json({
    transaction: {
      id: tx.id,
      kind: tx.kind,
      status: tx.status,
      more_info_url: tx.moreInfoUrl,
      amount_in: tx.amount || null,
      amount_out: tx.amount || null,
      stellar_transaction_id: tx.stellarTxHash ?? null,
      message: tx.message ?? null,
    },
  })
})
