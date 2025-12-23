<script setup lang="ts">
import { gsap } from 'gsap'
import { onMounted, onBeforeUnmount, ref } from 'vue'

interface Props {
  targetSelector?: string
  hideDefaultCursor?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  targetSelector: '.cursor-target',
  hideDefaultCursor: true
})

const cursorRef = ref<HTMLElement>()
const isTargeting = ref(false)

let corners: HTMLElement[] = []
let setX: any
let setY: any
let cornerSetters: { x: any; y: any }[] = []

let mouseX = 0
let mouseY = 0
let activeTarget: Element | null = null
let rafId = 0

const R = 16
const defaultPos = [
  { x: 0, y: -R },
  { x: -R * 0.866, y: R * 0.5 },
  { x: R * 0.866, y: R * 0.5 }
]

const update = () => {
  setX(mouseX)
  setY(mouseY)

  if (activeTarget) {
    const rect = activeTarget.getBoundingClientRect()
    const cx = (rect.left + rect.right) / 2 - mouseX
    const cy = (rect.top + rect.bottom) / 2 - mouseY
    const rx = rect.width / 2 + 12
    const ry = rect.height / 2 + 12

    cornerSetters[0].x(cx)
    cornerSetters[0].y(cy - ry)
    cornerSetters[1].x(cx - rx * 0.866)
    cornerSetters[1].y(cy + ry * 0.5)
    cornerSetters[2].x(cx + rx * 0.866)
    cornerSetters[2].y(cy + ry * 0.5)
  }

  rafId = requestAnimationFrame(update)
}

const onMove = (e: MouseEvent) => {
  mouseX = e.clientX
  mouseY = e.clientY
}

const findTarget = (el: Element | null): Element | null => {
  while (el && el !== document.body) {
    if (el.matches(props.targetSelector)) return el
    el = el.parentElement
  }
  return null
}

const onOver = (e: MouseEvent) => {
  const target = findTarget(e.target as Element)
  
  if (target === activeTarget) return
  
  // Kill any running corner animations for smooth transition
  corners.forEach(c => gsap.killTweensOf(c))
  
  if (target) {
    activeTarget = target
    isTargeting.value = true
  } else {
    activeTarget = null
    isTargeting.value = false
    // Animate back to default
    for (let i = 0; i < 3; i++) {
      gsap.to(corners[i], { 
        x: defaultPos[i].x, 
        y: defaultPos[i].y, 
        duration: 0.15, 
        ease: 'power2.out'
      })
    }
  }
}

onMounted(() => {
  if (!cursorRef.value) return
  if (props.hideDefaultCursor) document.body.style.cursor = 'none'

  corners = Array.from(cursorRef.value.querySelectorAll<HTMLElement>('.c'))
  
  setX = gsap.quickSetter(cursorRef.value, 'x', 'px')
  setY = gsap.quickSetter(cursorRef.value, 'y', 'px')

  cornerSetters = corners.map(c => ({
    x: gsap.quickSetter(c, 'x', 'px'),
    y: gsap.quickSetter(c, 'y', 'px')
  }))

  gsap.set(cursorRef.value, { xPercent: -50, yPercent: -50 })
  corners.forEach((c, i) => gsap.set(c, { x: defaultPos[i].x, y: defaultPos[i].y }))

  rafId = requestAnimationFrame(update)
  window.addEventListener('mousemove', onMove, { passive: true })
  window.addEventListener('mouseover', onOver, { passive: true })
})

onBeforeUnmount(() => {
  cancelAnimationFrame(rafId)
  window.removeEventListener('mousemove', onMove)
  window.removeEventListener('mouseover', onOver)
  document.body.style.cursor = ''
})
</script>

<template>
  <div ref="cursorRef" class="cur" :class="{ t: isTargeting }">
    <div class="inner">
      <div class="c c0" />
      <div class="c c1" />
      <div class="c c2" />
    </div>
  </div>
</template>

<style scoped>
.cur {
  position: fixed;
  top: 0;
  left: 0;
  z-index: 9999;
  pointer-events: none;
  mix-blend-mode: difference;
}

.inner {
  animation: s 0.8s linear infinite;
}

.cur.t .inner {
  animation: none;
}

@keyframes s {
  to { transform: rotate(360deg); }
}

.c {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0;
  height: 0;
}

.c0 {
  border-left: 5px solid transparent;
  border-right: 5px solid transparent;
  border-bottom: 9px solid white;
}

.c1 {
  border-top: 5px solid transparent;
  border-bottom: 5px solid transparent;
  border-right: 9px solid white;
}

.c2 {
  border-top: 5px solid transparent;
  border-bottom: 5px solid transparent;
  border-left: 9px solid white;
}
</style>
