<script setup lang="ts">
import { computed, ref } from 'vue'
import Masonry from './Masonry.vue'
import CircularGallery from './CircularGallery.vue'
import type { Photo } from '../types/photo'

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

// Mock photos for dev mode
const mockPhotos: Photo[] = Array.from({ length: 12 }, (_, i) => ({
  sha: `mock-${i}`,
  name: `Mock Photo ${i + 1}`,
  url: `https://picsum.photos/seed/gallery${i}/400/${300 + (i % 5) * 50}`,
}))

const mockAlbums: Album[] = [
  { name: 'Viagens', path: 'viagens', photo_count: 24, children: [
    { name: '2024', path: 'viagens/2024', photo_count: 12, children: [], coverUrl: 'https://picsum.photos/seed/sub1/400/350' },
    { name: '2023', path: 'viagens/2023', photo_count: 12, children: [], coverUrl: 'https://picsum.photos/seed/sub2/400/350' },
  ], coverUrl: 'https://picsum.photos/seed/album1/400/350' },
  { name: 'Família', path: 'familia', photo_count: 56, children: [], coverUrl: 'https://picsum.photos/seed/album2/400/400' },
]

const displayPhotos = computed(() => {
  if (import.meta.env.DEV && props.photos.length === 0) {
    return mockPhotos
  }
  return props.photos
})

// Find subalbums for current path
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
  
  // If inside an album, show subalbums
  if (props.currentAlbumPath) {
    return findSubalbums(allAlbums, props.currentAlbumPath)
  }
  
  return allAlbums
})

// Get cover image for album (first photo in that path)
function getAlbumCover(album: Album): string {
  if (album.coverUrl) return album.coverUrl
  const photo = props.photos.find(p => p.path?.startsWith(album.path))
  return photo?.url || `https://picsum.photos/seed/${album.path}/400/350`
}

// Convert albums to Masonry items
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

// Convert photos to Masonry items
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

// Combined items: folders first, then photos
// Also inject tag colors
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
    emit('photoClick', item.photo)
  }
}

// Lightbox (CircularGallery) Logic
const isLightboxOpen = ref(false);
const lightboxStartIndex = ref(0);

// Format photos for CircularGallery (only photos, not folders)
const circularItems = computed(() => 
    displayPhotos.value.map(p => ({
        image: p.url,
        text: p.name
    }))
);

function handleItemDblClick(item: any) {
    if (item.isFolder) return // Don't open lightbox for folders
    
    const index = displayPhotos.value.findIndex(p => p.sha === item.id);
    if (index !== -1) {
        lightboxStartIndex.value = index;
        isLightboxOpen.value = true;
    }
}

function handlePhotoListDblClick(photo: Photo) {
  const index = displayPhotos.value.findIndex(p => p.sha === photo.sha);
  if (index !== -1) {
    lightboxStartIndex.value = index;
    isLightboxOpen.value = true;
  }
}

function formatDate(sha: string): string {
  // Simple date formatting based on sha hash
  const hash = sha.split('').reduce((a, c) => a + c.charCodeAt(0), 0)
  const days = hash % 30 + 1
  const months = ['Jan', 'Fev', 'Mar', 'Abr', 'Mai', 'Jun', 'Jul', 'Ago', 'Set', 'Out', 'Nov', 'Dez']
  const month = months[hash % 12]
  return `${days} ${month}`
}

// --- Tag System Logic ---
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
  
  // Close menu on next click
  const closeListener = () => {
    contextMenu.value.visible = false
    window.removeEventListener('click', closeListener)
  }
  setTimeout(() => window.addEventListener('click', closeListener), 0)
}

function handleColorSelect(color: string) {
  if (!contextMenu.value.targetItemId) return
  
  // Find or create tag for this color
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
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
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
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
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
              @click="emit('photoClick', photo)"
              @dblclick="handlePhotoListDblClick(photo)"
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
              @click="emit('photoClick', photo)"
              @dblclick="handlePhotoListDblClick(photo)"
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
            enter-active-class="transition-opacity duration-300"
            enter-from-class="opacity-0"
            enter-to-class="opacity-100"
            leave-active-class="transition-opacity duration-300"
            leave-from-class="opacity-100"
            leave-to-class="opacity-0"
        >
            <div 
                v-if="isLightboxOpen" 
                class="fixed inset-0 z-[9999] bg-black"
            >
                <!-- Close Button -->
                <button 
                  class="absolute top-6 right-6 z-50 p-2 text-white/70 hover:text-white transition-colors cursor-pointer"
                  @click="isLightboxOpen = false"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </button>

                <div class="w-full h-full">
                    <CircularGallery
                        :items="circularItems"
                        :bend="1.5"
                        text-color="#ffffff"
                        :border-radius="0.05"
                        font="bold 60px Figtree"
                        :scroll-speed="2"
                        :scroll-ease="0.1"
                        :initial-index="lightboxStartIndex"
                    />
                </div>
            </div>
        </Transition>
    </Teleport>
  </div>
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
/* ... existing styles ... */

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
