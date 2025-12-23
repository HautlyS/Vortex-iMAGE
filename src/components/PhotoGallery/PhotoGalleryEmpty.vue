<script setup lang="ts">
const emit = defineEmits<{ refresh: [] }>()
</script>

<template>
  <div class="empty">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="18" rx="2"/>
        <circle cx="9" cy="9" r="2"/>
        <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
      </svg>
    </div>
    <h3>Nenhuma foto ainda</h3>
    <p>Arraste fotos ou clique em Upload para começar</p>
    <button class="btn-refresh" @click="emit('refresh')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12a9 9 0 11-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/>
        <path d="M21 3v5h-5"/>
      </svg>
      Atualizar
    </button>
  </div>
</template>

<style scoped>
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 40vh;
  text-align: center;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.empty-icon {
  width: 5rem;
  height: 5rem;
  color: var(--text-disabled, #3f3f46);
  margin-bottom: 1.5rem;
}

.empty-icon svg { 
  width: 100%; 
  height: 100%; 
}

.empty h3 { 
  font-size: 1.25rem; 
  font-weight: 600; 
  margin-bottom: 0.5rem; 
}

.empty p { 
  color: var(--text-muted, #71717a); 
  margin-bottom: 1.5rem; 
  max-width: 280px; 
}

.btn-refresh {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1rem;
  background: var(--bg-hover, rgba(255,255,255,0.04));
  border: 1px solid var(--border-default, rgba(255,255,255,0.1));
  color: var(--text-secondary, #a1a1aa);
  font-size: 0.875rem;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-refresh:hover { 
  background: var(--bg-active, rgba(255,255,255,0.08)); 
  color: var(--text-primary, #fafafa); 
}

.btn-refresh svg { 
  width: 1rem; 
  height: 1rem; 
}
</style>
