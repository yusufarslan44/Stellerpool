import type { IlloName } from '@/lib/icon-data'

const files = import.meta.glob<string>('../assets/illo/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
})

/** URL of an illustration under src/assets/illo (Fluent Emoji, MIT). */
export function illoUrl(name: IlloName): string {
  return files[`../assets/illo/${name}.svg`] as string
}
