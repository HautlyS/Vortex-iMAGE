/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

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

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('cancel')
}

onMounted(() => document.addEventListener('keydown', handleKeydown))
onUnmounted(() => document.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <Teleport to="body">
    <Transition name="confirm">
      <div class="confirm-overlay" @click.self="emit('cancel')">
        <div class="confirm-dialog">
          <div class="confirm-icon" :class="variant || 'warning'">
            <svg v-if="variant === 'danger'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/><path d="M15 9l-6 6M9 9l6 6"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
              <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
          </div>
          <h3>{{ title }}</h3>
          <p>{{ message }}</p>
          <div class="confirm-actions">
            <button class="btn-cancel" @click="emit('cancel')">{{ cancelText || 'Cancelar' }}</button>
            <button :class="['btn-confirm', variant || 'warning']" @click="emit('confirm')">
              {{ confirmText || 'Confirmar' }}
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
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}

.confirm-dialog {
  width: 100%;
  max-width: 400px;
  background: var(--surface-1, #1a1a1c);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 1rem;
  padding: 1.5rem;
  text-align: center;
}

.confirm-icon {
  width: 3rem;
  height: 3rem;
  margin: 0 auto 1rem;
}

.confirm-icon.warning { color: #f59e0b; }
.confirm-icon.danger { color: #ef4444; }
.confirm-icon.default { color: var(--accent-color, #6366f1); }

.confirm-icon svg { width: 100%; height: 100%; }

.confirm-dialog h3 {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: var(--text-primary, #fafafa);
}

.confirm-dialog p {
  font-size: 0.875rem;
  color: var(--text-secondary, #71717a);
  margin-bottom: 1.5rem;
}

.confirm-actions {
  display: flex;
  gap: 0.75rem;
  justify-content: center;
}

.btn-cancel {
  padding: 0.625rem 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: var(--text-secondary, #a1a1aa);
  font-size: 0.875rem;
  border-radius: 0.5rem;
  cursor: pointer;
}

.btn-cancel:hover { background: rgba(255, 255, 255, 0.1); }

.btn-confirm {
  padding: 0.625rem 1.25rem;
  border: none;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 0.5rem;
  cursor: pointer;
}

.btn-confirm.warning { background: #f59e0b; color: #000; }
.btn-confirm.danger { background: #ef4444; color: #fff; }
.btn-confirm.default { background: var(--accent-color, #6366f1); color: #fff; }

.btn-confirm:hover { filter: brightness(1.1); }

.confirm-enter-active, .confirm-leave-active { transition: opacity 0.2s; }
.confirm-enter-from, .confirm-leave-to { opacity: 0; }
</style>