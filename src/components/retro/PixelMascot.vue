<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';

const props = withDefaults(defineProps<{
  mood?: 'happy' | 'thinking' | 'working' | 'sleeping' | 'excited';
  size?: number;
  color?: string;
  animated?: boolean;
}>(), {
  mood: 'happy',
  size: 48,
  color: 'var(--retro-accent-green, #00ff87)',
  animated: true
});

const frame = ref(0);
let interval: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  if (props.animated) {
    interval = setInterval(() => {
      frame.value = (frame.value + 1) % 4;
    }, 400);
  }
});

onUnmounted(() => {
  if (interval) clearInterval(interval);
});

const eyeOffset = computed(() => frame.value % 2 === 0 ? 0 : 1);
const bounceOffset = computed(() => (frame.value === 1 || frame.value === 3) ? -1 : 0);
</script>

<template>
  <div 
    class="pixel-mascot"
    :style="{ 
      width: size + 'px', 
      height: size + 'px',
      '--mascot-color': color
    }"
  >
    <svg 
      viewBox="0 0 16 16" 
      class="mascot-svg"
      :style="{ transform: `translateY(${bounceOffset}px)` }"
    >
      <!-- Body -->
      <rect x="4" y="8" width="8" height="6" class="body-main" />
      <rect x="5" y="9" width="6" height="4" class="body-dark" />
      
      <!-- Head -->
      <rect x="3" y="2" width="10" height="6" class="head-main" />
      <rect x="4" y="3" width="8" height="4" class="head-inner" />
      
      <!-- Eyes -->
      <rect 
        :x="5 + eyeOffset" 
        y="4" 
        width="2" 
        height="2" 
        class="eye" 
      />
      <rect 
        :x="9 + eyeOffset" 
        y="4" 
        width="2" 
        height="2" 
        class="eye" 
      />
      
      <!-- Eye shine -->
      <rect x="5" y="4" width="1" height="1" class="eye-shine" />
      <rect x="9" y="4" width="1" height="1" class="eye-shine" />
      
      <!-- Antenna -->
      <rect x="7" y="0" width="2" height="2" class="antenna" />
      <rect 
        x="7" 
        :y="frame === 1 || frame === 3 ? -1 : 0" 
        width="2" 
        height="1" 
        :class="frame === 1 || frame === 3 ? 'antenna-glow' : 'antenna'" 
      />
      
      <!-- Arms -->
      <rect x="2" :y="10 + (frame % 2)" width="2" height="3" class="limb" />
      <rect x="12" :y="10 + ((frame + 1) % 2)" width="2" height="3" class="limb" />
      
      <!-- Legs -->
      <rect x="5" y="14" width="2" height="2" class="limb" />
      <rect x="9" y="14" width="2" height="2" class="limb" />
      
      <!-- Mouth based on mood -->
      <template v-if="mood === 'happy'">
        <rect x="6" y="6" width="4" height="1" class="mouth" />
        <rect x="5" y="5" width="1" height="1" class="mouth" />
        <rect x="10" y="5" width="1" height="1" class="mouth" />
      </template>
      <template v-else-if="mood === 'thinking'">
        <rect x="6" y="6" width="3" height="1" class="mouth" />
        <rect x="10" y="3" width="2" height="2" class="thought-bubble" />
      </template>
      <template v-else-if="mood === 'working'">
        <rect x="6" y="5" width="4" height="2" class="mouth" />
      </template>
      <template v-else-if="mood === 'sleeping'">
        <rect x="5" y="4" width="2" height="1" class="eye-closed" />
        <rect x="9" y="4" width="2" height="1" class="eye-closed" />
        <rect x="7" y="6" width="2" height="1" class="mouth" />
        <text x="12" y="3" class="zzz" font-size="2">Z</text>
      </template>
      <template v-else-if="mood === 'excited'">
        <rect x="6" y="5" width="4" height="2" class="mouth-open" />
        <rect x="3" y="1" width="1" height="1" class="sparkle" />
        <rect x="12" y="1" width="1" height="1" class="sparkle" />
      </template>
    </svg>
  </div>
</template>

<style scoped>
.pixel-mascot {
  display: inline-block;
  image-rendering: pixelated;
}

.mascot-svg {
  width: 100%;
  height: 100%;
  transition: transform 0.1s steps(2);
}

.body-main { fill: var(--retro-bg-lighter, #2d1f4d); }
.body-dark { fill: var(--retro-bg-dark, #0f0a1e); }

.head-main { fill: var(--mascot-color); }
.head-inner { fill: color-mix(in srgb, var(--mascot-color) 70%, black); }

.eye { fill: #000; }
.eye-shine { fill: #fff; }
.eye-closed { fill: #000; }

.antenna { fill: var(--retro-accent-blue, #00d4ff); }
.antenna-glow { 
  fill: var(--retro-accent-yellow, #ffd000);
  filter: drop-shadow(0 0 2px var(--retro-accent-yellow));
}

.limb { fill: var(--retro-bg-lighter, #2d1f4d); }

.mouth { fill: #000; }
.mouth-open { 
  fill: var(--retro-accent-pink, #ff2d95);
}

.thought-bubble { 
  fill: var(--retro-accent-blue, #00d4ff);
  opacity: 0.8;
}

.sparkle {
  fill: var(--retro-accent-yellow, #ffd000);
  animation: sparkle-blink 0.3s steps(1) infinite;
}

.zzz {
  fill: var(--retro-accent-blue, #00d4ff);
  font-family: 'Press Start 2P', monospace;
  animation: float-up 1s steps(4) infinite;
}

@keyframes sparkle-blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

@keyframes float-up {
  0%, 100% { transform: translateY(0); opacity: 1; }
  50% { transform: translateY(-2px); opacity: 0.5; }
}
</style>
