import type { IlloName } from '@/lib/icon-data'

const files = import.meta.glob<string>('../assets/illo/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
})

/** src/assets/illo altındaki çizimin (Fluent Emoji, MIT) URL'si. */
export function illoUrl(name: IlloName): string {
  return files[`../assets/illo/${name}.svg`] as string
}
