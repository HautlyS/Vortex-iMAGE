/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Vortex } from './ui/vortex'

const emit = defineEmits<{ complete: [] }>()
const progress = ref(0)

let interval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  let step = 0
  interval = setInterval(() => {
    step++
    progress.value = Math.min((step / 20) * 100, 100)
    if (step >= 25) {
      if (interval) clearInterval(interval)
      emit('complete')
    }
  }, 60)
})

onUnmounted(() => {
  if (interval) clearInterval(interval)
})
</script>

<template>
  <Vortex
    container-class="loader-vortex"
    :particle-count="700"
    :base-hue="210"
    :range-y="800"
    :base-speed="0.1"
    :range-speed="1.5"
    :base-radius="2"
    :range-radius="3"
    background-color="#000000"
  >
    <div class="loader-content">
      <!-- App icon -->
      <div class="app-icon-wrapper">
        <div class="app-icon">
          <svg viewBox="0 0 24 24" fill="none">
            <rect x="3" y="3" width="18" height="18" rx="4" fill="url(#iconGradient)" />
            <path d="M8 12L11 15L16 9" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
            <defs>
              <linearGradient id="iconGradient" x1="3" y1="3" x2="21" y2="21">
                <stop stop-color="#007AFF" />
                <stop offset="1" stop-color="#5856D6" />
              </linearGradient>
            </defs>
          </svg>
        </div>
      </div>
      
      <!-- Progress bar -->
      <div class="progress-wrapper">
        <div class="progress-track">
          <div class="progress-bar" :style="{ width: progress + '%' }" />
        </div>
      </div>
      
      <!-- Loading text -->
      <p class="loading-text">Loading...</p>
    </div>
  </Vortex>
</template>

<style scoped>
:deep(.loader-vortex) {
  position: fixed !important;
  inset: 0 !important;
  width: 100vw !important;
  height: 100vh !important;
  z-index: 9999;
  
  padding-top: env(safe-area-inset-top, 0);
  padding-bottom: env(safe-area-inset-bottom, 0);
}

.loader-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 24px;
  
  padding-top: env(safe-area-inset-top, 0);
  padding-bottom: env(safe-area-inset-bottom, 0);
}

.app-icon-wrapper {
  position: relative;
}

.app-icon {
  width: 80px;
  height: 80px;
  border-radius: 18px;
  background: rgba(28, 28, 30, 0.8);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  box-shadow: 
    0 0 0 0.5px rgba(255, 255, 255, 0.1),
    0 10px 40px rgba(0, 122, 255, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: icon-float 3s ease-in-out infinite;
}

@keyframes icon-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

@media (prefers-reduced-motion: reduce) {
  .app-icon {
    animation: none;
  }
}

.app-icon svg {
  width: 56px;
  height: 56px;
}

.progress-wrapper {
  width: 200px;
  max-width: 80vw;
}

.progress-track {
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--systemBlue, #007aff), var(--systemIndigo, #5856d6));
  border-radius: 2px;
  transition: width 0.1s ease-out;
}

.loading-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
  font-weight: 500;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', sans-serif;
}
</style>