import { randomUUID } from 'node:crypto'

/**
 * SEP-24 işlem kayıtları, yalnızca bellekte tutulur. Sunucu yeniden başlayınca kaybolur —
 * bu bir demo/hackathon anchor'ı için kabul edilebilir bir basitleştirme, gerçek bir anchor
 * kalıcı bir veritabanı kullanır (bkz. backend/README.md "Bilinçli basitleştirmeler").
 */
export type TxStatus =
  | 'incomplete'
  | 'pending_user_transfer_start'
  | 'pending_anchor'
  | 'pending_trust'
  | 'completed'
  | 'error'

export interface AnchorTx {
  id: string
  kind: 'deposit'
  account: string
  assetCode: string
  amount: string
  status: TxStatus
  createdAt: number
  moreInfoUrl: string
  stellarTxHash?: string
  message?: string
}

const transactions = new Map<string, AnchorTx>()

export function createDepositTx(account: string, assetCode: string, amount: string, moreInfoUrl: string): AnchorTx {
  const tx: AnchorTx = {
    id: randomUUID(),
    kind: 'deposit',
    account,
    assetCode,
    amount,
    status: 'incomplete',
    createdAt: Date.now(),
    moreInfoUrl,
  }
  transactions.set(tx.id, tx)
  return tx
}

export function getTx(id: string): AnchorTx | undefined {
  return transactions.get(id)
}

export function updateTx(id: string, patch: Partial<AnchorTx>): AnchorTx | undefined {
  const tx = transactions.get(id)
  if (!tx) return undefined
  Object.assign(tx, patch)
  return tx
}
