import 'dotenv/config'
import { Keypair } from '@stellar/stellar-sdk'

function required(name: string): string {
  const value = process.env[name]
  if (!value || !value.trim()) {
    throw new Error(`Missing environment variable: ${name} (see backend/.env.example)`)
  }
  return value.trim()
}

export const config = {
  port: Number(process.env.PORT ?? 3001),
  publicBaseUrl: (process.env.PUBLIC_BASE_URL ?? `http://localhost:${process.env.PORT ?? 3001}`).replace(/\/$/, ''),
  horizonUrl: process.env.HORIZON_URL?.trim() || 'https://horizon-testnet.stellar.org',
  networkPassphrase: process.env.NETWORK_PASSPHRASE?.trim() || 'Test SDF Network ; September 2015',
  assetCode: process.env.ASSET_CODE?.trim() || 'TRYT',
  jwtSecret: required('JWT_SECRET'),
  issuerKeypair: Keypair.fromSecret(required('ISSUER_SECRET')),
  distributionKeypair: Keypair.fromSecret(required('DISTRIBUTION_SECRET')),
  autoConfirmSeconds: Number(process.env.AUTO_CONFIRM_SECONDS ?? 0),
}

/** stellar.toml'un yayınlandığı, SEP-10 challenge'ında kullanılan home domain. */
export const homeDomain = new URL(config.publicBaseUrl).host
export const webAuthDomain = homeDomain
export const webAuthEndpoint = `${config.publicBaseUrl}/auth`
export const sep24Server = `${config.publicBaseUrl}/sep24`
