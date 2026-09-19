/**
 * Bir kerelik kurulum: issuer + distribution hesaplarını üretir, Friendbot ile fonlar,
 * distribution hesabına TRYT trustline açar, issuer'dan distribution'a arz gönderir.
 *
 * Kullanım: `npm run setup-issuer` (backend/ içinde). Çıktıdaki ISSUER_SECRET/
 * DISTRIBUTION_SECRET satırlarını backend/.env'e yapıştırın.
 */
import 'dotenv/config'
import {
  Asset,
  BASE_FEE,
  Horizon,
  Keypair,
  Networks,
  Operation,
  TransactionBuilder,
} from '@stellar/stellar-sdk'

const HORIZON_URL = process.env.HORIZON_URL?.trim() || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = process.env.NETWORK_PASSPHRASE?.trim() || Networks.TESTNET
const ASSET_CODE = process.env.ASSET_CODE?.trim() || 'TRYT'
const SUPPLY = '1000000000' // Distribution hesabına gönderilen başlangıç arzı.

const horizon = new Horizon.Server(HORIZON_URL)

async function fund(keypair: Keypair, label: string): Promise<void> {
  const res = await fetch(`https://friendbot.stellar.org?addr=${keypair.publicKey()}`)
  if (!res.ok) throw new Error(`${label} Friendbot fonlaması başarısız: ${res.status} ${await res.text()}`)
  console.log(`  ${label} fonlandı: ${keypair.publicKey()}`)
}

async function main() {
  console.log('==> issuer ve distribution hesapları üretiliyor')
  const issuer = Keypair.random()
  const distribution = Keypair.random()
  await fund(issuer, 'issuer')
  await fund(distribution, 'distribution')

  const asset = new Asset(ASSET_CODE, issuer.publicKey())

  console.log(`==> distribution hesabına ${ASSET_CODE} trustline açılıyor`)
  const distAccount = await horizon.loadAccount(distribution.publicKey())
  const trustTx = new TransactionBuilder(distAccount, { fee: BASE_FEE, networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(Operation.changeTrust({ asset }))
    .setTimeout(60)
    .build()
  trustTx.sign(distribution)
  await horizon.submitTransaction(trustTx)

  console.log(`==> issuer'dan distribution'a ${SUPPLY} ${ASSET_CODE} gönderiliyor`)
  const issuerAccount = await horizon.loadAccount(issuer.publicKey())
  const supplyTx = new TransactionBuilder(issuerAccount, { fee: BASE_FEE, networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(Operation.payment({ destination: distribution.publicKey(), asset, amount: SUPPLY }))
    .setTimeout(60)
    .build()
  supplyTx.sign(issuer)
  await horizon.submitTransaction(supplyTx)

  console.log('\n✅ Kurulum tamam. Aşağıdaki satırları backend/.env dosyasına yapıştırın:\n')
  console.log(`ISSUER_SECRET=${issuer.secret()}`)
  console.log(`DISTRIBUTION_SECRET=${distribution.secret()}`)
  console.log(`ASSET_CODE=${ASSET_CODE}`)
  console.log(`\nVarlık kodu: ${ASSET_CODE}  ·  issuer: ${issuer.publicKey()}`)
  console.log(
    `Frontend tarafında havuz varlığı olarak kullanmak için (frontend/.env):\n` +
      `VITE_POOL_ASSET_CODE=${ASSET_CODE}\nVITE_POOL_ASSET_ISSUER=${issuer.publicKey()}`,
  )
}

main().catch((err) => {
  console.error('Kurulum başarısız:', err)
  process.exit(1)
})
