<script setup lang="ts">
import { ref, computed, watch, useTemplateRef } from 'vue';

const props = withDefaults(defineProps<{
  modelValue?: number;
  min?: number;
  max?: number;
  step?: number;
  showValue?: boolean;
  color?: string;
}>(), {
  modelValue: 50,
  min: 0,
  max: 100,
  step: 1,
  showValue: true,
  color: 'var(--retro-accent-green, #00ff87)'
});

const emit = defineEmits<{
  'update:modelValue': [value: number];
}>();

const sliderRef = useTemplateRef<HTMLDivElement>('sliderRef');
const isDragging = ref(false);
const localValue = ref(props.modelValue);

watch(() => props.modelValue, (v) => { localValue.value = v; });

const percentage = computed(() => {
  return ((localValue.value - props.min) / (props.max - props.min)) * 100;
});

const updateValue = (clientX: number) => {
  if (!sliderRef.value) return;
  const rect = sliderRef.value.getBoundingClientRect();
  const percent = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
  let newValue = props.min + percent * (props.max - props.min);
  newValue = Math.round(newValue / props.step) * props.step;
  newValue = Math.max(props.min, Math.min(props.max, newValue));
  localValue.value = newValue;
  emit('update:modelValue', newValue);
};

const handlePointerDown = (e: PointerEvent) => {
  isDragging.value = true;
  updateValue(e.clientX);
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
};

const handlePointerMove = (e: PointerEvent) => {
  if (isDragging.value) updateValue(e.clientX);
};

const handlePointerUp = () => {
  isDragging.value = false;
};
</script>

<template>
  <div class="pixel-slider-wrapper">
    <div
      ref="sliderRef"
      class="pixel-slider"
      :class="{ dragging: isDragging }"
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
      @pointerup="handlePointerUp"
    >
      <div class="pixel-slider-track">
        <div 
          class="pixel-slider-fill" 
          :style="{ width: `${percentage}%`, backgroundColor: color }"
        />
        <div class="pixel-slider-segments">
          <div v-for="i in 10" :key="i" class="segment" />
        </div>
      </div>
      <div 
        class="pixel-slider-thumb"
        :style="{ left: `${percentage}%`, borderColor: color, boxShadow: `0 0 8px ${color}` }"
      />
    </div>
    <div v-if="showValue" class="pixel-slider-value" :style="{ color }">
      {{ Math.round(localValue) }}
    </div>
  </div>
</template>

<style scoped>
.pixel-slider-wrapper {
  display: flex;
  align-items: center;
  gap: 16px;
  width: 100%;
}

.pixel-slider {
  position: relative;
  flex: 1;
  height: 24px;
  cursor: pointer;
  touch-action: none;
  user-select: none;
}

.pixel-slider-track {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 8px;
  transform: translateY(-50%);
  background: var(--retro-bg-dark, #0f0a1e);
  border: 2px solid #000;
  overflow: hidden;
}

.pixel-slider-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  transition: width 0.05s steps(4);
}

.pixel-slider-segments {
  position: absolute;
  inset: 0;
  display: flex;
  gap: 2px;
  padding: 0 2px;
}

.segment {
  flex: 1;
  background: rgba(0, 0, 0, 0.3);
}

.pixel-slider-thumb {
  position: absolute;
  top: 50%;
  width: 16px;
  height: 20px;
  transform: translate(-50%, -50%);
  background: var(--retro-bg-lighter, #2d1f4d);
  border: 3px solid;
  transition: transform 0.1s steps(2);
}

.pixel-slider.dragging .pixel-slider-thumb {
  transform: translate(-50%, -50%) scale(1.1);
}

.pixel-slider-value {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  min-width: 40px;
  text-align: right;
  text-shadow: 0 0 8px currentColor;
}
</style>
