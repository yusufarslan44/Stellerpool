import { Router } from 'express'
import { WebAuth } from '@stellar/stellar-sdk'
import { config, homeDomain, webAuthDomain } from '../config.js'
import { issueToken } from '../lib/jwt.js'

export const authRouter = Router()

const CHALLENGE_TIMEOUT_SECONDS = 300

/**
 * SEP-10 adım 1: https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0010.md
 * `frontend/src/lib/anchor.ts` `authenticate()` bunu `?account=G...&home_domain=...` ile çağırır.
 * `WebAuth.*` yardımcıları frontend'in zaten kullandığı aynı SDK modülü (`WebAuth.readChallengeTx`,
 * bkz. frontend/src/lib/anchor.ts) — sunucu tarafı karşılığı.
 */
authRouter.get('/auth', (req, res) => {
  const account = String(req.query.account ?? '')
  if (!account.startsWith('G') || account.length !== 56) {
    res.status(400).json({ error: 'Geçersiz veya eksik account parametresi.' })
    return
  }
  const transaction = WebAuth.buildChallengeTx(
    config.issuerKeypair,
    account,
    homeDomain,
    CHALLENGE_TIMEOUT_SECONDS,
    config.networkPassphrase,
    webAuthDomain,
  )
  res.json({ transaction, network_passphrase: config.networkPassphrase })
})

/** SEP-10 adım 2: cüzdanın imzaladığı challenge'ı doğrular, oturum JWT'si döner. */
authRouter.post('/auth', (req, res) => {
  const transaction = String(req.body?.transaction ?? '')
  if (!transaction) {
    res.status(400).json({ error: 'transaction alanı zorunlu.' })
    return
  }
  try {
    const { clientAccountID } = WebAuth.readChallengeTx(
      transaction,
      config.issuerKeypair.publicKey(),
      config.networkPassphrase,
      homeDomain,
      webAuthDomain,
    )
    const signersFound = WebAuth.verifyChallengeTxSigners(
      transaction,
      config.issuerKeypair.publicKey(),
      config.networkPassphrase,
      [clientAccountID],
      homeDomain,
      webAuthDomain,
    )
    if (!signersFound.includes(clientAccountID)) {
      res.status(401).json({ error: 'İmza doğrulanamadı.' })
      return
    }
    res.json({ token: issueToken(clientAccountID) })
  } catch (err) {
    res.status(401).json({ error: err instanceof Error ? err.message : 'Challenge doğrulanamadı.' })
  }
})
