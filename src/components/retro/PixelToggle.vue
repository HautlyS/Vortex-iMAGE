<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  modelValue?: boolean;
  disabled?: boolean;
  size?: 'sm' | 'md' | 'lg';
  onColor?: string;
  offColor?: string;
}>(), {
  modelValue: false,
  disabled: false,
  size: 'md',
  onColor: 'var(--retro-accent-green, #00ff87)',
  offColor: 'var(--retro-bg-lighter, #2d1f4d)'
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const toggle = () => {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue);
  }
};

const toggleStyle = computed(() => ({
  '--on-color': props.onColor,
  '--off-color': props.offColor
}));
</script>

<template>
  <button
    class="pixel-toggle"
    :class="[size, { active: modelValue, disabled }]"
    :style="toggleStyle"
    :disabled="disabled"
    @click="toggle"
    role="switch"
    :aria-checked="modelValue"
  >
    <div class="pixel-toggle-track">
      <div class="pixel-toggle-segments">
        <div v-for="i in 4" :key="i" class="segment" />
      </div>
    </div>
    <div class="pixel-toggle-thumb">
      <div class="thumb-face" />
    </div>
    <div class="pixel-toggle-label">
      {{ modelValue ? 'ON' : 'OFF' }}
    </div>
  </button>
</template>

<style scoped>
.pixel-toggle {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0;
}

.pixel-toggle.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pixel-toggle-track {
  position: relative;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 3px solid #000;
  transition: all 0.1s steps(2);
  overflow: hidden;
}

.pixel-toggle.sm .pixel-toggle-track {
  width: 40px;
  height: 20px;
}

.pixel-toggle.md .pixel-toggle-track {
  width: 52px;
  height: 26px;
}

.pixel-toggle.lg .pixel-toggle-track {
  width: 64px;
  height: 32px;
}

.pixel-toggle.active .pixel-toggle-track {
  background: var(--on-color);
  box-shadow: 0 0 10px var(--on-color);
}

.pixel-toggle-segments {
  position: absolute;
  inset: 2px;
  display: flex;
  gap: 2px;
}

.segment {
  flex: 1;
  background: rgba(0, 0, 0, 0.2);
}

.pixel-toggle-thumb {
  position: absolute;
  background: var(--off-color);
  border: 2px solid #000;
  transition: all 0.15s steps(4);
  display: flex;
  align-items: center;
  justify-content: center;
}

.pixel-toggle.sm .pixel-toggle-thumb {
  width: 16px;
  height: 16px;
  left: 5px;
  top: 50%;
  transform: translateY(-50%);
}

.pixel-toggle.md .pixel-toggle-thumb {
  width: 20px;
  height: 20px;
  left: 6px;
  top: 50%;
  transform: translateY(-50%);
}

.pixel-toggle.lg .pixel-toggle-thumb {
  width: 24px;
  height: 24px;
  left: 7px;
  top: 50%;
  transform: translateY(-50%);
}

.pixel-toggle.active .pixel-toggle-thumb {
  background: #fff;
  box-shadow: 0 0 8px var(--on-color);
}

.pixel-toggle.sm.active .pixel-toggle-thumb {
  left: 21px;
}

.pixel-toggle.md.active .pixel-toggle-thumb {
  left: 26px;
}

.pixel-toggle.lg.active .pixel-toggle-thumb {
  left: 33px;
}

.thumb-face {
  width: 4px;
  height: 4px;
  background: #000;
}

.pixel-toggle.active .thumb-face {
  background: var(--on-color);
}

.pixel-toggle-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--retro-text-muted, #9d8ec2);
  transition: color 0.1s steps(2);
  min-width: 24px;
}

.pixel-toggle.active .pixel-toggle-label {
  color: var(--on-color);
  text-shadow: 0 0 8px var(--on-color);
}
</style>
