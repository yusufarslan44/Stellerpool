// Çalıştır: node scripts/build-icons.mjs
// Yalnızca kullanılan ikonları Iconify JSON paketlerinden ayıklayıp src/lib/icon-data.ts dosyasına yazar
// (tüm setler pakete girmesin). Lisanslar: Phosphor (MIT), Fluent Emoji (MIT, Microsoft).
import { mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const ph = require('@iconify-json/ph/icons.json')
const fe = require('@iconify-json/fluent-emoji/icons.json')

/** Arayüz simgeleri (Phosphor): denetim düğmeleri kalın, içerik simgeleri çift tonlu. */
const UI = {
  home: 'house-duotone',
  car: 'car-duotone',
  briefcase: 'briefcase-duotone',
  layers: 'stack-duotone',
  shield: 'shield-check-duotone',
  lock: 'lock-duotone',
  users: 'users-duotone',
  receipt: 'receipt-duotone',
  eye: 'eye-duotone',
  chevron: 'caret-down-bold',
  arrow: 'arrow-right-bold',
  check: 'check-bold',
  wallet: 'wallet-duotone',
  back: 'arrow-left-bold',
  sparkles: 'sparkle-duotone',
  clock: 'clock-duotone',
  coins: 'coins-duotone',
  store: 'storefront-duotone',
  alert: 'warning-duotone',
  info: 'info-duotone',
  copy: 'copy-bold',
  refresh: 'arrow-clockwise-bold',
  user: 'user-duotone',
  external: 'arrow-up-right-bold',
}

/** Renkli 3B tarzı çizimler (Fluent Emoji, renkli set). */
const ILLO = {
  home: 'house',
  car: 'automobile',
  work: 'briefcase',
  gift: 'wrapped-gift',
  rocket: 'rocket',
  seedling: 'seedling',
  shield: 'shield',
  handshake: 'handshake',
  magnifier: 'magnifying-glass-tilted-left',
  lock: 'locked',
  chain: 'link',
  eyes: 'eyes',
  pin: 'pushpin',
  coin: 'coin',
  moneybag: 'money-bag',
  receipt: 'receipt',
  sparkles: 'sparkles',
  hourglass: 'hourglass-not-done',
  alarm: 'alarm-clock',
  calendar: 'calendar',
  store: 'convenience-store',
  check: 'check-mark-button',
  warning: 'warning',
  bulb: 'light-bulb',
  purse: 'purse',
  crowd: 'busts-in-silhouette',
  party: 'party-popper',
  memo: 'memo',
  key: 'key',
  fox: 'fox',
  ayse: 'woman',
  mehmet: 'man',
  zeynep: 'woman-with-headscarf',
  can: 'man-beard',
}

function pick(set, map, label) {
  const out = {}
  for (const [name, src] of Object.entries(map)) {
    const icon = set.icons[src]
    if (!icon) throw new Error(`${label}: "${src}" bulunamadı`)
    out[name] = { body: icon.body, width: icon.width ?? set.width ?? 24, height: icon.height ?? set.height ?? 24 }
  }
  return out
}

const ui = pick(ph, UI, 'Phosphor')
const illo = pick(fe, ILLO, 'Fluent Emoji')

// Çizimler ayrı SVG dosyası olarak yazılır (<img> ile yalnızca kullanıldığında yüklenir, JS'i şişirmez).
const dir = new URL('../src/assets/illo/', import.meta.url)
mkdirSync(dir, { recursive: true })
for (const f of readdirSync(dir)) if (f.endsWith('.svg')) rmSync(new URL(f, dir))
let total = 0
for (const [name, i] of Object.entries(illo)) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${i.width} ${i.height}">${i.body}</svg>`
  writeFileSync(new URL(`${name}.svg`, dir), svg)
  total += svg.length
}

const ts = `// OTOMATİK ÜRETİLDİ: node scripts/build-icons.mjs (elle düzenleme)
// Phosphor Icons (MIT) kaynağından ayıklanmış arayüz simgeleri.
// Renkli çizimler src/assets/illo/*.svg (Microsoft Fluent Emoji, MIT).
export interface IconData {
  body: string
  width: number
  height: number
}

export const UI_ICONS = ${JSON.stringify(ui)} as const satisfies Record<string, IconData>

export type IlloName = ${Object.keys(illo).map((n) => `'${n}'`).join(' | ')}
`
writeFileSync(new URL('../src/lib/icon-data.ts', import.meta.url), ts)
console.log(`UI: ${Object.keys(ui).length} (${(ts.length / 1024).toFixed(1)} KB), çizim: ${Object.keys(illo).length} dosya (${(total / 1024).toFixed(0)} KB toplam)`)
