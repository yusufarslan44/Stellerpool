import jwt from 'jsonwebtoken'
import { config, homeDomain } from '../config.js'

const EXPIRY_SECONDS = 24 * 60 * 60

export function issueToken(account: string): string {
  return jwt.sign(
    { sub: account, iss: homeDomain },
    config.jwtSecret,
    { expiresIn: EXPIRY_SECONDS },
  )
}

export interface SessionClaims {
  sub: string
  iss: string
}

/** Doğrular ve hesabı döndürür; geçersiz/süresi dolmuş token'da fırlatır. */
export function verifyToken(token: string): SessionClaims {
  const claims = jwt.verify(token, config.jwtSecret) as SessionClaims
  if (!claims.sub) throw new Error('Token bir hesap içermiyor.')
  return claims
}
