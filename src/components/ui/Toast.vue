<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

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

const iconMap = {
  success: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6L9 17l-5-5"/></svg>`,
  error: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M15 9l-6 6M9 9l6 6"/></svg>`,
  warning: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 9v4M12 17h.01"/><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/></svg>`,
  info: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>`
}

const colorMap = computed(() => ({
  success: 'var(--retro-accent-green, #00ff87)',
  error: 'var(--retro-accent-red, #ff4757)',
  warning: 'var(--retro-accent-yellow, #ffc312)',
  info: 'var(--retro-accent-blue, #00d4ff)'
}))

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
</script>

<template>
  <div 
    class="toast"
    :style="{ '--toast-color': colorMap[type] }"
    @mouseenter="pauseTimer"
    @mouseleave="resumeTimer"
    role="alert"
  >
    <div class="toast-icon" v-html="iconMap[type]" />
    <div class="toast-content">
      <div class="toast-title">{{ title }}</div>
      <div v-if="message" class="toast-message">{{ message }}</div>
    </div>
    <button v-if="action" class="toast-action" @click="action.onClick">
      {{ action.label }}
    </button>
    <button class="toast-close" @click="emit('close', id)" aria-label="Fechar">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>
    <div class="toast-progress" :style="{ width: `${progress}%` }" />
  </div>
</template>

<style scoped>
.toast {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  background: var(--retro-bg-panel, #1a1030);
  border: 2px solid var(--toast-color);
  border-left-width: 4px;
  box-shadow: 4px 4px 0 rgba(0,0,0,0.5), 0 0 20px color-mix(in srgb, var(--toast-color) 30%, transparent);
  position: relative;
  overflow: hidden;
  min-width: 320px;
  max-width: 420px;
}

.toast-icon {
  width: 20px;
  height: 20px;
  color: var(--toast-color);
  flex-shrink: 0;
  margin-top: 2px;
}

.toast-icon :deep(svg) { width: 100%; height: 100%; }

.toast-content { flex: 1; min-width: 0; }

.toast-title {
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: var(--retro-text-main, #fff);
  line-height: 1.2;
}

.toast-message {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: var(--retro-text-muted, #9d8ec2);
  margin-top: 4px;
}

.toast-action {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  padding: 6px 10px;
  background: var(--toast-color);
  color: #000;
  border: 2px solid #000;
  box-shadow: 2px 2px 0 #000;
  white-space: nowrap;
}

.toast-action:hover { transform: translate(-1px, -1px); box-shadow: 3px 3px 0 #000; }
.toast-action:active { transform: translate(1px, 1px); box-shadow: 1px 1px 0 #000; }

.toast-close {
  width: 24px;
  height: 24px;
  padding: 4px;
  background: transparent;
  border: none;
  box-shadow: none;
  color: var(--retro-text-muted, #9d8ec2);
  flex-shrink: 0;
}

.toast-close:hover { color: var(--retro-accent-red, #ff4757); }

.toast-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 3px;
  background: var(--toast-color);
  transition: width 0.05s linear;
}
</style>
