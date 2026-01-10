<script setup lang="ts">
defineProps<{
  variant?: 'default' | 'primary' | 'success' | 'danger' | 'ghost' | 'retro'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
  glow?: boolean
}>()

defineEmits<{
  click: [event: MouseEvent]
}>()
</script>

<template>
  <button
    class="pixel-btn"
    :class="[variant, size, { loading, disabled, glow }]"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <span v-if="loading" class="pixel-spinner">
      <span class="spinner-dot"></span>
      <span class="spinner-dot"></span>
      <span class="spinner-dot"></span>
    </span>
    <span v-else class="btn-content">
      <slot />
    </span>
    <span class="btn-shine"></span>
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
  padding: 16px 24px;
  background: linear-gradient(180deg, #3a3a5c 0%, #2a2a4c 100%);
  color: #fff;
  border: 4px solid #000;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  image-rendering: pixelated;
  box-shadow: 
    inset -4px -4px 0 #1a1a3c,
    inset 4px 4px 0 #4a4a6c,
    4px 4px 0 #000;
  transition: none;
}

.pixel-btn:hover:not(:disabled) {
  transform: translate(-2px, -2px);
  box-shadow: 
    inset -4px -4px 0 #2a2a4c,
    inset 4px 4px 0 #5a5a7c,
    6px 6px 0 #000;
}

.pixel-btn:active:not(:disabled) {
  transform: translate(4px, 4px);
  box-shadow: 
    inset 4px 4px 0 #1a1a3c,
    inset -4px -4px 0 #3a3a5c;
}

.pixel-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Shine effect */
.btn-shine {
  position: absolute;
  top: 0;
  left: -100%;
  width: 50%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.2), transparent);
  transform: skewX(-20deg);
}

.pixel-btn:hover:not(:disabled) .btn-shine {
  animation: shine 0.4s steps(4);
}

@keyframes shine {
  to { left: 150%; }
}

/* Primary Variant - Green */
.pixel-btn.primary {
  background: linear-gradient(180deg, #63c74d 0%, #3e8948 100%);
  color: #000;
  text-shadow: 1px 1px 0 rgba(255,255,255,0.3);
  box-shadow: 
    inset -4px -4px 0 #2d6a30,
    inset 4px 4px 0 #8cff7a,
    4px 4px 0 #000;
}

.pixel-btn.primary:hover:not(:disabled) {
  box-shadow: 
    inset -4px -4px 0 #3e8948,
    inset 4px 4px 0 #a8ff96,
    6px 6px 0 #000,
    0 0 16px rgba(99, 199, 77, 0.5);
}

.pixel-btn.primary.glow {
  animation: glow-green 1s steps(4) infinite;
}

@keyframes glow-green {
  0%, 100% { box-shadow: inset -4px -4px 0 #2d6a30, inset 4px 4px 0 #8cff7a, 4px 4px 0 #000, 0 0 8px rgba(99, 199, 77, 0.3); }
  50% { box-shadow: inset -4px -4px 0 #2d6a30, inset 4px 4px 0 #8cff7a, 4px 4px 0 #000, 0 0 24px rgba(99, 199, 77, 0.6); }
}

/* Success Variant - Blue */
.pixel-btn.success {
  background: linear-gradient(180deg, #0099db 0%, #006b99 100%);
  color: #fff;
  box-shadow: 
    inset -4px -4px 0 #004d70,
    inset 4px 4px 0 #33b5e5,
    4px 4px 0 #000;
}

.pixel-btn.success:hover:not(:disabled) {
  box-shadow: 
    inset -4px -4px 0 #006b99,
    inset 4px 4px 0 #66ccff,
    6px 6px 0 #000,
    0 0 16px rgba(0, 153, 219, 0.5);
}

/* Danger Variant - Red */
.pixel-btn.danger {
  background: linear-gradient(180deg, #e43b44 0%, #a82835 100%);
  color: #fff;
  box-shadow: 
    inset -4px -4px 0 #7a1a22,
    inset 4px 4px 0 #ff6b6b,
    4px 4px 0 #000;
}

.pixel-btn.danger:hover:not(:disabled) {
  box-shadow: 
    inset -4px -4px 0 #a82835,
    inset 4px 4px 0 #ff8888,
    6px 6px 0 #000,
    0 0 16px rgba(228, 59, 68, 0.5);
}

/* Ghost Variant */
.pixel-btn.ghost {
  background: transparent;
  border-color: #3a3a5c;
  box-shadow: none;
  color: #808080;
}

.pixel-btn.ghost:hover:not(:disabled) {
  background: rgba(58, 58, 92, 0.3);
  border-color: #39ff14;
  color: #39ff14;
  box-shadow: 0 0 8px rgba(57, 255, 20, 0.3);
  transform: none;
}

/* Retro Variant - Rainbow */
.pixel-btn.retro {
  background: linear-gradient(180deg, #f15bb5 0%, #9b5de5 50%, #00bbf9 100%);
  color: #fff;
  text-shadow: 2px 2px 0 #000;
  box-shadow: 
    inset -4px -4px 0 #7b2d85,
    inset 4px 4px 0 #ff8ed4,
    4px 4px 0 #000;
  animation: rainbow-pulse 2s steps(8) infinite;
}

@keyframes rainbow-pulse {
  0%, 100% { filter: hue-rotate(0deg); }
  50% { filter: hue-rotate(30deg); }
}

/* Sizes */
.pixel-btn.sm {
  font-size: 8px;
  padding: 8px 16px;
  box-shadow: 
    inset -2px -2px 0 #1a1a3c,
    inset 2px 2px 0 #4a4a6c,
    2px 2px 0 #000;
}

.pixel-btn.sm:hover:not(:disabled) {
  box-shadow: 
    inset -2px -2px 0 #2a2a4c,
    inset 2px 2px 0 #5a5a7c,
    4px 4px 0 #000;
}

.pixel-btn.lg {
  font-size: 12px;
  padding: 20px 32px;
  box-shadow: 
    inset -6px -6px 0 #1a1a3c,
    inset 6px 6px 0 #4a4a6c,
    6px 6px 0 #000;
}

.pixel-btn.lg:hover:not(:disabled) {
  box-shadow: 
    inset -6px -6px 0 #2a2a4c,
    inset 6px 6px 0 #5a5a7c,
    8px 8px 0 #000;
}

/* Loading Spinner */
.pixel-spinner {
  display: flex;
  gap: 4px;
}

.spinner-dot {
  width: 8px;
  height: 8px;
  background: currentColor;
  animation: dot-bounce 0.6s steps(3) infinite;
}

.spinner-dot:nth-child(2) { animation-delay: 0.2s; }
.spinner-dot:nth-child(3) { animation-delay: 0.4s; }

@keyframes dot-bounce {
  0%, 100% { transform: translateY(0); opacity: 0.4; }
  50% { transform: translateY(-8px); opacity: 1; }
}

.btn-content {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
