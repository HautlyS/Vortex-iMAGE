/**
 * Vue Component - 4 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: Masonry, CircularGallery, FloatingTagPanel...
 */

<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Masonry from './Masonry.vue'
import CircularGallery from './CircularGallery.vue'
import type { Photo } from '../types/photo'
import { useGitHubAuth } from '../composables/useGitHubAuth'
import { useFavorites } from '../composables/useFavorites'

const { token, repo } = useGitHubAuth()
const { isFavorite, toggleFavorite } = useFavorites()

interface Album {
  name: string
  path: string
  photo_count: number
  children: Album[]
  coverUrl?: string
}

const props = defineProps<{
  photos: Photo[]
  loading: boolean
  albums?: Album[]
  showAlbums?: boolean
  viewMode?: 'grid' | 'list'
  currentAlbumPath?: string | null
}>()

const emit = defineEmits<{
  photoClick: [photo: Photo]
  albumClick: [album: Album]
}>()

const mockPhotos: Photo[] = Array.from({ length: 12 }, (_, i) => ({
  sha: `mock-${i}`,
  name: `Mock Photo ${i + 1}`,
  url: `https://picsum.photos/seed/${i}/400/400`
}))

const mockAlbums: Album[] = [
  { name: 'Viagens', path: 'viagens', photo_count: 24, children: [
    { name: '2024', path: 'viagens/2024', photo_count: 12, children: [], coverUrl: 'https://picsum.photos/seed/v2024/400/400' },
    { name: '2023', path: 'viagens/2023', photo_count: 12, children: [], coverUrl: 'https://picsum.photos/seed/v2023/400/400' },
  ], coverUrl: 'https://picsum.photos/seed/viagens/400/400' },
  { name: 'Família', path: 'familia', photo_count: 56, children: [], coverUrl: 'https://picsum.photos/seed/familia/400/400' },
]

const displayPhotos = computed(() => {
  if (import.meta.env.DEV && props.photos.length === 0) {
    return mockPhotos
  }
  return props.photos
})

function findSubalbums(albums: Album[], parentPath: string | null): Album[] {
  if (!parentPath) return albums
  
  const findInTree = (items: Album[], path: string): Album[] => {
    for (const album of items) {
      if (album.path === path) return album.children
      if (path.startsWith(album.path + '/')) {
        const found = findInTree(album.children, path)
        if (found.length) return found
      }
    }
    return []
  }
  
  return findInTree(albums, parentPath)
}

function getParentPath(path: string): string | null {
  const parts = path.split('/')
  return parts.length > 1 ? parts.slice(0, -1).join('/') : null
}

function getBreadcrumbs(path: string) {
    if (!path) return []
    const parts = path.split('/')
    return parts.map((part, index) => ({
        name: part.charAt(0).toUpperCase() + part.slice(1),
        path: parts.slice(0, index + 1).join('/')
    }))
}

const displayAlbums = computed(() => {
  const allAlbums = import.meta.env.DEV && (!props.albums || props.albums.length === 0) 
    ? mockAlbums 
    : (props.albums || [])
  
  if (!props.showAlbums) return []

  if (props.currentAlbumPath) {
    return findSubalbums(allAlbums, props.currentAlbumPath)
  }
  
  return allAlbums
})

function getAlbumCover(album: Album): string {
  if (album.coverUrl) return album.coverUrl
  const photo = props.photos.find(p => p.path?.startsWith(album.path))
  return photo?.url || `https://picsum.photos/seed/${album.path}/400/400`
}

const albumItems = computed(() =>
  displayAlbums.value.map(album => ({
    id: `folder-${album.path}`,
    img: getAlbumCover(album),
    url: '',
    height: 350,
    isFolder: true,
    folderName: album.name,
    photoCount: album.photo_count,
    album
  }))
)

const photoItems = computed(() => 
  displayPhotos.value.map(photo => {
    const hash = photo.sha.split('').reduce((a, c) => a + c.charCodeAt(0), 0)
    return {
      id: photo.sha,
      img: photo.url,
      url: photo.url,
      height: 300 + (hash % 200),
      isFolder: false,
      photo
    }
  })
)

const masonryItems = computed(() => {
  const allItems = [...albumItems.value, ...photoItems.value]
  return allItems.map(item => {
    const tag = getTagForItem(item.id)
    return {
      ...item,
      tagColor: tag?.color
    }
  })
})

function handleItemClick(item: any) {
  if (item.isFolder && item.album) {
    emit('albumClick', item.album)
  } else if (item.photo) {
    
    const index = displayPhotos.value.findIndex(p => p.sha === item.id)
    if (index !== -1) {
      lightboxStartIndex.value = index
      currentLightboxIndex.value = index
      isLightboxOpen.value = true
    }
  }
}

const isLightboxOpen = ref(false);
const lightboxStartIndex = ref(0);
const currentLightboxIndex = ref(0);
const isSaving = ref(false);
const isSyncing = ref(false);
const saveSuccess = ref(false);
const syncSuccess = ref(false);

const currentLightboxPhoto = computed(() => displayPhotos.value[currentLightboxIndex.value])

function handleLightboxIndexChange(index: number) {
  currentLightboxIndex.value = index
}

const circularItems = computed(() => 
    displayPhotos.value.map(p => ({
        image: p.url,
        text: p.name
    }))
);

async function saveToGallery() {
  if (!currentLightboxPhoto.value || !token.value || !repo.value) return
  
  isSaving.value = true
  saveSuccess.value = false
  
  try {
    await invoke('download_photo', {
      token: token.value,
      repo: repo.value,
      path: currentLightboxPhoto.value.name,
      savePath: null 
    })
    saveSuccess.value = true
    
    if ('vibrate' in navigator) navigator.vibrate(10)
    setTimeout(() => { saveSuccess.value = false }, 2000)
  } catch (e) {
    console.error('Save failed:', e)
  } finally {
    isSaving.value = false
  }
}

async function syncToGit() {
  if (!currentLightboxPhoto.value || !token.value || !repo.value) return
  
  isSyncing.value = true
  syncSuccess.value = false
  
  try {

    await invoke('download_photo', {
      token: token.value,
      repo: repo.value,
      path: currentLightboxPhoto.value.name,
      savePath: null
    })
    syncSuccess.value = true
    if ('vibrate' in navigator) navigator.vibrate([10, 50, 10])
    setTimeout(() => { syncSuccess.value = false }, 2000)
  } catch (e) {
    console.error('Sync failed:', e)
  } finally {
    isSyncing.value = false
  }
}

function toggleCurrentFavorite() {
  if (!currentLightboxPhoto.value) return
  toggleFavorite({ 
    type: 'photo', 
    id: currentLightboxPhoto.value.sha, 
    path: currentLightboxPhoto.value.name 
  })
  if ('vibrate' in navigator) navigator.vibrate(10)
}

function handleItemDblClick(item: any) {
    
    if (item.isFolder) return
}

function handlePhotoListClick(photo: Photo) {
  
  const index = displayPhotos.value.findIndex(p => p.sha === photo.sha);
  if (index !== -1) {
    lightboxStartIndex.value = index;
    currentLightboxIndex.value = index;
    isLightboxOpen.value = true;
  }
}

function formatDate(sha: string): string {
  
  const hash = sha.split('').reduce((a, c) => a + c.charCodeAt(0), 0)
  const days = hash % 30 + 1
  const months = ['Jan', 'Fev', 'Mar', 'Abr', 'Mai', 'Jun', 'Jul', 'Ago', 'Set', 'Out', 'Nov', 'Dez']
  const month = months[hash % 12]
  return `${days} ${month}`
}

import { useTags } from '../composables/useTags'
import FloatingTagPanel from './FloatingTagPanel.vue'
import ContextColorMenu from './ContextColorMenu.vue'

const { tags, getTagForItem, addTag, assignTag, updateTagName } = useTags()

const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  activeColor: undefined as string | undefined,
  targetItemId: null as string | null
})

function handleContextMenu(item: any, event: MouseEvent) {
  const currentTag = getTagForItem(item.id)
  
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    activeColor: currentTag?.color,
    targetItemId: item.id
  }

  const closeListener = () => {
    contextMenu.value.visible = false
    window.removeEventListener('click', closeListener)
  }
  setTimeout(() => window.addEventListener('click', closeListener), 0)
}

function handleColorSelect(color: string) {
  if (!contextMenu.value.targetItemId) return

  let tag = tags.value.find(t => t.color === color)
  if (!tag) {
    tag = addTag(color)
  }
  
  assignTag(contextMenu.value.targetItemId, tag.id)
}

function handleTagUpdate(id: string, name: string) {
  updateTagName(id, name)
}

function closeContextMenu() {
  contextMenu.value.visible = false
}

</script>

<template>
  <div class="gallery-container">
    <div v-if="loading" class="loading-state">
      <div class="spinner" />
      <span>Carregando fotos...</span>
    </div>

    <div v-else-if="masonryItems.length === 0" class="empty-state">
      <div v-if="currentAlbumPath" class="mb-4">

        <button class="btn btn-secondary" @click="emit('albumClick', { path: getParentPath(currentAlbumPath) || '', name: 'Back', photo_count: 0, children: [] })">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m15 18-6-6 6-6"/>
          </svg>
          Voltar
        </button>
      </div>
      <span>Nenhuma foto encontrada</span>
    </div>

    <!-- Grid View (Masonry) -->
    <div v-else-if="viewMode === 'grid'" class="masonry-wrapper">
       <!-- Breadcrumbs -->
       <div class="absolute top-4 left-4 z-10 flex items-center gap-2">
         <button 
            class="btn btn-secondary glass-btn !px-3"
            @click="emit('albumClick', { name: 'Home', path: '', photo_count: 0, children: [] })"
            :class="{ '!bg-white/10': !currentAlbumPath }"
         >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
              <polyline points="9 22 9 12 15 12 15 22"/>
            </svg>
         </button>
         
         <template v-if="currentAlbumPath">
            <span class="text-white/40 font-bold">/</span>
            <div class="flex items-center gap-2">
                <template v-for="(part, index) in getBreadcrumbs(currentAlbumPath)" :key="part.path">
                    <button 
                        class="btn btn-secondary glass-btn !px-3"
                        @click="emit('albumClick', { name: part.name, path: part.path, photo_count: 0, children: [] })"
                    >
                        {{ part.name }}
                    </button>
                    <span v-if="index < getBreadcrumbs(currentAlbumPath).length - 1" class="text-white/40 font-bold">/</span>
                </template>
            </div>
         </template>
      </div>
      <Masonry 
        :items="masonryItems"
        :scale-on-hover="true"
        :hover-scale="0.97"
        :blur-to-focus="true"
        animate-from="bottom"
        @item-click="handleItemClick"
        @item-dbl-click="handleItemDblClick"
        @context-menu="handleContextMenu"
      />
    </div>

    <!-- List View -->
    <div v-else class="list-wrapper">
      <div class="list-container">
        <!-- Column 1 -->
        <div class="list-column">
          <div class="column-header">Álbuns</div>
          <div class="column-items">
            <div 
              v-for="album in displayAlbums" 
              :key="`folder-${album.path}`"
              class="list-item folder-item"
              @click="emit('albumClick', album)"
            >
              <div class="list-item-icon">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z"/>
                </svg>
              </div>
              <div class="list-item-info">
                <div class="list-item-name">{{ album.name }}</div>
                <div class="list-item-meta">{{ album.photo_count }} fotos</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Column 2 -->
        <div class="list-column">
          <div class="column-header">Recentes</div>
          <div class="column-items">
            <div 
              v-for="photo in displayPhotos.slice(0, 8)" 
              :key="`recent-${photo.sha}`"
              class="list-item photo-item"
              @click="handlePhotoListClick(photo)"
            >
              <div class="list-item-icon">
                <img :src="photo.url" :alt="photo.name" class="photo-thumbnail" />
              </div>
              <div class="list-item-info">
                <div class="list-item-name">{{ photo.name }}</div>
                <div class="list-item-meta">{{ formatDate(photo.sha) }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Column 3 -->
        <div class="list-column">
          <div class="column-header">Todas as Fotos</div>
          <div class="column-items">
            <div 
              v-for="photo in displayPhotos" 
              :key="`all-${photo.sha}`"
              class="list-item photo-item"
              @click="handlePhotoListClick(photo)"
            >
              <div class="list-item-icon">
                <img :src="photo.url" :alt="photo.name" class="photo-thumbnail" />
              </div>
              <div class="list-item-info">
                <div class="list-item-name">{{ photo.name }}</div>
                <div class="list-item-meta">Imagem</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Circular Gallery Overlay -->
    <Teleport to="body">
        <Transition
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="opacity-0 scale-95"
            enter-to-class="opacity-100 scale-100"
            leave-active-class="transition-all duration-200 ease-in"
            leave-from-class="opacity-100 scale-100"
            leave-to-class="opacity-0 scale-95"
        >
            <div 
                v-if="isLightboxOpen" 
                class="lightbox-overlay"
                @click.self="isLightboxOpen = false"
            >
                <!-- Close Button - iOS safe area aware -->
                <button 
                  class="lightbox-close"
                  @click="isLightboxOpen = false"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </button>

                <!-- Photo counter -->
                <div class="lightbox-counter">
                  {{ currentLightboxIndex + 1 }} / {{ circularItems.length }}
                </div>

                <!-- Lightbox Tools -->
                <div class="lightbox-tools">
                  <!-- Save to Gallery -->
                  <button 
                    class="lightbox-tool"
                    :class="{ success: saveSuccess }"
                    :disabled="isSaving"
                    @click="saveToGallery"
                    title="Salvar no dispositivo"
                  >
                    <svg v-if="isSaving" class="spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="12" r="10" opacity="0.25"/>
                      <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
                    </svg>
                    <svg v-else-if="saveSuccess" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M20 6L9 17l-5-5"/>
                    </svg>
                    <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                      <polyline points="7,10 12,15 17,10"/>
                      <line x1="12" y1="15" x2="12" y2="3"/>
                    </svg>
                    <span>Salvar</span>
                  </button>

                  <!-- Favorite -->
                  <button 
                    class="lightbox-tool"
                    :class="{ active: currentLightboxPhoto && isFavorite(currentLightboxPhoto.sha) }"
                    @click="toggleCurrentFavorite"
                    title="Favoritar"
                  >
                    <svg viewBox="0 0 24 24" :fill="currentLightboxPhoto && isFavorite(currentLightboxPhoto.sha) ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
                      <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
                    </svg>
                    <span>Favorito</span>
                  </button>

                  <!-- Sync to Git -->
                  <button 
                    class="lightbox-tool"
                    :class="{ success: syncSuccess }"
                    :disabled="isSyncing"
                    @click="syncToGit"
                    title="Sincronizar com GitHub"
                  >
                    <svg v-if="isSyncing" class="spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="12" r="10" opacity="0.25"/>
                      <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
                    </svg>
                    <svg v-else-if="syncSuccess" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M20 6L9 17l-5-5"/>
                    </svg>
                    <svg v-else viewBox="0 0 24 24" fill="currentColor">
                      <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                    </svg>
                    <span>Sync</span>
                  </button>
                </div>

                <div class="w-full h-full">
                    <CircularGallery
                        :items="circularItems"
                        :bend="1.5"
                        text-color="#ffffff"
                        :border-radius="0.05"
                        font="bold 48px -apple-system, BlinkMacSystemFont, sans-serif"
                        :scroll-speed="2"
                        :scroll-ease="0.1"
                        :initial-index="lightboxStartIndex"
                        @index-change="handleLightboxIndexChange"
                    />
                </div>
            </div>
        </Transition>
    </Teleport>

    <!-- Tag System Components -->
    <FloatingTagPanel 
      :tags="tags" 
      @update-tag="handleTagUpdate"
    />
    
    <Teleport to="body">
      <ContextColorMenu 
        :visible="contextMenu.visible"
        :x="contextMenu.x"
        :y="contextMenu.y"
        :active-color="contextMenu.activeColor"
        @select="handleColorSelect"
        @close="closeContextMenu"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.gallery-container {
  width: 100%;
  height: 100%;
  position: relative;
  background: transparent;
}

.lightbox-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: #000;
  padding-top: env(safe-area-inset-top, 0);
  padding-bottom: env(safe-area-inset-bottom, 0);
}

.lightbox-close {
  position: absolute;
  top: calc(16px + env(safe-area-inset-top, 0));
  right: 16px;
  z-index: 50;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: none;
  border-radius: 50%;
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: all 0.2s;
  -webkit-tap-highlight-color: transparent;
}

.lightbox-close:active {
  transform: scale(0.9);
  background: rgba(255, 255, 255, 0.2);
}

.lightbox-counter {
  position: absolute;
  top: calc(20px + env(safe-area-inset-top, 0));
  left: 50%;
  transform: translateX(-50%);
  z-index: 50;
  padding: 8px 16px;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 20px;
  color: rgba(255, 255, 255, 0.8);
  font-size: 14px;
  font-weight: 500;
  font-family: -apple-system, BlinkMacSystemFont, sans-serif;
}

.lightbox-tools {
  position: absolute;
  bottom: calc(24px + env(safe-area-inset-bottom, 0));
  left: 50%;
  transform: translateX(-50%);
  z-index: 50;
  display: flex;
  gap: 8px;
  padding: 8px;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.lightbox-tool {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 16px;
  background: transparent;
  border: none;
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.8);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  -webkit-tap-highlight-color: transparent;
  min-width: 64px;
}

.lightbox-tool svg {
  width: 24px;
  height: 24px;
}

.lightbox-tool:active {
  transform: scale(0.95);
  background: rgba(255, 255, 255, 0.1);
}

.lightbox-tool:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.lightbox-tool.active {
  color: #ff3b5c;
}

.lightbox-tool.success {
  color: #34c759;
}

.lightbox-tool .spin {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  color: var(--text-muted);
  gap: 1rem;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-default);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.masonry-wrapper {
  width: 100%;
  height: calc(100vh - 200px);
  position: relative;
  background: transparent;
}

.list-wrapper {
  width: 100%;
  height: calc(100vh - 200px);
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.4);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.list-container {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  height: 100%;
  gap: 1px;
  background: var(--border-default);
}

.list-column {
  background: rgba(var(--surface-0-rgb, 8, 8, 12), 0.6);
  display: flex;
  flex-direction: column;
}

.column-header {
  padding: 1rem 1.25rem;
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.8);
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-default);
}

.column-items {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem 0;
}

.list-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1.25rem;
  cursor: pointer;
  transition: all var(--duration-fast);
}

.list-item:hover {
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.6);
}

.list-item-icon {
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.folder-item .list-item-icon {
  color: var(--accent-color);
}

.folder-item .list-item-icon svg {
  width: 1.75rem;
  height: 1.75rem;
}

.photo-thumbnail {
  width: 2.5rem;
  height: 2.5rem;
  object-fit: cover;
  border-radius: var(--radius-md);
}

.list-item-info {
  flex: 1;
  min-width: 0;
}

.list-item-name {
  font-size: 0.9375rem;
  color: var(--text-primary);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.list-item-meta {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  margin-top: 0.125rem;
}

@media (max-width: 768px) {
  .list-column:nth-child(3) {
    display: none;
  }
}

.glass-btn {
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.6);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: white;
  padding: 8px 16px;
  border-radius: 20px;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
}

.glass-btn:hover {
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.8);
  transform: translateY(-1px);
}
</style>