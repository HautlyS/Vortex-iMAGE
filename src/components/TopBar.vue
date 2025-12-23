<script setup lang="ts">
import { computed } from 'vue'
import HaloSearch from './HaloSearch.vue'
import RainbowButton from './RainbowButton.vue'

interface Breadcrumb {
  label: string
  path: string | null
}

const props = defineProps<{
  breadcrumbs: Breadcrumb[]
  photoCount: number
  selectedCount: number
  searchQuery: string
  viewMode: 'grid' | 'list'
  uploadProgress: number
  canGoBack: boolean
  canGoForward: boolean
  canGoUp: boolean
  canUpload: boolean
}>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  'update:viewMode': [value: 'grid' | 'list']
  back: []
  forward: []
  up: []
  upload: []
  uploadFolder: []
  breadcrumbClick: [crumb: Breadcrumb, index: number]
}>()

const localSearch = computed({
  get: () => props.searchQuery,
  set: (v) => emit('update:searchQuery', v)
})
</script>

<template>
  <header class="top-bar">
    <div class="top-bar-left">
      <div class="breadcrumb-nav">
        <button v-if="canGoBack" @click="emit('back')" class="nav-btn" title="Voltar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m15 18-6-6 6-6"/>
          </svg>
        </button>
        <button v-if="canGoForward" @click="emit('forward')" class="nav-btn" title="Avançar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m9 18 6-6-6-6"/>
          </svg>
        </button>
        <button v-if="canGoUp" @click="emit('up')" class="nav-btn" title="Subir">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m18 15-6-6-6 6"/>
          </svg>
        </button>
        
        <div class="breadcrumb-path">
          <template v-for="(crumb, i) in breadcrumbs" :key="i">
            <span v-if="i > 0" class="breadcrumb-sep">/</span>
            <button 
              class="breadcrumb-link" 
              :class="{ active: i === breadcrumbs.length - 1 }"
              @click="emit('breadcrumbClick', crumb, i)"
            >
              {{ crumb.label }}
            </button>
          </template>
        </div>
      </div>
      <div class="photo-stats">
        <span v-if="photoCount > 0" class="photo-count">{{ photoCount }} fotos</span>
        <span v-if="selectedCount > 0" class="selected-count">{{ selectedCount }} selecionadas</span>
      </div>
    </div>

    <div class="top-bar-center">
      <HaloSearch v-model="localSearch" placeholder="Pesquisar fotos..." class="desktop-search" />
    </div>

    <div class="top-bar-actions">
      <div class="upload-group">
        <RainbowButton @click="emit('upload')" :disabled="!canUpload" class="upload-btn">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17,8 12,3 7,8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
          <span>Upload</span>
          <span v-if="uploadProgress" class="upload-badge">{{ uploadProgress }}</span>
        </RainbowButton>
        <button class="btn-folder" @click="emit('uploadFolder')" :disabled="!canUpload" title="Upload pasta">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
        </button>
      </div>

      <div class="view-toggle">
        <button :class="{ active: viewMode === 'grid' }" @click="emit('update:viewMode', 'grid')" title="Grade">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
        </button>
        <button :class="{ active: viewMode === 'list' }" @click="emit('update:viewMode', 'list')" title="Lista">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.top-bar {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding: 1.25rem 1.75rem;
  min-height: var(--header-height);
  position: relative;
  margin: 1rem;
  border-radius: 16px;
  background: transparent;
}

.top-bar-left {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.top-bar-center {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
}

.top-bar-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-left: auto;
}

.breadcrumb-nav {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.breadcrumb-path {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  margin-left: 0.5rem;
}

.breadcrumb-sep {
  color: var(--text-muted);
  font-size: 0.875rem;
}

.breadcrumb-link {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 0.9375rem;
  font-weight: 500;
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}

.breadcrumb-link:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.05);
}

.breadcrumb-link.active {
  color: var(--text-primary);
  font-weight: 600;
}

.nav-btn {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.nav-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.nav-btn svg {
  width: 1rem;
  height: 1rem;
}

.photo-stats {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.photo-count {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.selected-count {
  font-size: 0.8125rem;
  color: var(--accent-color);
  font-weight: 600;
  background: var(--accent-light);
  padding: 0.25rem 0.5rem;
  border-radius: var(--radius-sm);
}

.desktop-search {
  width: 520px;
}

.upload-group {
  display: flex;
  gap: 0.5rem;
}

.upload-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  position: relative;
}

.upload-btn svg {
  width: 1.125rem;
  height: 1.125rem;
}

.upload-badge {
  margin-left: 0.25rem;
  padding: 0.125rem 0.5rem;
  font-size: 0.75rem;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 9999px;
}

.btn-folder {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 10px 14px;
  background: rgba(var(--accent-rgb), 0.15);
  border: none;
  border-radius: 9999px;
  color: var(--accent-color);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.btn-folder:hover {
  background: rgba(var(--accent-rgb), 0.25);
}

.btn-folder:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-folder svg {
  width: 1.125rem;
  height: 1.125rem;
}

.view-toggle {
  display: flex;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: var(--radius-md);
  padding: 0.25rem;
}

.view-toggle button {
  width: 2.25rem;
  height: 2.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}

.view-toggle button:hover {
  color: var(--text-primary);
}

.view-toggle button.active {
  background: linear-gradient(135deg, var(--accent-color), var(--accent-secondary));
  color: #000;
}

.view-toggle button svg {
  width: 1.125rem;
  height: 1.125rem;
}

@media (max-width: 768px) {
  .top-bar {
    padding: 1rem;
    margin: 0.5rem;
  }
  
  .top-bar-center {
    display: none;
  }
  
  .upload-btn span:not(.upload-badge) {
    display: none;
  }
}
</style>
