<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
  size?: 'sm' | 'md' | 'lg';
  glow?: boolean;
  animated?: boolean;
}>(), {
  variant: 'default',
  size: 'md',
  glow: false,
  animated: false
});

const variantColors = {
  default: { bg: 'var(--retro-bg-lighter)', color: 'var(--retro-text-main)', glow: 'var(--retro-accent-pink)' },
  success: { bg: 'var(--retro-accent-green)', color: '#000', glow: 'var(--retro-accent-green)' },
  warning: { bg: 'var(--retro-accent-yellow)', color: '#000', glow: 'var(--retro-accent-yellow)' },
  danger: { bg: 'var(--retro-accent-red)', color: '#fff', glow: 'var(--retro-accent-red)' },
  info: { bg: 'var(--retro-accent-blue)', color: '#000', glow: 'var(--retro-accent-blue)' }
};

const badgeStyle = computed(() => {
  const colors = variantColors[props.variant];
  return {
    '--badge-bg': colors.bg,
    '--badge-color': colors.color,
    '--badge-glow': colors.glow
  };
});
</script>

<template>
  <span 
    class="pixel-badge" 
    :class="[size, { glow, animated }]"
    :style="badgeStyle"
  >
    <slot />
  </span>
</template>

<style scoped>
.pixel-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: 'Press Start 2P', monospace;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: var(--badge-bg);
  color: var(--badge-color);
  border: 2px solid #000;
  box-shadow: 2px 2px 0 #000;
  white-space: nowrap;
}

/* Sizes */
.pixel-badge.sm {
  font-size: 6px;
  padding: 4px 6px;
}

.pixel-badge.md {
  font-size: 8px;
  padding: 6px 10px;
}

.pixel-badge.lg {
  font-size: 10px;
  padding: 8px 14px;
}

/* Glow effect */
.pixel-badge.glow {
  box-shadow: 
    2px 2px 0 #000,
    0 0 10px var(--badge-glow),
    0 0 20px var(--badge-glow);
}

/* Animation */
.pixel-badge.animated {
  animation: badge-pulse 1.5s steps(4) infinite;
}

@keyframes badge-pulse {
  0%, 100% { 
    transform: scale(1);
    box-shadow: 2px 2px 0 #000;
  }
  50% { 
    transform: scale(1.05);
    box-shadow: 2px 2px 0 #000, 0 0 15px var(--badge-glow);
  }
}
</style>
