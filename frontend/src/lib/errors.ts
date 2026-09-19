/** Cüzdan, SDK ve RPC hatalarını kullanıcıya gösterilecek tek bir metne çevirir. */
export function errorMessage(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  if (e && typeof e === 'object') {
    const obj = e as { message?: unknown; error?: unknown }
    if (typeof obj.message === 'string') return obj.message
    if (typeof obj.error === 'string') return obj.error
  }
  return 'Bilinmeyen bir hata oluştu.'
}

/** Kullanıcı cüzdan penceresini kapattıysa veya imzayı reddettiyse true. */
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
