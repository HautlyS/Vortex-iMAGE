<script setup lang="ts">
import { toRefs } from 'vue'
import type { Photo } from '../../types/photo'

const props = defineProps<{
  photos: Photo[]
  isSelected: (id: string) => boolean
  isFavorite: (id: string) => boolean
  getPhotoColorTag: (photo: Photo) => string | undefined
}>()

const emit = defineEmits<{
  select: [photo: Photo, additive: boolean, range: boolean]
  favorite: [photo: Photo]
  contextmenu: [e: MouseEvent, photo: Photo]
  'open-lightbox': [photo: Photo]
  'copy-url': [url: string]
}>()

// Memoize photos for better reactivity
const { photos } = toRefs(props)

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function handlePhotoSelect(photo: Photo, event: MouseEvent) {
  const additive = event.ctrlKey || event.metaKey
  const range = event.shiftKey
  emit('select', photo, additive, range)
}

function handlePhotoFavorite(photo: Photo) {
  emit('favorite', photo)
}

function handleContextMenu(event: MouseEvent, photo: Photo) {
  event.preventDefault()
  emit('contextmenu', event, photo)
}

function handleOpenLightbox(photo: Photo) {
  emit('open-lightbox', photo)
}

function handleCopyUrl(url: string) {
  emit('copy-url', url)
}
</script>

<template>
  <div class="photo-list" role="table" aria-label="Lista de fotos">
    <div class="list-header" role="row">
      <span class="col-thumb" role="columnheader">Miniatura</span>
      <span class="col-name" role="columnheader">Nome</span>
      <span class="col-size" role="columnheader">Tamanho</span>
      <span class="col-actions" role="columnheader">Ações</span>
    </div>
    <div
      v-for="photo in photos"
      :key="photo.sha"
      class="list-item"
      :class="{ selected: isSelected(photo.sha) }"
      role="row"
      :aria-selected="isSelected(photo.sha)"
      :aria-label="`Foto ${photo.name}`"
      @click="handlePhotoSelect(photo, $event)"
      @contextmenu="handleContextMenu($event, photo)"
      @dblclick="handleOpenLightbox(photo)"
    >
      <div class="col-thumb">
        <img :src="photo.url" :alt="photo.name" loading="lazy" />
        <span 
          v-if="getPhotoColorTag(photo)" 
          class="color-dot" 
          :style="{ backgroundColor: getPhotoColorTag(photo) }" 
        />
      </div>
      <span class="col-name">
        <svg 
          v-if="isFavorite(photo.sha)" 
          class="fav-icon" 
          viewBox="0 0 24 24" 
          fill="currentColor" 
          stroke="currentColor" 
          stroke-width="2"
        >
          <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
        </svg>
        {{ photo.name }}
      </span>
      <span class="col-size">{{ formatSize(photo.size) }}</span>
      <div class="col-actions">
        <button 
          @click.stop="handlePhotoFavorite(photo)" 
          :class="{ active: isFavorite(photo.sha) }"
          :aria-label="isFavorite(photo.sha) ? 'Remover dos favoritos' : 'Adicionar aos favoritos'"
        >
          <svg viewBox="0 0 24 24" :fill="isFavorite(photo.sha) ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
          </svg>
        </button>
        <button 
          @click.stop="handleCopyUrl(photo.url)"
          aria-label="Copiar URL da foto"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.photo-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
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

.list-item:hover { 
  background: var(--bg-hover, rgba(255,255,255,0.03)); 
}

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

.col-thumb img { 
  width: 100%; 
  height: 100%; 
  object-fit: cover; 
}

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

.col-name .fav-icon { 
  width: 1rem; 
  height: 1rem; 
  color: var(--error, #ef4444); 
  flex-shrink: 0; 
}

.col-size { 
  font-size: 0.875rem; 
  color: var(--text-muted, #71717a); 
}

.col-actions { 
  display: flex; 
  gap: 0.25rem; 
}

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

.col-actions button.active { 
  color: var(--error, #ef4444); 
}

.col-actions button svg { 
  width: 1rem; 
  height: 1rem; 
}
</style>
