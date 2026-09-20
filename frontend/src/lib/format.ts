// Stellar assets have 7 decimals: 1 unit = 10_000_000 stroops.
// Amounts are carried as bigint / string, never JS number, to avoid precision loss.
export const STROOPS = 10_000_000n

/** "12.5" -> 125000000n. Throws on invalid input. */
export function parseAmount(input: string): bigint {
  const s = input.trim().replace(',', '.')
  if (!/^\d+(\.\d{1,7})?$/.test(s)) {
    throw new Error('Invalid amount. At most 7 decimal places are allowed.')
  }
  const [whole, frac = ''] = s.split('.')
  return BigInt(whole) * STROOPS + BigInt(frac.padEnd(7, '0'))
}

/** 125000000n -> "12.5". For form fields and URLs; locale-independent. */
export function toPlainAmount(value: bigint): string {
  const whole = value / STROOPS
  const frac = (value % STROOPS).toString().padStart(7, '0').replace(/0+$/, '')
  return frac ? `${whole}.${frac}` : `${whole}`
}

/** 125000000n -> "12.50" (en-US). Shows at least 2 and at most maxDecimals decimals. */
export function formatStroops(value: bigint, maxDecimals = 2): string {
  const negative = value < 0n
  const abs = negative ? -value : value
  const whole = abs / STROOPS
  let frac = (abs % STROOPS).toString().padStart(7, '0').slice(0, Math.max(maxDecimals, 2))
  frac = frac.replace(/0+$/, '').padEnd(2, '0')
  return `${negative ? '-' : ''}${whole.toLocaleString('en-US')}.${frac}`
}

/** Formats a string amount such as "10.0000000" coming from Horizon. */
export function formatDecimalString(value: string, maxDecimals = 2): string {
  return formatStroops(parseAmount(value), maxDecimals)
}

export function shortAddress(address: string, chars = 4): string {
  if (address.length <= chars * 2 + 1) return address
  return `${address.slice(0, chars)}…${address.slice(-chars)}`
}

/** Shows remaining seconds as "02:59" or "1 d 03:20:00". */
export function formatDuration(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds))
  const days = Math.floor(s / 86400)
  const h = Math.floor((s % 86400) / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = s % 60
  const two = (n: number) => n.toString().padStart(2, '0')
  if (days > 0) return `${days} d ${two(h)}:${two(m)}:${two(sec)}`
  if (h > 0) return `${two(h)}:${two(m)}:${two(sec)}`
  return `${two(m)}:${two(sec)}`
}
