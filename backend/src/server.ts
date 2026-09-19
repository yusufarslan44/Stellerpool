import express from 'express'
import cors from 'cors'
import { config, homeDomain } from './config.js'
import { wellKnownRouter } from './routes/wellKnown.js'
import { authRouter } from './routes/auth.js'
import { sep24Router } from './routes/sep24.js'
import { interactiveRouter } from './routes/interactive.js'
import { tryAsset } from './lib/stellar.js'

const app = express()
app.use(cors())
app.use(express.json())
app.use(express.urlencoded({ extended: true }))

app.get('/health', (_req, res) => {
  res.json({ ok: true, homeDomain, asset: tryAsset.getCode(), issuer: tryAsset.getIssuer() })
})

app.use(wellKnownRouter)
app.use(authRouter)
app.use(sep24Router)
app.use(interactiveRouter)

app.listen(config.port, () => {
  console.log(`Stellerpool anchor dinliyor: http://localhost:${config.port}`)
  console.log(`Genel adres (PUBLIC_BASE_URL): ${config.publicBaseUrl}`)
  console.log(`stellar.toml: ${config.publicBaseUrl}/.well-known/stellar.toml`)
  console.log(`Varlık: ${tryAsset.getCode()}:${tryAsset.getIssuer()}`)
})
