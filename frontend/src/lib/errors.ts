/** Turns wallet, SDK and RPC errors into a single text to show the user. */
export function errorMessage(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  if (e && typeof e === 'object') {
    const obj = e as { message?: unknown; error?: unknown }
    if (typeof obj.message === 'string') return obj.message
    if (typeof obj.error === 'string') return obj.error
  }
  return 'An unknown error occurred.'
}

/** True if the user closed the wallet window or rejected the signature. */
export function isUserRejection(e: unknown): boolean {
  const msg = errorMessage(e).toLowerCase()
  return (
    msg.includes('reject') ||
    msg.includes('declin') ||
    msg.includes('denied') ||
    msg.includes('cancel') ||
    msg.includes('closed') ||
    msg.includes('user')
  )
}
