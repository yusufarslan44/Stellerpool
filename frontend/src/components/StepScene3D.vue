<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import StepArt from './StepArt.vue'
import type { BufferGeometry, Group, Material, MeshStandardMaterial } from 'three'

const props = defineProps<{ step: number; reduced: boolean; paused: boolean; previewDuration: number }>()
const host = ref<HTMLDivElement | null>(null)
const canvas = ref<HTMLCanvasElement | null>(null)
const ready = ref(false)
const failed = ref(false)
let disposed = false
let cleanup: (() => void) | undefined
let sync: ((stepChanged: boolean) => void) | undefined
let lazyObserver: IntersectionObserver | undefined

async function init() {
  if (!host.value || !canvas.value || disposed) return
  try {
    const [T, { RoomEnvironment }] = await Promise.all([
      import('three'),
      import('three/examples/jsm/environments/RoomEnvironment.js'),
    ])
    if (disposed || !host.value || !canvas.value) return
    const el = host.value
    const renderer = new T.WebGLRenderer({ canvas: canvas.value, alpha: true, antialias: true, powerPreference: 'low-power' })
    // Release partially initialized resources too, including a failed WebGL context.
    const releases: (() => void)[] = [() => { renderer.dispose(); renderer.forceContextLoss() }]
    cleanup = () => { releases.reverse().forEach((release) => release()); cleanup = undefined }
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.75))
    renderer.toneMapping = T.ACESFilmicToneMapping
    renderer.toneMappingExposure = 1.25
    const scene = new T.Scene()
    const pmrem = new T.PMREMGenerator(renderer)
    releases.push(() => pmrem.dispose())
    const room = new RoomEnvironment()
    const environment = pmrem.fromScene(room, .04)
    room.dispose()
    scene.environment = environment.texture
    releases.push(() => environment.dispose())
    const camera = new T.OrthographicCamera(-4.3, 4.3, 3, -3, .1, 60)
    camera.position.set(0, 4.2, 11)
    camera.lookAt(0, .25, 0)
    scene.add(new T.AmbientLight(0xfff8de, 1.5))
    const key = new T.DirectionalLight(0xffecc5, 4)
    key.position.set(-3, 7, 6)
    const rim = new T.DirectionalLight(0xdbffe4, 2.5)
    rim.position.set(5, 2, -3)
    scene.add(key, rim)
    const geometries = new Set<BufferGeometry>()
    const materials = new Set<Material>()
    releases.push(() => { geometries.forEach(g => g.dispose()); materials.forEach(m => m.dispose()) })
    function material(color: number, metalness = .1, roughness = .3) {
      const mat = new T.MeshStandardMaterial({ color, metalness, roughness })
      materials.add(mat)
      return mat
    }
    const green = material(0x137552, .48, .24)
    const dark = material(0x124b38, .4, .3)
    const mint = material(0xb7d8a2, .16, .32)
    const ivory = material(0xfff1d0, .1, .27)
    const gold = material(0xe7a523, .78, .23)
    const goldLight = material(0xffd36c, .62, .23)
    const root = new T.Group()
    root.position.y = .18
    scene.add(root)
    function mesh(geo: BufferGeometry, mat: Material, parent: Group, x = 0, y = 0, z = 0) {
      geometries.add(geo)
      const object = new T.Mesh(geo, mat)
      object.position.set(x, y, z)
      parent.add(object)
      return object
    }
    function block(w: number, h: number, depth: number, radius: number, mat: MeshStandardMaterial, parent: Group, x = 0, y = 0, z = 0) {
      const s = new T.Shape()
      s.moveTo(-w / 2 + radius, -h / 2)
      s.lineTo(w / 2 - radius, -h / 2)
      s.quadraticCurveTo(w / 2, -h / 2, w / 2, -h / 2 + radius)
      s.lineTo(w / 2, h / 2 - radius)
      s.quadraticCurveTo(w / 2, h / 2, w / 2 - radius, h / 2)
      s.lineTo(-w / 2 + radius, h / 2)
      s.quadraticCurveTo(-w / 2, h / 2, -w / 2, h / 2 - radius)
      s.lineTo(-w / 2, -h / 2 + radius)
      s.quadraticCurveTo(-w / 2, -h / 2, -w / 2 + radius, -h / 2)
      return mesh(new T.ExtrudeGeometry(s, { depth, bevelEnabled: true, bevelSize: .045, bevelThickness: .045, bevelSegments: 3, steps: 1, curveSegments: 12 }).translate(0, 0, -depth / 2), mat, parent, x, y, z)
    }
    function tube(points: number[][], radius: number, mat: MeshStandardMaterial, parent: Group) {
      return mesh(new T.TubeGeometry(new T.CatmullRomCurve3(points.map(p => new T.Vector3(p[0], p[1], p[2]))), 36, radius, 10, false), mat, parent)
    }
    function check(parent: Group, x = 0, y = 0, z = .3, size = 1) {
      const g = new T.Group()
      tube([[-.38, .02, 0], [-.12, -.22, 0], [.4, .35, 0]], .073, ivory, g)
      g.position.set(x, y, z)
      g.scale.setScalar(size)
      parent.add(g)
      return g
    }
    const coinBody = new T.CylinderGeometry(.51, .51, .14, 48).rotateX(Math.PI / 2)
    const coinFace = new T.CylinderGeometry(.41, .41, .158, 48).rotateX(Math.PI / 2)
    const coinRim = new T.TorusGeometry(.455, .022, 8, 48)
    const star = new T.Shape()
    for (let i = 0; i < 10; i++) {
      const a = i * Math.PI / 5 + Math.PI / 2
      const r = i % 2 ? .12 : .255
      if (i === 0) star.moveTo(Math.cos(a) * r, Math.sin(a) * r)
      else star.lineTo(Math.cos(a) * r, Math.sin(a) * r)
    }
    star.closePath()
    const starGeo = new T.ExtrudeGeometry(star, { depth: .025, bevelEnabled: true, bevelSize: .009, bevelThickness: .009, bevelSegments: 2 })
    function coin(parent: Group, x: number, y: number, z: number, scale = 1) {
      const g = new T.Group()
      mesh(coinBody, gold, g)
      mesh(coinFace, goldLight, g)
      mesh(coinRim, ivory, g, 0, 0, .085)
      mesh(starGeo, gold, g, 0, 0, .083)
      g.position.set(x, y, z)
      g.scale.setScalar(scale)
      g.rotation.set(-.12, -.2, -.08)
      parent.add(g)
      return g
    }
    function pedestal(parent: Group, radius = 1.72) {
      mesh(new T.CylinderGeometry(radius, radius * 1.04, .22, 80), dark, parent, 0, -.99, 0)
      mesh(new T.CylinderGeometry(radius * .99, radius * .99, .08, 80), green, parent, 0, -.84, 0)
      mesh(new T.TorusGeometry(radius * .94, .024, 8, 80).rotateX(Math.PI / 2), goldLight, parent, 0, -.785, 0)
    }
    // Soft contact shadow without an extra shadow render pass.
    const shadowCanvas = document.createElement('canvas')
    shadowCanvas.width = shadowCanvas.height = 128
    const ctx = shadowCanvas.getContext('2d')!
    const gradient = ctx.createRadialGradient(64, 64, 4, 64, 64, 64)
    gradient.addColorStop(0, 'rgba(26,60,28,.3)')
    gradient.addColorStop(.5, 'rgba(26,60,28,.14)')
    gradient.addColorStop(1, 'rgba(26,60,28,0)')
    ctx.fillStyle = gradient
    ctx.fillRect(0, 0, 128, 128)
    const shadowTexture = new T.CanvasTexture(shadowCanvas)
    releases.push(() => shadowTexture.dispose())
    const shadowMat = new T.MeshBasicMaterial({ map: shadowTexture, transparent: true, depthWrite: false })
    materials.add(shadowMat)
    const shadow = mesh(new T.PlaneGeometry(6, 4), shadowMat, root, 0, -1.14, 0)
    shadow.rotation.x = -Math.PI / 2
    const stages: { group: Group; animate: (time: number) => void }[] = []
    function stage(animate: (time: number) => void = () => {}) {
      const group = new T.Group()
      root.add(group)
      stages.push({ group, animate })
      return group
    }
    // 01: four members around the shared pool.
    const pool = stage(t => {
      members.forEach((m, i) => { m.position.y = .25 + Math.sin(t * 1.3 + i * 1.5) * .1; m.rotation.y = -.2 + Math.sin(t + i) * .16 })
      poolCoin.position.y = .4 + Math.sin(t * 1.2) * .08
    })
    pedestal(pool)
    mesh(new T.CylinderGeometry(.92, 1.04, .48, 64), ivory, pool, 0, -.5, 0)
    mesh(new T.CylinderGeometry(.8, .8, .06, 64), mint, pool, 0, -.23, 0)
    const poolCoin = coin(pool, 0, .4, .08, 1.35)
    const members = [-2.15, -1.05, 1.05, 2.15].map((x, i) => {
      const member = new T.Group()
      const z = i === 0 || i === 3 ? -.12 : -.8
      mesh(new T.SphereGeometry(.18, 24, 16), i % 2 ? goldLight : mint, member, 0, .21, 0)
      mesh(new T.CapsuleGeometry(.17, .2, 6, 16), i % 2 ? gold : green, member, 0, -.23, 0)
      member.position.set(x, .25, z)
      pool.add(member)
      tube([[x, -.48, z], [x * .7, -.64, z * .7], [0, -.68, 0]], .018, goldLight, pool)
      return member
    })
    // 02: onaylanan grup kuralları.
    const agreement = stage(t => { shieldGroup.position.y = .25 + Math.sin(t * 1.15) * .1; shieldGroup.rotation.y = -.18 + Math.sin(t * .7) * .13 })
    pedestal(agreement)
    const shieldGroup = new T.Group()
    agreement.add(shieldGroup)
    shieldGroup.position.set(0, .25, .15)
    const outline = new T.Shape()
    outline.moveTo(0, 1.28)
    outline.bezierCurveTo(.35, 1.02, .78, 1, .92, .98)
    outline.lineTo(.87, .05)
    outline.bezierCurveTo(.81, -.43, .35, -.78, 0, -.96)
    outline.bezierCurveTo(-.35, -.78, -.81, -.43, -.87, .05)
    outline.lineTo(-.92, .98)
    outline.bezierCurveTo(-.78, 1, -.35, 1.02, 0, 1.28)
    const shieldGeo = new T.ExtrudeGeometry(outline, { depth: .2, bevelEnabled: true, bevelSize: .07, bevelThickness: .07, bevelSegments: 4, curveSegments: 20 })
    mesh(shieldGeo, goldLight, shieldGroup)
    const shieldInset = mesh(shieldGeo, green, shieldGroup, 0, .03, .17)
    shieldInset.scale.set(.88, .88, .6)
    check(shieldGroup, 0, .18, .41, 1.1)
    // 03: lock and four approval markers.
    const rules = stage(t => { lock.rotation.y = -.2 + Math.sin(t * .8) * .13; lock.position.y = .15 + Math.sin(t * 1.2) * .08 })
    pedestal(rules)
    const lock = new T.Group()
    rules.add(lock)
    block(1.65, 1.24, .52, .23, green, lock, 0, -.05, .05)
    tube([[-.54, .45, 0], [-.54, 1.1, 0], [0, 1.53, 0], [.54, 1.1, 0], [.54, .45, 0]], .13, goldLight, lock)
    mesh(new T.SphereGeometry(.125, 24, 16), goldLight, lock, 0, .05, .34)
    block(.08, .24, .035, .02, goldLight, lock, 0, -.1, .36)
    for (let i = 0; i < 4; i++) {
      const marker = new T.Group()
      marker.position.set((i - 1.5) * .79, -.58, 1.02)
      rules.add(marker)
      mesh(new T.CylinderGeometry(.24, .24, .08, 32).rotateX(Math.PI / 2), mint, marker)
      check(marker, 0, 0, .09, .34).children.forEach(c => { if (c instanceof T.Mesh) c.material = dark })
    }
    // 04: contributions descend into the contract's vessel.
    const payments = stage(t => {
      falling.forEach((c, i) => {
        const p = (t * .23 + i / 4) % 1
        c.position.set((i - 1.5) * .61 * (1 - p * .62), 1.65 - p * 2.2, .04)
        c.rotation.y = -.2 + Math.sin(t * .7 + i) * .3
        c.scale.setScalar(.56 * (p > .83 ? 1 - (p - .83) / .17 : 1))
      })
    })
    pedestal(payments)
    const profile = [new T.Vector2(0, 0), new T.Vector2(.68, 0), new T.Vector2(.93, .15), new T.Vector2(1.04, .86), new T.Vector2(.9, .9), new T.Vector2(.79, .18), new T.Vector2(0, .18)]
    mesh(new T.LatheGeometry(profile, 72), green, payments, 0, -.8, 0)
    mesh(new T.TorusGeometry(.973, .065, 12, 72).rotateX(Math.PI / 2), goldLight, payments, 0, .075, 0)
    for (let i = 0; i < 3; i++) coin(payments, (i - 1) * .4, -.43, .05, .48).rotation.set(-1.2, 0, i * .5)
    const falling = Array.from({ length: 4 }, () => coin(payments, 0, 1, 0, .56))
    // 05: approved receipt, dimensional storefront, and direct transfer.
    const payout = stage(t => {
      transfer.position.set(-1.65 + ((t * .3) % 1) * 2.3, -.32 + Math.sin(((t * .3) % 1) * Math.PI) * .7, 1.05)
      transfer.rotation.y = t * .65
      receipt.rotation.z = -.13 + Math.sin(t * 1.1) * .035
    })
    pedestal(payout, 1.9)
    const shop = new T.Group()
    shop.position.set(.58, -.12, -.02)
    shop.rotation.y = -.2
    payout.add(shop)
    block(1.7, 1.45, .82, .1, ivory, shop)
    block(.48, .88, .07, .045, dark, shop, .37, -.28, .46)
    block(.63, .57, .07, .04, mint, shop, -.43, -.03, .46)
    block(.035, .57, .09, .01, ivory, shop, -.43, -.03, .5)
    block(1.83, .18, .98, .04, dark, shop, 0, .78, .05)
    for (let i = 0; i < 6; i++) {
      block(.29, .55, .09, .025, i % 2 ? ivory : green, shop, -.755 + i * .302, .57, .62).rotation.x = -.67
      mesh(new T.SphereGeometry(.147, 16, 12), i % 2 ? ivory : green, shop, -.755 + i * .302, .35, .8).scale.set(1, .7, .55)
    }
    const receipt = new T.Group()
    receipt.position.set(-1.23, .45, .13)
    receipt.rotation.set(-.05, .1, -.13)
    payout.add(receipt)
    block(.89, 1.28, .1, .055, ivory, receipt)
    for (let i = 0; i < 3; i++) block(i === 2 ? .28 : .53, .036, .015, .01, mint, receipt, i === 2 ? -.12 : 0, .4 - i * .14, .08)
    mesh(new T.CylinderGeometry(.22, .22, .05, 32).rotateX(Math.PI / 2), green, receipt, 0, -.25, .09)
    check(receipt, 0, -.25, .14, .34)
    const transfer = coin(payout, -1.65, -.3, 1.05, .53)

    let visible = true
    let raf = 0
    let time = 0
    let last = performance.now()
    let transitionUntil = 0
    let targetX = 0
    let targetY = 0
    const scales: number[] = stages.map((_, index) => index === props.step ? 1 : 0)
    function draw(dt: number, instant = false) {
      time += dt
      root.rotation.x += ((props.reduced ? 0 : targetY) - root.rotation.x) * (instant ? 1 : .06)
      root.rotation.y += ((props.reduced ? 0 : targetX) - root.rotation.y) * (instant ? 1 : .06)
      stages.forEach((stage, index) => {
        const target = index === props.step ? 1 : 0
        scales[index] = instant ? target : scales[index]! + (target - scales[index]!) * Math.min(1, dt * 9)
        const scale = scales[index]!
        stage.group.visible = scale > .015
        stage.group.scale.setScalar(Math.max(.001, scale))
        stage.group.position.y = (1 - scale) * -.45
        stage.group.rotation.y = (1 - scale) * -.35
        if (stage.group.visible) stage.animate(props.reduced ? 0 : time)
      })
      renderer.render(scene, camera)
    }
    function loop(now: number) {
      raf = 0
      if (!visible || document.hidden) return
      draw(Math.min((now - last) / 1000, .04))
      last = now
      if (!props.reduced && (!props.paused || now < transitionUntil)) raf = requestAnimationFrame(loop)
    }
    function schedule() {
      if (disposed || !visible || document.hidden) return
      if (props.reduced) { draw(0, true); return }
      if (!raf) { last = performance.now(); raf = requestAnimationFrame(loop) }
    }
    function resize() {
      const width = Math.max(el.clientWidth, 1)
      const height = Math.max(el.clientHeight, 1)
      const aspect = width / height
      const halfHeight = Math.max(2.5, 3.25 / aspect)
      camera.left = -halfHeight * aspect
      camera.right = halfHeight * aspect
      camera.top = halfHeight
      camera.bottom = -halfHeight
      camera.updateProjectionMatrix()
      renderer.setSize(width, height, false)
      draw(0, true)
    }
    const ro = new ResizeObserver(resize)
    ro.observe(el)
    releases.push(() => ro.disconnect())
    const io = new IntersectionObserver(([entry]) => {
      visible = !!entry?.isIntersecting
      if (visible) schedule()
      else { cancelAnimationFrame(raf); raf = 0 }
    })
    io.observe(el)
    releases.push(() => io.disconnect())
    const pointer = (event: PointerEvent) => {
      if (event.pointerType !== 'mouse' || props.reduced || props.paused) return
      const rect = el.getBoundingClientRect()
      targetX = ((event.clientX - rect.left) / rect.width - .5) * .3
      targetY = ((event.clientY - rect.top) / rect.height - .5) * .14
    }
    const resetPointer = () => { targetX = targetY = 0 }
    const contextLost = (event: Event) => {
      event.preventDefault()
      ready.value = false
      failed.value = true
      sync = undefined
      cleanup?.()
    }
    el.addEventListener('pointermove', pointer, { passive: true })
    el.addEventListener('pointerleave', resetPointer)
    canvas.value.addEventListener('webglcontextlost', contextLost)
    const currentCanvas = canvas.value
    document.addEventListener('visibilitychange', schedule)
    releases.push(() => {
      el.removeEventListener('pointermove', pointer)
      el.removeEventListener('pointerleave', resetPointer)
      currentCanvas.removeEventListener('webglcontextlost', contextLost)
      document.removeEventListener('visibilitychange', schedule)
      cancelAnimationFrame(raf)
    })
    sync = (stepChanged) => {
      // Keep a manually selected scene moving for a full step; explicit pause still stops it.
      transitionUntil = stepChanged ? performance.now() + props.previewDuration : 0
      schedule()
    }
    resize()
    schedule()
    ready.value = true
  } catch {
    cleanup?.()
    sync = undefined
    failed.value = true
  }
}
watch(() => [props.step, props.reduced, props.paused], (values, previous) => sync?.(values[0] !== previous[0]))
onMounted(() => {
  lazyObserver = new IntersectionObserver(([entry]) => {
    if (entry?.isIntersecting) { lazyObserver?.disconnect(); void init() }
  }, { rootMargin: '180px' })
  if (host.value) lazyObserver.observe(host.value)
})
onBeforeUnmount(() => {
  disposed = true
  lazyObserver?.disconnect()
  cleanup?.()
  sync = undefined
})
</script>

<template>
  <div ref="host" class="step-scene" aria-hidden="true">
    <canvas v-show="!failed" ref="canvas" :class="{ ready }" />
    <div v-if="!ready || failed" class="step-scene-fallback" :data-paused="paused || reduced"><StepArt :kind="(step + 1) as 1 | 2 | 3 | 4 | 5" /></div>
  </div>
</template>

<style scoped>
.step-scene { position: absolute; inset: 14px 0 24px; }
canvas { width: 100%; height: 100%; display: block; opacity: 0; transition: opacity .5s; }
canvas.ready { opacity: 1; }
.step-scene-fallback { position: absolute; inset: 32px 20% 48px; display: grid; place-items: center; }
.step-scene-fallback :deep(svg) { max-height: 100%; }
.step-scene-fallback[data-paused="true"] :deep(*) { animation: none !important; }
</style>
