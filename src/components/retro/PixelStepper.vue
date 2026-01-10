<script setup lang="ts">
import { computed } from 'vue';

interface Step {
  id: string | number;
  label: string;
  description?: string;
  icon?: string;
}

const props = withDefaults(defineProps<{
  steps: Step[];
  currentStep: number;
  variant?: 'default' | 'compact' | 'vertical';
}>(), {
  variant: 'default'
});

const emit = defineEmits<{
  'step-click': [index: number];
}>();

const getStepStatus = (index: number) => {
  if (index < props.currentStep) return 'completed';
  if (index === props.currentStep) return 'current';
  return 'upcoming';
};

const progress = computed(() => {
  return (props.currentStep / (props.steps.length - 1)) * 100;
});
</script>

<template>
  <div class="pixel-stepper" :class="[variant]">
    <!-- Progress track -->
    <div class="stepper-track">
      <div class="track-bg"></div>
      <div class="track-fill" :style="{ width: `${progress}%` }"></div>
    </div>
    
    <!-- Steps -->
    <div class="stepper-steps">
      <div 
        v-for="(step, index) in steps" 
        :key="step.id"
        class="stepper-step"
        :class="getStepStatus(index)"
        @click="emit('step-click', index)"
      >
        <!-- Step indicator -->
        <div class="step-indicator">
          <!-- Completed checkmark -->
          <svg v-if="getStepStatus(index) === 'completed'" class="step-icon" viewBox="0 0 16 16">
            <rect x="2" y="8" width="4" height="4" fill="currentColor"/>
            <rect x="6" y="10" width="4" height="4" fill="currentColor"/>
            <rect x="10" y="4" width="4" height="4" fill="currentColor"/>
            <rect x="14" y="0" width="2" height="4" fill="currentColor"/>
          </svg>
          
          <!-- Current - pulsing dot -->
          <div v-else-if="getStepStatus(index) === 'current'" class="step-current">
            <span class="pulse-ring"></span>
            <span class="pulse-dot"></span>
          </div>
          
          <!-- Upcoming - number -->
          <span v-else class="step-number">{{ index + 1 }}</span>
          
          <!-- Corner decorations -->
          <div class="indicator-corner tl"></div>
          <div class="indicator-corner tr"></div>
          <div class="indicator-corner bl"></div>
          <div class="indicator-corner br"></div>
        </div>
        
        <!-- Step content -->
        <div class="step-content">
          <span class="step-label">{{ step.label }}</span>
          <span v-if="step.description && variant !== 'compact'" class="step-description">
            {{ step.description }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pixel-stepper {
  position: relative;
  image-rendering: pixelated;
}

/* Track */
.stepper-track {
  position: absolute;
  top: 24px;
  left: 32px;
  right: 32px;
  height: 8px;
  background: #000;
  border: 2px solid #3a3a5c;
}

.track-bg {
  position: absolute;
  inset: 0;
  background: #1a1a2e;
}

.track-fill {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  background: linear-gradient(90deg, #39ff14 0%, #8cff7a 100%);
  box-shadow: 0 0 8px rgba(57, 255, 20, 0.5);
  transition: width 0.3s steps(8);
}

/* Steps container */
.stepper-steps {
  display: flex;
  justify-content: space-between;
  position: relative;
  z-index: 1;
}

/* Individual step */
.stepper-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  flex: 1;
}

/* Step indicator */
.step-indicator {
  position: relative;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1a1a2e;
  border: 4px solid #000;
  box-shadow: 4px 4px 0 rgba(0, 0, 0, 0.5);
}

.stepper-step.completed .step-indicator {
  background: linear-gradient(180deg, #39ff14 0%, #2d8a1a 100%);
  color: #000;
}

.stepper-step.current .step-indicator {
  background: linear-gradient(180deg, #0099db 0%, #006b99 100%);
  border-color: #0099db;
  box-shadow: 4px 4px 0 rgba(0, 0, 0, 0.5), 0 0 16px rgba(0, 153, 219, 0.5);
}

.stepper-step.upcoming .step-indicator {
  background: #2a2a4c;
  color: #808080;
}

/* Indicator corners */
.indicator-corner {
  position: absolute;
  width: 4px;
  height: 4px;
  background: #3a3a5c;
}

.indicator-corner.tl { top: -4px; left: -4px; }
.indicator-corner.tr { top: -4px; right: -4px; }
.indicator-corner.bl { bottom: -4px; left: -4px; }
.indicator-corner.br { bottom: -4px; right: -4px; }

.stepper-step.completed .indicator-corner { background: #39ff14; }
.stepper-step.current .indicator-corner { background: #0099db; }

/* Step icon */
.step-icon {
  width: 20px;
  height: 20px;
}

/* Step number */
.step-number {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
}

/* Current step pulse */
.step-current {
  position: relative;
  width: 16px;
  height: 16px;
}

.pulse-ring {
  position: absolute;
  inset: -4px;
  border: 2px solid #fff;
  animation: pulse-ring 1s steps(4) infinite;
}

.pulse-dot {
  position: absolute;
  inset: 0;
  background: #fff;
  animation: pulse-dot 1s steps(2) infinite;
}

@keyframes pulse-ring {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.5); opacity: 0; }
}

@keyframes pulse-dot {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(0.8); }
}

/* Step content */
.step-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-align: center;
}

.step-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #808080;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.stepper-step.completed .step-label,
.stepper-step.current .step-label {
  color: #fff;
}

.step-description {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: #4a4a6c;
  max-width: 120px;
}

.stepper-step.current .step-description {
  color: #808080;
}

/* Compact variant */
.pixel-stepper.compact .stepper-track {
  top: 16px;
  left: 20px;
  right: 20px;
  height: 4px;
}

.pixel-stepper.compact .step-indicator {
  width: 32px;
  height: 32px;
}

.pixel-stepper.compact .step-number {
  font-size: 10px;
}

.pixel-stepper.compact .step-icon {
  width: 14px;
  height: 14px;
}

.pixel-stepper.compact .step-content {
  display: none;
}

/* Vertical variant */
.pixel-stepper.vertical {
  display: flex;
  flex-direction: column;
}

.pixel-stepper.vertical .stepper-track {
  position: absolute;
  top: 32px;
  bottom: 32px;
  left: 24px;
  right: auto;
  width: 8px;
  height: auto;
}

.pixel-stepper.vertical .track-fill {
  width: 100%;
  height: var(--progress);
}

.pixel-stepper.vertical .stepper-steps {
  flex-direction: column;
  gap: 24px;
}

.pixel-stepper.vertical .stepper-step {
  flex-direction: row;
  align-items: flex-start;
  gap: 16px;
}

.pixel-stepper.vertical .step-content {
  align-items: flex-start;
  text-align: left;
}

/* Hover states */
.stepper-step:hover .step-indicator {
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 rgba(0, 0, 0, 0.5);
}

.stepper-step.current:hover .step-indicator {
  box-shadow: 6px 6px 0 rgba(0, 0, 0, 0.5), 0 0 20px rgba(0, 153, 219, 0.6);
}
</style>
