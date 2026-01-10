<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ complete: [] }>()
const progress = ref(0)
const loadingText = ref('INITIALIZING')
const frame = ref(0)

const loadingMessages = [
  'INITIALIZING',
  'LOADING SPRITES',
  'GENERATING WORLD',
  'SPAWNING PIXELS',
  'READY TO PLAY'
]

let interval: ReturnType<typeof setInterval> | null = null
let frameInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  let step = 0
  
  // Progress interval
  interval = setInterval(() => {
    step++
    progress.value = Math.min((step / 25) * 100, 100)
    
    // Update loading message
    const messageIndex = Math.floor((step / 25) * loadingMessages.length)
    loadingText.value = loadingMessages[Math.min(messageIndex, loadingMessages.length - 1)]
    
    if (step >= 30) {
      if (interval) clearInterval(interval)
      emit('complete')
    }
  }, 80)
  
  // Animation frame interval
  frameInterval = setInterval(() => {
    frame.value = (frame.value + 1) % 4
  }, 200)
})

onUnmounted(() => {
  if (interval) clearInterval(interval)
  if (frameInterval) clearInterval(frameInterval)
})

const filledSegments = (p: number) => Math.floor((p / 100) * 20)
</script>

<template>
  <div class="pixel-loader-screen">
    <!-- Scanlines -->
    <div class="scanlines"></div>
    
    <!-- Pixel Stars Background -->
    <div class="star-field">
      <div v-for="i in 50" :key="i" class="pixel-star" :style="{
        left: `${Math.random() * 100}%`,
        top: `${Math.random() * 100}%`,
        animationDelay: `${Math.random() * 2}s`
      }"></div>
    </div>
    
    <!-- Grid Floor -->
    <div class="grid-floor"></div>
    
    <div class="loader-content">
      <!-- 8-Bit Logo -->
      <div class="pixel-logo">
        <svg viewBox="0 0 64 64" class="logo-sprite">
          <!-- Camera body -->
          <rect x="8" y="20" width="48" height="36" fill="#1a1a2e"/>
          <rect x="8" y="20" width="48" height="4" fill="#16213e"/>
          <rect x="8" y="52" width="48" height="4" fill="#0f0f23"/>
          
          <!-- Camera lens -->
          <rect x="20" y="28" width="24" height="24" fill="#0f3460"/>
          <rect x="24" y="32" width="16" height="16" fill="#16213e"/>
          <rect x="28" y="36" width="8" height="8" fill="#39ff14"/>
          <rect x="30" y="38" width="4" height="4" fill="#8cff7a"/>
          
          <!-- Flash -->
          <rect x="44" y="24" width="8" height="8" :fill="frame % 2 === 0 ? '#feae34' : '#ffd700'"/>
          
          <!-- Viewfinder -->
          <rect x="12" y="24" width="8" height="6" fill="#0f3460"/>
          <rect x="14" y="26" width="4" height="2" fill="#39ff14"/>
          
          <!-- Decorative pixels -->
          <rect x="4" y="16" width="4" height="4" fill="#f15bb5"/>
          <rect x="56" y="16" width="4" height="4" fill="#f15bb5"/>
          <rect x="4" y="56" width="4" height="4" fill="#9b5de5"/>
          <rect x="56" y="56" width="4" height="4" fill="#9b5de5"/>
        </svg>
        
        <!-- Glow effect -->
        <div class="logo-glow"></div>
      </div>
      
      <!-- Title -->
      <h1 class="pixel-title">
        <span class="title-letter" v-for="(letter, i) in 'iMAGE'" :key="i" :style="{ animationDelay: `${i * 0.1}s` }">
          {{ letter }}
        </span>
      </h1>
      
      <!-- Progress Bar - 8-Bit Style -->
      <div class="pixel-progress-container">
        <div class="progress-frame">
          <div class="frame-corner tl"></div>
          <div class="frame-corner tr"></div>
          <div class="frame-corner bl"></div>
          <div class="frame-corner br"></div>
        </div>
        
        <div class="progress-track">
          <div 
            v-for="i in 20" 
            :key="i" 
            class="progress-segment"
            :class="{ filled: i <= filledSegments(progress) }"
          >
            <div class="segment-shine"></div>
          </div>
        </div>
        
        <div class="progress-label">{{ Math.round(progress) }}%</div>
      </div>
      
      <!-- Loading Text -->
      <p class="loading-text">
        {{ loadingText }}
        <span class="cursor" :class="{ blink: frame % 2 === 0 }">_</span>
      </p>
      
      <!-- Decorative Sprites -->
      <div class="floating-sprites">
        <div class="sprite coin" :style="{ animationDelay: '0s' }">
          <svg viewBox="0 0 16 16">
            <rect x="4" y="0" width="8" height="2" fill="#feae34"/>
            <rect x="2" y="2" width="12" height="12" fill="#ffd700"/>
            <rect x="4" y="14" width="8" height="2" fill="#c68b28"/>
            <rect x="6" y="6" width="2" height="4" fill="#fff"/>
          </svg>
        </div>
        <div class="sprite heart" :style="{ animationDelay: '0.5s' }">
          <svg viewBox="0 0 16 16">
            <rect x="2" y="2" width="4" height="4" fill="#e43b44"/>
            <rect x="10" y="2" width="4" height="4" fill="#e43b44"/>
            <rect x="0" y="4" width="16" height="4" fill="#ff6b6b"/>
            <rect x="2" y="8" width="12" height="4" fill="#e43b44"/>
            <rect x="6" y="12" width="4" height="4" fill="#a82835"/>
          </svg>
        </div>
        <div class="sprite star" :style="{ animationDelay: '1s' }">
          <svg viewBox="0 0 16 16">
            <rect x="6" y="0" width="4" height="4" fill="#feae34"/>
            <rect x="0" y="6" width="16" height="4" fill="#ffd700"/>
            <rect x="2" y="10" width="4" height="4" fill="#feae34"/>
            <rect x="10" y="10" width="4" height="4" fill="#feae34"/>
          </svg>
        </div>
      </div>
    </div>
    
    <!-- Version -->
    <div class="version-tag">v2.0.0</div>
  </div>
</template>

<style scoped>
.pixel-loader-screen {
  position: fixed;
  inset: 0;
  background: #0f0f23;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  image-rendering: pixelated;
  overflow: hidden;
}

.scanlines {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent 0px,
    transparent 2px,
    rgba(0, 0, 0, 0.15) 2px,
    rgba(0, 0, 0, 0.15) 4px
  );
  pointer-events: none;
  z-index: 100;
}

.star-field {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.pixel-star {
  position: absolute;
  width: 2px;
  height: 2px;
  background: #fff;
  animation: twinkle 1s steps(2) infinite;
}

@keyframes twinkle {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.grid-floor {
  position: absolute;
  bottom: -10%;
  left: -10%;
  width: 120%;
  height: 40%;
  background-image: 
    linear-gradient(rgba(57, 255, 20, 0.15) 2px, transparent 2px),
    linear-gradient(90deg, rgba(57, 255, 20, 0.15) 2px, transparent 2px);
  background-size: 40px 40px;
  transform: perspective(400px) rotateX(60deg);
  animation: grid-move 4s linear infinite;
}

@keyframes grid-move {
  0% { transform: perspective(400px) rotateX(60deg) translateY(0); }
  100% { transform: perspective(400px) rotateX(60deg) translateY(40px); }
}

.loader-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
  z-index: 10;
}

/* Logo */
.pixel-logo {
  position: relative;
  animation: logo-bounce 1s steps(4) infinite;
}

.logo-sprite {
  width: 96px;
  height: 96px;
  filter: drop-shadow(4px 4px 0 #000);
}

.logo-glow {
  position: absolute;
  inset: -20px;
  background: radial-gradient(circle, rgba(57, 255, 20, 0.3) 0%, transparent 70%);
  animation: glow-pulse 2s steps(4) infinite;
  pointer-events: none;
}

@keyframes logo-bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

@keyframes glow-pulse {
  0%, 100% { opacity: 0.5; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.1); }
}

/* Title */
.pixel-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 24px;
  color: #39ff14;
  text-shadow: 
    4px 4px 0 #000,
    0 0 20px rgba(57, 255, 20, 0.5);
  letter-spacing: 8px;
  display: flex;
}

.title-letter {
  animation: letter-wave 1s steps(4) infinite;
}

@keyframes letter-wave {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}

/* Progress Bar */
.pixel-progress-container {
  position: relative;
  width: 280px;
}

.progress-frame {
  position: absolute;
  inset: -8px;
  border: 4px solid #3a3a5c;
  pointer-events: none;
}

.frame-corner {
  position: absolute;
  width: 8px;
  height: 8px;
  background: #f15bb5;
}

.frame-corner.tl { top: -4px; left: -4px; }
.frame-corner.tr { top: -4px; right: -4px; }
.frame-corner.bl { bottom: -4px; left: -4px; }
.frame-corner.br { bottom: -4px; right: -4px; }

.progress-track {
  display: flex;
  gap: 2px;
  padding: 8px;
  background: #000;
  border: 4px solid #1a1a2e;
}

.progress-segment {
  flex: 1;
  height: 16px;
  background: #1a1a2e;
  position: relative;
}

.progress-segment.filled {
  background: linear-gradient(180deg, #8cff7a 0%, #39ff14 50%, #2d8a1a 100%);
  box-shadow: 0 0 8px rgba(57, 255, 20, 0.5);
}

.segment-shine {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 40%;
  background: rgba(255, 255, 255, 0.3);
  opacity: 0;
}

.progress-segment.filled .segment-shine {
  opacity: 1;
}

.progress-label {
  position: absolute;
  right: -50px;
  top: 50%;
  transform: translateY(-50%);
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: #39ff14;
  text-shadow: 0 0 8px rgba(57, 255, 20, 0.5);
}

/* Loading Text */
.loading-text {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: #808080;
  letter-spacing: 2px;
}

.cursor {
  color: #39ff14;
  opacity: 0;
}

.cursor.blink {
  opacity: 1;
}

/* Floating Sprites */
.floating-sprites {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.sprite {
  position: absolute;
  animation: sprite-float 8s linear infinite;
}

.sprite svg {
  width: 24px;
  height: 24px;
}

.sprite.coin {
  left: 10%;
  animation-delay: 0s;
}

.sprite.heart {
  left: 50%;
  animation-delay: 2s;
}

.sprite.star {
  left: 85%;
  animation-delay: 4s;
}

@keyframes sprite-float {
  0% { transform: translateY(100vh) rotate(0deg); opacity: 0; }
  10% { opacity: 0.8; }
  90% { opacity: 0.8; }
  100% { transform: translateY(-100px) rotate(360deg); opacity: 0; }
}

/* Version */
.version-tag {
  position: absolute;
  bottom: 20px;
  right: 20px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #3a3a5c;
}

@media (prefers-reduced-motion: reduce) {
  .pixel-logo,
  .title-letter,
  .pixel-star,
  .grid-floor,
  .sprite {
    animation: none;
  }
}
</style>
