<script setup lang="ts">
import { ref, computed } from 'vue';

const props = withDefaults(defineProps<{
  content: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
  color?: string;
  delay?: number;
}>(), {
  position: 'top',
  color: 'var(--retro-accent-pink, #ff2d95)',
  delay: 200
});

const isVisible = ref(false);
let showTimeout: ReturnType<typeof setTimeout> | null = null;

const show = () => {
  showTimeout = setTimeout(() => {
    isVisible.value = true;
  }, props.delay);
};

const hide = () => {
  if (showTimeout) clearTimeout(showTimeout);
  isVisible.value = false;
};

const tooltipStyle = computed(() => ({
  '--tooltip-color': props.color
}));
</script>

<template>
  <div 
    class="pixel-tooltip-wrapper"
    @mouseenter="show"
    @mouseleave="hide"
    @focus="show"
    @blur="hide"
  >
    <slot />
    <Transition name="pixel-fade">
      <div 
        v-if="isVisible" 
        class="pixel-tooltip"
        :class="position"
        :style="tooltipStyle"
      >
        <div class="pixel-tooltip-content">
          {{ content }}
        </div>
        <div class="pixel-tooltip-arrow" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.pixel-tooltip-wrapper {
  position: relative;
  display: inline-block;
}

.pixel-tooltip {
  position: absolute;
  z-index: 1000;
  pointer-events: none;
}

.pixel-tooltip-content {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  padding: 8px 12px;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 2px solid var(--tooltip-color);
  color: var(--retro-text-main, #fff);
  white-space: nowrap;
  box-shadow: 
    4px 4px 0 #000,
    0 0 10px var(--tooltip-color);
  text-shadow: 0 0 4px var(--tooltip-color);
}

.pixel-tooltip-arrow {
  position: absolute;
  width: 8px;
  height: 8px;
  background: var(--tooltip-color);
}

/* Positions */
.pixel-tooltip.top {
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 8px;
}

.pixel-tooltip.top .pixel-tooltip-arrow {
  bottom: -4px;
  left: 50%;
  transform: translateX(-50%) rotate(45deg);
}

.pixel-tooltip.bottom {
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 8px;
}

.pixel-tooltip.bottom .pixel-tooltip-arrow {
  top: -4px;
  left: 50%;
  transform: translateX(-50%) rotate(45deg);
}

.pixel-tooltip.left {
  right: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-right: 8px;
}

.pixel-tooltip.left .pixel-tooltip-arrow {
  right: -4px;
  top: 50%;
  transform: translateY(-50%) rotate(45deg);
}

.pixel-tooltip.right {
  left: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-left: 8px;
}

.pixel-tooltip.right .pixel-tooltip-arrow {
  left: -4px;
  top: 50%;
  transform: translateY(-50%) rotate(45deg);
}

/* Animation */
.pixel-fade-enter-active,
.pixel-fade-leave-active {
  transition: opacity 0.15s steps(4), transform 0.15s steps(4);
}

.pixel-fade-enter-from,
.pixel-fade-leave-to {
  opacity: 0;
}

.pixel-tooltip.top.pixel-fade-enter-from,
.pixel-tooltip.top.pixel-fade-leave-to {
  transform: translateX(-50%) translateY(4px);
}

.pixel-tooltip.bottom.pixel-fade-enter-from,
.pixel-tooltip.bottom.pixel-fade-leave-to {
  transform: translateX(-50%) translateY(-4px);
}
</style>
