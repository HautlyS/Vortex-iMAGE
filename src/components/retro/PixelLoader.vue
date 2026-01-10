<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg';
  color?: string;
  variant?: 'spinner' | 'dots' | 'bars' | 'pacman' | 'blocks' | 'hearts';
  text?: string;
}>(), {
  size: 'md',
  color: '#39ff14',
  variant: 'spinner'
});

const loaderStyle = computed(() => ({
  '--loader-color': props.color
}));
</script>

<template>
  <div class="pixel-loader" :class="size" :style="loaderStyle">
    <!-- Spinner variant - 8-bit style -->
    <div v-if="variant === 'spinner'" class="loader-spinner">
      <div class="spinner-ring">
        <div v-for="i in 8" :key="i" class="ring-segment" :style="{ '--i': i }"></div>
      </div>
    </div>

    <!-- Dots variant -->
    <div v-else-if="variant === 'dots'" class="loader-dots">
      <div v-for="i in 4" :key="i" class="pixel-dot" :style="{ '--i': i }"></div>
    </div>

    <!-- Bars variant - Equalizer style -->
    <div v-else-if="variant === 'bars'" class="loader-bars">
      <div v-for="i in 5" :key="i" class="pixel-bar" :style="{ '--i': i }"></div>
    </div>

    <!-- Pacman variant -->
    <div v-else-if="variant === 'pacman'" class="loader-pacman">
      <div class="pacman">
        <div class="pacman-top"></div>
        <div class="pacman-bottom"></div>
        <div class="pacman-eye"></div>
      </div>
      <div class="pellets">
        <div v-for="i in 4" :key="i" class="pellet" :style="{ '--i': i }"></div>
      </div>
    </div>

    <!-- Blocks variant - Tetris style -->
    <div v-else-if="variant === 'blocks'" class="loader-blocks">
      <div v-for="i in 4" :key="i" class="pixel-block" :style="{ '--i': i }"></div>
    </div>

    <!-- Hearts variant - Game lives style -->
    <div v-else-if="variant === 'hearts'" class="loader-hearts">
      <div v-for="i in 3" :key="i" class="pixel-heart" :style="{ '--i': i }">
        <svg viewBox="0 0 16 16">
          <rect x="2" y="2" width="4" height="4" fill="currentColor"/>
          <rect x="10" y="2" width="4" height="4" fill="currentColor"/>
          <rect x="0" y="4" width="16" height="4" fill="currentColor"/>
          <rect x="2" y="8" width="12" height="4" fill="currentColor"/>
          <rect x="4" y="12" width="8" height="2" fill="currentColor"/>
          <rect x="6" y="14" width="4" height="2" fill="currentColor"/>
        </svg>
      </div>
    </div>

    <span v-if="text" class="loader-text">{{ text }}</span>
  </div>
</template>

<style scoped>
.pixel-loader {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  image-rendering: pixelated;
}

.loader-text {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: var(--loader-color);
  text-shadow: 0 0 8px var(--loader-color);
  animation: text-blink 0.8s steps(1) infinite;
  letter-spacing: 2px;
}

@keyframes text-blink {
  0%, 60% { opacity: 1; }
  61%, 100% { opacity: 0; }
}

/* Spinner */
.loader-spinner {
  position: relative;
}

.pixel-loader.sm .loader-spinner { width: 32px; height: 32px; }
.pixel-loader.md .loader-spinner { width: 48px; height: 48px; }
.pixel-loader.lg .loader-spinner { width: 64px; height: 64px; }

.spinner-ring {
  width: 100%;
  height: 100%;
  position: relative;
}

.ring-segment {
  position: absolute;
  width: 8px;
  height: 8px;
  background: var(--loader-color);
  animation: segment-pulse 0.8s steps(1) infinite;
  animation-delay: calc(var(--i) * 0.1s);
  opacity: 0.2;
}

.ring-segment:nth-child(1) { top: 0; left: 50%; transform: translateX(-50%); }
.ring-segment:nth-child(2) { top: 15%; right: 15%; }
.ring-segment:nth-child(3) { top: 50%; right: 0; transform: translateY(-50%); }
.ring-segment:nth-child(4) { bottom: 15%; right: 15%; }
.ring-segment:nth-child(5) { bottom: 0; left: 50%; transform: translateX(-50%); }
.ring-segment:nth-child(6) { bottom: 15%; left: 15%; }
.ring-segment:nth-child(7) { top: 50%; left: 0; transform: translateY(-50%); }
.ring-segment:nth-child(8) { top: 15%; left: 15%; }

@keyframes segment-pulse {
  0%, 12.5% { opacity: 1; box-shadow: 0 0 8px var(--loader-color); }
  12.6%, 100% { opacity: 0.2; box-shadow: none; }
}

/* Dots */
.loader-dots {
  display: flex;
  gap: 8px;
}

.pixel-dot {
  background: var(--loader-color);
  animation: dot-jump 0.8s steps(4) infinite;
  animation-delay: calc(var(--i) * 0.15s);
}

.pixel-loader.sm .pixel-dot { width: 8px; height: 8px; }
.pixel-loader.md .pixel-dot { width: 12px; height: 12px; }
.pixel-loader.lg .pixel-dot { width: 16px; height: 16px; }

@keyframes dot-jump {
  0%, 100% { transform: translateY(0); opacity: 0.4; }
  50% { transform: translateY(-16px); opacity: 1; box-shadow: 0 0 12px var(--loader-color); }
}

/* Bars */
.loader-bars {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  height: 40px;
}

.pixel-bar {
  width: 8px;
  background: var(--loader-color);
  animation: bar-dance 0.8s steps(4) infinite;
  animation-delay: calc(var(--i) * 0.1s);
}

@keyframes bar-dance {
  0%, 100% { height: 8px; opacity: 0.4; }
  50% { height: 40px; opacity: 1; box-shadow: 0 0 8px var(--loader-color); }
}

/* Pacman */
.loader-pacman {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pacman {
  position: relative;
  width: 32px;
  height: 32px;
}

.pacman-top, .pacman-bottom {
  position: absolute;
  width: 32px;
  height: 16px;
  background: #feae34;
  left: 0;
}

.pacman-top {
  top: 0;
  border-radius: 16px 16px 0 0;
  animation: chomp-top 0.3s steps(2) infinite;
  transform-origin: bottom center;
}

.pacman-bottom {
  bottom: 0;
  border-radius: 0 0 16px 16px;
  animation: chomp-bottom 0.3s steps(2) infinite;
  transform-origin: top center;
}

.pacman-eye {
  position: absolute;
  top: 6px;
  left: 18px;
  width: 4px;
  height: 4px;
  background: #000;
}

@keyframes chomp-top {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(-20deg); }
}

@keyframes chomp-bottom {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(20deg); }
}

.pellets {
  display: flex;
  gap: 12px;
}

.pellet {
  width: 8px;
  height: 8px;
  background: var(--loader-color);
  animation: pellet-eat 1.2s steps(1) infinite;
  animation-delay: calc(var(--i) * 0.3s);
}

@keyframes pellet-eat {
  0%, 75% { opacity: 1; }
  76%, 100% { opacity: 0.2; }
}

/* Blocks - Tetris style */
.loader-blocks {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
}

.pixel-block {
  width: 16px;
  height: 16px;
  border: 2px solid #000;
  animation: block-fall 1.2s steps(4) infinite;
  animation-delay: calc(var(--i) * 0.2s);
}

.pixel-block:nth-child(1) { background: #e43b44; }
.pixel-block:nth-child(2) { background: #0099db; }
.pixel-block:nth-child(3) { background: #63c74d; }
.pixel-block:nth-child(4) { background: #feae34; }

@keyframes block-fall {
  0% { transform: translateY(-20px) rotate(0deg); opacity: 0; }
  20% { opacity: 1; }
  80% { opacity: 1; }
  100% { transform: translateY(20px) rotate(90deg); opacity: 0; }
}

/* Hearts */
.loader-hearts {
  display: flex;
  gap: 8px;
}

.pixel-heart {
  color: #e43b44;
  animation: heart-pulse 0.6s steps(2) infinite;
  animation-delay: calc(var(--i) * 0.2s);
}

.pixel-loader.sm .pixel-heart svg { width: 16px; height: 16px; }
.pixel-loader.md .pixel-heart svg { width: 24px; height: 24px; }
.pixel-loader.lg .pixel-heart svg { width: 32px; height: 32px; }

@keyframes heart-pulse {
  0%, 100% { transform: scale(1); opacity: 0.6; }
  50% { transform: scale(1.2); opacity: 1; filter: drop-shadow(0 0 8px #e43b44); }
}
</style>
