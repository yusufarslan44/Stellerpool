import type { NextFunction, Request, Response } from 'express'
import { verifyToken } from './jwt.js'

export interface AuthedRequest extends Request {
  account?: string
}

/** SEP-24 uçları `Authorization: Bearer <SEP-10 token>` ister. */
export function requireBearer(req: AuthedRequest, res: Response, next: NextFunction): void {
  const header = req.header('authorization') ?? ''
  const token = header.startsWith('Bearer ') ? header.slice('Bearer '.length) : ''
  if (!token) {
    res.status(401).json({ error: 'Authorization: Bearer <token> gerekli.' })
    return
  }
  try {
    req.account = verifyToken(token).sub
    next()
  } catch {
    res.status(401).json({ error: 'Token geçersiz veya süresi dolmuş.' })
  }
}
