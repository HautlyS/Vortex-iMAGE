/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRepoManager, validateRepoName } from '../composables/useRepoManager'
import { useGitHubAuth } from '../composables/useGitHubAuth'
import { registerOverlay } from '../composables/useKeyboardShortcuts'

const emit = defineEmits<{
  (e: 'created', repo: string): void
  (e: 'close'): void
}>()

// Register ESC key handler
let unregisterOverlay: (() => void) | null = null;

onMounted(() => {
  unregisterOverlay = registerOverlay('repo-creator', () => emit('close'));
});

onUnmounted(() => {
  if (unregisterOverlay) {
    unregisterOverlay();
  }
});

const { token, setRepo } = useGitHubAuth()
const { createRepo, creating, error, clearError } = useRepoManager()

const name = ref('')
const description = ref('')
const visibility = ref<'public' | 'private'>('private')

const nameValidation = computed(() => {
  if (!name.value) return { valid: true, error: undefined }
  return validateRepoName(name.value)
})

const canSubmit = computed(() => {
  return name.value.length > 0 && nameValidation.value.valid && !creating.value
})

watch(name, () => {
  clearError()
})

async function handleSubmit() {
  if (!canSubmit.value || !token.value) return

  try {
    const result = await createRepo(
      {
        name: name.value,
        description: description.value,
        visibility: visibility.value,
      },
      token.value
    )

    setRepo(result.full_name)
    emit('created', result.full_name)
  } catch {
    
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="dialog-overlay" @click.self="emit('close')">
      <div class="dialog">
        <div class="dialog-header">
          <h2>Criar Novo Repositório</h2>
          <button class="close-btn" @click="emit('close')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <form class="dialog-content" @submit.prevent="handleSubmit">
          <!-- Repository Name -->
          <div class="form-group">
            <label for="repo-name">Nome do Repositório</label>
            <input
              id="repo-name"
              v-model="name"
              type="text"
              placeholder="meu-album-de-fotos"
              :class="{ error: name && !nameValidation.valid }"
              autocomplete="off"
            />
            <span v-if="name && !nameValidation.valid" class="error-text">
              {{ nameValidation.error }}
            </span>
            <span v-else class="hint-text">
              Use letras, números, hífens, underscores e pontos
            </span>
          </div>

          <!-- Description -->
          <div class="form-group">
            <label for="repo-desc">Descrição (opcional)</label>
            <textarea
              id="repo-desc"
              v-model="description"
              placeholder="Minha coleção de fotos pessoais"
              rows="2"
            />
          </div>

          <!-- Visibility -->
          <div class="form-group">
            <label>Visibilidade</label>
            <div class="visibility-options">
              <button
                type="button"
                class="visibility-btn"
                :class="{ active: visibility === 'private' }"
                @click="visibility = 'private'"
              >
                <div class="visibility-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                  </svg>
                </div>
                <div class="visibility-content">
                  <span class="visibility-title">Privado</span>
                  <span class="visibility-desc">Apenas você pode ver</span>
                </div>
              </button>

              <button
                type="button"
                class="visibility-btn"
                :class="{ active: visibility === 'public' }"
                @click="visibility = 'public'"
              >
                <div class="visibility-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10" />
                    <line x1="2" y1="12" x2="22" y2="12" />
                    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
                  </svg>
                </div>
                <div class="visibility-content">
                  <span class="visibility-title">Público</span>
                  <span class="visibility-desc">Qualquer pessoa pode ver</span>
                </div>
              </button>
            </div>
          </div>

          <!-- Error Message -->
          <div v-if="error" class="error-message">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 8v4M12 16h.01" />
            </svg>
            <span>{{ error }}</span>
          </div>

          <!-- Actions -->
          <div class="dialog-actions">
            <button type="button" class="btn-secondary" @click="emit('close')">
              Cancelar
            </button>
            <button type="submit" class="btn-primary" :disabled="!canSubmit">
              <span v-if="creating" class="spinner" />
              <span v-else>Criar Repositório</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.dialog {
  width: 100%;
  max-width: 480px;
  background: #1a1a1c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 1rem;
  overflow: hidden;
  animation: slideUp 0.2s ease-out;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.dialog-header h2 {
  font-size: 1.125rem;
  font-weight: 600;
  color: #fafafa;
}

.close-btn {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #71717a;
  cursor: pointer;
  border-radius: 0.375rem;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fafafa;
}

.close-btn svg {
  width: 1.25rem;
  height: 1.25rem;
}

.dialog-content {
  padding: 1.5rem;
}

.form-group {
  margin-bottom: 1.25rem;
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: #a1a1aa;
  margin-bottom: 0.5rem;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 0.75rem 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.5rem;
  color: #fafafa;
  font-size: 0.875rem;
  transition: all 0.2s;
}

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.15);
}

.form-group input.error {
  border-color: #ef4444;
}

.form-group input::placeholder,
.form-group textarea::placeholder {
  color: #52525b;
}

.form-group textarea {
  resize: vertical;
  min-height: 60px;
}

.hint-text {
  display: block;
  font-size: 0.75rem;
  color: #52525b;
  margin-top: 0.375rem;
}

.error-text {
  display: block;
  font-size: 0.75rem;
  color: #ef4444;
  margin-top: 0.375rem;
}

.visibility-options {
  display: flex;
  gap: 0.75rem;
}

.visibility-btn {
  flex: 1;
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 1rem;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
}

.visibility-btn:hover {
  background: rgba(255, 255, 255, 0.05);
}

.visibility-btn.active {
  background: rgba(99, 102, 241, 0.1);
  border-color: #6366f1;
}

.visibility-icon {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.15);
  border-radius: 0.375rem;
  color: #818cf8;
  flex-shrink: 0;
}

.visibility-icon svg {
  width: 1rem;
  height: 1rem;
}

.visibility-title {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  color: #fafafa;
  margin-bottom: 0.125rem;
}

.visibility-desc {
  display: block;
  font-size: 0.75rem;
  color: #71717a;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 0.5rem;
  color: #ef4444;
  font-size: 0.875rem;
  margin-bottom: 1.25rem;
}

.error-message svg {
  width: 1.25rem;
  height: 1.25rem;
  flex-shrink: 0;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding-top: 0.5rem;
}

.btn-secondary {
  padding: 0.75rem 1.25rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.5rem;
  color: #a1a1aa;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fafafa;
}

.btn-primary {
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border: none;
  border-radius: 0.5rem;
  color: white;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 140px;
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  width: 1rem;
  height: 1rem;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>