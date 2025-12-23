/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface FolderScanResult {
  path: string
  name: string
  image_count: number
  total_size: number
  subfolders: FolderScanResult[]
}

const props = defineProps<{
  folderPath: string
}>()

const emit = defineEmits<{
  (e: 'confirm', mode: 'album' | 'recursive'): void
  (e: 'cancel'): void
}>()

const loading = ref(true)
const scanResult = ref<FolderScanResult | null>(null)
const error = ref<string | null>(null)

function countTotalImages(folder: FolderScanResult): number {
  return folder.image_count + folder.subfolders.reduce((sum, sub) => sum + countTotalImages(sub), 0)
}

function countTotalSubfolders(folder: FolderScanResult): number {
  return folder.subfolders.length + folder.subfolders.reduce((sum, sub) => sum + countTotalSubfolders(sub), 0)
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

onMounted(async () => {
  try {
    scanResult.value = await invoke<FolderScanResult>('scan_folder', { path: props.folderPath })
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <Teleport to="body">
    <div class="dialog-overlay" @click.self="emit('cancel')">
      <div class="dialog">
        <div class="dialog-header">
          <h2>Upload de Pasta</h2>
          <button class="close-btn" @click="emit('cancel')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="dialog-content">
          <!-- Loading State -->
          <div v-if="loading" class="loading-state">
            <div class="spinner" />
            <p>Analisando pasta...</p>
          </div>

          <!-- Error State -->
          <div v-else-if="error" class="error-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 8v4M12 16h.01" />
            </svg>
            <p>{{ error }}</p>
          </div>

          <!-- Scan Result -->
          <template v-else-if="scanResult">
            <div class="folder-info">
              <div class="folder-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                </svg>
              </div>
              <div class="folder-details">
                <h3>{{ scanResult.name }}</h3>
                <div class="folder-stats">
                  <span>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="3" width="18" height="18" rx="2" />
                      <circle cx="9" cy="9" r="2" />
                      <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />
                    </svg>
                    {{ countTotalImages(scanResult) }} imagens
                  </span>
                  <span v-if="scanResult.subfolders.length > 0">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                    </svg>
                    {{ countTotalSubfolders(scanResult) }} subpastas
                  </span>
                  <span>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                      <polyline points="7,10 12,15 17,10" />
                      <line x1="12" y1="15" x2="12" y2="3" />
                    </svg>
                    {{ formatSize(scanResult.total_size) }}
                  </span>
                </div>
              </div>
            </div>

            <div class="upload-options">
              <h4>Como deseja fazer o upload?</h4>

              <button class="option-btn" @click="emit('confirm', 'album')">
                <div class="option-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                    <path d="M12 11v6M9 14h6" />
                  </svg>
                </div>
                <div class="option-content">
                  <span class="option-title">Criar Álbum</span>
                  <span class="option-desc">
                    Mantém a estrutura de pastas como álbuns e sub-álbuns
                  </span>
                </div>
              </button>

              <button class="option-btn" @click="emit('confirm', 'recursive')">
                <div class="option-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="3" width="18" height="18" rx="2" />
                    <circle cx="9" cy="9" r="2" />
                    <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />
                  </svg>
                </div>
                <div class="option-content">
                  <span class="option-title">Importar Todas as Imagens</span>
                  <span class="option-desc">
                    Extrai todas as imagens para a pasta raiz de fotos
                  </span>
                </div>
              </button>
            </div>
          </template>
        </div>
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

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2rem;
}

.spinner {
  width: 2.5rem;
  height: 2.5rem;
  border: 3px solid rgba(99, 102, 241, 0.2);
  border-top-color: #6366f1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-state p {
  color: #71717a;
  font-size: 0.875rem;
}

.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2rem;
  color: #ef4444;
}

.error-state svg {
  width: 3rem;
  height: 3rem;
  margin-bottom: 1rem;
}

.error-state p {
  font-size: 0.875rem;
  text-align: center;
}

.folder-info {
  display: flex;
  gap: 1rem;
  padding: 1rem;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 0.75rem;
  margin-bottom: 1.5rem;
}

.folder-icon {
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.15);
  border-radius: 0.5rem;
  color: #818cf8;
  flex-shrink: 0;
}

.folder-icon svg {
  width: 1.5rem;
  height: 1.5rem;
}

.folder-details h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #fafafa;
  margin-bottom: 0.5rem;
}

.folder-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.folder-stats span {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: #71717a;
}

.folder-stats svg {
  width: 0.875rem;
  height: 0.875rem;
}

.upload-options h4 {
  font-size: 0.875rem;
  font-weight: 500;
  color: #a1a1aa;
  margin-bottom: 1rem;
}

.option-btn {
  width: 100%;
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 1rem;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
  margin-bottom: 0.75rem;
}

.option-btn:last-child {
  margin-bottom: 0;
}

.option-btn:hover {
  background: rgba(99, 102, 241, 0.1);
  border-color: rgba(99, 102, 241, 0.3);
}

.option-icon {
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.15);
  border-radius: 0.5rem;
  color: #818cf8;
  flex-shrink: 0;
}

.option-icon svg {
  width: 1.25rem;
  height: 1.25rem;
}

.option-content {
  flex: 1;
}

.option-title {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  color: #fafafa;
  margin-bottom: 0.25rem;
}

.option-desc {
  display: block;
  font-size: 0.75rem;
  color: #71717a;
  line-height: 1.4;
}
</style>