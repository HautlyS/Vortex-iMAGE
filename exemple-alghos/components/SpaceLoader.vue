<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ complete: [] }>()
const progress = ref(0)
const messages = ['INITIALIZING', 'CONNECTING', 'DECRYPTING', 'READY']
const currentMessage = ref(messages[0])

let interval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  let step = 0
  interval = setInterval(() => {
    step++
    progress.value = Math.min((step / 20) * 100, 100)
    const msgIndex = Math.min(Math.floor((progress.value / 100) * messages.length), messages.length - 1)
    currentMessage.value = messages[msgIndex]
    if (step >= 24) {
      if (interval) clearInterval(interval)
      emit('complete')
    }
  }, 80)
})

onUnmounted(() => { if (interval) clearInterval(interval) })
</script>

<template>
  <div class="loader">
    <div class="loader-ring">
      <div class="ring ring-1"></div>
      <div class="ring ring-2"></div>
      <div class="ring ring-3"></div>
      <div class="core"></div>
    </div>
    
    <div class="loader-progress">
      <div class="progress-track">
        <div class="progress-bar" :style="{ width: progress + '%' }"></div>
      </div>
      <p class="loader-text">{{ currentMessage }}</p>
      <p class="loader-percent">{{ Math.round(progress) }}%</p>
    </div>
  </div>
</template>

<style scoped>
.loader {
  position: fixed;
  inset: 0;
  background: #000;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.loader-ring {
  position: relative;
  width: clamp(60px, 15vw, 80px);
  height: clamp(60px, 15vw, 80px);
  margin-bottom: clamp(2rem, 6vw, 3rem);
}

.ring {
  position: absolute;
  border-radius: 50%;
  border: 2px solid transparent;
}

.ring-1 {
  inset: 0;
  border-color: rgba(34,211,238,0.2);
  animation: spin 3s linear infinite;
}

.ring-2 {
  inset: 8px;
  border-color: rgba(34,211,238,0.3);
  animation: spin 2s linear infinite reverse;
}

.ring-3 {
  inset: 16px;
  border-top-color: #22d3ee;
  animation: spin 1s linear infinite;
}

.core {
  position: absolute;
  inset: 24px;
  background: radial-gradient(circle, #22d3ee 0%, transparent 70%);
  border-radius: 50%;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse { 0%, 100% { opacity: 0.5; } 50% { opacity: 1; } }

.loader-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.progress-track {
  width: clamp(120px, 40vw, 180px);
  height: 3px;
  background: rgba(255,255,255,0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #22d3ee, #06b6d4);
  border-radius: 2px;
  transition: width 0.2s ease;
  box-shadow: 0 0 10px rgba(34,211,238,0.5);
}

.loader-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: clamp(0.6rem, 2vw, 0.7rem);
  color: rgba(255,255,255,0.6);
  letter-spacing: 0.2em;
  text-transform: uppercase;
}

.loader-percent {
  font-family: 'JetBrains Mono', monospace;
  font-size: clamp(0.55rem, 1.5vw, 0.65rem);
  color: rgba(255,255,255,0.3);
}
</style>
