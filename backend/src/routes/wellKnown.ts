import { Router } from 'express'
import { config, homeDomain, sep24Server, webAuthEndpoint } from '../config.js'

export const wellKnownRouter = Router()

/**
 * SEP-1: https://developers.stellar.org/docs/tokens/anchor-assets#stellartoml
 * Frontend (`frontend/src/lib/anchor.ts` `resolveAnchor`) bu alanları okur:
 * WEB_AUTH_ENDPOINT, TRANSFER_SERVER_SEP0024, SIGNING_KEY, NETWORK_PASSPHRASE.
 */
wellKnownRouter.get('/.well-known/stellar.toml', (_req, res) => {
  const toml = `# Stellarpool anchor — operated on Testnet for TEST purposes. It is NOT a real TRY / bank rail.
# See "Deliberate simplifications" in backend/README.md.
NETWORK_PASSPHRASE="${config.networkPassphrase}"
WEB_AUTH_ENDPOINT="${webAuthEndpoint}"
SIGNING_KEY="${config.issuerKeypair.publicKey()}"
TRANSFER_SERVER_SEP0024="${sep24Server}"
ACCOUNTS=["${config.issuerKeypair.publicKey()}", "${config.distributionKeypair.publicKey()}"]

[DOCUMENTATION]
ORG_NAME="Stellerpool (hackathon demo)"
ORG_URL="${homeDomain}"
ORG_DESCRIPTION="Demo anchor for the Rise In x Stellar Pro Hackathon 2026. The TRYT test asset REPRESENTS Turkish lira; it is not real money or a bank integration."

[[CURRENCIES]]
code="${config.assetCode}"
issuer="${config.issuerKeypair.publicKey()}"
display_decimals=2
name="Stellerpool Test TRY"
desc="A demo asset representing TRY on Testnet. It is NOT real Turkish lira; there is no real bank/payment-institution integration."
is_asset_anchored=false
anchor_asset_type="other"
`
  res.type('text/plain').send(toml)
})
