<script setup lang="ts">
defineProps<{
  variant?: 'default' | 'primary' | 'success' | 'danger' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
}>()

defineEmits<{
  click: [event: MouseEvent]
}>()
</script>

<template>
  <button
    class="pixel-btn"
    :class="[variant, size, { loading, disabled }]"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <span v-if="loading" class="spinner"></span>
    <slot v-else />
  </button>
</template>

<style scoped>
.pixel-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 12px 20px;
  background: var(--retro-bg-lighter, #2d1f4d);
  color: var(--retro-text-main, #fff);
  border: 2px solid #000;
  box-shadow: 4px 4px 0 #000;
  cursor: pointer;
  transition: all 0.1s;
  position: relative;
  overflow: hidden;
}

.pixel-btn:hover:not(:disabled) {
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000;
}

.pixel-btn:active:not(:disabled) {
  transform: translate(2px, 2px);
  box-shadow: 0 0 0 #000;
}

.pixel-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Variants */
.pixel-btn.primary {
  background: linear-gradient(180deg, var(--retro-accent-green, #00ff87), #00cc6a);
  color: #000;
}

.pixel-btn.success {
  background: linear-gradient(180deg, var(--retro-accent-blue, #00d4ff), #00a8cc);
  color: #000;
}

.pixel-btn.danger {
  background: linear-gradient(180deg, var(--retro-accent-red, #ff3b30), #cc2020);
  color: #fff;
}

.pixel-btn.ghost {
  background: transparent;
  border-color: var(--retro-bg-lighter, #2d1f4d);
  box-shadow: none;
}

.pixel-btn.ghost:hover:not(:disabled) {
  border-color: var(--retro-accent-green, #00ff87);
  color: var(--retro-accent-green, #00ff87);
  transform: none;
  box-shadow: none;
}

/* Sizes */
.pixel-btn.sm {
  font-size: 8px;
  padding: 8px 12px;
  box-shadow: 2px 2px 0 #000;
}

.pixel-btn.sm:hover:not(:disabled) {
  box-shadow: 3px 3px 0 #000;
}

.pixel-btn.lg {
  font-size: 12px;
  padding: 16px 28px;
  box-shadow: 5px 5px 0 #000;
}

.pixel-btn.lg:hover:not(:disabled) {
  box-shadow: 7px 7px 0 #000;
}

/* Loading Spinner */
.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  animation: spin 0.6s steps(8) infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Hover Glow Effect */
.pixel-btn.primary:hover:not(:disabled),
.pixel-btn.success:hover:not(:disabled) {
  box-shadow: 6px 6px 0 #000, 0 0 15px var(--retro-accent-green, #00ff87);
}

.pixel-btn.danger:hover:not(:disabled) {
  box-shadow: 6px 6px 0 #000, 0 0 15px var(--retro-accent-red, #ff3b30);
}
</style>
