<script setup lang="ts">
import { ref, computed, useSlots, watch, nextTick, useTemplateRef } from 'vue';

const props = withDefaults(defineProps<{
  initialStep?: number;
  backText?: string;
  nextText?: string;
  completeText?: string;
}>(), {
  initialStep: 1,
  backText: '< BACK',
  nextText: 'NEXT >',
  completeText: 'COMPLETE!'
});

const emit = defineEmits<{
  stepChange: [step: number];
  complete: [];
}>();

const slots = useSlots();
const currentStep = ref(props.initialStep);
const isCompleted = ref(false);
const contentRef = useTemplateRef<HTMLDivElement>('contentRef');
const contentHeight = ref(0);

const stepsArray = computed(() => slots.default?.() || []);
const totalSteps = computed(() => stepsArray.value.length);
const isLastStep = computed(() => currentStep.value === totalSteps.value);

const getStepStatus = (step: number) => {
  if (isCompleted.value || currentStep.value > step) return 'complete';
  if (currentStep.value === step) return 'active';
  return 'inactive';
};

const measureHeight = () => {
  nextTick(() => {
    if (contentRef.value) {
      contentHeight.value = contentRef.value.offsetHeight;
    }
  });
};

const goToStep = (step: number) => {
  if (isCompleted.value) return;
  if (step >= 1 && step <= totalSteps.value) {
    currentStep.value = step;
    emit('stepChange', step);
    measureHeight();
  }
};

const handleBack = () => goToStep(currentStep.value - 1);
const handleNext = () => {
  if (isLastStep.value) {
    isCompleted.value = true;
    emit('complete');
  } else {
    goToStep(currentStep.value + 1);
  }
};

watch(currentStep, measureHeight);
</script>

<template>
  <div class="pixel-stepper">
    <!-- Step indicators -->
    <div class="pixel-stepper-header">
      <template v-for="(_, index) in stepsArray" :key="index">
        <button
          class="pixel-step-indicator"
          :class="getStepStatus(index + 1)"
          @click="goToStep(index + 1)"
          :disabled="isCompleted"
        >
          <span v-if="getStepStatus(index + 1) === 'complete'" class="check">✓</span>
          <span v-else>{{ index + 1 }}</span>
        </button>
        <div 
          v-if="index < totalSteps - 1" 
          class="pixel-step-connector"
          :class="{ filled: currentStep > index + 1 }"
        >
          <div class="connector-fill" />
        </div>
      </template>
    </div>

    <!-- Content area -->
    <div 
      class="pixel-stepper-content"
      :style="{ height: isCompleted ? '0px' : `${contentHeight}px` }"
    >
      <div ref="contentRef" v-if="!isCompleted">
        <component 
          v-if="stepsArray[currentStep - 1]" 
          :is="stepsArray[currentStep - 1]" 
        />
      </div>
    </div>

    <!-- Completed state -->
    <div v-if="isCompleted" class="pixel-stepper-complete">
      <div class="complete-icon">★</div>
      <span>ALL STEPS COMPLETE!</span>
    </div>

    <!-- Navigation -->
    <div v-if="!isCompleted" class="pixel-stepper-footer">
      <button 
        v-if="currentStep > 1"
        class="pixel-stepper-btn back"
        @click="handleBack"
      >
        {{ backText }}
      </button>
      <div v-else />
      <button 
        class="pixel-stepper-btn next"
        @click="handleNext"
      >
        {{ isLastStep ? completeText : nextText }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.pixel-stepper {
  background: var(--retro-bg-panel, #1a1030);
  border: 3px solid #000;
  box-shadow: 6px 6px 0 rgba(0, 0, 0, 0.6);
  padding: 20px;
}

.pixel-stepper-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
  margin-bottom: 24px;
}

.pixel-step-indicator {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  border: 3px solid #000;
  background: var(--retro-bg-dark, #0f0a1e);
  color: var(--retro-text-muted, #9d8ec2);
  cursor: pointer;
  transition: all 0.1s steps(2);
  box-shadow: 2px 2px 0 #000;
}

.pixel-step-indicator:hover:not(:disabled) {
  transform: translate(-1px, -1px);
  box-shadow: 3px 3px 0 #000;
}

.pixel-step-indicator.active {
  background: var(--retro-accent-pink, #ff2d95);
  color: #fff;
  box-shadow: 2px 2px 0 #000, 0 0 10px var(--retro-accent-pink);
}

.pixel-step-indicator.complete {
  background: var(--retro-accent-green, #00ff87);
  color: #000;
  box-shadow: 2px 2px 0 #000, 0 0 10px var(--retro-accent-green);
}

.pixel-step-indicator .check {
  font-size: 12px;
}

.pixel-step-connector {
  width: 40px;
  height: 4px;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 1px solid #000;
  position: relative;
  overflow: hidden;
}

.connector-fill {
  position: absolute;
  inset: 0;
  background: var(--retro-accent-green, #00ff87);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s steps(4);
}

.pixel-step-connector.filled .connector-fill {
  transform: scaleX(1);
}

.pixel-stepper-content {
  overflow: hidden;
  transition: height 0.3s steps(6);
  margin-bottom: 20px;
}

.pixel-stepper-complete {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px;
  color: var(--retro-accent-green, #00ff87);
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  text-shadow: 0 0 10px var(--retro-accent-green);
  animation: complete-pulse 1s steps(4) infinite;
}

.complete-icon {
  font-size: 32px;
  animation: spin 2s steps(8) infinite;
}

@keyframes complete-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.pixel-stepper-footer {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.pixel-stepper-btn {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  padding: 12px 20px;
  border: 2px solid #000;
  cursor: pointer;
  transition: all 0.1s steps(2);
}

.pixel-stepper-btn.back {
  background: var(--retro-bg-lighter, #2d1f4d);
  color: var(--retro-text-muted, #9d8ec2);
  box-shadow: 2px 2px 0 #000;
}

.pixel-stepper-btn.back:hover {
  color: var(--retro-text-main, #fff);
  transform: translate(-1px, -1px);
  box-shadow: 3px 3px 0 #000;
}

.pixel-stepper-btn.next {
  background: linear-gradient(180deg, var(--retro-accent-green, #00ff87), #00cc6a);
  color: #000;
  box-shadow: 3px 3px 0 #000;
}

.pixel-stepper-btn.next:hover {
  transform: translate(-2px, -2px);
  box-shadow: 5px 5px 0 #000, 0 0 15px var(--retro-accent-green);
}
</style>
