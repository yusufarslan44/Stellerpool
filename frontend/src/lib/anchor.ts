import { StellarToml, WebAuth } from '@stellar/stellar-sdk'
import { anchorHomeDomain, config } from '@/lib/stellar'

/**
 * Anchor client: SEP-1 (stellar.toml) → SEP-10 (web auth) → SEP-24 (interactive deposit/withdrawal).
 *
 * The default provider is the SDF reference TEST anchor: it runs the real protocol flow but
 * issues a test asset and is NOT Turkish lira. Once a real TRY provider is verified, only
 * `VITE_ANCHOR_HOME_DOMAIN` changes (docs/altin-gunu-legal-boundary.md "Anchor selection gate").
 * The anchor's interactive window address is accepted only if it is https, is shown to the user by domain,
 * and the window opens only when the user clicks (the SDF test anchor serves its interface on a sibling
 * subdomain, so a home-domain match is not required).
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
  /** All asset codes that appear in stellar.toml and in the SEP-24 info. */
  assetCodes: string[]
  /** Does the anchor offer a real TRY asset (TRY / TRYB)? If not, the interface says "not TRY". */
  supportsTry: boolean
  /** Is there a test asset that merely REPRESENTS TRY (e.g. TRYT)? This is not real Turkish lira. */
  representsTry: boolean
  /** Asset code → issuer mapping from the stellar.toml [[CURRENCIES]] entries (native has no issuer). */
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

/** Is the address under the anchor's home domain (or a subdomain) and https? */
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
    /* Not JSON; handled as an error below */
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

/** SEP-1: reads the endpoints from stellar.toml, then the supported assets from SEP-24 /info. */
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
 * Does the anchor offer the asset the pool uses (same code AND issuer) for deposit? Matching the code
 * alone is not enough: an asset with the same code from another issuer is not valid in the pool.
 */
export function supportsPoolAsset(anchor: AnchorInfo, code: string, issuer: string): boolean {
  return anchor.deposit[code]?.enabled === true && anchor.issuers[code] === issuer
}

/**
 * SEP-10: takes the anchor's challenge transaction, VERIFIES the server signature and domains,
 * has the wallet sign it and returns the JWT. The token is kept only in memory.
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

  // Server signature, home domain and web_auth_domain checks (SEP-10 client verification).
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
  /** The domain the window will open on; shown to the user. */
  host: string
  /** Is the address under the anchor's home domain (or a subdomain)? */
  sameDomain: boolean
}

/** SEP-24: starts an interactive deposit or withdrawal session. */
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

/** Human-readable labels for the SEP-24 status codes. */
export const STATUS_LABELS: Record<string, string> = {
  incomplete: 'Waiting for the anchor form',
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
