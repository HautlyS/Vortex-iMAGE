/**
 * Vue Component - 1 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: GlassSurface
 */

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { cn } from '../lib/utils'
import GlassSurface from './GlassSurface.vue'

export interface DockApp {
  id: string
  name: string
  icon: string
}

const props = withDefaults(defineProps<{
  apps: DockApp[]
  openApps?: string[]
  className?: string
  isMobile?: boolean
}>(), {
  openApps: () => [],
  className: '',
  isMobile: false
})

const emit = defineEmits<{
  (e: 'appClick', appId: string): void
}>()

const mouseX = ref<number | null>(null)
const currentScales = ref<number[]>([])
const currentPositions = ref<number[]>([])
const dockRef = ref<HTMLDivElement | null>(null)
const iconRefs = ref<(HTMLDivElement | null)[]>([])
const animationFrame = ref<number | undefined>(undefined)
const lastMouseMoveTime = ref(0)
const config = ref({ baseIconSize: 64, maxScale: 1.6, effectWidth: 240 })

watch(() => props.apps, () => {
  currentScales.value = props.apps.map(() => 1)
  iconRefs.value = new Array(props.apps.length).fill(null)
}, { immediate: true })

const getResponsiveConfig = () => {
  if (typeof window === 'undefined') {
    return { baseIconSize: 64, maxScale: 1.6, effectWidth: 240 }
  }
  const smallerDimension = Math.min(window.innerWidth, window.innerHeight)
  
  if (smallerDimension < 480) {
    return { baseIconSize: Math.max(40, smallerDimension * 0.08), maxScale: 1.4, effectWidth: smallerDimension * 0.4 }
  } else if (smallerDimension < 768) {
    return { baseIconSize: Math.max(48, smallerDimension * 0.07), maxScale: 1.5, effectWidth: smallerDimension * 0.35 }
  } else if (smallerDimension < 1024) {
    return { baseIconSize: Math.max(56, smallerDimension * 0.06), maxScale: 1.6, effectWidth: smallerDimension * 0.3 }
  }
  return { baseIconSize: Math.max(64, Math.min(80, smallerDimension * 0.05)), maxScale: 1.8, effectWidth: 300 }
}

const minScale = 1.0
const baseSpacing = computed(() => Math.max(4, config.value.baseIconSize * 0.08))
const padding = computed(() => Math.max(8, config.value.baseIconSize * 0.12))

const updateConfig = () => { config.value = getResponsiveConfig() }

onMounted(() => {
  updateConfig()
  window.addEventListener('resize', updateConfig)
})

onUnmounted(() => {
  window.removeEventListener('resize', updateConfig)
  if (animationFrame.value) cancelAnimationFrame(animationFrame.value)
})

const calculateTargetMagnification = (mousePosition: number | null) => {
  if (mousePosition === null) return props.apps.map(() => minScale)
  return props.apps.map((_, index) => {
    const normalIconCenter = (index * (config.value.baseIconSize + baseSpacing.value)) + (config.value.baseIconSize / 2)
    const minX = mousePosition - (config.value.effectWidth / 2)
    const maxX = mousePosition + (config.value.effectWidth / 2)
    if (normalIconCenter < minX || normalIconCenter > maxX) return minScale
    const theta = ((normalIconCenter - minX) / config.value.effectWidth) * 2 * Math.PI
    const cappedTheta = Math.min(Math.max(theta, 0), 2 * Math.PI)
    const scaleFactor = (1 - Math.cos(cappedTheta)) / 2
    return minScale + (scaleFactor * (config.value.maxScale - minScale))
  })
}

const calculatePositions = (scales: number[]) => {
  let currentX = 0
  return scales.map((scale) => {
    const scaledWidth = config.value.baseIconSize * scale
    const centerX = currentX + (scaledWidth / 2)
    currentX += scaledWidth + baseSpacing.value
    return centerX
  })
}

watch([() => props.apps, config], () => {
  const initialScales = props.apps.map(() => minScale)
  currentScales.value = initialScales 
  currentPositions.value = calculatePositions(initialScales)
}, { immediate: true })

const animateToTarget = () => {
  const targetScales = calculateTargetMagnification(mouseX.value)
  const targetPositions = calculatePositions(targetScales)
  const lerpFactor = mouseX.value !== null ? 0.2 : 0.12
  
  let scalesChanged = false
  currentScales.value = currentScales.value.map((currentScale, index) => {
    const diff = targetScales[index] - currentScale
    const newScale = currentScale + (diff * lerpFactor)
    if (Math.abs(newScale - targetScales[index]) > 0.002) scalesChanged = true
    return newScale
  })

  let positionsChanged = false
  currentPositions.value = currentPositions.value.map((currentPos, index) => {
    const diff = targetPositions[index] - currentPos
    const newPos = currentPos + (diff * lerpFactor)
    if (Math.abs(newPos - targetPositions[index]) > 0.1) positionsChanged = true
    return newPos
  })

  if (scalesChanged || positionsChanged || mouseX.value !== null) {
    animationFrame.value = requestAnimationFrame(animateToTarget)
  }
}

watch([mouseX, () => props.apps], () => {
  if (animationFrame.value) cancelAnimationFrame(animationFrame.value)
  animationFrame.value = requestAnimationFrame(animateToTarget)
})

const handleMouseMove = (e: MouseEvent) => {
  const now = performance.now()
  if (now - lastMouseMoveTime.value < 16) return
  lastMouseMoveTime.value = now
  if (dockRef.value) {
    const rect = dockRef.value.getBoundingClientRect()
    mouseX.value = e.clientX - rect.left - padding.value
  }
}

const handleMouseLeave = () => { mouseX.value = null }

const handleAppClick = (appId: string, index: number) => {
  const element = iconRefs.value[index]
  if (element) {
    const bounceHeight = Math.max(-8, -config.value.baseIconSize * 0.15)
    element.style.transition = 'transform 0.2s ease-out'
    element.style.transform = `translateY(${bounceHeight}px)`
    setTimeout(() => { element.style.transform = 'translateY(0px)' }, 200)
  }
  emit('appClick', appId)
}

const contentWidth = computed(() => {
  if (currentPositions.value.length > 0 && currentScales.value.length === currentPositions.value.length) {
    const lastIndex = currentPositions.value.length - 1
    return currentPositions.value[lastIndex] + (config.value.baseIconSize * currentScales.value[lastIndex] / 2)
  }
  return (props.apps.length * (config.value.baseIconSize + baseSpacing.value)) - baseSpacing.value
})

</script>

<template>
  <div 
    ref="dockRef"
    :class="cn('fixed z-50', className, props.isMobile ? 'bottom-6 left-1/2 -translate-x-1/2 scale-110' : 'bottom-4 left-1/2 -translate-x-1/2')"
    @mousemove="handleMouseMove"
    @mouseleave="handleMouseLeave"
  >
    <GlassSurface
      :width="contentWidth + 32"
      :height="config.baseIconSize + 24"
      :border-radius="Math.max(16, config.baseIconSize * 0.5)"
      :border-width="0.08"
    >
      <div class="flex items-center justify-center gap-2">
        <div
          v-for="(app, index) in apps"
          :key="app.id"
          :ref="(el) => { if (el) iconRefs[index] = el as HTMLDivElement }"
          class="cursor-pointer flex items-center justify-center group relative"
          :title="app.name"
          @click="handleAppClick(app.id, index)"
          :style="{
            width: `${config.baseIconSize * (currentScales[index] || 1)}px`,
            height: `${config.baseIconSize * (currentScales[index] || 1)}px`
          }"
        >
          <div 
            class="absolute bottom-full mb-2 px-2 py-1 bg-gray-900/90 text-white text-xs rounded-md opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none whitespace-nowrap"
            style="transform: translateX(-50%); left: 50%"
          >
            {{ app.name }}
          </div>
          
          <div class="w-full h-full flex items-center justify-center text-white" v-html="app.icon" />
          
          <div 
            v-if="openApps.includes(app.id)"
            class="absolute -bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-white/80"
          />
        </div>
      </div>
    </GlassSurface>
  </div>
</template>