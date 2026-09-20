import { StellarToml, WebAuth } from '@stellar/stellar-sdk'
import { anchorHomeDomain, config } from '@/lib/stellar'

/**
 * Anchor istemcisi: SEP-1 (stellar.toml) → SEP-10 (web auth) → SEP-24 (interaktif yatırma/çekme).
 *
 * Varsayılan sağlayıcı SDF'nin referans TEST anchor'ıdır: gerçek protokol akışını çalıştırır ama
 * test varlığı üretir, Türk lirası DEĞİLDİR. Gerçek bir TRY sağlayıcısı doğrulanınca yalnızca
 * `VITE_ANCHOR_HOME_DOMAIN` değişir (docs/altin-gunu-legal-boundary.md "Anchor seçim kapısı").
 * Anchor'ın interaktif pencere adresi yalnızca https ise kabul edilir, kullanıcıya alan adıyla
 * gösterilir ve pencere ancak kullanıcı tıklayınca açılır (SDF test anchor'ı arayüzü kardeş bir
 * alt alan adında sunar, bu yüzden home domain eşleşmesi zorunlu tutulmaz).
 */

export const TEST_ANCHOR_DOMAIN = 'testanchor.stellar.org'
export const anchorDomain = anchorHomeDomain ?? TEST_ANCHOR_DOMAIN
export const usingTestAnchor = anchorDomain === TEST_ANCHOR_DOMAIN

export type AnchorKind = 'deposit' | 'withdraw'

export interface AssetSupport {
  enabled: boolean
  minAmount?: number
  maxAmount?: number
}

export interface AnchorInfo {
  domain: string
  webAuthEndpoint: string
  sep24Server: string
  signingKey: string
  deposit: Record<string, AssetSupport>
  withdraw: Record<string, AssetSupport>
  /** stellar.toml'da ve SEP-24 bilgisinde geçen tüm varlık kodları. */
  assetCodes: string[]
  /** Anchor gerçek bir TRY varlığı (TRY / TRYB) sunuyor mu? Sunmuyorsa arayüz "TRY değil" der. */
  supportsTry: boolean
  /** TRY'yi yalnızca TEMSİL eden bir test varlığı (örn. TRYT) var mı? Bu gerçek Türk lirası değildir. */
  representsTry: boolean
  /** stellar.toml [[CURRENCIES]] kayıtlarındaki varlık kodu → ihraççı eşlemesi (native'in ihraççısı yoktur). */
  issuers: Record<string, string>
}

export type Signer = (
  xdr: string,
  opts: { networkPassphrase: string; address: string },
) => Promise<{ signedTxXdr: string }>

const TIMEOUT_MS = 15_000

function requireHttps(url: string, what: string): URL {
  let parsed: URL
  try {
    parsed = new URL(url)
  } catch {
    throw new Error(`The anchor ${what} address is invalid.`)
  }
  if (parsed.protocol !== 'https:') throw new Error(`The anchor ${what} address must use https.`)
  return parsed
}

/** Adres anchor'ın home domain'i (veya alt alan adı) altında ve https mi? */
export function isAnchorUrl(url: string, domain: string): boolean {
  try {
    const u = new URL(url)
    return u.protocol === 'https:' && (u.hostname === domain || u.hostname.endsWith(`.${domain}`))
  } catch {
    return false
  }
}

async function requestJson<T>(url: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(url, {
    ...init,
    headers: { Accept: 'application/json', ...init.headers },
    signal: AbortSignal.timeout(TIMEOUT_MS),
  })
  const text = await res.text()
  let body: unknown = null
  try {
    body = JSON.parse(text)
  } catch {
    /* JSON değil; aşağıda hata olarak ele alınır */
  }
  if (!res.ok) {
    const detail = (body as { error?: string } | null)?.error ?? text.slice(0, 140)
    throw new Error(`Anchor response ${res.status}: ${detail || 'no details'}`)
  }
  if (body === null || typeof body !== 'object') throw new Error('The anchor returned an unexpected response.')
  return body as T
}

interface RawSupport {
  enabled?: boolean
  min_amount?: number
  max_amount?: number
}

function mapSupport(raw: Record<string, RawSupport> | undefined): Record<string, AssetSupport> {
  const out: Record<string, AssetSupport> = {}
  for (const [code, v] of Object.entries(raw ?? {})) {
    out[code] = { enabled: v.enabled === true, minAmount: v.min_amount, maxAmount: v.max_amount }
  }
  return out
}

/** SEP-1: stellar.toml'dan uç noktaları, ardından SEP-24 /info ile desteklenen varlıkları okur. */
export async function resolveAnchor(domain: string = anchorDomain): Promise<AnchorInfo> {
  const toml = await StellarToml.Resolver.resolve(domain, { timeout: TIMEOUT_MS })
  const { WEB_AUTH_ENDPOINT, TRANSFER_SERVER_SEP0024, SIGNING_KEY } = toml
  if (!WEB_AUTH_ENDPOINT || !TRANSFER_SERVER_SEP0024 || !SIGNING_KEY) {
    throw new Error(`${domain} does not publish SEP-10 and SEP-24 endpoints.`)
  }
  if (toml.NETWORK_PASSPHRASE && toml.NETWORK_PASSPHRASE !== config.passphrase) {
    throw new Error(`${domain} is configured for a different network.`)
  }
  requireHttps(WEB_AUTH_ENDPOINT, 'web auth')
  const sep24 = requireHttps(TRANSFER_SERVER_SEP0024, 'SEP-24')

  const info = await requestJson<{
    deposit?: Record<string, RawSupport>
    withdraw?: Record<string, RawSupport>
  }>(`${sep24.origin}${sep24.pathname.replace(/\/$/, '')}/info`)

  const deposit = mapSupport(info.deposit)
  const withdraw = mapSupport(info.withdraw)
  const assetCodes = [...new Set([...Object.keys(deposit), ...Object.keys(withdraw)])]
  const issuers: Record<string, string> = {}
  for (const c of toml.CURRENCIES ?? []) {
    if (c.code && c.issuer) issuers[c.code] = c.issuer
  }
  return {
    domain,
    webAuthEndpoint: WEB_AUTH_ENDPOINT,
    sep24Server: `${sep24.origin}${sep24.pathname.replace(/\/$/, '')}`,
    signingKey: SIGNING_KEY,
    deposit,
    withdraw,
    assetCodes,
    supportsTry: assetCodes.some((c) => /^TRYB?$/i.test(c)),
    representsTry: assetCodes.some((c) => /^TRY/i.test(c) && !/^TRYB?$/i.test(c)),
    issuers,
  }
}

/**
 * Anchor, havuzun kullandığı varlığı (kod VE ihraççı aynı) yatırma için sunuyor mu? Yalnızca kod
 * eşleşmesi yetmez: aynı kodlu başka ihraççının varlığı havuzda geçmez.
 */
export function supportsPoolAsset(anchor: AnchorInfo, code: string, issuer: string): boolean {
  return anchor.deposit[code]?.enabled === true && anchor.issuers[code] === issuer
}

/**
 * SEP-10: anchor'ın challenge işlemini alır, sunucu imzasını ve alan adlarını DOĞRULAR,
 * cüzdanla imzalatır ve JWT döndürür. Token yalnızca bellekte tutulur.
 */
export async function authenticate(anchor: AnchorInfo, account: string, sign: Signer): Promise<string> {
  const challengeUrl = new URL(anchor.webAuthEndpoint)
  challengeUrl.searchParams.set('account', account)
  challengeUrl.searchParams.set('home_domain', anchor.domain)
  const challenge = await requestJson<{ transaction?: string; network_passphrase?: string }>(challengeUrl.toString())
  if (!challenge.transaction) throw new Error('The anchor did not send a challenge transaction.')
  if (challenge.network_passphrase && challenge.network_passphrase !== config.passphrase) {
    throw new Error('The anchor challenge was prepared for a different network.')
  }

  // Sunucu imzası, home domain ve web_auth_domain kontrolü (SEP-10 istemci doğrulaması).
  const { clientAccountID } = WebAuth.readChallengeTx(
    challenge.transaction,
    anchor.signingKey,
    config.passphrase,
    anchor.domain,
    new URL(anchor.webAuthEndpoint).host,
  )
  if (clientAccountID !== account) throw new Error('The anchor challenge was generated for a different account.')

  const { signedTxXdr } = await sign(challenge.transaction, { networkPassphrase: config.passphrase, address: account })
  const res = await requestJson<{ token?: string }>(anchor.webAuthEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ transaction: signedTxXdr }),
  })
  if (!res.token) throw new Error('The anchor did not provide a session key.')
  return res.token
}

export interface InteractiveSession {
  url: string
  id: string
  /** Pencerenin açılacağı alan adı; kullanıcıya gösterilir. */
  host: string
  /** Adres anchor'ın home domain'i (veya alt alan adı) altında mı? */
  sameDomain: boolean
}

/** SEP-24: interaktif yatırma veya çekme oturumu başlatır. */
export async function startInteractive(
  anchor: AnchorInfo,
  token: string,
  kind: AnchorKind,
  assetCode: string,
  account: string,
): Promise<InteractiveSession> {
  const res = await requestJson<{ url?: string; id?: string }>(`${anchor.sep24Server}/transactions/${kind}/interactive`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
    body: JSON.stringify({ asset_code: assetCode, account, lang: 'tr' }),
  })
  if (!res.url || !res.id) throw new Error('Anchor interaktif oturum adresi vermedi.')
  const popup = requireHttps(res.url, 'interaktif pencere')
  const sameDomain = isAnchorUrl(res.url, anchor.domain)
  popup.searchParams.set('callback', 'postMessage')
  return { url: popup.toString(), id: res.id, host: popup.host, sameDomain }
}

export interface AnchorTransaction {
  id: string
  status: string
  moreInfoUrl: string | null
  amountIn: string | null
  amountOut: string | null
}

export async function getTransaction(anchor: AnchorInfo, token: string, id: string): Promise<AnchorTransaction> {
  const res = await requestJson<{
    transaction?: { id?: string; status?: string; more_info_url?: string; amount_in?: string; amount_out?: string }
  }>(`${anchor.sep24Server}/transaction?id=${encodeURIComponent(id)}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
  const t = res.transaction
  if (!t?.status) throw new Error('The anchor did not provide the transaction status.')
  return {
    id: t.id ?? id,
    status: t.status,
    moreInfoUrl: t.more_info_url && isAnchorUrl(t.more_info_url, anchor.domain) ? t.more_info_url : null,
    amountIn: t.amount_in ?? null,
    amountOut: t.amount_out ?? null,
  }
}

/** SEP-24 durum kodlarının Türkçe karşılıkları. */
export const STATUS_LABELS: Record<string, string> = {
  incomplete: 'Anchor formu bekleniyor',
  pending_user_transfer_start: 'Waiting for your payment',
  pending_user_transfer_complete: 'Your payment is being processed',
  pending_external: 'Processing in an external system',
  pending_anchor: 'The anchor is processing',
  pending_stellar: 'Waiting for the Stellar transaction',
  pending_trust: 'A trustline is required for the asset',
  pending_user: 'Waiting for action from you',
  completed: 'Completed',
  refunded: 'Refunded',
  expired: 'Expired',
  no_market: 'Uygun piyasa yok',
  too_small: 'Amount too small',
  too_large: 'Amount too large',
  error: 'Anchor hata bildirdi',
}

export const TERMINAL_STATUSES = new Set(['completed', 'refunded', 'expired', 'error', 'no_market', 'too_small', 'too_large'])
