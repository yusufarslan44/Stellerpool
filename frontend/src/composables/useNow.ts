import { onMounted, onUnmounted, ref } from 'vue'

/** "Now" as unix seconds, updated every second (for countdowns). */
export function useNow() {
  const now = ref(Math.floor(Date.now() / 1000))
  let timer: ReturnType<typeof setInterval> | undefined

  onMounted(() => {
    timer = setInterval(() => {
      now.value = Math.floor(Date.now() / 1000)
    }, 1000)
  })
  onUnmounted(() => clearInterval(timer))

  return now
}
