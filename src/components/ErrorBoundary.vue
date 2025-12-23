/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, onErrorCaptured, onMounted, onUnmounted } from 'vue'

interface ErrorInfo {
  message: string
  timestamp: number
  id: string
}

const errors = ref<ErrorInfo[]>([])
const MAX_ERRORS = 5

function addError(message: string) {
  const error: ErrorInfo = {
    message,
    timestamp: Date.now(),
    id: Math.random().toString(36).slice(2)
  }
  errors.value = [error, ...errors.value].slice(0, MAX_ERRORS)

  setTimeout(() => dismissError(error.id), 5000)
}

function dismissError(id: string) {
  errors.value = errors.value.filter(e => e.id !== id)
}

onErrorCaptured((err) => {
  addError(err.message || 'Erro desconhecido')
  return false
})

function handleGlobalError(event: ErrorEvent) {
  addError(event.message || 'Erro inesperado')
}

function handleRejection(event: PromiseRejectionEvent) {
  const msg = event.reason?.message || event.reason || 'Erro de promessa'
  addError(String(msg))
}

onMounted(() => {
  window.addEventListener('error', handleGlobalError)
  window.addEventListener('unhandledrejection', handleRejection)
})

onUnmounted(() => {
  window.removeEventListener('error', handleGlobalError)
  window.removeEventListener('unhandledrejection', handleRejection)
})
</script>

<template>
  <slot />
  
  <Teleport to="body">
    <TransitionGroup name="error-toast" tag="div" class="error-container">
      <div v-for="error in errors" :key="error.id" class="error-toast" role="alert">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="error-icon">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <span class="error-message">{{ error.message }}</span>
        <button class="error-dismiss" @click="dismissError(error.id)" aria-label="Fechar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </TransitionGroup>
  </Teleport>
</template>

<style scoped>
.error-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 400px;
}

.error-toast {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 12px;
  backdrop-filter: blur(12px);
  color: #fca5a5;
}

.error-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  color: #ef4444;
}

.error-message {
  flex: 1;
  font-size: 0.875rem;
  line-height: 1.4;
}

.error-dismiss {
  width: 24px;
  height: 24px;
  padding: 4px;
  background: transparent;
  border: none;
  color: #fca5a5;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.15s;
}

.error-dismiss:hover {
  background: rgba(255, 255, 255, 0.1);
}

.error-dismiss svg {
  width: 100%;
  height: 100%;
}

.error-toast-enter-active {
  animation: slide-in 0.3s ease-out;
}

.error-toast-leave-active {
  animation: slide-out 0.2s ease-in forwards;
}

@keyframes slide-in {
  from {
    opacity: 0;
    transform: translateX(100%);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes slide-out {
  to {
    opacity: 0;
    transform: translateX(100%);
  }
}

@media (max-width: 768px) {
  .error-container {
    top: auto;
    bottom: 80px;
    left: 16px;
    right: 16px;
    max-width: none;
  }
}
</style>