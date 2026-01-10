<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

export interface ToastProps {
  id: string
  type?: 'success' | 'error' | 'warning' | 'info'
  title: string
  message?: string
  duration?: number
  action?: { label: string; onClick: () => void }
}

const props = withDefaults(defineProps<ToastProps>(), {
  type: 'info',
  duration: 4000
})

const emit = defineEmits<{
  close: [id: string]
}>()

const progress = ref(100)
const isPaused = ref(false)
let intervalId: ReturnType<typeof setInterval> | null = null

const startTimer = () => {
  if (props.duration <= 0) return
  const step = 100 / (props.duration / 50)
  intervalId = setInterval(() => {
    if (!isPaused.value) {
      progress.value -= step
      if (progress.value <= 0) {
        emit('close', props.id)
      }
    }
  }, 50)
}

const pauseTimer = () => { isPaused.value = true }
const resumeTimer = () => { isPaused.value = false }

onMounted(startTimer)
onUnmounted(() => { if (intervalId) clearInterval(intervalId) })

const colorMap = {
  success: '#39ff14',
  error: '#e43b44',
  warning: '#feae34',
  info: '#0099db'
}

const bgColorMap = {
  success: '#1a3a1a',
  error: '#3a1a1a',
  warning: '#3a2a1a',
  info: '#1a2a3a'
}
</script>

<template>
  <div 
    class="pixel-toast"
    :class="type"
    :style="{ 
      '--toast-color': colorMap[type],
      '--toast-bg': bgColorMap[type]
    }"
    @mouseenter="pauseTimer"
    @mouseleave="resumeTimer"
    role="alert"
  >
    <!-- Corner decorations -->
    <div class="corner tl"></div>
    <div class="corner tr"></div>
    <div class="corner bl"></div>
    <div class="corner br"></div>
    
    <!-- Icon -->
    <div class="toast-icon">
      <!-- Success -->
      <svg v-if="type === 'success'" viewBox="0 0 16 16">
        <rect x="2" y="8" width="4" height="4" fill="currentColor"/>
        <rect x="6" y="10" width="4" height="4" fill="currentColor"/>
        <rect x="10" y="4" width="4" height="4" fill="currentColor"/>
        <rect x="14" y="0" width="2" height="4" fill="currentColor"/>
      </svg>
      
      <!-- Error -->
      <svg v-else-if="type === 'error'" viewBox="0 0 16 16">
        <rect x="2" y="2" width="4" height="4" fill="currentColor"/>
        <rect x="10" y="2" width="4" height="4" fill="currentColor"/>
        <rect x="6" y="6" width="4" height="4" fill="currentColor"/>
        <rect x="2" y="10" width="4" height="4" fill="currentColor"/>
        <rect x="10" y="10" width="4" height="4" fill="currentColor"/>
      </svg>
      
      <!-- Warning -->
      <svg v-else-if="type === 'warning'" viewBox="0 0 16 16">
        <rect x="6" y="0" width="4" height="2" fill="currentColor"/>
        <rect x="4" y="2" width="8" height="10" fill="currentColor"/>
        <rect x="2" y="12" width="12" height="4" fill="currentColor"/>
        <rect x="6" y="4" width="4" height="4" fill="#000"/>
        <rect x="6" y="10" width="4" height="2" fill="#000"/>
      </svg>
      
      <!-- Info -->
      <svg v-else viewBox="0 0 16 16">
        <rect x="4" y="0" width="8" height="2" fill="currentColor"/>
        <rect x="2" y="2" width="12" height="12" fill="currentColor"/>
        <rect x="4" y="14" width="8" height="2" fill="currentColor"/>
        <rect x="6" y="4" width="4" height="2" fill="#000"/>
        <rect x="6" y="8" width="4" height="4" fill="#000"/>
      </svg>
    </div>
    
    <!-- Content -->
    <div class="toast-content">
      <div class="toast-title">{{ title }}</div>
      <div v-if="message" class="toast-message">{{ message }}</div>
    </div>
    
    <!-- Action button -->
    <button v-if="action" class="toast-action" @click="action.onClick">
      {{ action.label }}
    </button>
    
    <!-- Close button -->
    <button class="toast-close" @click="emit('close', id)" aria-label="Fechar">
      <svg viewBox="0 0 16 16">
        <rect x="2" y="6" width="4" height="4" fill="currentColor"/>
        <rect x="6" y="2" width="4" height="4" fill="currentColor"/>
        <rect x="6" y="10" width="4" height="4" fill="currentColor"/>
        <rect x="10" y="6" width="4" height="4" fill="currentColor"/>
      </svg>
    </button>
    
    <!-- Progress bar -->
    <div class="toast-progress-track">
      <div class="toast-progress-fill" :style="{ width: `${progress}%` }"></div>
    </div>
  </div>
</template>

<style scoped>
.pixel-toast {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px 20px;
  padding-bottom: 20px;
  background: var(--toast-bg);
  border: 4px solid #000;
  box-shadow: 6px 6px 0 rgba(0,0,0,0.8);
  min-width: 320px;
  max-width: 420px;
  image-rendering: pixelated;
}

/* Corner decorations */
.corner {
  position: absolute;
  width: 6px;
  height: 6px;
  background: var(--toast-color);
}

.corner.tl { top: -3px; left: -3px; }
.corner.tr { top: -3px; right: -3px; }
.corner.bl { bottom: -3px; left: -3px; }
.corner.br { bottom: -3px; right: -3px; }

/* Icon */
.toast-icon {
  width: 24px;
  height: 24px;
  color: var(--toast-color);
  flex-shrink: 0;
  filter: drop-shadow(0 0 4px var(--toast-color));
}

.toast-icon svg {
  width: 100%;
  height: 100%;
}

/* Content */
.toast-content {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: #fff;
  line-height: 1.4;
  text-shadow: 1px 1px 0 #000;
}

.toast-message {
  font-family: 'VT323', monospace;
  font-size: 16px;
  color: #808080;
  margin-top: 6px;
}

/* Action button */
.toast-action {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  padding: 8px 12px;
  background: var(--toast-color);
  color: #000;
  border: 2px solid #000;
  box-shadow: 2px 2px 0 #000;
  white-space: nowrap;
  text-shadow: none;
}

.toast-action:hover {
  transform: translate(-2px, -2px);
  box-shadow: 4px 4px 0 #000;
}

.toast-action:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

/* Close button */
.toast-close {
  width: 24px;
  height: 24px;
  padding: 4px;
  background: transparent;
  border: none;
  box-shadow: none;
  color: #808080;
  flex-shrink: 0;
}

.toast-close:hover {
  color: #e43b44;
  transform: none;
}

.toast-close svg {
  width: 100%;
  height: 100%;
}

/* Progress bar */
.toast-progress-track {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: rgba(0, 0, 0, 0.5);
}

.toast-progress-fill {
  height: 100%;
  background: var(--toast-color);
  box-shadow: 0 0 8px var(--toast-color);
  transition: width 0.05s steps(10);
}

/* Type-specific styles */
.pixel-toast.success {
  border-color: #2d8a1a;
}

.pixel-toast.error {
  border-color: #a82835;
}

.pixel-toast.warning {
  border-color: #c68b28;
}

.pixel-toast.info {
  border-color: #006b99;
}
</style>
