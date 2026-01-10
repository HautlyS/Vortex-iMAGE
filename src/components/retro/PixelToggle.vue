<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'power' | 'switch';
}>(), {
  disabled: false,
  size: 'md',
  variant: 'default'
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const toggle = () => {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue);
  }
};

const isOn = computed(() => props.modelValue);
</script>

<template>
  <div class="pixel-toggle-wrapper" :class="[size, variant, { disabled }]">
    <span v-if="label" class="toggle-label">{{ label }}</span>
    
    <button 
      class="pixel-toggle"
      :class="{ on: isOn }"
      :disabled="disabled"
      @click="toggle"
      role="switch"
      :aria-checked="isOn"
    >
      <!-- Track -->
      <div class="toggle-track">
        <div class="track-bg"></div>
        <div class="track-fill"></div>
        
        <!-- Power variant indicators -->
        <template v-if="variant === 'power'">
          <span class="power-label off">OFF</span>
          <span class="power-label on">ON</span>
        </template>
        
        <!-- Switch variant notches -->
        <template v-if="variant === 'switch'">
          <div class="switch-notch" v-for="i in 3" :key="i"></div>
        </template>
      </div>
      
      <!-- Thumb -->
      <div class="toggle-thumb">
        <div class="thumb-shine"></div>
        <div class="thumb-shadow"></div>
        
        <!-- Power icon for power variant -->
        <svg v-if="variant === 'power'" class="power-icon" viewBox="0 0 16 16">
          <rect x="7" y="2" width="2" height="6" fill="currentColor"/>
          <rect x="4" y="4" width="2" height="2" fill="currentColor"/>
          <rect x="10" y="4" width="2" height="2" fill="currentColor"/>
          <rect x="2" y="6" width="2" height="4" fill="currentColor"/>
          <rect x="12" y="6" width="2" height="4" fill="currentColor"/>
          <rect x="4" y="10" width="2" height="2" fill="currentColor"/>
          <rect x="10" y="10" width="2" height="2" fill="currentColor"/>
          <rect x="6" y="12" width="4" height="2" fill="currentColor"/>
        </svg>
      </div>
    </button>
  </div>
</template>

<style scoped>
.pixel-toggle-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  image-rendering: pixelated;
}

.toggle-label {
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: #c0c0c0;
}

.pixel-toggle {
  position: relative;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  outline: none;
}

.pixel-toggle:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Track */
.toggle-track {
  position: relative;
  background: #1a1a2e;
  border: 4px solid #000;
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    4px 4px 0 rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.pixel-toggle-wrapper.sm .toggle-track {
  width: 48px;
  height: 24px;
}

.pixel-toggle-wrapper.md .toggle-track {
  width: 64px;
  height: 32px;
}

.pixel-toggle-wrapper.lg .toggle-track {
  width: 80px;
  height: 40px;
}

.track-bg {
  position: absolute;
  inset: 0;
  background: #2a2a4c;
}

.track-fill {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, #39ff14 0%, #2d8a1a 100%);
  transform: translateX(-100%);
  transition: transform 0.1s steps(4);
}

.pixel-toggle.on .track-fill {
  transform: translateX(0);
}

/* Thumb */
.toggle-thumb {
  position: absolute;
  top: 50%;
  left: 4px;
  transform: translateY(-50%);
  background: linear-gradient(180deg, #808080 0%, #4a4a4a 100%);
  border: 2px solid #000;
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5);
  transition: left 0.1s steps(4);
  display: flex;
  align-items: center;
  justify-content: center;
}

.pixel-toggle-wrapper.sm .toggle-thumb {
  width: 16px;
  height: 16px;
}

.pixel-toggle-wrapper.md .toggle-thumb {
  width: 24px;
  height: 24px;
}

.pixel-toggle-wrapper.lg .toggle-thumb {
  width: 32px;
  height: 32px;
}

.pixel-toggle.on .toggle-thumb {
  background: linear-gradient(180deg, #8cff7a 0%, #39ff14 100%);
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 12px rgba(57, 255, 20, 0.5);
}

.pixel-toggle-wrapper.sm .pixel-toggle.on .toggle-thumb {
  left: calc(100% - 20px);
}

.pixel-toggle-wrapper.md .pixel-toggle.on .toggle-thumb {
  left: calc(100% - 28px);
}

.pixel-toggle-wrapper.lg .pixel-toggle.on .toggle-thumb {
  left: calc(100% - 36px);
}

.thumb-shine {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 40%;
  height: 40%;
  background: rgba(255, 255, 255, 0.4);
}

.thumb-shadow {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 40%;
  height: 40%;
  background: rgba(0, 0, 0, 0.3);
}

/* Power variant */
.pixel-toggle-wrapper.power .power-label {
  position: absolute;
  font-family: 'Press Start 2P', monospace;
  font-size: 6px;
  top: 50%;
  transform: translateY(-50%);
  z-index: 1;
}

.power-label.off {
  right: 8px;
  color: #e43b44;
}

.power-label.on {
  left: 8px;
  color: #000;
  opacity: 0;
}

.pixel-toggle.on .power-label.off {
  opacity: 0;
}

.pixel-toggle.on .power-label.on {
  opacity: 1;
}

.power-icon {
  width: 60%;
  height: 60%;
  color: #000;
}

/* Switch variant */
.pixel-toggle-wrapper.switch .toggle-track {
  background: #2a2a4c;
}

.pixel-toggle-wrapper.switch .track-fill {
  background: linear-gradient(180deg, #0099db 0%, #006b99 100%);
}

.pixel-toggle-wrapper.switch .pixel-toggle.on .toggle-thumb {
  background: linear-gradient(180deg, #66ccff 0%, #0099db 100%);
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 12px rgba(0, 153, 219, 0.5);
}

.switch-notch {
  position: absolute;
  width: 4px;
  height: 8px;
  background: rgba(0, 0, 0, 0.3);
  top: 50%;
  transform: translateY(-50%);
}

.switch-notch:nth-child(3) { left: 25%; }
.switch-notch:nth-child(4) { left: 50%; transform: translate(-50%, -50%); }
.switch-notch:nth-child(5) { right: 25%; }

/* Focus state */
.pixel-toggle:focus-visible .toggle-track {
  outline: 4px solid #39ff14;
  outline-offset: 2px;
}
</style>
