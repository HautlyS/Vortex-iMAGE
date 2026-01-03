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
}>(), {
  max: 100,
  showLabel: true,
  color: 'var(--retro-accent-green, #00ff87)',
  height: 20,
  animated: true,
  segments: 10
});

const percentage = computed(() => Math.min(100, Math.max(0, (props.value / props.max) * 100)));
const filledSegments = computed(() => Math.floor((percentage.value / 100) * props.segments));
</script>

<template>
  <div class="pixel-progress-wrapper">
    <div 
      class="pixel-progress" 
      :style="{ height: `${height}px`, '--bar-color': color }"
    >
      <div class="pixel-progress-track">
        <div 
          v-for="i in segments" 
          :key="i"
          class="pixel-segment"
          :class="{ 
            filled: i <= filledSegments,
            animated: animated && i <= filledSegments
          }"
          :style="{ animationDelay: `${i * 0.05}s` }"
        />
      </div>
      <div class="pixel-progress-border" />
    </div>
    <span v-if="showLabel" class="pixel-progress-label" :style="{ color }">
      {{ Math.round(percentage) }}%
    </span>
  </div>
</template>

<style scoped>
.pixel-progress-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pixel-progress {
  flex: 1;
  position: relative;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 3px solid #000;
  box-shadow: inset 2px 2px 0 rgba(0, 0, 0, 0.5);
}

.pixel-progress-track {
  display: flex;
  gap: 2px;
  padding: 3px;
  height: 100%;
}

.pixel-segment {
  flex: 1;
  background: var(--retro-bg-lighter, #2d1f4d);
  transition: background 0.1s steps(2);
}

.pixel-segment.filled {
  background: var(--bar-color);
  box-shadow: 0 0 4px var(--bar-color);
}

.pixel-segment.animated {
  animation: segment-pulse 1s steps(4) infinite;
}

@keyframes segment-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.pixel-progress-border {
  position: absolute;
  inset: 0;
  border: 2px solid rgba(255, 255, 255, 0.1);
  pointer-events: none;
}

.pixel-progress-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  min-width: 50px;
  text-align: right;
  text-shadow: 0 0 8px currentColor;
}
</style>
