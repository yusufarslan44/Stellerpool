// Uçtan uca doğrulama: SEP-10 login + SEP-24 interactive deposit + confirm + poll.
// Gerçek bir cüzdan yerine burada test amaçlı bir Keypair üretip Friendbot ile fonluyoruz.
// Kullanım: node scripts/test-flow.mjs [baseUrl] [issuerPublicKey]
//   baseUrl varsayılan: http://localhost:3001 (deploy sonrası doğrulamak için gerçek
//   PUBLIC_BASE_URL'i ilk argüman olarak verin).
//   issuerPublicKey varsayılan: .env'deki ISSUER_SECRET'tan türetilir.
import 'dotenv/config'
import { Keypair, TransactionBuilder, Networks, Asset, BASE_FEE, Operation, Horizon } from '@stellar/stellar-sdk'

const BASE = (process.argv[2] || 'http://localhost:3001').replace(/\/$/, '')
const issuerArg = process.argv[3] || (process.env.ISSUER_SECRET ? Keypair.fromSecret(process.env.ISSUER_SECRET).publicKey() : '')
if (!issuerArg) throw new Error('issuerPublicKey verilmedi ve .env\'de ISSUER_SECRET yok.')
const passphrase = Networks.TESTNET
const horizon = new Horizon.Server('https://horizon-testnet.stellar.org')
console.log('Hedef:', BASE, ' issuer:', issuerArg)

const user = Keypair.random()
console.log('Test kullanıcı hesabı:', user.publicKey())

console.log('==> Friendbot ile fonlanıyor')
await fetch(`https://friendbot.stellar.org?addr=${user.publicKey()}`)

console.log('==> SEP-10 challenge isteniyor')
const homeDomain = new URL(BASE).host
const challengeRes = await fetch(`${BASE}/auth?account=${user.publicKey()}&home_domain=${homeDomain}`)
const { transaction } = await challengeRes.json()

const tx = TransactionBuilder.fromXDR(transaction, passphrase)
tx.sign(user)
const signedXdr = tx.toXDR()

console.log('==> imzalı challenge gönderiliyor, token isteniyor')
const tokenRes = await fetch(`${BASE}/auth`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ transaction: signedXdr }),
})
const { token, error: authErr } = await tokenRes.json()
if (authErr) throw new Error('SEP-10 hata: ' + authErr)
console.log('    token alındı (ilk 20 karakter):', token.slice(0, 20) + '...')

console.log('==> SEP-24 /info')
const info = await (await fetch(`${BASE}/sep24/info`)).json()
console.log('   ', JSON.stringify(info))

console.log('==> SEP-24 interactive deposit başlatılıyor')
const depositRes = await fetch(`${BASE}/sep24/transactions/deposit/interactive`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
  body: JSON.stringify({ asset_code: 'TRYT', account: user.publicKey(), lang: 'tr' }),
})
const { url, id } = await depositRes.json()
console.log('    interactive url:', url, ' id:', id)

console.log('==> kullanıcı henüz trustline açmadan formu onaylıyor (pending_trust bekleniyor)')
const confirmRes1 = await fetch(`${BASE}/sep24/interactive/${id}/confirm`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
  body: `amount=250`,
})
console.log('    confirm HTTP', confirmRes1.status)

let poll = await (
  await fetch(`${BASE}/sep24/transaction?id=${id}`, { headers: { Authorization: `Bearer ${token}` } })
).json()
console.log('==> poll #1 (trustline açılmadan önce):', JSON.stringify(poll.transaction))
if (poll.transaction.status !== 'pending_trust') throw new Error('Beklenen durum pending_trust değil: ' + poll.transaction.status)

console.log('==> şimdi kullanıcı TRYT için trustline açıyor (gerçek on-chain işlem)')
const account = await horizon.loadAccount(user.publicKey())
const trustTx = new TransactionBuilder(account, { fee: BASE_FEE, networkPassphrase: passphrase })
  .addOperation(Operation.changeTrust({ asset: new Asset('TRYT', issuerArg) }))
  .setTimeout(60)
  .build()
trustTx.sign(user)
await horizon.submitTransaction(trustTx)
console.log('    trustline açıldı')

poll = await (
  await fetch(`${BASE}/sep24/transaction?id=${id}`, { headers: { Authorization: `Bearer ${token}` } })
).json()
console.log('==> poll #2 (trustline sonrası, ödeme burada otomatik tekrar denenmeli):', JSON.stringify(poll.transaction))
if (poll.transaction.status !== 'completed') throw new Error('Beklenen durum completed değil: ' + poll.transaction.status)

const finalAccount = await horizon.loadAccount(user.publicKey())
const bal = finalAccount.balances.find((b) => b.asset_type !== 'native' && b.asset_code === 'TRYT')
console.log('==> son bakiye:', bal?.balance, 'TRYT')
console.log('\n✅ Uçtan uca SEP-1/10/24 akışı başarılı.')
