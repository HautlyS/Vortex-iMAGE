<script setup lang="ts">
import { computed, type CSSProperties } from 'vue';

const props = withDefaults(defineProps<{
  color?: string;
  thickness?: number;
  animated?: boolean;
  glowIntensity?: number;
  cornerSize?: number;
}>(), {
  color: 'var(--retro-accent-green, #00ff87)',
  thickness: 3,
  animated: true,
  glowIntensity: 1,
  cornerSize: 8
});

const borderStyle = computed<CSSProperties>(() => ({
  '--border-color': props.color,
  '--border-thickness': `${props.thickness}px`,
  '--glow-intensity': props.glowIntensity,
  '--corner-size': `${props.cornerSize}px`
}));
</script>

<template>
  <div class="pixel-border" :class="{ animated }" :style="borderStyle">
    <!-- Corner decorations -->
    <div class="corner tl" />
    <div class="corner tr" />
    <div class="corner bl" />
    <div class="corner br" />
    
    <!-- Animated border segments -->
    <div class="border-segment top" />
    <div class="border-segment right" />
    <div class="border-segment bottom" />
    <div class="border-segment left" />
    
    <!-- Glow layer -->
    <div class="glow-layer" />
    
    <!-- Content -->
    <div class="pixel-border-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.pixel-border {
  position: relative;
  background: var(--retro-bg-panel, #1a1030);
  padding: var(--border-thickness);
}

/* Corners */
.corner {
  position: absolute;
  width: var(--corner-size);
  height: var(--corner-size);
  background: var(--border-color);
  z-index: 2;
  box-shadow: 0 0 calc(8px * var(--glow-intensity)) var(--border-color);
}

.corner.tl { top: calc(var(--corner-size) / -2); left: calc(var(--corner-size) / -2); }
.corner.tr { top: calc(var(--corner-size) / -2); right: calc(var(--corner-size) / -2); }
.corner.bl { bottom: calc(var(--corner-size) / -2); left: calc(var(--corner-size) / -2); }
.corner.br { bottom: calc(var(--corner-size) / -2); right: calc(var(--corner-size) / -2); }

/* Border segments */
.border-segment {
  position: absolute;
  background: var(--border-color);
}

.border-segment.top,
.border-segment.bottom {
  left: var(--corner-size);
  right: var(--corner-size);
  height: var(--border-thickness);
}

.border-segment.left,
.border-segment.right {
  top: var(--corner-size);
  bottom: var(--corner-size);
  width: var(--border-thickness);
}

.border-segment.top { top: 0; }
.border-segment.bottom { bottom: 0; }
.border-segment.left { left: 0; }
.border-segment.right { right: 0; }

/* Animation */
.animated .border-segment {
  background: linear-gradient(
    90deg,
    transparent 0%,
    var(--border-color) 20%,
    var(--border-color) 80%,
    transparent 100%
  );
  background-size: 200% 100%;
}

.animated .border-segment.top {
  animation: flow-right 2s linear infinite;
}

.animated .border-segment.bottom {
  animation: flow-left 2s linear infinite;
}

.animated .border-segment.left,
.animated .border-segment.right {
  background: linear-gradient(
    180deg,
    transparent 0%,
    var(--border-color) 20%,
    var(--border-color) 80%,
    transparent 100%
  );
  background-size: 100% 200%;
}

.animated .border-segment.right {
  animation: flow-down 2s linear infinite;
}

.animated .border-segment.left {
  animation: flow-up 2s linear infinite;
}

@keyframes flow-right {
  0% { background-position: -100% 0; }
  100% { background-position: 100% 0; }
}

@keyframes flow-left {
  0% { background-position: 100% 0; }
  100% { background-position: -100% 0; }
}

@keyframes flow-down {
  0% { background-position: 0 -100%; }
  100% { background-position: 0 100%; }
}

@keyframes flow-up {
  0% { background-position: 0 100%; }
  100% { background-position: 0 -100%; }
}

/* Glow layer */
.glow-layer {
  position: absolute;
  inset: -2px;
  border: var(--border-thickness) solid transparent;
  box-shadow: 
    0 0 calc(10px * var(--glow-intensity)) var(--border-color),
    inset 0 0 calc(5px * var(--glow-intensity)) rgba(0, 0, 0, 0.5);
  pointer-events: none;
  opacity: 0.6;
}

/* Content */
.pixel-border-content {
  position: relative;
  z-index: 1;
  background: var(--retro-bg-card, #251842);
}
</style>
