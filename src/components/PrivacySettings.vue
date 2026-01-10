/**
 * Vue Component - 1 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: ConfirmDialog
 */

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { load } from '@tauri-apps/plugin-store'
import { useRepoManager } from '../composables/useRepoManager'
import { useGitHubAuth } from '../composables/useGitHubAuth'
import { registerOverlay } from '../composables/useKeyboardShortcuts'
import ConfirmDialog from './ui/ConfirmDialog.vue'

const emit = defineEmits<{ close: [] }>()

// Register ESC key handler
let unregisterOverlay: (() => void) | null = null;

onMounted(() => {
  unregisterOverlay = registerOverlay('privacy-settings', () => emit('close'));
  loadSettings();
});

onUnmounted(() => {
  if (unregisterOverlay) {
    unregisterOverlay();
  }
});

const { token, repo } = useGitHubAuth()
const { syncPrivacy, updateVisibility, syncing } = useRepoManager()

const privacyLevel = ref<'public' | 'private' | 'unlisted'>('private')
const autoBackup = ref(true)
const deleteAfterUpload = ref(false)
const compressImages = ref(false)
const stripMetadata = ref(true)
const loading = ref(true)
const showConfirmPublic = ref(false)
const pendingPublicChange = ref(false)

async function loadSettings() {
  loading.value = true
  try {
    const store = await load('settings.json')
    privacyLevel.value = await store.get('privacyLevel') || 'private'
    autoBackup.value = await store.get('autoBackup') ?? true
    deleteAfterUpload.value = await store.get('deleteAfterUpload') ?? false
    compressImages.value = await store.get('compressImages') ?? false
    stripMetadata.value = await store.get('stripMetadata') ?? true

    if (token.value && repo.value) {
      const isPrivate = await syncPrivacy(repo.value, token.value)
      privacyLevel.value = isPrivate ? 'private' : 'public'
    }
  } catch {}
  loading.value = false
}

async function handlePrivacyChange(level: 'public' | 'private' | 'unlisted') {
  
  if (privacyLevel.value === 'private' && level === 'public') {
    pendingPublicChange.value = true
    showConfirmPublic.value = true
    return
  }
  privacyLevel.value = level
}

async function confirmPublicChange() {
  showConfirmPublic.value = false
  privacyLevel.value = 'public'
  pendingPublicChange.value = false
}

function cancelPublicChange() {
  showConfirmPublic.value = false
  pendingPublicChange.value = false
}

async function saveSettings() {
  const store = await load('settings.json')
  await store.set('privacyLevel', privacyLevel.value)
  await store.set('autoBackup', autoBackup.value)
  await store.set('deleteAfterUpload', deleteAfterUpload.value)
  await store.set('compressImages', compressImages.value)
  await store.set('stripMetadata', stripMetadata.value)
  await store.save()

  if (token.value && repo.value && privacyLevel.value !== 'unlisted') {
    try {
      await updateVisibility(repo.value, privacyLevel.value === 'private', token.value)
    } catch (e) {
      console.error('Failed to update repo visibility:', e)
    }
  }
  
  emit('close')
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <header class="modal-header">
        <h2>Privacidade e Segurança</h2>
        <button class="btn-close" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>
      </header>

      <div class="modal-body">
        <!-- Loading State -->
        <div v-if="loading" class="loading-state">
          <div class="spinner" />
          <p>Sincronizando com GitHub...</p>
        </div>

        <template v-else>
          <!-- Privacy Level -->
          <section class="setting-section">
            <h3>Visibilidade do Repositório</h3>
            <p class="setting-desc">Controle quem pode ver suas fotos</p>
            
            <div class="privacy-options">
              <label class="privacy-option" :class="{ active: privacyLevel === 'private' }">
                <input type="radio" :checked="privacyLevel === 'private'" @change="handlePrivacyChange('private')" />
                <div class="option-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg>
                </div>
                <div class="option-content">
                  <span class="option-title">Privado</span>
                  <span class="option-desc">Apenas você pode ver</span>
                </div>
              </label>

              <label class="privacy-option" :class="{ active: privacyLevel === 'unlisted' }">
                <input type="radio" :checked="privacyLevel === 'unlisted'" @change="handlePrivacyChange('unlisted')" />
                <div class="option-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                </div>
                <div class="option-content">
                  <span class="option-title">Não listado</span>
                  <span class="option-desc">Quem tem o link pode ver</span>
                </div>
              </label>

              <label class="privacy-option" :class="{ active: privacyLevel === 'public' }">
                <input type="radio" :checked="privacyLevel === 'public'" @change="handlePrivacyChange('public')" />
                <div class="option-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
                </div>
                <div class="option-content">
                  <span class="option-title">Público</span>
                  <span class="option-desc">Qualquer pessoa pode ver</span>
                </div>
              </label>
            </div>
            
            <p v-if="syncing" class="sync-status">
              <span class="sync-spinner" /> Sincronizando...
            </p>
          </section>

          <!-- Security Options -->
          <section class="setting-section">
            <h3>Segurança</h3>
            
            <label class="toggle-option">
              <div class="toggle-content">
                <span class="toggle-title">Remover metadados EXIF</span>
                <span class="toggle-desc">Remove localização e dados da câmera</span>
              </div>
              <div class="toggle" :class="{ active: stripMetadata }">
                <input type="checkbox" v-model="stripMetadata" />
                <span class="toggle-slider"></span>
              </div>
            </label>

            <label class="toggle-option">
              <div class="toggle-content">
                <span class="toggle-title">Comprimir imagens</span>
                <span class="toggle-desc">Reduz tamanho mantendo qualidade</span>
              </div>
              <div class="toggle" :class="{ active: compressImages }">
                <input type="checkbox" v-model="compressImages" />
                <span class="toggle-slider"></span>
              </div>
            </label>
          </section>

          <!-- Backup Options -->
          <section class="setting-section">
            <h3>Backup</h3>
            
            <label class="toggle-option">
              <div class="toggle-content">
                <span class="toggle-title">Backup automático</span>
                <span class="toggle-desc">Sincroniza novas fotos automaticamente</span>
              </div>
              <div class="toggle" :class="{ active: autoBackup }">
                <input type="checkbox" v-model="autoBackup" />
                <span class="toggle-slider"></span>
              </div>
            </label>

            <label class="toggle-option">
              <div class="toggle-content">
                <span class="toggle-title">Excluir após upload</span>
                <span class="toggle-desc">Remove arquivo local após backup</span>
              </div>
              <div class="toggle" :class="{ active: deleteAfterUpload }">
                <input type="checkbox" v-model="deleteAfterUpload" />
                <span class="toggle-slider"></span>
              </div>
            </label>
          </section>

          <!-- Future Integrations -->
          <section class="setting-section">
            <h3>Integrações</h3>
            <p class="setting-desc">Em breve</p>
            
            <div class="integrations">
              <div class="integration disabled">
                <div class="integration-icon google">
                  <svg viewBox="0 0 24 24" fill="currentColor"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/></svg>
                </div>
                <span>Google Photos</span>
                <span class="badge">Em breve</span>
              </div>
              <div class="integration disabled">
                <div class="integration-icon dropbox">
                  <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 6.134L6.069 9.797 12 13.459l5.931-3.662L12 6.134zM6.069 14.534L12 18.196l5.931-3.662L12 10.872l-5.931 3.662zm5.931-12L1.069 9.797 6.069 13.459 12 9.797l5.931 3.662 5-3.662L12 2.534z"/></svg>
                </div>
                <span>Dropbox</span>
                <span class="badge">Em breve</span>
              </div>
              <div class="integration disabled">
                <div class="integration-icon icloud">
                  <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96z"/></svg>
                </div>
                <span>iCloud</span>
                <span class="badge">Em breve</span>
              </div>
            </div>
          </section>
        </template>
      </div>

      <footer class="modal-footer">
        <button class="btn-cancel" @click="emit('close')">Cancelar</button>
        <button class="btn-save" @click="saveSettings" :disabled="loading || syncing">
          <span v-if="syncing" class="btn-spinner" />
          <span v-else>Salvar</span>
        </button>
      </footer>
    </div>

    <!-- Confirm Public Dialog -->
    <ConfirmDialog
      v-if="showConfirmPublic"
      title="Tornar repositório público?"
      message="Suas fotos ficarão visíveis para qualquer pessoa na internet. Esta ação pode ser revertida."
      confirm-text="Tornar Público"
      variant="warning"
      @confirm="confirmPublicChange"
      @cancel="cancelPublicChange"
    />
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  padding: 1rem;
}

.modal {
  width: 100%;
  max-width: 520px;
  max-height: 90vh;
  background: #18181b;
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 1rem;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid rgba(255,255,255,0.06);
}
.modal-header h2 { font-size: 1.125rem; font-weight: 600; }
.btn-close {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255,255,255,0.05);
  border: none;
  border-radius: 0.375rem;
  color: #71717a;
  cursor: pointer;
}
.btn-close:hover { background: rgba(255,255,255,0.1); color: #fafafa; }
.btn-close svg { width: 1.25rem; height: 1.25rem; }

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
}
.spinner {
  width: 2.5rem;
  height: 2.5rem;
  border: 3px solid rgba(var(--accent-rgb, 99, 102, 241), 0.2);
  border-top-color: var(--accent-color, #6366f1);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}
@keyframes spin { to { transform: rotate(360deg); } }
.loading-state p { color: #71717a; font-size: 0.875rem; }

.setting-section { margin-bottom: 2rem; }
.setting-section:last-child { margin-bottom: 0; }
.setting-section h3 { font-size: 0.875rem; font-weight: 600; margin-bottom: 0.25rem; }
.setting-desc { font-size: 0.75rem; color: #71717a; margin-bottom: 1rem; }

.sync-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: #71717a;
  margin-top: 0.75rem;
}
.sync-spinner {
  width: 0.875rem;
  height: 0.875rem;
  border: 2px solid rgba(var(--accent-rgb, 99, 102, 241), 0.2);
  border-top-color: var(--accent-color, #6366f1);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.privacy-options { display: flex; flex-direction: column; gap: 0.5rem; }
.privacy-option {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.875rem;
  background: rgba(255,255,255,0.02);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
}
.privacy-option:hover { background: rgba(255,255,255,0.05); }
.privacy-option.active { background: rgba(var(--accent-rgb, 99, 102, 241), 0.1); border-color: rgba(var(--accent-rgb, 99, 102, 241), 0.3); }
.privacy-option input { display: none; }
.option-icon {
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255,255,255,0.05);
  border-radius: 0.5rem;
}
.privacy-option.active .option-icon { background: rgba(var(--accent-rgb, 99, 102, 241), 0.2); color: var(--accent-color, #818cf8); }
.option-icon svg { width: 1.25rem; height: 1.25rem; }
.option-content { display: flex; flex-direction: column; }
.option-title { font-size: 0.875rem; font-weight: 500; }
.option-desc { font-size: 0.75rem; color: #71717a; }

.toggle-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.875rem;
  background: rgba(255,255,255,0.02);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 0.5rem;
  margin-bottom: 0.5rem;
  cursor: pointer;
}
.toggle-option:hover { background: rgba(255,255,255,0.05); }
.toggle-content { display: flex; flex-direction: column; }
.toggle-title { font-size: 0.875rem; font-weight: 500; }
.toggle-desc { font-size: 0.75rem; color: #71717a; }

.toggle {
  position: relative;
  width: 2.75rem;
  height: 1.5rem;
  background: rgba(255,255,255,0.1);
  border-radius: 1rem;
  transition: background 0.2s;
}
.toggle.active { background: var(--accent-color, #6366f1); }
.toggle input { display: none; }
.toggle-slider {
  position: absolute;
  top: 0.125rem;
  left: 0.125rem;
  width: 1.25rem;
  height: 1.25rem;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s;
}
.toggle.active .toggle-slider { transform: translateX(1.25rem); }

.integrations { display: flex; flex-direction: column; gap: 0.5rem; }
.integration {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.875rem;
  background: rgba(255,255,255,0.02);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 0.5rem;
}
.integration.disabled { opacity: 0.5; }
.integration-icon {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.375rem;
}
.integration-icon svg { width: 1.25rem; height: 1.25rem; }
.integration-icon.google { background: #fff; color: #4285f4; }
.integration-icon.dropbox { background: #0061fe; color: white; }
.integration-icon.icloud { background: #3693f3; color: white; }
.integration span { font-size: 0.875rem; }
.badge {
  margin-left: auto;
  padding: 0.25rem 0.5rem;
  background: rgba(255,255,255,0.1);
  border-radius: 1rem;
  font-size: 0.625rem;
  color: #71717a;
  text-transform: uppercase;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid rgba(255,255,255,0.06);
}
.btn-cancel {
  padding: 0.625rem 1rem;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  color: #a1a1aa;
  font-size: 0.875rem;
  border-radius: 0.5rem;
  cursor: pointer;
}
.btn-cancel:hover { background: rgba(255,255,255,0.1); }
.btn-save {
  padding: 0.625rem 1.5rem;
  background: linear-gradient(135deg, var(--accent-color, #6366f1), #8b5cf6);
  border: none;
  color: white;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 0.5rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 80px;
}
.btn-save:hover:not(:disabled) { filter: brightness(1.1); }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-spinner {
  width: 1rem;
  height: 1rem;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>