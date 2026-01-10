<script setup lang="ts">
import { ref, computed } from 'vue';

const props = withDefaults(defineProps<{
  content: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
  variant?: 'default' | 'info' | 'success' | 'warning' | 'danger';
  delay?: number;
}>(), {
  position: 'top',
  variant: 'default',
  delay: 300
});

const isVisible = ref(false);
let showTimeout: ReturnType<typeof setTimeout> | null = null;
let hideTimeout: ReturnType<typeof setTimeout> | null = null;

const show = () => {
  if (hideTimeout) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
  showTimeout = setTimeout(() => {
    isVisible.value = true;
  }, props.delay);
};

const hide = () => {
  if (showTimeout) {
    clearTimeout(showTimeout);
    showTimeout = null;
  }
  hideTimeout = setTimeout(() => {
    isVisible.value = false;
  }, 100);
};

const variantColors = {
  default: { bg: '#1a1a2e', border: '#3a3a5c', text: '#fff' },
  info: { bg: '#0099db', border: '#006b99', text: '#fff' },
  success: { bg: '#39ff14', border: '#2d8a1a', text: '#000' },
  warning: { bg: '#feae34', border: '#c68b28', text: '#000' },
  danger: { bg: '#e43b44', border: '#a82835', text: '#fff' }
};

const colors = computed(() => variantColors[props.variant]);
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
    
    <Transition name="tooltip">
      <div 
        v-if="isVisible" 
        class="pixel-tooltip"
        :class="[position, variant]"
        :style="{
          '--tooltip-bg': colors.bg,
          '--tooltip-border': colors.border,
          '--tooltip-text': colors.text
        }"
      >
        <!-- Corner decorations -->
        <div class="corner tl"></div>
        <div class="corner tr"></div>
        <div class="corner bl"></div>
        <div class="corner br"></div>
        
        <!-- Arrow -->
        <div class="tooltip-arrow"></div>
        
        <!-- Content -->
        <span class="tooltip-content">{{ content }}</span>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.pixel-tooltip-wrapper {
  position: relative;
  display: inline-flex;
}

.pixel-tooltip {
  position: absolute;
  z-index: 1000;
  padding: 8px 12px;
  background: var(--tooltip-bg);
  border: 4px solid #000;
  box-shadow: 4px 4px 0 rgba(0, 0, 0, 0.8);
  white-space: nowrap;
  pointer-events: none;
  image-rendering: pixelated;
}

.tooltip-content {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--tooltip-text);
  letter-spacing: 1px;
  text-transform: uppercase;
}

/* Corners */
.corner {
  position: absolute;
  width: 4px;
  height: 4px;
  background: var(--tooltip-border);
}

.corner.tl { top: -4px; left: -4px; }
.corner.tr { top: -4px; right: -4px; }
.corner.bl { bottom: -4px; left: -4px; }
.corner.br { bottom: -4px; right: -4px; }

/* Arrow */
.tooltip-arrow {
  position: absolute;
  width: 0;
  height: 0;
}

/* Positions */
.pixel-tooltip.top {
  bottom: calc(100% + 12px);
  left: 50%;
  transform: translateX(-50%);
}

.pixel-tooltip.top .tooltip-arrow {
  bottom: -8px;
  left: 50%;
  transform: translateX(-50%);
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-top: 8px solid #000;
}

.pixel-tooltip.bottom {
  top: calc(100% + 12px);
  left: 50%;
  transform: translateX(-50%);
}

.pixel-tooltip.bottom .tooltip-arrow {
  top: -8px;
  left: 50%;
  transform: translateX(-50%);
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 8px solid #000;
}

.pixel-tooltip.left {
  right: calc(100% + 12px);
  top: 50%;
  transform: translateY(-50%);
}

.pixel-tooltip.left .tooltip-arrow {
  right: -8px;
  top: 50%;
  transform: translateY(-50%);
  border-top: 8px solid transparent;
  border-bottom: 8px solid transparent;
  border-left: 8px solid #000;
}

.pixel-tooltip.right {
  left: calc(100% + 12px);
  top: 50%;
  transform: translateY(-50%);
}

.pixel-tooltip.right .tooltip-arrow {
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  border-top: 8px solid transparent;
  border-bottom: 8px solid transparent;
  border-right: 8px solid #000;
}

/* Variants */
.pixel-tooltip.info .corner { background: #0099db; }
.pixel-tooltip.success .corner { background: #39ff14; }
.pixel-tooltip.warning .corner { background: #feae34; }
.pixel-tooltip.danger .corner { background: #e43b44; }

/* Animation */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: all 0.15s steps(4);
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
}

.tooltip-enter-from.top,
.tooltip-leave-to.top {
  transform: translateX(-50%) translateY(8px);
}

.tooltip-enter-from.bottom,
.tooltip-leave-to.bottom {
  transform: translateX(-50%) translateY(-8px);
}

.tooltip-enter-from.left,
.tooltip-leave-to.left {
  transform: translateY(-50%) translateX(8px);
}

.tooltip-enter-from.right,
.tooltip-leave-to.right {
  transform: translateY(-50%) translateX(-8px);
}
</style>
