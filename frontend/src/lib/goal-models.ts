import type * as Three from 'three'

export type SceneGoal = 'home' | 'car' | 'work'

/** Small, shared-material models for the landing page. No textures or remote assets. */
export function createGoalModels(T: typeof Three) {
  const geometries = new Set<Three.BufferGeometry>()
  const materials: Three.Material[] = []
  const mat = (color: number, metalness = .15, roughness = .3) => {
    const result = new T.MeshStandardMaterial({ color, metalness, roughness })
    materials.push(result)
    return result
  }
  const cream = mat(0xffedcb)
  const green = mat(0x1c6448, .4)
  const mint = mat(0x9fbd8c)
  const gold = mat(0xe8b851, .7, .24)
  const glass = mat(0x47675c, .65, .18)
  const tire = mat(0x24372c, .05, .75)
  const white = mat(0xfff8df, .25)
  function mesh(geometry: Three.BufferGeometry, material: Three.Material, parent: Three.Group, x = 0, y = 0, z = 0) {
    geometries.add(geometry)
    const object = new T.Mesh(geometry, material)
    object.position.set(x, y, z)
    parent.add(object)
    return object
  }
  function box(w: number, h: number, d: number, material: Three.Material, parent: Three.Group, x = 0, y = 0, z = 0) {
    const r = Math.min(.09, w / 3, h / 3)
    const shape = new T.Shape()
    shape.moveTo(-w/2+r,-h/2)
    shape.lineTo(w/2-r,-h/2); shape.quadraticCurveTo(w/2,-h/2,w/2,-h/2+r)
    shape.lineTo(w/2,h/2-r); shape.quadraticCurveTo(w/2,h/2,w/2-r,h/2)
    shape.lineTo(-w/2+r,h/2); shape.quadraticCurveTo(-w/2,h/2,-w/2,h/2-r)
    shape.lineTo(-w/2,-h/2+r); shape.quadraticCurveTo(-w/2,-h/2,-w/2+r,-h/2)
    return mesh(new T.ExtrudeGeometry(shape, { depth: d, bevelEnabled: true, bevelSize: .035, bevelThickness: .035, bevelSegments: 3, curveSegments: 8 }).translate(0,0,-d/2), material, parent, x,y,z)
  }
  const base = new T.Group()
  mesh(new T.CylinderGeometry(2.1,2.2,.26,80), green, base,0,-1.1)
  mesh(new T.CylinderGeometry(2.04,2.04,.07,80), mint, base,0,-.94)
  mesh(new T.TorusGeometry(1.98,.028,10,80).rotateX(Math.PI/2), gold,base,0,-.89)
  const home = new T.Group()
  home.rotation.y = -.38
  box(1.85,1.5,1.48,cream,home,0,-.08)
  const roofShape = new T.Shape()
  roofShape.moveTo(-1.12,0); roofShape.lineTo(0,.88); roofShape.lineTo(1.12,0); roofShape.closePath()
  mesh(new T.ExtrudeGeometry(roofShape,{depth:1.8,bevelEnabled:true,bevelSize:.05,bevelThickness:.04,bevelSegments:3}).translate(0,0,-.9),green,home,0,.7)
  box(.4,.7,.38,cream,home,.55,1.1,-.36)
  box(.49,.84,.06,green,home,.4,-.36,.78)
  mesh(new T.SphereGeometry(.035,12,8),gold,home,.54,-.37,.84)
  box(.6,.61,.055,glass,home,-.43,.03,.78)
  box(.045,.62,.065,cream,home,-.43,.03,.82)
  box(.62,.045,.065,cream,home,-.43,.03,.82)
  box(.7,.12,.35,white,home,.4,-.85,.8)
  for (const x of [-1.38,1.4]) {
    mesh(new T.CylinderGeometry(.055,.07,.42,12),gold,home,x,-.67,.05)
    mesh(new T.SphereGeometry(.3,24,16),mint,home,x,-.22,.05).scale.y = 1.4
  }
  const car = new T.Group()
  car.rotation.y = -.5
  box(2.65,.62,1.26,green,car,0,-.38)
  box(1.47,.65,1.16,mint,car,-.15,.21)
  box(1.27,.39,.03,glass,car,-.15,.26,.61)
  box(.065,.43,.07,green,car,-.08,.26,.64)
  box(1.27,.39,.03,glass,car,-.15,.26,-.61)
  box(.065,.43,.07,green,car,-.08,.26,-.64)
  for (const x of [-.85,.88]) for (const z of [-.66,.66]) {
    mesh(new T.CylinderGeometry(.34,.34,.17,36).rotateX(Math.PI/2),tire,car,x,-.6,z)
    mesh(new T.CylinderGeometry(.19,.19,.19,32).rotateX(Math.PI/2),gold,car,x,-.6,z)
    mesh(new T.CylinderGeometry(.09,.09,.2,24).rotateX(Math.PI/2),white,car,x,-.6,z)
  }
  for (const z of [-.42,.42]) {
    mesh(new T.SphereGeometry(.12,16,12),white,car,1.34,-.32,z).scale.x=.35
    mesh(new T.SphereGeometry(.085,16,12),gold,car,-1.36,-.32,z).scale.x=.35
  }
  box(.06,.1,1.08,gold,car,1.37,-.57)
  const work = new T.Group()
  work.rotation.y = -.35
  box(2.08,1.6,1.36,cream,work,0,-.08)
  box(.56,1.1,.05,green,work,.57,-.3,.72)
  box(.83,.73,.05,glass,work,-.41,-.07,.72)
  box(.04,.76,.06,cream,work,-.41,-.07,.76)
  box(2.23,.34,1.47,green,work,0,.91)
  for (let i=0;i<7;i++) {
    box(.31,.65,.09,i%2 ? cream:mint,work,-.96+i*.32,.52,.94).rotation.x=-.72
    mesh(new T.SphereGeometry(.155,16,12),i%2?cream:mint,work,-.96+i*.32,.28,1.14).scale.set(1,.65,.55)
  }
  box(.73,.11,.04,gold,work,0,.93,.76)
  return {
    base,
    models: { home, car, work },
    dispose() { geometries.forEach(g=>g.dispose()); materials.forEach(m=>m.dispose()) },
  }
}
