<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg';
  color?: string;
  variant?: 'spinner' | 'dots' | 'bars' | 'pacman';
  text?: string;
}>(), {
  size: 'md',
  color: 'var(--retro-accent-green, #00ff87)',
  variant: 'spinner'
});

const loaderStyle = computed(() => ({
  '--loader-color': props.color
}));


</script>

<template>
  <div class="pixel-loader" :class="size" :style="loaderStyle">
    <!-- Spinner variant -->
    <div v-if="variant === 'spinner'" class="loader-spinner">
      <div v-for="i in 8" :key="i" class="spinner-segment" :style="{ '--i': i }" />
    </div>

    <!-- Dots variant -->
    <div v-else-if="variant === 'dots'" class="loader-dots">
      <div v-for="i in 3" :key="i" class="dot" :style="{ '--i': i }" />
    </div>

    <!-- Bars variant -->
    <div v-else-if="variant === 'bars'" class="loader-bars">
      <div v-for="i in 5" :key="i" class="bar" :style="{ '--i': i }" />
    </div>

    <!-- Pacman variant -->
    <div v-else-if="variant === 'pacman'" class="loader-pacman">
      <div class="pacman" />
      <div class="pellets">
        <div v-for="i in 3" :key="i" class="pellet" :style="{ '--i': i }" />
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
  gap: 12px;
}

.loader-text {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--loader-color);
  text-shadow: 0 0 8px var(--loader-color);
  animation: text-blink 1s steps(1) infinite;
}

@keyframes text-blink {
  0%, 70% { opacity: 1; }
  71%, 100% { opacity: 0; }
}

/* Spinner */
.loader-spinner {
  position: relative;
}

.pixel-loader.sm .loader-spinner { width: 24px; height: 24px; }
.pixel-loader.md .loader-spinner { width: 40px; height: 40px; }
.pixel-loader.lg .loader-spinner { width: 56px; height: 56px; }

.spinner-segment {
  position: absolute;
  width: 20%;
  height: 20%;
  background: var(--loader-color);
  animation: spinner-fade 0.8s steps(1) infinite;
  animation-delay: calc(var(--i) * 0.1s);
  opacity: 0.2;
}

.spinner-segment:nth-child(1) { top: 0; left: 40%; }
.spinner-segment:nth-child(2) { top: 10%; right: 10%; }
.spinner-segment:nth-child(3) { top: 40%; right: 0; }
.spinner-segment:nth-child(4) { bottom: 10%; right: 10%; }
.spinner-segment:nth-child(5) { bottom: 0; left: 40%; }
.spinner-segment:nth-child(6) { bottom: 10%; left: 10%; }
.spinner-segment:nth-child(7) { top: 40%; left: 0; }
.spinner-segment:nth-child(8) { top: 10%; left: 10%; }

@keyframes spinner-fade {
  0%, 12.5% { opacity: 1; box-shadow: 0 0 8px var(--loader-color); }
  12.6%, 100% { opacity: 0.2; box-shadow: none; }
}

/* Dots */
.loader-dots {
  display: flex;
  gap: 8px;
}

.dot {
  background: var(--loader-color);
  animation: dot-bounce 0.6s steps(4) infinite;
  animation-delay: calc(var(--i) * 0.15s);
}

.pixel-loader.sm .dot { width: 6px; height: 6px; }
.pixel-loader.md .dot { width: 10px; height: 10px; }
.pixel-loader.lg .dot { width: 14px; height: 14px; }

@keyframes dot-bounce {
  0%, 100% { transform: translateY(0); opacity: 0.4; }
  50% { transform: translateY(-8px); opacity: 1; box-shadow: 0 0 10px var(--loader-color); }
}

/* Bars */
.loader-bars {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  height: 32px;
}

.bar {
  background: var(--loader-color);
  animation: bar-grow 0.8s steps(4) infinite;
  animation-delay: calc(var(--i) * 0.1s);
}

.pixel-loader.sm .bar { width: 4px; }
.pixel-loader.md .bar { width: 6px; }
.pixel-loader.lg .bar { width: 8px; }

@keyframes bar-grow {
  0%, 100% { height: 20%; opacity: 0.4; }
  50% { height: 100%; opacity: 1; box-shadow: 0 0 8px var(--loader-color); }
}

/* Pacman */
.loader-pacman {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pacman {
  width: 24px;
  height: 24px;
  background: var(--retro-accent-yellow, #ffd000);
  clip-path: polygon(
    50% 50%,
    100% 0%,
    100% 100%
  );
  animation: pacman-chomp 0.4s steps(2) infinite;
}

@keyframes pacman-chomp {
  0%, 100% { 
    clip-path: polygon(50% 50%, 100% 10%, 100% 90%);
  }
  50% { 
    clip-path: polygon(50% 50%, 100% 35%, 100% 65%);
  }
}

.pellets {
  display: flex;
  gap: 8px;
}

.pellet {
  width: 8px;
  height: 8px;
  background: var(--loader-color);
  animation: pellet-fade 0.6s steps(1) infinite;
  animation-delay: calc(var(--i) * 0.2s);
}

@keyframes pellet-fade {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0.3; }
}
</style>
