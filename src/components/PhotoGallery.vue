<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import PhotoPreview from './PhotoPreview.vue'
import PixelCard from '../component/PixelCard/PixelCard.vue'
import { useSelection } from '../composables/useSelection'
import { useFavorites } from '../composables/useFavorites'
import { useColorTags } from '../composables/useColorTags'
import { useSyncStatus } from '../composables/useSyncStatus'
import { GALLERY, SHORTCUTS } from '../config'

interface Photo {
  name: string
  url: string
  sha: string
  path?: string
  size?: number
}

const props = defineProps<{
  photos: Photo[]
  loading: boolean
  viewMode: 'grid' | 'list'
  previewSize?: number
}>()

const emit = defineEmits<{ 
  refresh: []
  contextmenu: [e: MouseEvent]
  resize: [size: number]
}>()

const { select, isSelected } = useSelection()
const { isFavorite, toggleFavorite } = useFavorites()
const { getItemTags } = useColorTags()
const { getStatus, uploadPhoto, downloadPhoto, removeLocalCopy, deleteFromRemote } = useSyncStatus()

const selectedPhoto = ref<Photo | null>(null)
const selectedIndex = computed(() => props.photos.findIndex(p => p.sha === selectedPhoto.value?.sha))
const galleryRef = ref<HTMLElement | null>(null)
const imageLoading = ref(false)

// Use config for grid sizing
const gridStyle = computed(() => ({
  gridTemplateColumns: `repeat(auto-fill, minmax(${props.previewSize || GALLERY.preview.defaultSize}px, 1fr))`,
  gap: `${GALLERY.grid.gap}px`
}))

const allPhotoIds = computed(() => props.photos.map(p => p.sha))

function getPhotoColorTag(photo: Photo): string | undefined {
  const tags = getItemTags(photo.sha)
  return tags.length > 0 ? tags[0].color : undefined
}

function handlePhotoSelect(photo: Photo, additive: boolean, range: boolean) {
  select(photo.sha, { additive, range }, allPhotoIds.value)
}

function handlePhotoFavorite(photo: Photo) {
  toggleFavorite({ type: 'photo', id: photo.sha, path: photo.name })
}

function handlePhotoContextMenu(e: MouseEvent, photo: Photo) {
  // Ensure photo is selected
  if (!isSelected(photo.sha)) {
    select(photo.sha, { additive: false, range: false }, allPhotoIds.value)
  }
  emit('contextmenu', e)
}

function handlePhotoSyncAction(action: string, photoId: string) {
  switch (action) {
    case 'upload':
      uploadPhoto(photoId)
      break
    case 'download':
      downloadPhoto(photoId)
      break
    case 'remove-local':
      removeLocalCopy(photoId)
      break
    case 'delete-remote':
      deleteFromRemote(photoId)
      break
  }
}

function handleResize(newSize: number) {
  emit('resize', newSize)
}

function openLightbox(photo: Photo) {
  selectedPhoto.value = photo
  imageLoading.value = true
}

function closeLightbox() {
  selectedPhoto.value = null
  imageLoading.value = false
}

function onImageLoad() {
  // Add a small delay to let the transition finish smoothly if needed, 
  // or just hide immediately. Using slight delay for effect.
  setTimeout(() => {
    imageLoading.value = false
  }, 500)
}

function nextPhoto() {
  if (selectedIndex.value < props.photos.length - 1) {
    selectedPhoto.value = props.photos[selectedIndex.value + 1]
  }
}

function prevPhoto() {
  if (selectedIndex.value > 0) {
    selectedPhoto.value = props.photos[selectedIndex.value - 1]
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!selectedPhoto.value) return
  if (e.key === SHORTCUTS.escape.key) closeLightbox()
  if (e.key === 'ArrowRight') nextPhoto()
  if (e.key === 'ArrowLeft') prevPhoto()
  if (e.key === SHORTCUTS.favorite.key) handlePhotoFavorite(selectedPhoto.value)
}

// Focus management for accessibility - instance-specific
const keydownHandler = ref<((e: KeyboardEvent) => void) | null>(null)

onMounted(() => {
  keydownHandler.value = handleKeydown
  document.addEventListener('keydown', keydownHandler.value)
  galleryRef.value?.focus()
})

onUnmounted(() => {
  if (keydownHandler.value) {
    document.removeEventListener('keydown', keydownHandler.value)
    keydownHandler.value = null
  }
})

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function copyUrl(url: string) {
  navigator.clipboard.writeText(url)
}
</script>

<template>
  <div 
    ref="galleryRef" 
    class="gallery" 
    @keydown="handleKeydown" 
    tabindex="0"
    role="grid"
    :aria-label="`Galeria com ${photos.length} fotos`"
    aria-busy="false"
  >
    <!-- Loading with skeleton -->
    <div v-if="loading" class="loading" role="status" aria-live="polite">
      <div class="spinner" aria-hidden="true"></div>
      <p>Carregando fotos...</p>
      <span class="sr-only">Carregando galeria de fotos</span>
    </div>

    <!-- Empty -->
    <div v-else-if="photos.length === 0" class="empty">
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
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/></svg>
        Atualizar
      </button>
    </div>

    <!-- Grid View -->
    <div v-else-if="viewMode === 'grid'" class="photo-grid" :style="gridStyle" role="list">
      <PhotoPreview
        v-for="(photo, index) in photos"
        :key="photo.sha"
        :photo="photo"
        :size="previewSize || GALLERY.preview.defaultSize"
        :selected="isSelected(photo.sha)"
        :favorited="isFavorite(photo.sha)"
        :color-tag="getPhotoColorTag(photo)"
        :sync-status="getStatus(photo.sha)"
        :tabindex="index === 0 ? 0 : -1"
        role="listitem"
        @select="handlePhotoSelect"
        @favorite="handlePhotoFavorite"
        @sync-action="handlePhotoSyncAction"
        @contextmenu="handlePhotoContextMenu"
        @resize="handleResize"
        @dblclick="openLightbox(photo)"
      />
    </div>

    <!-- List View -->
    <div v-else class="photo-list">
      <div class="list-header">
        <span class="col-thumb">Miniatura</span>
        <span class="col-name">Nome</span>
        <span class="col-size">Tamanho</span>
        <span class="col-actions">Ações</span>
      </div>
      <div
        v-for="photo in photos"
        :key="photo.sha"
        class="list-item"
        :class="{ selected: isSelected(photo.sha) }"
        @click="handlePhotoSelect(photo, $event.ctrlKey || $event.metaKey, $event.shiftKey)"
        @contextmenu.prevent="handlePhotoContextMenu($event, photo)"
        @dblclick="openLightbox(photo)"
      >
        <div class="col-thumb">
          <img :src="photo.url" :alt="photo.name" loading="lazy" />
          <span v-if="getPhotoColorTag(photo)" class="color-dot" :style="{ backgroundColor: getPhotoColorTag(photo) }" />
        </div>
        <span class="col-name">
          <svg v-if="isFavorite(photo.sha)" class="fav-icon" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
          </svg>
          {{ photo.name }}
        </span>
        <span class="col-size">{{ formatSize(photo.size) }}</span>
        <div class="col-actions">
          <button @click.stop="handlePhotoFavorite(photo)" :class="{ active: isFavorite(photo.sha) }">
            <svg viewBox="0 0 24 24" :fill="isFavorite(photo.sha) ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
              <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
            </svg>
          </button>
          <button @click.stop="copyUrl(photo.url)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
              <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Lightbox -->
    <Teleport to="body">
      <Transition name="lightbox">
        <div v-if="selectedPhoto" class="lightbox" @click.self="closeLightbox">
          <button class="lb-close" @click="closeLightbox">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
          </button>

          <button v-if="selectedIndex > 0" class="lb-nav lb-prev" @click="prevPhoto">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg>
          </button>

          <div class="lb-content">
            <PixelCard 
              v-if="imageLoading" 
              variant="pink" 
              :speed="80"
              class-name="fixed !w-full !h-full !inset-0 z-20 pointer-events-none"
              :no-focus="true"
            />
            <img 
              v-show="!imageLoading" 
              :src="selectedPhoto.url" 
              :alt="selectedPhoto.name" 
              @load="onImageLoad"
            />
          </div>

          <button v-if="selectedIndex < photos.length - 1" class="lb-nav lb-next" @click="nextPhoto">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg>
          </button>

          <div class="lb-info">
            <div class="lb-details">
              <span class="lb-name">{{ selectedPhoto.name }}</span>
              <span class="lb-meta">{{ formatSize(selectedPhoto.size) }}</span>
            </div>
            <div class="lb-actions">
              <button @click="handlePhotoFavorite(selectedPhoto)" :class="{ active: isFavorite(selectedPhoto.sha) }">
                <svg viewBox="0 0 24 24" :fill="isFavorite(selectedPhoto.sha) ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
                  <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
                </svg>
              </button>
              <button @click="copyUrl(selectedPhoto.url)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
                  <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
                </svg>
              </button>
            </div>
            <span class="lb-counter">{{ selectedIndex + 1 }} / {{ photos.length }}</span>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.gallery { outline: none; }

/* Screen reader only */
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

/* Loading */
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
@keyframes spin { to { transform: rotate(360deg); } }
.loading p { color: var(--text-muted, #71717a); font-size: 0.875rem; }

/* Empty */
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
.empty-icon svg { width: 100%; height: 100%; }
.empty h3 { font-size: 1.25rem; font-weight: 600; margin-bottom: 0.5rem; }
.empty p { color: var(--text-muted, #71717a); margin-bottom: 1.5rem; max-width: 280px; }
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
.btn-refresh svg { width: 1rem; height: 1rem; }

/* Grid */
.photo-grid {
  display: grid;
  gap: 0.75rem;
  animation: fadeIn 0.3s ease-out;
}

/* List View */
.photo-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  animation: fadeIn 0.3s ease-out;
}
.list-header {
  display: grid;
  grid-template-columns: 60px 1fr 100px 80px;
  gap: 1rem;
  padding: 0.75rem 1rem;
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--text-disabled, #52525b);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.06));
}
.list-item {
  display: grid;
  grid-template-columns: 60px 1fr 100px 80px;
  gap: 1rem;
  padding: 0.5rem 1rem;
  align-items: center;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.15s ease;
}
.list-item:hover { background: var(--bg-hover, rgba(255,255,255,0.03)); }
.list-item.selected { 
  background: var(--accent-light, rgba(99, 102, 241, 0.1)); 
  outline: 1px solid var(--accent-medium, rgba(99, 102, 241, 0.25));
}
.col-thumb {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 0.375rem;
  overflow: hidden;
  background: var(--bg-tertiary, #1a1a1c);
}
.col-thumb img { width: 100%; height: 100%; object-fit: cover; }
.col-thumb .color-dot {
  position: absolute;
  top: 4px;
  left: 4px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  border: 1px solid rgba(255,255,255,0.5);
  box-shadow: 0 1px 3px rgba(0,0,0,0.3);
}
.col-name {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-primary, #fafafa);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.col-name .fav-icon { width: 1rem; height: 1rem; color: var(--error, #ef4444); flex-shrink: 0; }
.col-size { font-size: 0.875rem; color: var(--text-muted, #71717a); }
.col-actions { display: flex; gap: 0.25rem; }
.col-actions button {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-disabled, #52525b);
  cursor: pointer;
  border-radius: 0.375rem;
  transition: all 0.15s ease;
}
.col-actions button:hover { 
  background: var(--bg-active, rgba(255,255,255,0.1)); 
  color: var(--text-primary, #fafafa); 
}
.col-actions button.active { color: var(--error, #ef4444); }
.col-actions button svg { width: 1rem; height: 1rem; }

/* Lightbox */
.lightbox {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(8px);
  z-index: 300;
  display: flex;
  align-items: center;
  justify-content: center;
}
.lb-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255,255,255,0.1);
  border: none;
  border-radius: 50%;
  color: white;
  cursor: pointer;
  z-index: 10;
  transition: all 0.15s ease;
}
.lb-close:hover { background: rgba(255,255,255,0.2); transform: scale(1.05); }
.lb-close svg { width: 1.5rem; height: 1.5rem; }

.lb-nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255,255,255,0.1);
  border: none;
  border-radius: 50%;
  color: white;
  cursor: pointer;
  z-index: 10;
  transition: all 0.15s ease;
}
.lb-nav:hover { background: rgba(255,255,255,0.2); transform: translateY(-50%) scale(1.05); }
.lb-nav svg { width: 1.5rem; height: 1.5rem; }
.lb-prev { left: 1rem; }
.lb-next { right: 1rem; }

.lb-content {
  max-width: 90vw;
  max-height: 85vh;
  animation: scaleIn 0.2s ease-out;
}
@keyframes scaleIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
.lb-content img {
  max-width: 100%;
  max-height: 85vh;
  object-fit: contain;
  border-radius: 0.5rem;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
}

.lb-info {
  position: absolute;
  bottom: 1rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 0.75rem 1.25rem;
  background: rgba(0,0,0,0.8);
  backdrop-filter: blur(12px);
  border-radius: 2rem;
  border: 1px solid rgba(255,255,255,0.1);
}
.lb-details { display: flex; flex-direction: column; gap: 0.125rem; }
.lb-name { font-size: 0.875rem; color: white; font-weight: 500; }
.lb-meta { font-size: 0.75rem; color: var(--text-muted, #71717a); }
.lb-actions { display: flex; gap: 0.5rem; }
.lb-actions button {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255,255,255,0.1);
  border: none;
  border-radius: 50%;
  color: white;
  cursor: pointer;
  transition: all 0.15s ease;
}
.lb-actions button:hover { background: rgba(255,255,255,0.2); }
.lb-actions button.active { color: var(--error, #ef4444); }
.lb-actions button svg { width: 1rem; height: 1rem; }
.lb-counter { font-size: 0.75rem; color: var(--text-muted, #71717a); }

/* Transitions */
.lightbox-enter-active, .lightbox-leave-active { transition: opacity 0.2s ease; }
.lightbox-enter-from, .lightbox-leave-to { opacity: 0; }
</style>
