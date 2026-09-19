<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

/**
 * Üç boyutlu altın para sahnesi (three.js, yalnızca bu bileşen açıldığında yüklenir).
 * Ortadaki büyük para "havuzu", çevresinde dönen paralar üyeleri temsil eder.
 *  - coins: yörüngedeki para sayısı (üye sayısı)
 *  - filled: kaç tanesi altın (ödemiş/katılmış); geri kalanı soluk. -1 = hepsi altın.
 * Hareket azaltma tercihinde animasyon döngüsü çalışmaz, yalnızca tek kare çizilir.
 */
const props = withDefaults(defineProps<{ coins?: number; filled?: number; label?: string }>(), {
  coins: 4,
  filled: -1,
  label: 'Havuzu temsil eden üç boyutlu altın paralar',
})

type Three = typeof import('three')

interface OrbitCoin {
  group: import('three').Group
  mats: import('three').MeshStandardMaterial[]
  scale: number
  targetScale: number
  fill: number
  targetFill: number
  angle: number
  slot: number
  phase: number
}

const host = ref<HTMLDivElement | null>(null)
const canvas = ref<HTMLCanvasElement | null>(null)
const failed = ref(false)
const ready = ref(false)

let disposed = false
let cleanup: (() => void) | null = null
let sync: (() => void) | null = null

function webglAvailable(): boolean {
  try {
    const c = document.createElement('canvas')
    return !!(c.getContext('webgl2') ?? c.getContext('webgl'))
  } catch {
    return false
  }
}

async function init() {
  if (!host.value || !canvas.value) return
  if (!webglAvailable()) {
    failed.value = true
    return
  }
  let THREE: Three
  let RoomEnvironment: typeof import('three/examples/jsm/environments/RoomEnvironment.js').RoomEnvironment
  try {
    ;[THREE, { RoomEnvironment }] = await Promise.all([
      import('three'),
      import('three/examples/jsm/environments/RoomEnvironment.js'),
    ])
  } catch {
    failed.value = true
    return
  }
  if (disposed || !host.value || !canvas.value) return

  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  const el = host.value

  const renderer = new THREE.WebGLRenderer({
    canvas: canvas.value,
    antialias: true,
    alpha: true,
    powerPreference: 'high-performance',
  })
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2))
  renderer.toneMapping = THREE.ACESFilmicToneMapping
  renderer.toneMappingExposure = 1.05

  const scene = new THREE.Scene()
  const pmrem = new THREE.PMREMGenerator(renderer)
  const envTexture = pmrem.fromScene(new RoomEnvironment(), 0.04).texture
  scene.environment = envTexture

  const camera = new THREE.PerspectiveCamera(34, 1, 0.1, 100)
  const key = new THREE.DirectionalLight(0xffe0b8, 2.4)
  key.position.set(4, 6, 6)
  scene.add(key, new THREE.AmbientLight(0xffead0, 0.35))

  // --- Para geometrisi (ortak) -------------------------------------------------------------
  const bodyGeo = new THREE.CylinderGeometry(1, 1, 0.18, 72).rotateX(Math.PI / 2)
  const faceGeo = new THREE.CylinderGeometry(0.8, 0.8, 0.2, 72).rotateX(Math.PI / 2)
  const rimGeo = new THREE.TorusGeometry(0.9, 0.055, 14, 72)
  const starShape = new THREE.Shape()
  for (let i = 0; i < 10; i++) {
    const a = (i * Math.PI) / 5 - Math.PI / 2
    const r = i % 2 === 0 ? 0.5 : 0.21
    const x = Math.cos(a) * r
    const y = Math.sin(a) * r
    if (i === 0) starShape.moveTo(x, y)
    else starShape.lineTo(x, y)
  }
  starShape.closePath()
  const starGeo = new THREE.ExtrudeGeometry(starShape, {
    depth: 0.03,
    bevelEnabled: true,
    bevelSize: 0.012,
    bevelThickness: 0.012,
    bevelSegments: 2,
  })

  const GOLD = new THREE.Color(0xf2b134)
  const GOLD_FACE = new THREE.Color(0xf7c85a)
  const PALE = new THREE.Color(0xb59d7e)
  const PALE_FACE = new THREE.Color(0xcdb99b)

  const allMats: import('three').Material[] = []
  const makeMat = (color: import('three').Color) => {
    const m = new THREE.MeshStandardMaterial({
      color,
      metalness: 0.92,
      roughness: 0.3,
      envMapIntensity: 1.5,
      emissive: new THREE.Color(0xb86a00),
      emissiveIntensity: 0.22,
    })
    allMats.push(m)
    return m
  }

  function makeCoin(): { group: import('three').Group; mats: import('three').MeshStandardMaterial[] } {
    const rimMat = makeMat(GOLD)
    const faceMat = makeMat(GOLD_FACE)
    const starMat = makeMat(GOLD)
    const group = new THREE.Group()
    group.add(new THREE.Mesh(bodyGeo, rimMat), new THREE.Mesh(faceGeo, faceMat))
    for (const side of [1, -1]) {
      const rim = new THREE.Mesh(rimGeo, rimMat)
      rim.position.z = 0.09 * side
      const star = new THREE.Mesh(starGeo, starMat)
      star.position.z = 0.1 * side
      if (side === -1) star.rotation.y = Math.PI
      group.add(rim, star)
    }
    return { group, mats: [rimMat, faceMat, starMat] }
  }

  // --- Sahne düzeni ------------------------------------------------------------------------
  const root = new THREE.Group()
  scene.add(root)

  const centre = makeCoin()
  centre.group.scale.setScalar(1.55)
  root.add(centre.group)

  const ring = new THREE.Group()
  ring.rotation.set(0.28, 0, 0.1)
  root.add(ring)

  const orbitLine = new THREE.Mesh(
    new THREE.TorusGeometry(3.05, 0.014, 8, 160).rotateX(Math.PI / 2),
    new THREE.MeshBasicMaterial({ color: 0x1f9d6b, transparent: true, opacity: 0.35 }),
  )
  ring.add(orbitLine)
  allMats.push(orbitLine.material as import('three').Material)

  // Yıldız tozu: derinlik hissi için küçük altın noktalar.
  const dustCount = 70
  const dustPos = new Float32Array(dustCount * 3)
  for (let i = 0; i < dustCount; i++) {
    const r = 3.4 + Math.random() * 2.2
    const th = Math.random() * Math.PI * 2
    const ph = Math.acos(2 * Math.random() - 1)
    dustPos[i * 3] = r * Math.sin(ph) * Math.cos(th)
    dustPos[i * 3 + 1] = (r * Math.cos(ph)) * 0.7
    dustPos[i * 3 + 2] = r * Math.sin(ph) * Math.sin(th)
  }
  const dustGeo = new THREE.BufferGeometry()
  dustGeo.setAttribute('position', new THREE.BufferAttribute(dustPos, 3))
  const dust = new THREE.Points(
    dustGeo,
    new THREE.PointsMaterial({ color: 0xf5a623, size: 0.06, transparent: true, opacity: 0.8, sizeAttenuation: true }),
  )
  root.add(dust)
  allMats.push(dust.material as import('three').Material)

  const orbit: OrbitCoin[] = []

  function reconcile() {
    const wanted = Math.max(0, Math.min(12, props.coins))
    const alive = orbit.filter((c) => c.targetScale > 0)
    while (alive.length < wanted) {
      const { group, mats } = makeCoin()
      group.scale.setScalar(0.0001)
      ring.add(group)
      const coin: OrbitCoin = {
        group,
        mats,
        scale: 0,
        targetScale: 1,
        fill: 1,
        targetFill: 1,
        angle: alive.length ? alive[alive.length - 1]!.slot : 0,
        slot: 0,
        phase: Math.random() * Math.PI * 2,
      }
      orbit.push(coin)
      alive.push(coin)
    }
    while (alive.length > wanted) alive.pop()!.targetScale = 0
    alive.forEach((c, i) => {
      c.slot = (i * Math.PI * 2) / Math.max(1, wanted)
      c.targetScale = 1
      const filled = props.filled < 0 ? wanted : props.filled
      c.targetFill = i < filled ? 1 : 0
    })
  }
  reconcile()
  orbit.forEach((c) => (c.fill = c.targetFill))

  // --- Boyut ve kamera ---------------------------------------------------------------------
  function resize() {
    const w = Math.max(1, el.clientWidth)
    const h = Math.max(1, el.clientHeight)
    renderer.setSize(w, h, false)
    camera.aspect = w / h
    const dist = 10.6 / Math.min(1, Math.max(0.55, camera.aspect / 1.15)) ** 0.7
    camera.position.set(0, 1.5, dist)
    camera.lookAt(0, 0, 0)
    camera.updateProjectionMatrix()
  }
  const ro = new ResizeObserver(() => {
    resize()
    if (reduced) frame(0)
  })
  ro.observe(el)
  resize()

  // --- Etkileşim ---------------------------------------------------------------------------
  let tx = 0
  let ty = 0
  const onPointer = (e: PointerEvent) => {
    const r = el.getBoundingClientRect()
    tx = ((e.clientX - r.left) / r.width - 0.5) * 0.6
    ty = ((e.clientY - r.top) / r.height - 0.5) * 0.3
  }
  if (!reduced) window.addEventListener('pointermove', onPointer, { passive: true })

  // --- Çizim döngüsü -----------------------------------------------------------------------
  const tmp = new THREE.Color()
  let time = 0
  let last = performance.now()
  let raf = 0
  let visible = true

  function frame(dt: number) {
    time += dt
    root.rotation.y += (tx - root.rotation.y) * 0.04
    root.rotation.x += (ty - root.rotation.x) * 0.04

    centre.group.rotation.y = time * 0.55
    centre.group.position.y = Math.sin(time * 1.1) * 0.12
    ring.rotation.y = time * 0.22
    dust.rotation.y = -time * 0.05

    for (let i = orbit.length - 1; i >= 0; i--) {
      const c = orbit[i]!
      c.scale += (c.targetScale - c.scale) * Math.min(1, dt * 5)
      c.fill += (c.targetFill - c.fill) * Math.min(1, dt * 5)
      let d = c.slot - c.angle
      d = Math.atan2(Math.sin(d), Math.cos(d))
      c.angle += d * Math.min(1, dt * 4)

      const R = 3.05
      c.group.position.set(Math.cos(c.angle) * R, Math.sin(time * 1.3 + c.phase) * 0.16, Math.sin(c.angle) * R)
      c.group.rotation.y = Math.PI / 2 - c.angle + Math.sin(time * 0.8 + c.phase) * 0.18
      c.group.scale.setScalar(Math.max(0.0001, c.scale * 0.62))

      const [rim, face, star] = c.mats
      tmp.lerpColors(PALE, GOLD, c.fill)
      rim!.color.copy(tmp)
      star!.color.copy(tmp)
      face!.color.lerpColors(PALE_FACE, GOLD_FACE, c.fill)
      for (const m of c.mats) {
        m.metalness = 0.25 + 0.67 * c.fill
        m.roughness = 0.62 - 0.32 * c.fill
        m.transparent = c.fill < 0.98
        m.opacity = 0.78 + 0.22 * c.fill
        m.emissiveIntensity = 0.03 + 0.19 * c.fill
      }

      if (c.targetScale === 0 && c.scale < 0.01) {
        ring.remove(c.group)
        c.mats.forEach((m) => {
          allMats.splice(allMats.indexOf(m), 1)
          m.dispose()
        })
        orbit.splice(i, 1)
      }
    }
    renderer.render(scene, camera)
  }

  function loop(now: number) {
    raf = requestAnimationFrame(loop)
    const dt = Math.min(0.05, (now - last) / 1000)
    last = now
    if (visible && !document.hidden) frame(dt)
  }

  const io = new IntersectionObserver(([entry]) => (visible = !!entry?.isIntersecting), { threshold: 0.01 })
  io.observe(el)

  if (reduced) {
    frame(0)
  } else {
    raf = requestAnimationFrame(loop)
  }
  ready.value = true

  sync = () => {
    reconcile()
    if (reduced) {
      // Hareketsiz modda hedef değerlere hemen geç ve tek kare çiz.
      for (let i = orbit.length - 1; i >= 0; i--) {
        const c = orbit[i]!
        c.scale = c.targetScale
        c.fill = c.targetFill
        c.angle = c.slot
      }
      frame(0)
    }
  }

  cleanup = () => {
    cancelAnimationFrame(raf)
    io.disconnect()
    ro.disconnect()
    window.removeEventListener('pointermove', onPointer)
    ;[bodyGeo, faceGeo, rimGeo, starGeo, dustGeo, orbitLine.geometry].forEach((g) => g.dispose())
    allMats.forEach((m) => m.dispose())
    envTexture.dispose()
    pmrem.dispose()
    renderer.dispose()
    renderer.forceContextLoss()
  }
}

onMounted(() => void init())
watch(
  () => [props.coins, props.filled],
  () => sync?.(),
)
onBeforeUnmount(() => {
  disposed = true
  cleanup?.()
  cleanup = null
  sync = null
})
</script>

<template>
  <div ref="host" class="relative size-full" role="img" :aria-label="label">
    <canvas
      v-show="!failed"
      ref="canvas"
      class="absolute inset-0 size-full transition-opacity duration-700"
      :class="ready ? 'opacity-100' : 'opacity-0'"
      aria-hidden="true"
    />
    <!-- WebGL yoksa veya yüklenemezse: durağan çizim. -->
    <svg
      v-if="failed"
      viewBox="0 0 200 200"
      class="absolute inset-0 m-auto size-3/4 max-h-full max-w-full"
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="coinfill" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stop-color="#f7c85a" />
          <stop offset="1" stop-color="#e39a12" />
        </linearGradient>
      </defs>
      <g class="float">
        <circle cx="100" cy="100" r="62" fill="url(#coinfill)" stroke="#c98a10" stroke-width="4" />
        <circle cx="100" cy="100" r="48" fill="none" stroke="#fdeec3" stroke-width="3" />
        <path
          d="m100 68 8.5 17.6 19.3 2.6-14 13.6 3.4 19.2L100 112l-17.2 8.9 3.4-19.2-14-13.6 19.3-2.6Z"
          fill="#fdeec3"
        />
      </g>
    </svg>
  </div>
</template>
