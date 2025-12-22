<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  loading?: boolean
  error?: string | null
  empty?: boolean
  emptyMessage?: string
  retryable?: boolean
}>(), {
  loading: false,
  error: null,
  empty: false,
  emptyMessage: 'Nenhum item encontrado',
  retryable: true
})

const emit = defineEmits<{
  retry: []
}>()

const state = computed(() => {
  if (props.loading) return 'loading'
  if (props.error) return 'error'
  if (props.empty) return 'empty'
  return 'ready'
})
</script>

<template>
  <div class="async-state">
    <!-- Loading -->
    <div v-if="state === 'loading'" class="state-loading">
      <div class="spinner" />
      <span>Carregando...</span>
    </div>

    <!-- Error -->
    <div v-else-if="state === 'error'" class="state-error">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p>{{ error }}</p>
      <button v-if="retryable" class="retry-btn" @click="emit('retry')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 4v6h6M23 20v-6h-6"/>
          <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15"/>
        </svg>
        Tentar novamente
      </button>
    </div>

    <!-- Empty -->
    <div v-else-if="state === 'empty'" class="state-empty">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="18" rx="2"/>
        <circle cx="9" cy="9" r="2"/>
        <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
      </svg>
      <p>{{ emptyMessage }}</p>
      <slot name="empty-action" />
    </div>

    <!-- Content -->
    <slot v-else />
  </div>
</template>

<style scoped>
.async-state {
  width: 100%;
  height: 100%;
}

.state-loading,
.state-error,
.state-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 48px 24px;
  text-align: center;
  color: var(--text-muted, #71717a);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-top-color: var(--accent-color, #00ff41);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.state-error svg,
.state-empty svg {
  width: 48px;
  height: 48px;
  opacity: 0.5;
}

.state-error {
  color: #fca5a5;
}

.state-error svg {
  color: #ef4444;
}

.retry-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  color: var(--text-primary, #fff);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.15s;
}

.retry-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: var(--accent-color, #00ff41);
}

.retry-btn svg {
  width: 16px;
  height: 16px;
}
</style>
