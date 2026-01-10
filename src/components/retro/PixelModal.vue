<script setup lang="ts">
import { watch, onUnmounted } from 'vue';
import { registerOverlay } from '../../composables/useKeyboardShortcuts';

const props = withDefaults(defineProps<{
  modelValue: boolean;
  title?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closable?: boolean;
  closeOnClickOutside?: boolean;
  closeOnEsc?: boolean;
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
}>(), {
  size: 'md',
  closable: true,
  closeOnClickOutside: true,
  closeOnEsc: true,
  variant: 'default'
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  close: [];
}>();

let unregisterOverlay: (() => void) | null = null;

const close = () => {
  emit('update:modelValue', false);
  emit('close');
};

const handleOverlayClick = (e: MouseEvent) => {
  if (props.closable && props.closeOnClickOutside && e.target === e.currentTarget) {
    close();
  }
};

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    document.body.style.overflow = 'hidden';
    if (props.closable && props.closeOnEsc) {
      unregisterOverlay = registerOverlay(`pixel-modal-${Date.now()}`, close);
    }
  } else {
    document.body.style.overflow = '';
    if (unregisterOverlay) {
      unregisterOverlay();
      unregisterOverlay = null;
    }
  }
}, { immediate: true });

onUnmounted(() => {
  if (unregisterOverlay) {
    unregisterOverlay();
  }
  document.body.style.overflow = '';
});
</script>

<template>
  <Teleport to="body">
    <Transition name="pixel-modal">
      <div v-if="modelValue" class="pixel-modal-overlay" @click="handleOverlayClick">
        <!-- Scanline effect on modal -->
        <div class="modal-scanlines"></div>
        
        <div class="pixel-modal" :class="[size, variant]">
          <!-- 8-Bit Corner decorations -->
          <div class="corner tl"><div class="corner-shine"></div></div>
          <div class="corner tr"><div class="corner-shine"></div></div>
          <div class="corner bl"><div class="corner-shine"></div></div>
          <div class="corner br"><div class="corner-shine"></div></div>

          <!-- Header -->
          <div v-if="title || closable" class="pixel-modal-header">
            <div class="header-pattern"></div>
            <div class="header-deco-left">
              <span class="deco-block"></span>
              <span class="deco-block"></span>
              <span class="deco-block"></span>
            </div>
            <h3 class="pixel-modal-title">{{ title }}</h3>
            <button v-if="closable" class="pixel-modal-close" @click="close">
              <svg viewBox="0 0 16 16" width="16" height="16">
                <rect x="2" y="6" width="4" height="4" fill="currentColor"/>
                <rect x="6" y="2" width="4" height="4" fill="currentColor"/>
                <rect x="6" y="10" width="4" height="4" fill="currentColor"/>
                <rect x="10" y="6" width="4" height="4" fill="currentColor"/>
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="pixel-modal-body">
            <slot />
          </div>

          <!-- Footer -->
          <div v-if="$slots.footer" class="pixel-modal-footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.pixel-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.92);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 20px;
  image-rendering: pixelated;
}

.modal-scanlines {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent 0px,
    transparent 2px,
    rgba(0, 0, 0, 0.2) 2px,
    rgba(0, 0, 0, 0.2) 4px
  );
  pointer-events: none;
  animation: scanline-scroll 0.1s steps(2) infinite;
}

@keyframes scanline-scroll {
  0% { transform: translateY(0); }
  100% { transform: translateY(4px); }
}

.pixel-modal {
  position: relative;
  background: #1a1a2e;
  border: 4px solid #000;
  box-shadow: 
    8px 8px 0 rgba(0, 0, 0, 0.9),
    inset 0 0 40px rgba(57, 255, 20, 0.03);
  max-height: 90vh;
  display: flex;
  flex-direction: column;
}

/* Sizes */
.pixel-modal.sm { width: 320px; }
.pixel-modal.md { width: 480px; }
.pixel-modal.lg { width: 640px; }
.pixel-modal.xl { width: 800px; }

/* Corners */
.corner {
  position: absolute;
  width: 16px;
  height: 16px;
  background: #f15bb5;
  z-index: 1;
}

.corner-shine {
  position: absolute;
  width: 4px;
  height: 4px;
  background: rgba(255,255,255,0.6);
}

.corner.tl { top: -8px; left: -8px; }
.corner.tl .corner-shine { top: 2px; left: 2px; }

.corner.tr { top: -8px; right: -8px; }
.corner.tr .corner-shine { top: 2px; right: 2px; }

.corner.bl { bottom: -8px; left: -8px; }
.corner.bl .corner-shine { bottom: 2px; left: 2px; }

.corner.br { bottom: -8px; right: -8px; }
.corner.br .corner-shine { bottom: 2px; right: 2px; }

/* Variants */
.pixel-modal.success .corner { background: #63c74d; }
.pixel-modal.warning .corner { background: #feae34; }
.pixel-modal.danger .corner { background: #e43b44; }
.pixel-modal.info .corner { background: #9b5de5; }

/* Header */
.pixel-modal-header {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  padding-left: 40px;
  background: linear-gradient(180deg, #0099db 0%, #006b99 100%);
  border-bottom: 4px solid #000;
  overflow: hidden;
}

.header-pattern {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    90deg,
    transparent 0px,
    transparent 8px,
    rgba(0,0,0,0.1) 8px,
    rgba(0,0,0,0.1) 16px
  );
}

.header-deco-left {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.deco-block {
  width: 8px;
  height: 4px;
  background: rgba(255,255,255,0.5);
}

.pixel-modal.success .pixel-modal-header {
  background: linear-gradient(180deg, #63c74d 0%, #3e8948 100%);
}

.pixel-modal.warning .pixel-modal-header {
  background: linear-gradient(180deg, #feae34 0%, #c68b28 100%);
}

.pixel-modal.danger .pixel-modal-header {
  background: linear-gradient(180deg, #e43b44 0%, #a82835 100%);
}

.pixel-modal.info .pixel-modal-header {
  background: linear-gradient(180deg, #9b5de5 0%, #7b3dc5 100%);
}

.pixel-modal-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: #fff;
  text-shadow: 2px 2px 0 #000;
  margin: 0;
  letter-spacing: 2px;
  text-transform: uppercase;
  position: relative;
}

.pixel-modal-close {
  position: relative;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0,0,0,0.3);
  border: 2px solid #000;
  color: #fff;
  cursor: pointer;
  padding: 0;
  box-shadow: 2px 2px 0 #000;
}

.pixel-modal-close:hover {
  background: #e43b44;
  transform: translate(-2px, -2px);
  box-shadow: 4px 4px 0 #000;
}

.pixel-modal-close:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

/* Body */
.pixel-modal-body {
  padding: 24px;
  overflow-y: auto;
  background: linear-gradient(180deg, #16213e 0%, #1a1a2e 100%);
}

/* Footer */
.pixel-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 4px solid #000;
  background: #0f0f23;
}

/* Animation */
.pixel-modal-enter-active,
.pixel-modal-leave-active {
  transition: opacity 0.2s steps(4);
}

.pixel-modal-enter-active .pixel-modal,
.pixel-modal-leave-active .pixel-modal {
  transition: transform 0.2s steps(4);
}

.pixel-modal-enter-from,
.pixel-modal-leave-to {
  opacity: 0;
}

.pixel-modal-enter-from .pixel-modal,
.pixel-modal-leave-to .pixel-modal {
  transform: scale(0.8) translateY(-40px);
}
</style>
