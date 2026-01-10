<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const props = withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg';
  mood?: 'happy' | 'excited' | 'thinking' | 'sleeping';
  animated?: boolean;
}>(), {
  size: 'md',
  mood: 'happy',
  animated: true
});

const frame = ref(0);
let animationInterval: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  if (props.animated) {
    animationInterval = setInterval(() => {
      frame.value = (frame.value + 1) % 4;
    }, 300);
  }
});

onUnmounted(() => {
  if (animationInterval) {
    clearInterval(animationInterval);
  }
});
</script>

<template>
  <div class="pixel-mascot" :class="[size, mood, { animated }]">
    <svg viewBox="0 0 32 32" class="mascot-sprite">
      <!-- Body -->
      <rect x="8" y="12" width="16" height="16" fill="#9b5de5"/>
      <rect x="6" y="14" width="2" height="12" fill="#9b5de5"/>
      <rect x="24" y="14" width="2" height="12" fill="#9b5de5"/>
      
      <!-- Body highlight -->
      <rect x="8" y="12" width="4" height="4" fill="#b87dff"/>
      <rect x="12" y="12" width="2" height="2" fill="#d4a8ff"/>
      
      <!-- Body shadow -->
      <rect x="20" y="24" width="4" height="4" fill="#7b3dc5"/>
      
      <!-- Head -->
      <rect x="6" y="2" width="20" height="14" fill="#f15bb5"/>
      <rect x="4" y="4" width="2" height="10" fill="#f15bb5"/>
      <rect x="26" y="4" width="2" height="10" fill="#f15bb5"/>
      
      <!-- Head highlight -->
      <rect x="6" y="2" width="6" height="4" fill="#ff8ed4"/>
      <rect x="8" y="2" width="2" height="2" fill="#ffb8e8"/>
      
      <!-- Head shadow -->
      <rect x="22" y="12" width="4" height="4" fill="#c93d8f"/>
      
      <!-- Eyes -->
      <template v-if="mood === 'sleeping'">
        <rect x="8" y="8" width="6" height="2" fill="#000"/>
        <rect x="18" y="8" width="6" height="2" fill="#000"/>
      </template>
      <template v-else-if="mood === 'excited'">
        <rect x="10" y="6" width="4" height="6" fill="#fff"/>
        <rect x="18" y="6" width="4" height="6" fill="#fff"/>
        <rect :x="frame % 2 === 0 ? 10 : 12" y="8" width="2" height="2" fill="#000"/>
        <rect :x="frame % 2 === 0 ? 18 : 20" y="8" width="2" height="2" fill="#000"/>
        <!-- Sparkles -->
        <rect x="4" y="2" width="2" height="2" fill="#feae34" :opacity="frame % 2"/>
        <rect x="26" y="4" width="2" height="2" fill="#feae34" :opacity="(frame + 1) % 2"/>
      </template>
      <template v-else>
        <rect x="10" y="6" width="4" height="6" fill="#fff"/>
        <rect x="18" y="6" width="4" height="6" fill="#fff"/>
        <rect :x="10 + (frame % 2)" y="8" width="2" height="2" fill="#000"/>
        <rect :x="18 + (frame % 2)" y="8" width="2" height="2" fill="#000"/>
        <!-- Eye shine -->
        <rect x="10" y="6" width="2" height="2" fill="rgba(255,255,255,0.5)"/>
        <rect x="18" y="6" width="2" height="2" fill="rgba(255,255,255,0.5)"/>
      </template>
      
      <!-- Mouth -->
      <template v-if="mood === 'happy'">
        <rect x="12" y="12" width="8" height="2" fill="#000"/>
        <rect x="10" y="10" width="2" height="2" fill="#000"/>
        <rect x="20" y="10" width="2" height="2" fill="#000"/>
      </template>
      <template v-else-if="mood === 'excited'">
        <rect x="12" y="10" width="8" height="4" fill="#000"/>
        <rect x="14" y="12" width="4" height="2" fill="#e43b44"/>
      </template>
      <template v-else-if="mood === 'thinking'">
        <rect x="14" y="12" width="4" height="2" fill="#000"/>
        <!-- Thought bubble -->
        <rect x="26" y="0" width="4" height="4" fill="#fff"/>
        <rect x="24" y="4" width="2" height="2" fill="#fff"/>
      </template>
      <template v-else-if="mood === 'sleeping'">
        <rect x="14" y="12" width="4" height="2" fill="#000"/>
        <!-- Z's -->
        <text x="26" y="6" fill="#fff" font-family="'Press Start 2P'" font-size="4" :opacity="frame % 2">Z</text>
      </template>
      
      <!-- Arms -->
      <rect x="2" :y="16 + (frame % 2) * 2" width="4" height="6" fill="#9b5de5"/>
      <rect x="26" :y="16 + ((frame + 1) % 2) * 2" width="4" height="6" fill="#9b5de5"/>
      
      <!-- Feet -->
      <rect x="8" y="28" width="6" height="4" fill="#7b3dc5"/>
      <rect x="18" y="28" width="6" height="4" fill="#7b3dc5"/>
      
      <!-- Antenna -->
      <rect x="14" y="0" width="4" height="4" fill="#39ff14"/>
      <rect x="15" y="0" width="2" height="2" fill="#8cff7a"/>
    </svg>
    
    <!-- Shadow -->
    <div class="mascot-shadow" :class="{ bounce: animated }"></div>
  </div>
</template>

<style scoped>
.pixel-mascot {
  display: flex;
  flex-direction: column;
  align-items: center;
  image-rendering: pixelated;
}

.mascot-sprite {
  filter: drop-shadow(2px 2px 0 #000);
}

.pixel-mascot.sm .mascot-sprite { width: 48px; height: 48px; }
.pixel-mascot.md .mascot-sprite { width: 64px; height: 64px; }
.pixel-mascot.lg .mascot-sprite { width: 96px; height: 96px; }

.mascot-shadow {
  width: 60%;
  height: 8px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 50%;
  margin-top: 4px;
}

.pixel-mascot.animated .mascot-sprite {
  animation: mascot-bounce 0.6s steps(2) infinite;
}

.pixel-mascot.animated .mascot-shadow.bounce {
  animation: shadow-pulse 0.6s steps(2) infinite;
}

@keyframes mascot-bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}

@keyframes shadow-pulse {
  0%, 100% { transform: scaleX(1); opacity: 0.3; }
  50% { transform: scaleX(0.8); opacity: 0.2; }
}

/* Mood-specific animations */
.pixel-mascot.excited .mascot-sprite {
  animation: mascot-jump 0.4s steps(2) infinite;
}

@keyframes mascot-jump {
  0%, 100% { transform: translateY(0) rotate(-2deg); }
  50% { transform: translateY(-8px) rotate(2deg); }
}

.pixel-mascot.sleeping .mascot-sprite {
  animation: mascot-sleep 2s steps(4) infinite;
}

@keyframes mascot-sleep {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(2px); }
}

.pixel-mascot.thinking .mascot-sprite {
  animation: mascot-think 1s steps(2) infinite;
}

@keyframes mascot-think {
  0%, 100% { transform: rotate(0deg); }
  25% { transform: rotate(-3deg); }
  75% { transform: rotate(3deg); }
}
</style>
