<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { SHORTCUTS } from '../../config'
import type { Photo } from '../../types/photo'

const props = defineProps<{
  photo: Photo | null
  photos: Photo[]
  isFavorite: (id: string) => boolean
}>()

const emit = defineEmits<{
  close: []
  next: []
  prev: []
  favorite: [photo: Photo]
  'copy-url': [url: string]
}>()

const imageLoading = ref(false)
const imageError = ref(false)

const selectedIndex = computed(() => 
  props.photos.findIndex(p => p.sha === props.photo?.sha)
)

const canGoNext = computed(() => 
  selectedIndex.value < props.photos.length - 1
)

const canGoPrev = computed(() => 
  selectedIndex.value > 0
)

function onImageLoad() {
  imageError.value = false
  imageLoading.value = false
}

function onImageError() {
  imageError.value = true
  imageLoading.value = false
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.photo) return
  
  switch (e.key) {
    case SHORTCUTS.escape.key:
      emit('close')
      break
    case 'ArrowRight':
      if (canGoNext.value) emit('next')
      break
    case 'ArrowLeft':
      if (canGoPrev.value) emit('prev')
      break
    case SHORTCUTS.favorite.key:
      emit('favorite', props.photo)
      break
  }
}

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

// Watch for photo changes to reset loading state
watch(() => props.photo, (newPhoto) => {
  if (newPhoto) {
    imageLoading.value = true
    imageError.value = false
  }
})

const keydownHandler = ref<((e: KeyboardEvent) => void) | null>(null)

onMounted(() => {
  keydownHandler.value = handleKeydown
  document.addEventListener('keydown', keydownHandler.value)
  if (props.photo) {
    imageLoading.value = true
  }
})

onUnmounted(() => {
  if (keydownHandler.value) {
    document.removeEventListener('keydown', keydownHandler.value)
    keydownHandler.value = null
  }
})
</script>

<template>
  <Teleport to="body">
    <Transition name="lightbox">
      <div v-if="photo" class="lightbox" @click.self="emit('close')">
        <button class="lb-close" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>

        <button 
          v-if="canGoPrev" 
          class="lb-nav lb-prev" 
          @click="emit('prev')"
          aria-label="Foto anterior"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m15 18-6-6 6-6"/>
          </svg>
        </button>

        <div class="lb-content">
          <div class="lb-image-container">
            <img 
              :src="photo.url" 
              :alt="photo.name" 
              @load="onImageLoad"
              @error="onImageError"
            />
          </div>
          <div v-if="imageError" class="lb-error">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="m21 21-6-6m0 0L9 9m6 6 6-6M15 15l-6-6"/>
            </svg>
            <p>Erro ao carregar imagem</p>
          </div>
        </div>

        <button 
          v-if="canGoNext" 
          class="lb-nav lb-next" 
          @click="emit('next')"
          aria-label="Próxima foto"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m9 18 6-6-6-6"/>
          </svg>
        </button>

        <div class="lb-info">
          <div class="lb-details">
            <span class="lb-name">{{ photo.name }}</span>
            <span class="lb-meta">{{ formatSize(photo.size) }}</span>
          </div>
          <div class="lb-actions">
            <button 
              @click="emit('favorite', photo)" 
              :class="{ active: isFavorite(photo.sha) }"
              :aria-label="isFavorite(photo.sha) ? 'Remover dos favoritos' : 'Adicionar aos favoritos'"
            >
              <svg viewBox="0 0 24 24" :fill="isFavorite(photo.sha) ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
                <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
              </svg>
            </button>
            <button 
              @click="emit('copy-url', photo.url)"
              aria-label="Copiar URL da foto"
            >
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
</template>

<style scoped>
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

.lb-close:hover { 
  background: rgba(255,255,255,0.2); 
  transform: scale(1.05); 
}

.lb-close svg { 
  width: 1.5rem; 
  height: 1.5rem; 
}

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

.lb-nav:hover { 
  background: rgba(255,255,255,0.2); 
  transform: translateY(-50%) scale(1.05); 
}

.lb-nav svg { 
  width: 1.5rem; 
  height: 1.5rem; 
}

.lb-prev { left: 1rem; }
.lb-next { right: 1rem; }

.lb-content {
  max-width: 90vw;
  max-height: 85vh;
  animation: scaleIn 0.2s ease-out;
  position: relative;
}

.lb-image-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.lb-content img {
  max-width: 100%;
  max-height: 85vh;
  object-fit: contain;
  border-radius: 0.5rem;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  transition: opacity 0.3s ease;
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

.lb-details { 
  display: flex; 
  flex-direction: column; 
  gap: 0.125rem; 
}

.lb-name { 
  font-size: 0.875rem; 
  color: white; 
  font-weight: 500; 
}

.lb-meta { 
  font-size: 0.75rem; 
  color: var(--text-muted, #71717a); 
}

.lb-actions { 
  display: flex; 
  gap: 0.5rem; 
}

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

.lb-actions button:hover { 
  background: rgba(255,255,255,0.2); 
}

.lb-actions button.active { 
  color: var(--error, #ef4444); 
}

.lb-actions button svg { 
  width: 1rem; 
  height: 1rem; 
}

.lb-counter { 
  font-size: 0.75rem; 
  color: var(--text-muted, #71717a); 
}

.lb-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  color: var(--text-muted, #71717a);
  gap: 1rem;
}

.lb-error svg {
  width: 3rem;
  height: 3rem;
  opacity: 0.5;
}

.lb-error p {
  font-size: 0.875rem;
}

@keyframes scaleIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.lightbox-enter-active, .lightbox-leave-active { 
  transition: opacity 0.2s ease; 
}

.lightbox-enter-from, .lightbox-leave-to { 
  opacity: 0; 
}
</style>
