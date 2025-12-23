<script setup lang="ts">
defineEmits<{ refresh: [] }>()
</script>

<template>
  <div class="loading" role="status" aria-live="polite">
    <div class="spinner" aria-hidden="true"></div>
    <p>Carregando fotos...</p>
    <span class="sr-only">Carregando galeria de fotos</span>
  </div>
</template>

<style scoped>
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 40vh;
  gap: 1rem;
}

.spinner {
  width: 2.5rem;
  height: 2.5rem;
  border: 3px solid rgba(var(--accent-rgb, 99, 102, 241), 0.15);
  border-top-color: var(--accent-color, #6366f1);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin { 
  to { transform: rotate(360deg); } 
}

.loading p { 
  color: var(--text-muted, #71717a); 
  font-size: 0.875rem; 
}
</style>
