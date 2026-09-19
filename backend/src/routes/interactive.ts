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
    res.status(404).send(page('Bulunamadı', '<h1>İşlem bulunamadı</h1>'))
    return
  }
  if (tx.status !== 'incomplete') {
    res.send(page('Zaten işlendi', `<h1>Bu işlem zaten "${tx.status}" durumunda.</h1>`))
    return
  }
  res.send(
    page(
      'Stellerpool test anchor — TRY yatır',
      `<h1>${config.assetCode} yatır (test)</h1>
       <div class="warn">Bu, hackathon demo anchor'ıdır. <b>Gerçek Türk lirası veya banka
       entegrasyonu yoktur.</b> "Yatırdım" butonu bankayı değil, bu sunucunun kendi onayını tetikler.</div>
       <form method="post" action="/sep24/interactive/${tx.id}/confirm">
         <label for="amount">Tutar (${config.assetCode})</label>
         <input id="amount" name="amount" type="number" min="10" max="100000" step="1" value="100" required>
         <label for="account">Alıcı Stellar hesabı</label>
         <input value="${tx.account}" disabled>
         <button type="submit">TRY yatırdım, onayla (test)</button>
       </form>
       <p class="muted">Hesabınızda ${config.assetCode} için trustline yoksa (issuer
       ${(tryAsset.getIssuer() ?? '').slice(0, 6)}…), onay sonrası ödeme "trustline bekleniyor" durumunda
       kalır; trustline'ı açtığınızda arayüz bir sonraki kontrolde otomatik tamamlar.</p>`,
    ),
  )
})

interactiveRouter.post('/sep24/interactive/:id/confirm', async (req, res) => {
  const tx = getTx(req.params.id)
  if (!tx) {
    res.status(404).send(page('Bulunamadı', '<h1>İşlem bulunamadı</h1>'))
    return
  }
  const amount = String(req.body?.amount ?? '').trim()
  if (!amount || Number.isNaN(Number(amount)) || Number(amount) <= 0) {
    res.status(400).send(page('Geçersiz tutar', '<h1>Geçerli bir tutar girin.</h1>'))
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
      updateTx(tx.id, { status: 'error', message: err instanceof Error ? err.message : 'Ödeme başarısız.' })
    }
  }

  const finalTx = getTx(tx.id)!
  res.send(
    page(
      'Onaylandı',
      `<h1>${finalTx.status === 'completed' ? 'Tamamlandı' : finalTx.status === 'pending_trust' ? 'Trustline bekleniyor' : 'İşleniyor'}</h1>
       <p>Durum: <b>${finalTx.status}</b></p>
       ${finalTx.status === 'pending_trust' ? '<p class="muted">Cüzdanınızda bu varlık için trustline açın, arayüz otomatik tamamlayacak.</p>' : ''}
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
