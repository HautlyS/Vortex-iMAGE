<script setup lang="ts">
import { watch, onMounted, onUnmounted } from 'vue';

const props = withDefaults(defineProps<{
  modelValue: boolean;
  title?: string;
  size?: 'sm' | 'md' | 'lg';
  closable?: boolean;
  variant?: 'default' | 'success' | 'warning' | 'danger';
}>(), {
  size: 'md',
  closable: true,
  variant: 'default'
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  close: [];
}>();

const close = () => {
  emit('update:modelValue', false);
  emit('close');
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.closable) {
    close();
  }
};

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    document.body.style.overflow = 'hidden';
  } else {
    document.body.style.overflow = '';
  }
});

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  document.body.style.overflow = '';
});
</script>

<template>
  <Teleport to="body">
    <Transition name="pixel-modal">
      <div v-if="modelValue" class="pixel-modal-overlay" @click.self="closable && close()">
        <div class="pixel-modal" :class="[size, variant]">
          <!-- Corner decorations -->
          <div class="corner tl" />
          <div class="corner tr" />
          <div class="corner bl" />
          <div class="corner br" />

          <!-- Header -->
          <div v-if="title || closable" class="pixel-modal-header">
            <div class="header-decoration" />
            <h3 class="pixel-modal-title">{{ title }}</h3>
            <button v-if="closable" class="pixel-modal-close" @click="close">
              ✕
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
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 20px;
}

.pixel-modal {
  position: relative;
  background: var(--retro-bg-panel, #1a1030);
  border: 4px solid #000;
  box-shadow: 8px 8px 0 rgba(0, 0, 0, 0.8);
  max-height: 90vh;
  display: flex;
  flex-direction: column;
}

/* Sizes */
.pixel-modal.sm { width: 320px; }
.pixel-modal.md { width: 480px; }
.pixel-modal.lg { width: 640px; }

/* Corners */
.corner {
  position: absolute;
  width: 10px;
  height: 10px;
  background: var(--retro-accent-pink, #ff2d95);
  z-index: 1;
}

.corner.tl { top: -5px; left: -5px; }
.corner.tr { top: -5px; right: -5px; }
.corner.bl { bottom: -5px; left: -5px; }
.corner.br { bottom: -5px; right: -5px; }

/* Variants */
.pixel-modal.success .corner { background: var(--retro-accent-green, #00ff87); }
.pixel-modal.warning .corner { background: var(--retro-accent-yellow, #ffd000); }
.pixel-modal.danger .corner { background: var(--retro-accent-red, #ff3b30); }

/* Header */
.pixel-modal-header {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: linear-gradient(90deg, var(--retro-accent-blue, #00d4ff), var(--retro-accent-purple, #b24dff));
  border-bottom: 4px solid #000;
}

.pixel-modal.success .pixel-modal-header {
  background: linear-gradient(90deg, var(--retro-accent-green, #00ff87), #00cc6a);
}

.pixel-modal.warning .pixel-modal-header {
  background: linear-gradient(90deg, var(--retro-accent-yellow, #ffd000), #ffaa00);
}

.pixel-modal.danger .pixel-modal-header {
  background: linear-gradient(90deg, var(--retro-accent-red, #ff3b30), #cc2020);
}

.header-decoration {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 8px;
  background: repeating-linear-gradient(
    0deg,
    #000 0px,
    #000 4px,
    transparent 4px,
    transparent 8px
  );
}

.pixel-modal-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: #fff;
  text-shadow: 2px 2px 0 #000;
  margin: 0;
  padding-left: 12px;
}

.pixel-modal-close {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  background: transparent;
  border: none;
  color: #fff;
  cursor: pointer;
  padding: 4px 8px;
  transition: all 0.1s steps(2);
  text-shadow: 2px 2px 0 #000;
}

.pixel-modal-close:hover {
  color: var(--retro-accent-yellow, #ffd000);
  transform: scale(1.2);
}

/* Body */
.pixel-modal-body {
  padding: 20px;
  overflow-y: auto;
  background: var(--retro-bg-card, #251842);
}

/* Footer */
.pixel-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 3px solid #000;
  background: var(--retro-bg-panel, #1a1030);
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
  transform: scale(0.9) translateY(-20px);
}
</style>
