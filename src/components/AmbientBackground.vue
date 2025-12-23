/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  imageUrl?: string
  color?: string
  active?: boolean
}>()

const loaded = ref(false)
const isVisible = ref(true)
const containerRef = ref<HTMLElement | null>(null)

let observer: IntersectionObserver | null = null

onMounted(() => {
  if (props.imageUrl) {
    const img = new Image()
    img.onload = () => loaded.value = true
    img.src = props.imageUrl
  }
  
  if (containerRef.value) {
    observer = new IntersectionObserver(
      ([entry]) => isVisible.value = entry.isIntersecting,
      { threshold: 0 }
    )
    observer.observe(containerRef.value)
  }
})

onUnmounted(() => observer?.disconnect())

watch(() => props.imageUrl, (url) => {
  if (url) {
    loaded.value = false
    const img = new Image()
    img.onload = () => loaded.value = true
    img.src = url
  }
})
</script>

<template>
  <div 
    ref="containerRef"
    class="ambient-bg"
    :class="{ active: active !== false, loaded, paused: !isVisible }"
    :style="{
      '--bg-image': imageUrl ? `url(${imageUrl})` : 'none',
      '--bg-color': color || 'var(--cyber-purple, #b026ff)'
    }"
  >
    <div class="overlay" />
  </div>
</template>

<style scoped>
.ambient-bg {
  --veil: rgba(0, 0, 0, 0.6);
  --speed: 0.66s;
  position: absolute;
  inset: 0;
  opacity: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: -1;
  
  background-image: 
    linear-gradient(180deg, transparent 50%, var(--void, #000) 85%),
    linear-gradient(0deg, var(--veil) 0%, var(--veil) 100%),
    linear-gradient(0deg, var(--bg-color) 0%, var(--bg-color) 100%);
  background-position: center;
  background-size: 120%;
  
  filter: blur(40px) saturate(1.5);
  transform: scale(1.2);
  transition: opacity calc(var(--speed) * 2) ease-out, background-size var(--speed) ease-in;
}

.ambient-bg.loaded {
  background-image: 
    linear-gradient(180deg, transparent 50%, var(--void, #000) 85%),
    linear-gradient(0deg, var(--veil) 0%, var(--veil) 100%),
    var(--bg-image);
}

.ambient-bg.active {
  opacity: 1;
  background-size: 100%;
  transition: opacity calc(var(--speed) / 2) ease-in;
}

.overlay {
  position: absolute;
  inset: 0;
  opacity: 0;
  background-image: var(--bg-image);
  background-position: 100% 100%;
  background-size: 250%;
  filter: brightness(1.3) saturate(0);
  mix-blend-mode: overlay;
  animation: shift-bg 60s infinite linear alternate;
  animation-play-state: paused;
  transition: opacity var(--speed) ease-in;
}

.active .overlay {
  opacity: 0.3;
  animation-play-state: running;
  transition-delay: calc(var(--speed) * 2);
}

.paused .overlay {
  animation-play-state: paused;
  opacity: 0;
}

@keyframes shift-bg {
  0% { background-position: 0% 50%; background-size: 250%; }
  25% { background-position: 60% 20%; background-size: 300%; }
  50% { background-position: 100% 50%; background-size: 320%; }
  75% { background-position: 40% 100%; background-size: 220%; }
  100% { background-position: 20% 50%; background-size: 300%; }
}
</style>