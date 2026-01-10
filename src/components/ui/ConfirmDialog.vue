<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { registerOverlay } from '../../composables/useKeyboardShortcuts'

defineProps<{
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'danger' | 'warning' | 'default'
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

let unregisterOverlay: (() => void) | null = null;

onMounted(() => {
  unregisterOverlay = registerOverlay(`confirm-dialog-${Date.now()}`, () => emit('cancel'));
});

onUnmounted(() => {
  if (unregisterOverlay) {
    unregisterOverlay();
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="confirm">
      <div class="confirm-overlay" @click.self="emit('cancel')">
        <!-- Scanlines -->
        <div class="overlay-scanlines"></div>
        
        <div class="confirm-dialog" :class="variant || 'warning'">
          <!-- Corner decorations -->
          <div class="corner tl"></div>
          <div class="corner tr"></div>
          <div class="corner bl"></div>
          <div class="corner br"></div>
          
          <!-- Icon -->
          <div class="confirm-icon">
            <!-- Danger X -->
            <svg v-if="variant === 'danger'" viewBox="0 0 32 32">
              <rect x="4" y="4" width="8" height="8" fill="currentColor"/>
              <rect x="20" y="4" width="8" height="8" fill="currentColor"/>
              <rect x="12" y="12" width="8" height="8" fill="currentColor"/>
              <rect x="4" y="20" width="8" height="8" fill="currentColor"/>
              <rect x="20" y="20" width="8" height="8" fill="currentColor"/>
            </svg>
            
            <!-- Warning Triangle -->
            <svg v-else viewBox="0 0 32 32">
              <rect x="14" y="4" width="4" height="4" fill="currentColor"/>
              <rect x="12" y="8" width="8" height="4" fill="currentColor"/>
              <rect x="10" y="12" width="12" height="4" fill="currentColor"/>
              <rect x="8" y="16" width="16" height="4" fill="currentColor"/>
              <rect x="6" y="20" width="20" height="4" fill="currentColor"/>
              <rect x="4" y="24" width="24" height="4" fill="currentColor"/>
              <rect x="14" y="12" width="4" height="6" fill="#000"/>
              <rect x="14" y="20" width="4" height="4" fill="#000"/>
            </svg>
          </div>
          
          <h3>{{ title }}</h3>
          <p>{{ message }}</p>
          
          <div class="confirm-actions">
            <button class="btn-cancel" @click="emit('cancel')">
              {{ cancelText || 'CANCEL' }}
            </button>
            <button class="btn-confirm" :class="variant || 'warning'" @click="emit('confirm')">
              {{ confirmText || 'CONFIRM' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.92);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
  image-rendering: pixelated;
}

.overlay-scanlines {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent 0px,
    transparent 2px,
    rgba(0, 0, 0, 0.15) 2px,
    rgba(0, 0, 0, 0.15) 4px
  );
  pointer-events: none;
}

.confirm-dialog {
  position: relative;
  width: 100%;
  max-width: 400px;
  background: #1a1a2e;
  border: 4px solid #000;
  box-shadow: 8px 8px 0 rgba(0, 0, 0, 0.8);
  padding: 24px;
  text-align: center;
}

/* Corner decorations */
.corner {
  position: absolute;
  width: 8px;
  height: 8px;
  background: #feae34;
}

.corner.tl { top: -4px; left: -4px; }
.corner.tr { top: -4px; right: -4px; }
.corner.bl { bottom: -4px; left: -4px; }
.corner.br { bottom: -4px; right: -4px; }

.confirm-dialog.danger .corner { background: #e43b44; }
.confirm-dialog.default .corner { background: #0099db; }

/* Icon */
.confirm-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 20px;
  color: #feae34;
  filter: drop-shadow(0 0 8px currentColor);
}

.confirm-dialog.danger .confirm-icon { color: #e43b44; }
.confirm-dialog.default .confirm-icon { color: #0099db; }

.confirm-icon svg {
  width: 100%;
  height: 100%;
}

/* Title */
.confirm-dialog h3 {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  color: #fff;
  text-shadow: 2px 2px 0 #000;
  margin-bottom: 12px;
  letter-spacing: 2px;
}

/* Message */
.confirm-dialog p {
  font-family: 'VT323', monospace;
  font-size: 20px;
  color: #808080;
  margin-bottom: 24px;
  line-height: 1.4;
}

/* Actions */
.confirm-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.btn-cancel {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  padding: 12px 20px;
  background: #2a2a4c;
  border: 4px solid #000;
  color: #808080;
  cursor: pointer;
  box-shadow: 4px 4px 0 #000;
}

.btn-cancel:hover {
  background: #3a3a5c;
  color: #fff;
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000;
}

.btn-cancel:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

.btn-confirm {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  padding: 12px 20px;
  border: 4px solid #000;
  cursor: pointer;
  box-shadow: 4px 4px 0 #000;
}

.btn-confirm.warning {
  background: linear-gradient(180deg, #feae34 0%, #c68b28 100%);
  color: #000;
}

.btn-confirm.danger {
  background: linear-gradient(180deg, #e43b44 0%, #a82835 100%);
  color: #fff;
}

.btn-confirm.default {
  background: linear-gradient(180deg, #0099db 0%, #006b99 100%);
  color: #fff;
}

.btn-confirm:hover {
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000;
  filter: brightness(1.1);
}

.btn-confirm:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

/* Animation */
.confirm-enter-active,
.confirm-leave-active {
  transition: opacity 0.2s steps(4);
}

.confirm-enter-active .confirm-dialog,
.confirm-leave-active .confirm-dialog {
  transition: transform 0.2s steps(4);
}

.confirm-enter-from,
.confirm-leave-to {
  opacity: 0;
}

.confirm-enter-from .confirm-dialog,
.confirm-leave-to .confirm-dialog {
  transform: scale(0.9) translateY(-20px);
}
</style>
