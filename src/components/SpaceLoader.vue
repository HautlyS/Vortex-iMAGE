<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ complete: [] }>()

const progress = ref(0)
const glitchActive = ref(false)
const messages = ['BREACH_INIT', 'DECRYPT_SYS', 'BYPASS_AUTH', 'INJECT_CODE', 'ACCESS_GRANTED']
const currentMessage = ref(messages[0])
const hexStream = ref('')

let interval: ReturnType<typeof setInterval> | null = null
let glitchInterval: ReturnType<typeof setInterval> | null = null
let hexInterval: ReturnType<typeof setInterval> | null = null

function generateHex(): string {
  return Array.from({ length: 32 }, () => 
    Math.floor(Math.random() * 16).toString(16)
  ).join('').toUpperCase()
}

onMounted(() => {
  let step = 0
  
  // Main progress
  interval = setInterval(() => {
    step++
    progress.value = Math.min((step / 25) * 100, 100)
    const msgIndex = Math.min(Math.floor((progress.value / 100) * messages.length), messages.length - 1)
    currentMessage.value = messages[msgIndex]
    
    if (step >= 30) {
      if (interval) clearInterval(interval)
      if (glitchInterval) clearInterval(glitchInterval)
      if (hexInterval) clearInterval(hexInterval)
      emit('complete')
    }
  }, 70)
  
  // Random glitch effect
  glitchInterval = setInterval(() => {
    glitchActive.value = true
    setTimeout(() => glitchActive.value = false, 150)
  }, 800 + Math.random() * 1200)
  
  // Hex stream
  hexInterval = setInterval(() => {
    hexStream.value = generateHex()
  }, 50)
})

onUnmounted(() => {
  if (interval) clearInterval(interval)
  if (glitchInterval) clearInterval(glitchInterval)
  if (hexInterval) clearInterval(hexInterval)
})
</script>

<template>
  <div class="loader" :class="{ glitch: glitchActive }">
    <!-- Scanlines -->
    <div class="scanlines" />
    
    <!-- Glitch layers -->
    <div class="glitch-layer cyan" />
    <div class="glitch-layer pink" />
    
    <!-- Main content -->
    <div class="loader-content">
      <!-- Anarchist symbol -->
      <div class="symbol-container">
        <div class="symbol">
          <div class="ring ring-outer" />
          <div class="ring ring-middle" />
          <div class="ring ring-inner" />
          <div class="core">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 2L12 22M2 12L22 12M4.93 4.93L19.07 19.07M19.07 4.93L4.93 19.07" />
            </svg>
          </div>
        </div>
        
        <!-- Orbiting particles -->
        <div class="particles">
          <span v-for="i in 8" :key="i" class="particle" :style="{ '--i': i }" />
        </div>
      </div>
      
      <!-- Hex stream -->
      <div class="hex-stream">{{ hexStream }}</div>
      
      <!-- Progress -->
      <div class="progress-container">
        <div class="progress-track">
          <div class="progress-bar" :style="{ width: progress + '%' }" />
          <div class="progress-glow" :style="{ left: progress + '%' }" />
        </div>
        
        <div class="progress-info">
          <span class="message">{{ currentMessage }}</span>
          <span class="percent">{{ Math.round(progress) }}%</span>
        </div>
      </div>
      
      <!-- Brand -->
      <div class="brand">
        <span class="brand-text" data-text="iMAGE">iMAGE</span>
        <span class="brand-sub">SYSTEM v2.0</span>
      </div>
    </div>
    
    <!-- Corner decorations -->
    <div class="corner corner-tl" />
    <div class="corner corner-tr" />
    <div class="corner corner-bl" />
    <div class="corner corner-br" />
  </div>
</template>

<style scoped>
.loader {
  position: fixed;
  inset: 0;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  overflow: hidden;
}

/* Scanlines */
.scanlines {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 0, 0, 0.1) 2px,
    rgba(0, 0, 0, 0.1) 4px
  );
  pointer-events: none;
  opacity: 0.3;
}

/* Glitch layers */
.glitch-layer {
  position: absolute;
  inset: 0;
  opacity: 0;
  pointer-events: none;
  mix-blend-mode: screen;
}

.glitch-layer.cyan {
  background: linear-gradient(90deg, transparent 30%, rgba(0, 240, 255, 0.1) 50%, transparent 70%);
}

.glitch-layer.pink {
  background: linear-gradient(90deg, transparent 40%, rgba(255, 45, 106, 0.1) 60%, transparent 80%);
}

.loader.glitch .glitch-layer.cyan {
  opacity: 1;
  animation: glitch-cyan 0.15s steps(2) infinite;
}

.loader.glitch .glitch-layer.pink {
  opacity: 1;
  animation: glitch-pink 0.15s steps(2) infinite;
}

@keyframes glitch-cyan {
  0%, 100% { transform: translateX(-5px); }
  50% { transform: translateX(5px); }
}

@keyframes glitch-pink {
  0%, 100% { transform: translateX(5px); }
  50% { transform: translateX(-5px); }
}

/* Content */
.loader-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
  z-index: 1;
}

/* Symbol */
.symbol-container {
  position: relative;
  width: clamp(80px, 20vw, 120px);
  height: clamp(80px, 20vw, 120px);
}

.symbol {
  position: relative;
  width: 100%;
  height: 100%;
}

.ring {
  position: absolute;
  border-radius: 50%;
  border: 1px solid;
}

.ring-outer {
  inset: 0;
  border-color: rgba(0, 240, 255, 0.3);
  animation: spin-slow 8s linear infinite;
}

.ring-middle {
  inset: 15%;
  border-color: rgba(176, 38, 255, 0.4);
  animation: spin-slow 6s linear infinite reverse;
}

.ring-inner {
  inset: 30%;
  border-color: rgba(255, 45, 106, 0.5);
  animation: spin-slow 4s linear infinite;
}

.core {
  position: absolute;
  inset: 35%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--cyber-cyan, #00f0ff);
  animation: pulse-core 2s ease-in-out infinite;
}

.core svg {
  width: 100%;
  height: 100%;
}

@keyframes spin-slow {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes pulse-core {
  0%, 100% { 
    opacity: 0.7;
    filter: drop-shadow(0 0 5px currentColor);
  }
  50% { 
    opacity: 1;
    filter: drop-shadow(0 0 15px currentColor) drop-shadow(0 0 30px currentColor);
  }
}

/* Particles */
.particles {
  position: absolute;
  inset: -20%;
}

.particle {
  position: absolute;
  width: 4px;
  height: 4px;
  background: var(--cyber-cyan);
  border-radius: 50%;
  top: 50%;
  left: 50%;
  animation: orbit 3s linear infinite;
  animation-delay: calc(var(--i) * -0.375s);
  box-shadow: 0 0 10px var(--cyber-cyan);
}

@keyframes orbit {
  0% { transform: rotate(0deg) translateX(60px) rotate(0deg); opacity: 0; }
  10% { opacity: 1; }
  90% { opacity: 1; }
  100% { transform: rotate(360deg) translateX(60px) rotate(-360deg); opacity: 0; }
}

/* Hex stream */
.hex-stream {
  font-family: 'JetBrains Mono', 'SF Mono', monospace;
  font-size: clamp(0.5rem, 1.5vw, 0.65rem);
  color: rgba(0, 240, 255, 0.4);
  letter-spacing: 0.1em;
  max-width: 280px;
  text-align: center;
  word-break: break-all;
}

/* Progress */
.progress-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  width: clamp(180px, 50vw, 280px);
}

.progress-track {
  position: relative;
  width: 100%;
  height: 2px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 1px;
  overflow: visible;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--cyber-cyan), var(--cyber-purple));
  border-radius: 1px;
  transition: width 0.1s ease;
  box-shadow: 0 0 10px var(--cyber-cyan);
}

.progress-glow {
  position: absolute;
  top: 50%;
  width: 20px;
  height: 20px;
  background: var(--cyber-cyan);
  border-radius: 50%;
  transform: translate(-50%, -50%);
  filter: blur(10px);
  opacity: 0.6;
  transition: left 0.1s ease;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  width: 100%;
  font-family: 'JetBrains Mono', 'SF Mono', monospace;
}

.message {
  font-size: clamp(0.6rem, 2vw, 0.7rem);
  color: var(--cyber-cyan);
  letter-spacing: 0.15em;
  text-transform: uppercase;
  animation: flicker 0.1s infinite;
}

.percent {
  font-size: clamp(0.55rem, 1.5vw, 0.65rem);
  color: rgba(255, 255, 255, 0.5);
}

@keyframes flicker {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.8; }
}

/* Brand */
.brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  margin-top: 1rem;
}

.brand-text {
  position: relative;
  font-size: clamp(1.5rem, 5vw, 2rem);
  font-weight: 700;
  letter-spacing: 0.3em;
  color: #fff;
}

.brand-text::before,
.brand-text::after {
  content: attr(data-text);
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

.brand-text::before {
  color: var(--cyber-pink);
  animation: glitch-text 3s infinite;
  clip-path: polygon(0 0, 100% 0, 100% 45%, 0 45%);
}

.brand-text::after {
  color: var(--cyber-cyan);
  animation: glitch-text 3s infinite reverse;
  clip-path: polygon(0 55%, 100% 55%, 100% 100%, 0 100%);
}

@keyframes glitch-text {
  0%, 90%, 100% { transform: translate(0); }
  92% { transform: translate(-2px, 1px); }
  94% { transform: translate(2px, -1px); }
  96% { transform: translate(-1px, 2px); }
  98% { transform: translate(1px, -2px); }
}

.brand-sub {
  font-family: 'JetBrains Mono', 'SF Mono', monospace;
  font-size: clamp(0.5rem, 1.5vw, 0.6rem);
  color: rgba(255, 255, 255, 0.3);
  letter-spacing: 0.2em;
}

/* Corners */
.corner {
  position: absolute;
  width: 30px;
  height: 30px;
  border: 1px solid rgba(0, 240, 255, 0.3);
}

.corner-tl { top: 20px; left: 20px; border-right: none; border-bottom: none; }
.corner-tr { top: 20px; right: 20px; border-left: none; border-bottom: none; }
.corner-bl { bottom: 20px; left: 20px; border-right: none; border-top: none; }
.corner-br { bottom: 20px; right: 20px; border-left: none; border-top: none; }

/* Glitch state */
.loader.glitch .brand-text {
  animation: glitch-skew 0.15s ease-in-out;
}

.loader.glitch .symbol {
  animation: glitch-skew 0.15s ease-in-out;
}

@keyframes glitch-skew {
  0%, 100% { transform: skew(0deg); }
  25% { transform: skew(-2deg); }
  50% { transform: skew(1deg); }
  75% { transform: skew(-1deg); }
}
</style>
