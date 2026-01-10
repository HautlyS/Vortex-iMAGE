<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useFolder, validateFolderName } from '../composables/useFolder'
import { registerOverlay } from '../composables/useKeyboardShortcuts'

const props = defineProps<{
  parentPath?: string | null
}>()

const emit = defineEmits<{
  (e: 'created', path: string): void
  (e: 'close'): void
}>()

// Register ESC key handler
let unregisterOverlay: (() => void) | null = null;

onMounted(() => {
  unregisterOverlay = registerOverlay('folder-creator', () => emit('close'));
});

onUnmounted(() => {
  if (unregisterOverlay) {
    unregisterOverlay();
  }
});

const { createFolder, creating, error, clearError } = useFolder()

const folderName = ref('')

const validation = computed(() => validateFolderName(folderName.value))
const isValid = computed(() => folderName.value.trim() && validation.value.valid)

const fullPath = computed(() => {
  const name = folderName.value.trim()
  if (!name) return ''
  return props.parentPath ? `${props.parentPath}/${name}` : name
})

async function handleCreate() {
  if (!isValid.value) return
  
  clearError()
  
  try {
    const path = await createFolder(fullPath.value)
    emit('created', path)
  } catch {
    // Error is handled by composable
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && isValid.value && !creating.value) handleCreate()
  if (e.key === 'Escape') emit('close')
}
</script>

<template>
  <div class="folder-creator-overlay" @click.self="$emit('close')">
    <div class="folder-creator" @keydown="handleKeydown">
      <div class="creator-header">
        <div class="header-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 10v6m-3-3h6" />
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
        </div>
        <div class="header-text">
          <h2>Nova Pasta</h2>
          <p v-if="parentPath">em {{ parentPath }}</p>
          <p v-else>na raiz</p>
        </div>
        <button class="close-btn" @click="$emit('close')" aria-label="Fechar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="creator-body">
        <div class="input-group">
          <label for="folder-name">Nome da pasta</label>
          <input
            id="folder-name"
            v-model="folderName"
            type="text"
            placeholder="Minha pasta"
            autofocus
            :class="{ invalid: folderName && !validation.valid }"
          />
          <span v-if="fullPath" class="path-preview">photos/{{ fullPath }}</span>
          <span v-if="folderName && !validation.valid" class="validation-error">{{ validation.error }}</span>
        </div>

        <Transition name="fade">
          <div v-if="error" class="error-message">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 8v4m0 4h.01" />
            </svg>
            {{ error }}
          </div>
        </Transition>
      </div>

      <div class="creator-footer">
        <button class="btn-cancel" @click="$emit('close')">Cancelar</button>
        <button 
          class="btn-create" 
          :disabled="!isValid || creating"
          @click="handleCreate"
        >
          <svg v-if="creating" class="spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" stroke-dasharray="32" stroke-dashoffset="32" />
          </svg>
          <template v-else>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 5v14m-7-7h14" />
            </svg>
            Criar
          </template>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.folder-creator-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.folder-creator {
  width: 100%;
  max-width: 420px;
  background: var(--systemStandardThickMaterialSover, rgba(28, 28, 30, 0.88));
  backdrop-filter: blur(40px);
  border: 1px solid var(--labelDivider, rgba(255, 255, 255, 0.1));
  border-radius: var(--global-border-radius-xlarge, 24px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.05) inset;
  animation: slideUp 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  overflow: hidden;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px) scale(0.96); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.creator-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--labelDivider, rgba(255, 255, 255, 0.1));
}

.header-icon {
  width: 3rem;
  height: 3rem;
  background: linear-gradient(135deg, var(--keyColor, #0a84ff), color-mix(in srgb, var(--keyColor, #0a84ff), #000 20%));
  border-radius: var(--global-border-radius-medium, 12px);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(10, 132, 255, 0.3);
}

.header-icon svg {
  width: 1.5rem;
  height: 1.5rem;
  color: #fff;
}

.header-text {
  flex: 1;
}

.header-text h2 {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--systemPrimary, rgba(255, 255, 255, 0.85));
  margin: 0;
}

.header-text p {
  font-size: 0.8125rem;
  color: var(--systemSecondary, rgba(255, 255, 255, 0.55));
  margin: 0.125rem 0 0;
}

.close-btn {
  width: 2rem;
  height: 2rem;
  background: var(--systemQuinary, rgba(255, 255, 255, 0.05));
  border: none;
  border-radius: 50%;
  color: var(--systemSecondary, rgba(255, 255, 255, 0.55));
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.close-btn:hover {
  background: var(--systemQuaternary, rgba(255, 255, 255, 0.1));
  color: var(--systemPrimary, rgba(255, 255, 255, 0.85));
}

.close-btn svg {
  width: 1rem;
  height: 1rem;
}

.creator-body {
  padding: 1.5rem;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.input-group label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--systemSecondary, rgba(255, 255, 255, 0.55));
}

.input-group input {
  padding: 0.875rem 1rem;
  font-size: 1rem;
  background: var(--systemGray6, #1c1c1e);
  border: 1px solid transparent;
  border-radius: var(--global-border-radius-medium, 12px);
  color: var(--systemPrimary, rgba(255, 255, 255, 0.85));
  transition: all 0.2s;
}

.input-group input:focus {
  outline: none;
  border-color: var(--keyColor, #0a84ff);
  box-shadow: 0 0 0 4px rgba(10, 132, 255, 0.15);
}

.input-group input.invalid {
  border-color: var(--systemRed, #ff453a);
}

.path-preview {
  font-size: 0.75rem;
  color: var(--systemTertiary, rgba(255, 255, 255, 0.25));
  font-family: 'SF Mono', ui-monospace, monospace;
  padding: 0.25rem 0;
}

.validation-error {
  font-size: 0.75rem;
  color: var(--systemRed, #ff453a);
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 1rem;
  padding: 0.75rem 1rem;
  background: rgba(255, 69, 58, 0.1);
  border: 1px solid rgba(255, 69, 58, 0.2);
  border-radius: var(--global-border-radius-small, 9px);
  color: var(--systemRed, #ff453a);
  font-size: 0.875rem;
}

.error-message svg {
  width: 1rem;
  height: 1rem;
  flex-shrink: 0;
}

.creator-footer {
  display: flex;
  gap: 0.75rem;
  padding: 1rem 1.5rem 1.5rem;
}

.btn-cancel {
  flex: 1;
  padding: 0.75rem 1.25rem;
  background: var(--systemQuinary, rgba(255, 255, 255, 0.05));
  border: none;
  border-radius: var(--global-border-radius-medium, 12px);
  color: var(--systemPrimary, rgba(255, 255, 255, 0.85));
  font-size: 0.9375rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-cancel:hover {
  background: var(--systemQuaternary, rgba(255, 255, 255, 0.1));
}

.btn-create {
  flex: 1;
  padding: 0.75rem 1.25rem;
  background: var(--keyColor, #0a84ff);
  border: none;
  border-radius: var(--global-border-radius-medium, 12px);
  color: #fff;
  font-size: 0.9375rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.btn-create:hover:not(:disabled) {
  background: color-mix(in srgb, var(--keyColor, #0a84ff), #000 10%);
}

.btn-create:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-create svg {
  width: 1rem;
  height: 1rem;
}

.spinner {
  width: 1.25rem;
  height: 1.25rem;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
