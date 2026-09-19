import { Router } from 'express'
import { config, homeDomain, sep24Server, webAuthEndpoint } from '../config.js'

export const wellKnownRouter = Router()

/**
 * SEP-1: https://developers.stellar.org/docs/tokens/anchor-assets#stellartoml
 * Frontend (`frontend/src/lib/anchor.ts` `resolveAnchor`) bu alanları okur:
 * WEB_AUTH_ENDPOINT, TRANSFER_SERVER_SEP0024, SIGNING_KEY, NETWORK_PASSPHRASE.
 */
wellKnownRouter.get('/.well-known/stellar.toml', (_req, res) => {
  const toml = `# Stellerpool anchor — Testnet, TEST amacıyla işletilir. Gerçek TRY / banka rayı DEĞİLDİR.
# Bkz. backend/README.md "Bilinçli basitleştirmeler".
NETWORK_PASSPHRASE="${config.networkPassphrase}"
WEB_AUTH_ENDPOINT="${webAuthEndpoint}"
SIGNING_KEY="${config.issuerKeypair.publicKey()}"
TRANSFER_SERVER_SEP0024="${sep24Server}"
ACCOUNTS=["${config.issuerKeypair.publicKey()}", "${config.distributionKeypair.publicKey()}"]

[DOCUMENTATION]
ORG_NAME="Stellerpool (hackathon demo)"
ORG_URL="${homeDomain}"
ORG_DESCRIPTION="Rise In x Stellar Pro Hackathon 2026 demo anchor'ı. TRYT test varlığı gerçek Türk lirasını TEMSİL EDER, gerçek para veya banka entegrasyonu değildir."

[[CURRENCIES]]
code="${config.assetCode}"
issuer="${config.issuerKeypair.publicKey()}"
display_decimals=2
name="Stellerpool Test TRY"
desc="Testnet'te TRY'yi temsil eden demo varlığı. Gerçek Türk lirası DEĞİLDİR; gerçek banka/ödeme kuruluşu entegrasyonu yoktur."
is_asset_anchored=false
anchor_asset_type="other"
`
  res.type('text/plain').send(toml)
})
