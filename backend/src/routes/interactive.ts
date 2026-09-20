import { Router } from 'express'
import { config } from '../config.js'
import { getTx, updateTx } from '../lib/store.js'
import { NoTrustlineError, sendTry, tryAsset } from '../lib/stellar.js'

export const interactiveRouter = Router()

function page(title: string, body: string): string {
  return `<!doctype html>
<html lang="tr"><head><meta charset="utf-8"><title>${title}</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
  body{font-family:system-ui,sans-serif;max-width:420px;margin:40px auto;padding:0 16px;color:#1a1a1a}
  h1{font-size:1.1rem}
  .warn{background:#fff3cd;border:1px solid #ffe69c;border-radius:8px;padding:10px 12px;font-size:.85rem;margin:12px 0}
  label{display:block;margin:14px 0 4px;font-size:.9rem}
  input{width:100%;box-sizing:border-box;padding:8px;font-size:1rem;border:1px solid #ccc;border-radius:6px}
  button{margin-top:18px;width:100%;padding:10px;font-size:1rem;border:0;border-radius:6px;background:#111;color:#fff;cursor:pointer}
  .muted{color:#666;font-size:.85rem}
</style></head><body>${body}</body></html>`
}

/** SEP-24 interaktif deposit formu: tutar alır, "TRY yatırdım" onayını simüle eder. */
interactiveRouter.get('/sep24/interactive/:id', (req, res) => {
  const tx = getTx(req.params.id)
  if (!tx) {
    res.status(404).send(page('Not found', '<h1>Transaction not found</h1>'))
    return
  }
  if (tx.status !== 'incomplete') {
    res.send(page('Already processed', `<h1>This transaction is already "${tx.status}".</h1>`))
    return
  }
  res.send(
    page(
      'Stellarpool test anchor — deposit TRY',
      `<h1>Deposit ${config.assetCode} (test)</h1>
       <div class="warn">This is the hackathon demo anchor. <b>There is no real Turkish lira or bank
       integration.</b> The "I deposited" button triggers this server's own confirmation, not a bank.</div>
       <form method="post" action="/sep24/interactive/${tx.id}/confirm">
         <label for="amount">Tutar (${config.assetCode})</label>
         <input id="amount" name="amount" type="number" min="10" max="100000" step="1" value="100" required>
         <label for="account">Recipient Stellar account</label>
         <input value="${tx.account}" disabled>
         <button type="submit">I deposited TRY, confirm (test)</button>
       </form>
       <p class="muted">If your account has no trustline for ${config.assetCode} (issuer
       ${(tryAsset.getIssuer() ?? '').slice(0, 6)}…), after confirmation the payment stays in the "waiting for trustline" state;
       once you open the trustline, the interface completes it automatically on its next check.</p>`,
    ),
  )
})

interactiveRouter.post('/sep24/interactive/:id/confirm', async (req, res) => {
  const tx = getTx(req.params.id)
  if (!tx) {
    res.status(404).send(page('Not found', '<h1>Transaction not found</h1>'))
    return
  }
  const amount = String(req.body?.amount ?? '').trim()
  if (!amount || Number.isNaN(Number(amount)) || Number(amount) <= 0) {
    res.status(400).send(page('Invalid amount', '<h1>Enter a valid amount.</h1>'))
    return
  }
  updateTx(tx.id, { amount, status: 'pending_anchor' })

  try {
    const hash = await sendTry(tx.account, amount)
    updateTx(tx.id, { status: 'completed', stellarTxHash: hash })
  } catch (err) {
    if (err instanceof NoTrustlineError) {
      updateTx(tx.id, { status: 'pending_trust' })
    } else {
      updateTx(tx.id, { status: 'error', message: err instanceof Error ? err.message : 'Payment failed.' })
    }
  }

  const finalTx = getTx(tx.id)!
  res.send(
    page(
      'Confirmed',
      `<h1>${finalTx.status === 'completed' ? 'Completed' : finalTx.status === 'pending_trust' ? 'Waiting for trustline' : 'Processing'}</h1>
       <p>Durum: <b>${finalTx.status}</b></p>
       ${finalTx.status === 'pending_trust' ? '<p class="muted">Open a trustline for this asset in your wallet; the interface will complete it automatically.</p>' : ''}
       <p class="muted">Bu pencereyi kapatabilirsiniz.</p>
       <script>
         try {
           if (window.opener) {
             window.opener.postMessage({ transaction: { id: "${finalTx.id}", status: "${finalTx.status}" } }, '*')
           }
         } catch (e) {}
         setTimeout(() => { try { window.close() } catch (e) {} }, 2500)
       </script>`,
    ),
  )
})
