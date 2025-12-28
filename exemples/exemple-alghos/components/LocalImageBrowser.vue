<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useDataDriver } from '../composables/useDataDriver'

interface LocalImage {
  path: string
  name: string
  size: number
  modified: number
  selected: boolean
}

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'import', images: string[], targetDriverId: string): void
}>()

const { githubDrivers, loadDrivers } = useDataDriver()

const currentFolder = ref<string | null>(null)
const images = ref<LocalImage[]>([])
const loading = ref(false)
const targetDriverId = ref<string | null>(null)
const selectAll = ref(false)
const viewSize = ref(120)

const selectedImages = computed(() => images.value.filter(img => img.selected))
const selectedCount = computed(() => selectedImages.value.length)
const folderName = computed(() => currentFolder.value?.split('/').pop() || '')

onMounted(async () => {
  await loadDrivers()
  if (githubDrivers.value.length > 0) {
    targetDriverId.value = githubDrivers.value[0].id
  }
})

async function handleSelectFolder() {
  const folder = await open({ multiple: false, directory: true })
  if (folder && typeof folder === 'string') {
    currentFolder.value = folder
    await scanFolder(folder)
  }
}

async function scanFolder(folderPath: string) {
  loading.value = true
  images.value = []
  
  try {
    const result = await invoke<{ path: string; name: string; size: number; modified: number }[]>(
      'scan_folder', 
      { path: folderPath }
    )
    
    images.value = result.map(img => ({
      ...img,
      selected: false
    }))
  } catch (e) {
    console.error('Failed to scan folder:', e)
  } finally {
    loading.value = false
  }
}

function toggleSelectAll() {
  selectAll.value = !selectAll.value
  images.value.forEach(img => img.selected = selectAll.value)
}

function toggleImage(image: LocalImage) {
  image.selected = !image.selected
  selectAll.value = images.value.every(img => img.selected)
}

function handleImport() {
  if (selectedCount.value === 0 || !targetDriverId.value) return
  
  const paths = selectedImages.value.map(img => img.path)
  emit('import', paths, targetDriverId.value)
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function getImageUrl(path: string): string {
  // Use Tauri's convertFileSrc for proper asset loading
  return convertFileSrc(path)
}
</script>

<template>
  <div class="browser-overlay" @click.self="emit('close')">
    <div class="browser-panel">
      <!-- Header -->
      <div class="panel-header">
        <div class="header-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <circle cx="9" cy="9" r="2"/>
            <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
          </svg>
          <h2>Importar Imagens Locais</h2>
        </div>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Toolbar -->
      <div class="toolbar">
        <div class="folder-section">
          <button class="btn-folder" @click="handleSelectFolder">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            {{ currentFolder ? 'Alterar Pasta' : 'Selecionar Pasta' }}
          </button>
          <span v-if="currentFolder" class="folder-path">{{ folderName }}</span>
        </div>
        
        <div class="target-section">
          <label>Importar para:</label>
          <select v-model="targetDriverId" :disabled="githubDrivers.length === 0">
            <option v-for="driver in githubDrivers" :key="driver.id" :value="driver.id">
              {{ driver.name }}
            </option>
          </select>
        </div>
      </div>

      <!-- Content -->
      <div class="panel-content">
        <div v-if="!currentFolder" class="empty-state">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
          <p>Selecione uma pasta para visualizar as imagens</p>
          <button class="btn-primary" @click="handleSelectFolder">Selecionar Pasta</button>
        </div>

        <div v-else-if="loading" class="loading-state">
          <div class="spinner"></div>
          <p>Escaneando pasta...</p>
        </div>

        <div v-else-if="images.length === 0" class="empty-state">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <circle cx="9" cy="9" r="2"/>
            <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
          </svg>
          <p>Nenhuma imagem encontrada nesta pasta</p>
        </div>

        <template v-else>
          <div class="selection-bar">
            <label class="select-all">
              <input type="checkbox" :checked="selectAll" @change="toggleSelectAll" />
              <span>Selecionar todas ({{ images.length }})</span>
            </label>
            <span class="selected-count">{{ selectedCount }} selecionadas</span>
          </div>

          <div class="image-grid" :style="{ '--grid-size': viewSize + 'px' }">
            <div 
              v-for="image in images" 
              :key="image.path"
              :class="['image-item', { selected: image.selected }]"
              @click="toggleImage(image)"
            >
              <img :src="getImageUrl(image.path)" :alt="image.name" loading="lazy" />
              <div class="image-overlay">
                <div class="checkbox" :class="{ checked: image.selected }">
                  <svg v-if="image.selected" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                  </svg>
                </div>
              </div>
              <div class="image-info">
                <span class="image-name">{{ image.name }}</span>
                <span class="image-size">{{ formatSize(image.size) }}</span>
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- Footer -->
      <div class="panel-footer">
        <button class="btn-cancel" @click="emit('close')">Cancelar</button>
        <button 
          class="btn-import" 
          :disabled="selectedCount === 0 || !targetDriverId"
          @click="handleImport"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          Importar {{ selectedCount }} {{ selectedCount === 1 ? 'imagem' : 'imagens' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.browser-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.browser-panel {
  width: 95%;
  max-width: 900px;
  height: 85vh;
  background: var(--surface-1, #111113);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  border-radius: var(--radius-xl, 20px);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.header-title svg {
  width: 1.5rem;
  height: 1.5rem;
  color: var(--accent-color, #00ff41);
}

.header-title h2 {
  font-size: 1.125rem;
  font-weight: 600;
}

.close-btn {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  cursor: pointer;
  border-radius: var(--radius-sm, 6px);
}

.close-btn:hover {
  background: var(--surface-2, #18181b);
  color: var(--text-primary, #fafafa);
}

.close-btn svg { width: 1.25rem; height: 1.25rem; }

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  background: var(--surface-0, #0a0a0b);
  border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
}

.folder-section {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.btn-folder {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  color: var(--text-primary, #fafafa);
  font-size: 0.875rem;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
}

.btn-folder:hover {
  background: var(--surface-3, #27272a);
}

.btn-folder svg { width: 1rem; height: 1rem; }

.folder-path {
  font-size: 0.875rem;
  color: var(--text-secondary, #a1a1aa);
  font-family: var(--font-mono, monospace);
}

.target-section {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.target-section label {
  font-size: 0.875rem;
  color: var(--text-secondary, #a1a1aa);
}

.target-section select {
  padding: 0.5rem 0.75rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  border-radius: var(--radius-sm, 6px);
  color: var(--text-primary, #fafafa);
  font-size: 0.875rem;
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.empty-state, .loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 1rem;
  text-align: center;
}

.empty-state svg {
  width: 4rem;
  height: 4rem;
  color: var(--text-muted, #52525b);
}

.empty-state p, .loading-state p {
  color: var(--text-secondary, #a1a1aa);
}

.btn-primary {
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, var(--accent-color, #00ff41), var(--accent-secondary, #008f11));
  border: none;
  color: var(--void, #000);
  font-weight: 600;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
}

.spinner {
  width: 2.5rem;
  height: 2.5rem;
  border: 3px solid var(--surface-3, #27272a);
  border-top-color: var(--accent-color, #00ff41);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.selection-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
  padding: 0.75rem 1rem;
  background: var(--surface-2, #18181b);
  border-radius: var(--radius-md, 10px);
}

.select-all {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
}

.select-all input {
  width: 1rem;
  height: 1rem;
  accent-color: var(--accent-color, #00ff41);
}

.selected-count {
  font-size: 0.875rem;
  color: var(--accent-color, #00ff41);
  font-weight: 500;
}

.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--grid-size, 120px), 1fr));
  gap: 0.75rem;
}

.image-item {
  position: relative;
  aspect-ratio: 1;
  border-radius: var(--radius-md, 10px);
  overflow: hidden;
  cursor: pointer;
  background: var(--surface-2, #18181b);
  transition: all 0.2s;
}

.image-item:hover {
  transform: scale(1.02);
}

.image-item.selected {
  outline: 2px solid var(--accent-color, #00ff41);
  outline-offset: 2px;
}

.image-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.image-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  opacity: 0;
  transition: opacity 0.2s;
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  padding: 0.5rem;
}

.image-item:hover .image-overlay,
.image-item.selected .image-overlay {
  opacity: 1;
}

.checkbox {
  width: 1.5rem;
  height: 1.5rem;
  border: 2px solid white;
  border-radius: var(--radius-sm, 6px);
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
}

.checkbox.checked {
  background: var(--accent-color, #00ff41);
  border-color: var(--accent-color, #00ff41);
}

.checkbox svg {
  width: 1rem;
  height: 1rem;
  color: var(--void, #000);
}

.image-info {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.5rem;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.8));
  opacity: 0;
  transition: opacity 0.2s;
}

.image-item:hover .image-info {
  opacity: 1;
}

.image-name {
  display: block;
  font-size: 0.6875rem;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-size {
  font-size: 0.625rem;
  color: rgba(255, 255, 255, 0.7);
}

.panel-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
  background: var(--surface-0, #0a0a0b);
}

.btn-cancel {
  padding: 0.625rem 1.25rem;
  background: transparent;
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  color: var(--text-secondary, #a1a1aa);
  font-size: 0.875rem;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
}

.btn-cancel:hover {
  background: var(--surface-2, #18181b);
  color: var(--text-primary, #fafafa);
}

.btn-import {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1.25rem;
  background: linear-gradient(135deg, var(--accent-color, #00ff41), var(--accent-secondary, #008f11));
  border: none;
  color: var(--void, #000);
  font-size: 0.875rem;
  font-weight: 600;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
}

.btn-import:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-import svg {
  width: 1rem;
  height: 1rem;
}
</style>
