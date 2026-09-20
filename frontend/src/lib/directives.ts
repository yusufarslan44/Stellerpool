import type { Directive } from 'vue'

const reducedMotion = () =>
  typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

let observer: IntersectionObserver | null = null

function getObserver(): IntersectionObserver | null {
  if (typeof IntersectionObserver === 'undefined') return null
  observer ??= new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue
        entry.target.classList.add('is-in')
        observer?.unobserve(entry.target)
      }
    },
    { threshold: 0.12, rootMargin: '0px 0px -6% 0px' },
  )
  return observer
}

/**
 * An element that appears on scroll. Value: stagger index (0, 1, 2…), delay = index × 80 ms.
 * If IntersectionObserver is missing or motion is reduced, the element is shown immediately.
 */
export const vReveal: Directive<HTMLElement, number | undefined> = {
  mounted(el, binding) {
    el.setAttribute('data-reveal', '')
    el.style.setProperty('--d', String(binding.value ?? 0))
    const io = getObserver()
    if (!io || reducedMotion()) {
      el.classList.add('is-in')
      return
    }
    io.observe(el)
  },
  unmounted(el) {
    observer?.unobserve(el)
  },
}

/** A card that tilts slightly with mouse movement (only with a mouse and when motion is on). */
export const vTilt: Directive<HTMLElement, number | undefined> = {
  mounted(el, binding) {
    if (reducedMotion() || !window.matchMedia('(hover: hover)').matches) return
    const max = binding.value ?? 6
    el.setAttribute('data-tilt', '')
    const onMove = (e: PointerEvent) => {
      const r = el.getBoundingClientRect()
      const px = (e.clientX - r.left) / r.width - 0.5
      const py = (e.clientY - r.top) / r.height - 0.5
      el.style.setProperty('--ry', `${(px * max * 2).toFixed(2)}deg`)
      el.style.setProperty('--rx', `${(-py * max * 2).toFixed(2)}deg`)
    }
    const onLeave = () => {
      el.style.setProperty('--rx', '0deg')
      el.style.setProperty('--ry', '0deg')
    }
    el.addEventListener('pointermove', onMove)
    el.addEventListener('pointerleave', onLeave)
    ;(el as HTMLElement & { _tiltOff?: () => void })._tiltOff = () => {
      el.removeEventListener('pointermove', onMove)
      el.removeEventListener('pointerleave', onLeave)
    }
  },
  unmounted(el) {
    ;(el as HTMLElement & { _tiltOff?: () => void })._tiltOff?.()
  },
}
