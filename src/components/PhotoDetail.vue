/**
 * Vue Component - 1 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: SecureImage
 */

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import SecureImage from './SecureImage.vue'
import { useImageMetadata, type ImageMetadata } from '../composables/useImageMetadata'
import { useFavorites } from '../composables/useFavorites'
import { useColorTags, PREDEFINED_COLORS } from '../composables/useColorTags'

interface Photo {
  sha: string
  name: string
  url: string
  path?: string
  size?: number
}

const props = defineProps<{
  photo: Photo
  photos: Photo[]
}>()

const emit = defineEmits<{
  close: []
  navigate: [direction: 'prev' | 'next']
}>()

const { extractMetadata, formatFileSize, formatDate, formatCoordinates, loading } = useImageMetadata()
const { isFavorite, toggleFavorite } = useFavorites()
const { getPhotoTag, tagItems } = useColorTags()

const metadata = ref<ImageMetadata | null>(null)
const showMetadata = ref(false)
const showColorPicker = ref(false)

const currentIndex = computed(() => props.photos.findIndex(p => p.sha === props.photo.sha))
const hasPrev = computed(() => currentIndex.value > 0)
const hasNext = computed(() => currentIndex.value < props.photos.length - 1)
const photoTags = computed(() => {
  const tag = getPhotoTag(props.photo.sha)
  return tag ? [tag] : []
})

watch(() => props.photo, async (newPhoto) => {
  if (newPhoto?.url) {
    metadata.value = await extractMetadata(newPhoto.url)
  }
}, { immediate: true })

function handleFavorite() {
  toggleFavorite({ type: 'photo', id: props.photo.sha, path: props.photo.name })
}

function handleTagSelect(colorId: string) {
  tagItems([props.photo.sha], colorId)
  showColorPicker.value = false
}

function copyUrl() {
  navigator.clipboard.writeText(props.photo.url)
}

function handleSwipe(direction: 'left' | 'right') {
  if (direction === 'left' && hasNext.value) {
    emit('navigate', 'next')
  } else if (direction === 'right' && hasPrev.value) {
    emit('navigate', 'prev')
  }
}

let touchStartX = 0
let touchStartY = 0

function onTouchStart(e: TouchEvent) {
  touchStartX = e.touches[0].clientX
  touchStartY = e.touches[0].clientY
}

function onTouchEnd(e: TouchEvent) {
  const touchEndX = e.changedTouches[0].clientX
  const touchEndY = e.changedTouches[0].clientY
  const deltaX = touchEndX - touchStartX
  const deltaY = touchEndY - touchStartY

  if (Math.abs(deltaX) > Math.abs(deltaY) && Math.abs(deltaX) > 50) {
    handleSwipe(deltaX > 0 ? 'right' : 'left')
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
  if (e.key === 'ArrowLeft' && hasPrev.value) emit('navigate', 'prev')
  if (e.key === 'ArrowRight' && hasNext.value) emit('navigate', 'next')
  if (e.key === 'f') handleFavorite()
  if (e.key === 'i') showMetadata.value = !showMetadata.value
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div 
    class="photo-detail" 
    @click.self="emit('close')"
    @touchstart="onTouchStart"
    @touchend="onTouchEnd"
  >
    <!-- Close Button -->
    <button class="close-btn" @click="emit('close')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>

    <!-- Navigation Buttons (Desktop) -->
    <button v-if="hasPrev" class="nav-btn prev" @click="emit('navigate', 'prev')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m15 18-6-6 6-6"/>
      </svg>
    </button>
    <button v-if="hasNext" class="nav-btn next" @click="emit('navigate', 'next')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m9 18 6-6-6-6"/>
      </svg>
    </button>

    <!-- Image -->
    <div class="image-container">
      <SecureImage :src="photo.url" :alt="photo.name" />
    </div>

    <!-- Bottom Info Bar -->
    <div class="info-bar" :class="{ expanded: showMetadata }">
      <!-- Drag Handle -->
      <div class="drag-handle" @click="showMetadata = !showMetadata">
        <span class="handle-bar" />
      </div>

      <!-- Quick Actions -->
      <div class="quick-actions">
        <button 
          class="action-btn" 
          :class="{ active: isFavorite(photo.sha) }"
          @click="handleFavorite"
        >
          <svg viewBox="0 0 24 24" :fill="isFavorite(photo.sha) ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
          </svg>
        </button>

        <div class="color-picker-wrapper">
          <button 
            class="action-btn color-btn"
            :style="photoTags.length ? { backgroundColor: photoTags[0].color } : {}"
            @click="showColorPicker = !showColorPicker"
          >
            <svg v-if="!photoTags.length" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
            </svg>
          </button>
          
          <Transition name="fade">
            <div v-if="showColorPicker" class="color-dropdown">
              <button 
                v-for="color in PREDEFINED_COLORS" 
                :key="color.id"
                class="color-option"
                :style="{ backgroundColor: color.color }"
                @click="handleTagSelect(color.id)"
              />
            </div>
          </Transition>
        </div>

        <button class="action-btn" @click="copyUrl">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
          </svg>
        </button>

        <button class="action-btn" @click="showMetadata = !showMetadata">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 16v-4M12 8h.01"/>
          </svg>
        </button>
      </div>

      <!-- Basic Info -->
      <div class="basic-info">
        <span class="photo-name">{{ photo.name }}</span>
        <span class="photo-meta">
          {{ formatFileSize(photo.size || metadata?.fileSize || 0) }}
          <template v-if="metadata?.dimensions">
            • {{ metadata.dimensions.width }} × {{ metadata.dimensions.height }}
          </template>
        </span>
        <span class="photo-counter">{{ currentIndex + 1 }} / {{ photos.length }}</span>
      </div>

      <!-- Expanded Metadata -->
      <Transition name="slide">
        <div v-if="showMetadata && metadata" class="metadata-panel">
          <!-- Camera Info -->
          <div v-if="metadata.camera?.make || metadata.camera?.model" class="metadata-section">
            <h4 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
                <circle cx="12" cy="13" r="4"/>
              </svg>
              Câmera
            </h4>
            <div class="metadata-grid">
              <div v-if="metadata.camera.make" class="metadata-item">
                <span class="label">Fabricante</span>
                <span class="value">{{ metadata.camera.make }}</span>
              </div>
              <div v-if="metadata.camera.model" class="metadata-item">
                <span class="label">Modelo</span>
                <span class="value">{{ metadata.camera.model }}</span>
              </div>
              <div v-if="metadata.camera.lens" class="metadata-item full">
                <span class="label">Lente</span>
                <span class="value">{{ metadata.camera.lens }}</span>
              </div>
            </div>
          </div>

          <!-- Settings -->
          <div v-if="metadata.settings?.aperture || metadata.settings?.shutterSpeed" class="metadata-section">
            <h4 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
              </svg>
              Configurações
            </h4>
            <div class="settings-grid">
              <div v-if="metadata.settings.aperture" class="setting-chip">
                <span class="setting-icon">ƒ</span>
                <span>{{ metadata.settings.aperture }}</span>
              </div>
              <div v-if="metadata.settings.shutterSpeed" class="setting-chip">
                <span class="setting-icon">⏱</span>
                <span>{{ metadata.settings.shutterSpeed }}</span>
              </div>
              <div v-if="metadata.settings.iso" class="setting-chip">
                <span class="setting-icon">ISO</span>
                <span>{{ metadata.settings.iso }}</span>
              </div>
              <div v-if="metadata.settings.focalLength" class="setting-chip">
                <span class="setting-icon">🔭</span>
                <span>{{ metadata.settings.focalLength }}</span>
              </div>
              <div v-if="metadata.settings.flash" class="setting-chip">
                <span class="setting-icon">⚡</span>
                <span>{{ metadata.settings.flash }}</span>
              </div>
              <div v-if="metadata.settings.whiteBalance" class="setting-chip">
                <span class="setting-icon">☀️</span>
                <span>WB: {{ metadata.settings.whiteBalance }}</span>
              </div>
            </div>
          </div>

          <!-- Date & Time -->
          <div v-if="metadata.dateTime?.original" class="metadata-section">
            <h4 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                <line x1="16" y1="2" x2="16" y2="6"/>
                <line x1="8" y1="2" x2="8" y2="6"/>
                <line x1="3" y1="10" x2="21" y2="10"/>
              </svg>
              Data
            </h4>
            <div class="metadata-grid">
              <div class="metadata-item full">
                <span class="label">Capturada</span>
                <span class="value">{{ formatDate(metadata.dateTime.original) }}</span>
              </div>
            </div>
          </div>

          <!-- Location -->
          <div v-if="metadata.location?.latitude" class="metadata-section">
            <h4 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/>
                <circle cx="12" cy="10" r="3"/>
              </svg>
              Localização
            </h4>
            <div class="metadata-grid">
              <div class="metadata-item full">
                <span class="label">Coordenadas</span>
                <span class="value">{{ formatCoordinates(metadata.location.latitude, metadata.location.longitude) }}</span>
              </div>
              <div v-if="metadata.location.altitude" class="metadata-item">
                <span class="label">Altitude</span>
                <span class="value">{{ metadata.location.altitude.toFixed(1) }}m</span>
              </div>
            </div>
            <a 
              :href="`https:
              target="_blank"
              class="map-link"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                <polyline points="15,3 21,3 21,9"/>
                <line x1="10" y1="14" x2="21" y2="3"/>
              </svg>
              Ver no Google Maps
            </a>
          </div>

          <!-- File Info -->
          <div class="metadata-section">
            <h4 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14,2 14,8 20,8"/>
              </svg>
              Arquivo
            </h4>
            <div class="metadata-grid">
              <div class="metadata-item">
                <span class="label">Tipo</span>
                <span class="value">{{ metadata.fileType }}</span>
              </div>
              <div class="metadata-item">
                <span class="label">Tamanho</span>
                <span class="value">{{ formatFileSize(metadata.fileSize) }}</span>
              </div>
              <div v-if="metadata.dimensions" class="metadata-item">
                <span class="label">Dimensões</span>
                <span class="value">{{ metadata.dimensions.width }} × {{ metadata.dimensions.height }}</span>
              </div>
              <div v-if="metadata.colorSpace" class="metadata-item">
                <span class="label">Espaço de Cor</span>
                <span class="value">{{ metadata.colorSpace }}</span>
              </div>
            </div>
          </div>

          <!-- Loading State -->
          <div v-if="loading" class="metadata-loading">
            <div class="spinner" />
            <span>Carregando metadados...</span>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.photo-detail {
  position: fixed;
  inset: 0;
  background: var(--amoled-black, #000);
  z-index: 300;
  display: flex;
  flex-direction: column;
}

.close-btn {
  position: absolute;
  top: calc(var(--mobile-gap-md, 12px) + env(safe-area-inset-top, 0px));
  right: var(--mobile-gap-md, 12px);
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(12px);
  border: none;
  border-radius: 50%;
  color: white;
  cursor: pointer;
  z-index: 10;
  transition: all 0.2s ease;
}

.close-btn:active {
  transform: scale(0.92);
  background: rgba(255, 255, 255, 0.2);
}

.close-btn svg {
  width: 1.25rem;
  height: 1.25rem;
}

.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(12px);
  border: none;
  border-radius: 50%;
  color: white;
  cursor: pointer;
  z-index: 10;
  transition: all 0.2s ease;
}

.nav-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.nav-btn svg {
  width: 1.5rem;
  height: 1.5rem;
}

.nav-btn.prev { left: 1rem; }
.nav-btn.next { right: 1rem; }

@media (max-width: 768px) {
  .nav-btn { display: none; }
}

.image-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 1rem;
}

.image-container img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: var(--radius-md, 8px);
}

.info-bar {
  background: var(--amoled-surface-1, #0a0a0a);
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--radius-xl, 20px) var(--radius-xl, 20px) 0 0;
  padding: var(--mobile-gap-md, 12px) var(--mobile-gap-lg, 16px);
  padding-bottom: calc(var(--mobile-gap-lg, 16px) + env(safe-area-inset-bottom, 0px));
  transition: max-height 0.3s ease;
  max-height: 180px;
  overflow: hidden;
}

.info-bar.expanded {
  max-height: 80vh;
  overflow-y: auto;
}

.drag-handle {
  display: flex;
  justify-content: center;
  padding: var(--mobile-gap-sm, 8px) 0;
  cursor: pointer;
}

.handle-bar {
  width: 36px;
  height: 4px;
  background: var(--surface-4, #3f3f46);
  border-radius: var(--radius-full, 9999px);
}

.quick-actions {
  display: flex;
  justify-content: center;
  gap: var(--mobile-gap-md, 12px);
  margin-bottom: var(--mobile-gap-md, 12px);
}

.action-btn {
  width: 2.75rem;
  height: 2.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--amoled-surface-3, #1a1a1a);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 50%;
  color: var(--text-secondary, #a1a1aa);
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:active {
  transform: scale(0.92);
}

.action-btn.active {
  color: var(--error, #ef4444);
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.3);
}

.action-btn svg {
  width: 1.25rem;
  height: 1.25rem;
}

.color-btn.active {
  border-width: 2px;
}

.color-picker-wrapper {
  position: relative;
}

.color-dropdown {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: var(--mobile-gap-sm, 8px);
  padding: var(--mobile-gap-sm, 8px);
  background: var(--amoled-surface-2, #121212);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: var(--radius-lg, 14px);
  margin-bottom: var(--mobile-gap-sm, 8px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.color-option {
  width: 2rem;
  height: 2rem;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.2);
  cursor: pointer;
  transition: all 0.2s ease;
}

.color-option:active {
  transform: scale(0.9);
}

.basic-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  text-align: center;
}

.photo-name {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--text-primary, #fafafa);
}

.photo-meta {
  font-size: 0.75rem;
  color: var(--text-muted, #71717a);
}

.photo-counter {
  font-size: 0.6875rem;
  color: var(--text-muted, #71717a);
  margin-top: 4px;
}

.metadata-panel {
  margin-top: var(--mobile-gap-lg, 16px);
  display: flex;
  flex-direction: column;
  gap: var(--mobile-gap-lg, 16px);
}

.metadata-section {
  background: var(--amoled-surface-2, #121212);
  border-radius: var(--radius-lg, 14px);
  padding: var(--mobile-gap-md, 12px);
}

.section-title {
  display: flex;
  align-items: center;
  gap: var(--mobile-gap-sm, 8px);
  font-size: 0.6875rem;
  font-weight: 700;
  color: var(--accent-color, #00ff41);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: var(--mobile-gap-md, 12px);
}

.section-title svg {
  width: 1rem;
  height: 1rem;
}

.metadata-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--mobile-gap-sm, 8px);
}

.metadata-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metadata-item.full {
  grid-column: span 2;
}

.metadata-item .label {
  font-size: 0.6875rem;
  color: var(--text-muted, #71717a);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.metadata-item .value {
  font-size: 0.8125rem;
  color: var(--text-primary, #fafafa);
  font-weight: 500;
  word-break: break-word;
}

.settings-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--mobile-gap-sm, 8px);
}

.setting-chip {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background: var(--amoled-surface-3, #1a1a1a);
  border-radius: var(--radius-full, 9999px);
  font-size: 0.75rem;
  color: var(--text-secondary, #a1a1aa);
}

.setting-icon {
  font-size: 0.625rem;
  opacity: 0.7;
}

.map-link {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--mobile-gap-sm, 8px);
  margin-top: var(--mobile-gap-md, 12px);
  padding: var(--mobile-gap-sm, 8px) var(--mobile-gap-md, 12px);
  background: var(--accent-light, rgba(0, 255, 65, 0.1));
  border: 1px solid var(--accent-color, #00ff41);
  border-radius: var(--radius-md, 10px);
  color: var(--accent-color, #00ff41);
  font-size: 0.8125rem;
  font-weight: 500;
  text-decoration: none;
  transition: all 0.2s ease;
}

.map-link:active {
  transform: scale(0.98);
  background: var(--accent-medium, rgba(0, 255, 65, 0.25));
}

.map-link svg {
  width: 1rem;
  height: 1rem;
}

.metadata-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--mobile-gap-sm, 8px);
  padding: var(--mobile-gap-xl, 24px);
  color: var(--text-muted, #71717a);
  font-size: 0.8125rem;
}

.spinner {
  width: 1.5rem;
  height: 1.5rem;
  border: 2px solid rgba(var(--accent-rgb, 0, 255, 65), 0.2);
  border-top-color: var(--accent-color, #00ff41);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateY(20px);
}
</style>