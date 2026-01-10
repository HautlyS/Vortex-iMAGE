<script setup lang="ts">
import { computed, ref } from 'vue';

const props = withDefaults(defineProps<{
  modelValue: number;
  min?: number;
  max?: number;
  step?: number;
  label?: string;
  showValue?: boolean;
  variant?: 'default' | 'volume' | 'health' | 'mana';
  disabled?: boolean;
}>(), {
  min: 0,
  max: 100,
  step: 1,
  showValue: true,
  variant: 'default',
  disabled: false
});

const emit = defineEmits<{
  'update:modelValue': [value: number];
}>();

const isDragging = ref(false);

const percentage = computed(() => {
  return ((props.modelValue - props.min) / (props.max - props.min)) * 100;
});

const segments = computed(() => {
  return Math.ceil((props.max - props.min) / props.step);
});

const filledSegments = computed(() => {
  return Math.round((percentage.value / 100) * Math.min(segments.value, 20));
});

const handleInput = (e: Event) => {
  const value = Number((e.target as HTMLInputElement).value);
  emit('update:modelValue', value);
};

const variantColors = {
  default: { fill: '#39ff14', track: '#1a3a1a' },
  volume: { fill: '#0099db', track: '#1a2a3a' },
  health: { fill: '#e43b44', track: '#3a1a1a' },
  mana: { fill: '#9b5de5', track: '#2a1a3a' }
};

const colors = computed(() => variantColors[props.variant]);
</script>

<template>
  <div class="pixel-slider-wrapper" :class="[variant, { disabled, dragging: isDragging }]">
    <label v-if="label" class="slider-label">{{ label }}</label>
    
    <div class="slider-container">
      <div class="slider-track" :style="{ '--fill-color': colors.fill, '--track-color': colors.track }">
        <!-- Visual segments -->
        <div class="track-segments">
          <div 
            v-for="i in Math.min(segments, 20)" 
            :key="i" 
            class="track-segment"
            :class="{ filled: i <= filledSegments }"
          >
            <div class="segment-shine"></div>
          </div>
        </div>
        
        <!-- Actual input -->
        <input
          type="range"
          class="slider-input"
          :value="modelValue"
          :min="min"
          :max="max"
          :step="step"
          :disabled="disabled"
          @input="handleInput"
          @mousedown="isDragging = true"
          @mouseup="isDragging = false"
          @touchstart="isDragging = true"
          @touchend="isDragging = false"
        />
        
        <!-- Thumb -->
        <div 
          class="slider-thumb"
          :style="{ left: `${percentage}%` }"
        >
          <div class="thumb-body">
            <div class="thumb-shine"></div>
            <div class="thumb-grip">
              <span></span>
              <span></span>
              <span></span>
            </div>
          </div>
        </div>
      </div>
      
      <span v-if="showValue" class="slider-value" :style="{ color: colors.fill }">
        {{ modelValue }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.pixel-slider-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
  image-rendering: pixelated;
}

.slider-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #808080;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 16px;
}

.slider-track {
  position: relative;
  flex: 1;
  height: 24px;
  background: #000;
  border: 4px solid #3a3a5c;
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    4px 4px 0 rgba(0, 0, 0, 0.5);
}

.track-segments {
  position: absolute;
  inset: 0;
  display: flex;
  gap: 2px;
  padding: 2px;
}

.track-segment {
  flex: 1;
  background: var(--track-color, #1a1a2e);
  position: relative;
}

.track-segment.filled {
  background: var(--fill-color, #39ff14);
  box-shadow: 0 0 4px var(--fill-color, #39ff14);
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

.track-segment.filled .segment-shine {
  opacity: 1;
}

/* Hidden input */
.slider-input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
  z-index: 10;
}

.slider-input:disabled {
  cursor: not-allowed;
}

/* Thumb */
.slider-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  pointer-events: none;
  z-index: 5;
}

.thumb-body {
  width: 20px;
  height: 32px;
  background: linear-gradient(180deg, #808080 0%, #4a4a4a 100%);
  border: 2px solid #000;
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5);
  position: relative;
}

.pixel-slider-wrapper.dragging .thumb-body {
  background: linear-gradient(180deg, var(--fill-color, #39ff14) 0%, #2d8a1a 100%);
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 12px var(--fill-color, #39ff14);
}

.thumb-shine {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 6px;
  height: 6px;
  background: rgba(255, 255, 255, 0.4);
}

.thumb-grip {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.thumb-grip span {
  width: 8px;
  height: 2px;
  background: rgba(0, 0, 0, 0.3);
}

/* Value display */
.slider-value {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  min-width: 48px;
  text-align: right;
  text-shadow: 0 0 8px currentColor;
}

/* Disabled state */
.pixel-slider-wrapper.disabled {
  opacity: 0.5;
}

.pixel-slider-wrapper.disabled .slider-track {
  cursor: not-allowed;
}

/* Volume variant - horizontal bars */
.pixel-slider-wrapper.volume .track-segment.filled {
  background: linear-gradient(180deg, #66ccff 0%, #0099db 100%);
}

/* Health variant */
.pixel-slider-wrapper.health .track-segment.filled {
  background: linear-gradient(180deg, #ff6b6b 0%, #e43b44 100%);
}

/* Mana variant */
.pixel-slider-wrapper.mana .track-segment.filled {
  background: linear-gradient(180deg, #b87dff 0%, #9b5de5 100%);
}

/* Focus state */
.slider-input:focus-visible + .slider-thumb .thumb-body {
  outline: 4px solid var(--fill-color, #39ff14);
  outline-offset: 2px;
}
</style>
