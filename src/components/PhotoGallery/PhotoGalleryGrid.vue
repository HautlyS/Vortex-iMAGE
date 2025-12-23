/**
 * Vue Component - 1 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: PhotoPreview
 */

<script setup lang="ts">
import { computed, toRefs } from 'vue'
import PhotoPreview from '../PhotoPreview.vue'
import { GALLERY } from '../../config'
import type { Photo } from '../../types/photo'

const props = defineProps<{
  photos: Photo[]
  previewSize?: number
  isSelected: (id: string) => boolean
  isFavorite: (id: string) => boolean
  getPhotoColorTag: (photo: Photo) => string | undefined
  getStatus: (id: string) => any
}>()

const emit = defineEmits<{
  select: [photo: Photo, additive: boolean, range: boolean]
  favorite: [photo: Photo]
  'sync-action': [action: string, photoId: string]
  contextmenu: [e: MouseEvent, photo: Photo]
  resize: [size: number]
  'open-lightbox': [photo: Photo]
}>()

const gridStyle = computed(() => ({
  gridTemplateColumns: `repeat(auto-fill, minmax(${props.previewSize || GALLERY.preview.defaultSize}px, 1fr))`,
  gap: `${GALLERY.grid.gap}px`
}))

const { photos } = toRefs(props)

function handlePhotoSelect(photo: Photo, additive: boolean, range: boolean) {
  emit('select', photo, additive, range)
}

function handlePhotoFavorite(photo: Photo) {
  emit('favorite', photo)
}

function handleSyncAction(action: string, photoId: string) {
  emit('sync-action', action, photoId)
}

function handleContextMenu(e: MouseEvent, photo: Photo) {
  emit('contextmenu', e, photo)
}

function handleResize(size: number) {
  emit('resize', size)
}

function handleOpenLightbox(photo: Photo) {
  emit('open-lightbox', photo)
}
</script>

<template>
  <div class="photo-grid" :style="gridStyle" role="grid" :aria-label="`Grid com ${photos.length} fotos`">
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
      role="gridcell"
      :aria-label="`Foto ${photo.name}, ${isFavorite(photo.sha) ? 'favorita' : 'não favorita'}`"
      @select="handlePhotoSelect"
      @favorite="handlePhotoFavorite"
      @sync-action="handleSyncAction"
      @contextmenu="handleContextMenu"
      @resize="handleResize"
      @dblclick="handleOpenLightbox(photo)"
    />
  </div>
</template>

<style scoped>
.photo-grid {
  display: grid;
  gap: 0.75rem;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>