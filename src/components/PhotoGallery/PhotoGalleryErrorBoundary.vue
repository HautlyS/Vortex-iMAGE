<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'

defineProps<{
  fallback?: string
}>()

const emit = defineEmits<{
  error: [error: Error]
}>()

const hasError = ref(false)
const error = ref<Error | null>(null)

onErrorCaptured((err: Error) => {
  hasError.value = true
  error.value = err
  emit('error', err)
  return false
})

function retry() {
  hasError.value = false
  error.value = null
}
</script>

<template>
  <div v-if="hasError" class="error-boundary">
    <div class="error-content">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <h3>Algo deu errado</h3>
      <p>{{ fallback || 'Ocorreu um erro inesperado na galeria de fotos.' }}</p>
      <button @click="retry" class="retry-btn">
        Tentar novamente
      </button>
    </div>
  </div>
  <slot v-else />
</template>

<style scoped>
.error-boundary {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  padding: 2rem;
}

.error-content {
  text-align: center;
  max-width: 400px;
}

.error-content svg {
  width: 3rem;
  height: 3rem;
  color: var(--error, #ef4444);
  margin-bottom: 1rem;
}

.error-content h3 {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: var(--text-primary, #fafafa);
}

.error-content p {
  color: var(--text-muted, #71717a);
  margin-bottom: 1.5rem;
  line-height: 1.5;
}

.retry-btn {
  padding: 0.5rem 1rem;
  background: var(--accent-color, #6366f1);
  color: white;
  border: none;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.retry-btn:hover {
  background: var(--accent-hover, #5855eb);
  transform: translateY(-1px);
}
</style>
