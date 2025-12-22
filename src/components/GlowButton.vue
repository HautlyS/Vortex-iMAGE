<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
  pulse?: boolean
}>()

defineEmits<{ click: [e: MouseEvent] }>()
</script>

<template>
  <button 
    class="glow-btn" 
    :class="[variant || 'primary', size || 'md', { disabled, loading, pulse }]"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <span class="glow-btn-bg" />
    <span class="glow-btn-content">
      <span v-if="loading" class="spinner" />
      <slot v-else />
    </span>
  </button>
</template>

<style scoped>
.glow-btn {
  --btn-color: var(--accent, var(--cyber-cyan, #00f0ff));
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border: none;
  border-radius: var(--radius-md, 12px);
  font-weight: 600;
  cursor: pointer;
  overflow: hidden;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

/* Sizes */
.glow-btn.sm { padding: 0.5rem 1rem; font-size: 0.8125rem; }
.glow-btn.md { padding: 0.75rem 1.5rem; font-size: 0.9375rem; }
.glow-btn.lg { padding: 1rem 2rem; font-size: 1rem; }

/* Background layer */
.glow-btn-bg {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  transition: opacity 0.2s ease;
}

/* Content layer */
.glow-btn-content {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

/* Primary variant */
.glow-btn.primary {
  color: #000;
  box-shadow: 0 0 20px rgba(var(--accent-rgb, 0, 240, 255), 0.4);
}
.glow-btn.primary .glow-btn-bg {
  background: var(--btn-color);
}
.glow-btn.primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 0 30px rgba(var(--accent-rgb, 0, 240, 255), 0.6), 0 4px 20px rgba(0, 0, 0, 0.3);
}

/* Secondary variant */
.glow-btn.secondary {
  color: var(--btn-color);
  box-shadow: inset 0 0 0 1px var(--btn-color);
}
.glow-btn.secondary .glow-btn-bg {
  background: transparent;
}
.glow-btn.secondary:hover {
  box-shadow: inset 0 0 0 1px var(--btn-color), 0 0 20px rgba(var(--accent-rgb, 0, 240, 255), 0.3);
}
.glow-btn.secondary:hover .glow-btn-bg {
  background: rgba(var(--accent-rgb, 0, 240, 255), 0.1);
}

/* Ghost variant */
.glow-btn.ghost {
  color: var(--text-secondary, #a1a1aa);
}
.glow-btn.ghost .glow-btn-bg {
  background: transparent;
}
.glow-btn.ghost:hover {
  color: var(--text-primary, #fff);
}
.glow-btn.ghost:hover .glow-btn-bg {
  background: rgba(255, 255, 255, 0.08);
}

/* Pulse animation */
.glow-btn.pulse.primary {
  animation: btn-pulse 2s ease-in-out infinite;
}

@keyframes btn-pulse {
  0%, 100% { box-shadow: 0 0 20px rgba(var(--accent-rgb, 0, 240, 255), 0.4); }
  50% { box-shadow: 0 0 35px rgba(var(--accent-rgb, 0, 240, 255), 0.7), 0 0 60px rgba(var(--accent-rgb, 0, 240, 255), 0.3); }
}

/* Disabled state */
.glow-btn.disabled,
.glow-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none !important;
  box-shadow: none !important;
  animation: none !important;
}

/* Loading spinner */
.spinner {
  width: 1em;
  height: 1em;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
