<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  value: number;
  max?: number;
  showLabel?: boolean;
  color?: string;
  height?: number;
  animated?: boolean;
  segments?: number;
  variant?: 'default' | 'health' | 'mana' | 'exp' | 'rainbow';
}>(), {
  max: 100,
  showLabel: true,
  color: '#39ff14',
  height: 24,
  animated: true,
  segments: 10,
  variant: 'default'
});

const percentage = computed(() => Math.min(100, Math.max(0, (props.value / props.max) * 100)));
const filledSegments = computed(() => Math.floor((percentage.value / 100) * props.segments));

const variantColors = {
  default: { fill: '#39ff14', bg: '#1a3a1a' },
  health: { fill: '#e43b44', bg: '#3a1a1a' },
  mana: { fill: '#0099db', bg: '#1a2a3a' },
  exp: { fill: '#feae34', bg: '#3a2a1a' },
  rainbow: { fill: 'rainbow', bg: '#2a2a2a' }
};

const colors = computed(() => variantColors[props.variant] || variantColors.default);
</script>

<template>
  <div class="pixel-progress-wrapper">
    <div 
      class="pixel-progress" 
      :class="[variant, { animated }]"
      :style="{ 
        height: `${height}px`, 
        '--bar-color': color,
        '--fill-color': colors.fill,
        '--bg-color': colors.bg
      }"
    >
      <!-- Border frame -->
      <div class="progress-frame">
        <div class="frame-corner tl"></div>
        <div class="frame-corner tr"></div>
        <div class="frame-corner bl"></div>
        <div class="frame-corner br"></div>
      </div>
      
      <!-- Segments -->
      <div class="pixel-progress-track">
        <div 
          v-for="i in segments" 
          :key="i"
          class="pixel-segment"
          :class="{ 
            filled: i <= filledSegments,
            'last-filled': i === filledSegments && animated
          }"
          :style="{ 
            animationDelay: `${i * 0.05}s`,
            '--segment-index': i
          }"
        >
          <div class="segment-shine"></div>
        </div>
      </div>
      
      <!-- Shine overlay -->
      <div class="progress-shine"></div>
    </div>
    
    <span v-if="showLabel" class="pixel-progress-label" :style="{ color: colors.fill }">
      {{ Math.round(percentage) }}%
    </span>
  </div>
</template>

<style scoped>
.pixel-progress-wrapper {
  display: flex;
  align-items: center;
  gap: 16px;
  image-rendering: pixelated;
}

.pixel-progress {
  flex: 1;
  position: relative;
  background: #000;
  border: 4px solid #000;
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    4px 4px 0 rgba(0, 0, 0, 0.5);
}

/* Frame corners */
.progress-frame {
  position: absolute;
  inset: -4px;
  pointer-events: none;
}

.frame-corner {
  position: absolute;
  width: 8px;
  height: 8px;
  background: #808080;
}

.frame-corner.tl { top: 0; left: 0; }
.frame-corner.tr { top: 0; right: 0; }
.frame-corner.bl { bottom: 0; left: 0; }
.frame-corner.br { bottom: 0; right: 0; }

.pixel-progress-track {
  display: flex;
  gap: 2px;
  padding: 4px;
  height: 100%;
  background: var(--bg-color, #1a1a1a);
}

.pixel-segment {
  flex: 1;
  position: relative;
  background: #2a2a2a;
  border: 1px solid #000;
  transition: none;
}

.pixel-segment.filled {
  background: var(--fill-color, var(--bar-color));
  box-shadow: 0 0 4px var(--fill-color, var(--bar-color));
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

.pixel-segment.filled .segment-shine {
  opacity: 1;
}

/* Animated last segment */
.pixel-segment.last-filled {
  animation: segment-pulse 0.5s steps(2) infinite;
}

@keyframes segment-pulse {
  0%, 100% { 
    box-shadow: 0 0 4px var(--fill-color, var(--bar-color)); 
    filter: brightness(1);
  }
  50% { 
    box-shadow: 0 0 12px var(--fill-color, var(--bar-color)); 
    filter: brightness(1.3);
  }
}

/* Progress shine effect */
.progress-shine {
  position: absolute;
  top: 4px;
  left: 4px;
  right: 4px;
  height: 30%;
  background: linear-gradient(180deg, rgba(255,255,255,0.2) 0%, transparent 100%);
  pointer-events: none;
}

/* Label */
.pixel-progress-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  min-width: 60px;
  text-align: right;
  text-shadow: 0 0 8px currentColor;
}

/* Variants */
.pixel-progress.health .pixel-segment.filled {
  background: linear-gradient(180deg, #ff6b6b 0%, #e43b44 50%, #a82835 100%);
}

.pixel-progress.mana .pixel-segment.filled {
  background: linear-gradient(180deg, #66ccff 0%, #0099db 50%, #006b99 100%);
}

.pixel-progress.exp .pixel-segment.filled {
  background: linear-gradient(180deg, #ffd700 0%, #feae34 50%, #c68b28 100%);
}

.pixel-progress.rainbow .pixel-segment.filled {
  animation: rainbow-segment 2s linear infinite;
  animation-delay: calc(var(--segment-index) * 0.1s);
}

@keyframes rainbow-segment {
  0% { background: #e43b44; }
  16% { background: #f77622; }
  33% { background: #feae34; }
  50% { background: #63c74d; }
  66% { background: #0099db; }
  83% { background: #9b5de5; }
  100% { background: #e43b44; }
}

/* Animated variant */
.pixel-progress.animated .pixel-segment.filled {
  animation: fill-glow 1s steps(4) infinite;
  animation-delay: calc(var(--segment-index) * 0.1s);
}

@keyframes fill-glow {
  0%, 100% { filter: brightness(1); }
  50% { filter: brightness(1.2); }
}
</style>
