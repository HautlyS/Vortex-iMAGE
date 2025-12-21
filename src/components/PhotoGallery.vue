<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { useAccentColor } from '../composables/useAccentColor'

const props = defineProps<{ photos: string[], loading?: boolean }>()
const { accentHex } = useAccentColor()

const lightbox = ref<string | null>(null)
// Use reactive object instead of Set for proper reactivity
const imageLoaded = reactive<Record<string, boolean>>({})

function openLightbox(url: string) { lightbox.value = url }
function closeLightbox() { lightbox.value = null }
function onImageLoad(url: string) { imageLoaded[url] = true }

function nextImage() {
  if (!lightbox.value) return
  const idx = props.photos.indexOf(lightbox.value)
  if (idx < props.photos.length - 1) lightbox.value = props.photos[idx + 1]
}

function prevImage() {
  if (!lightbox.value) return
  const idx = props.photos.indexOf(lightbox.value)
  if (idx > 0) lightbox.value = props.photos[idx - 1]
}

function onKeydown(e: KeyboardEvent) {
  if (!lightbox.value) return
  if (e.key === 'Escape') closeLightbox()
  if (e.key === 'ArrowRight') nextImage()
  if (e.key === 'ArrowLeft') prevImage()
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <!-- Loading State -->
  <div v-if="loading" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
    <div v-for="i in 8" :key="i" class="aspect-square bg-amoled-gray/50 rounded-lg overflow-hidden">
      <div class="w-full h-full animate-pulse bg-gradient-to-br from-amoled-gray to-amoled-light"></div>
    </div>
  </div>

  <!-- Gallery Grid -->
  <TransitionGroup v-else-if="photos.length" name="gallery" tag="div" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
    <div v-for="(url, i) in photos" :key="url" @click="openLightbox(url)" class="group relative aspect-square overflow-hidden rounded-lg border border-amoled-gray/50 cursor-pointer transition-all duration-300 hover:border-current hover:scale-[1.02] hover:z-10" :style="{ animationDelay: `${i * 50}ms`, '--tw-border-opacity': 1 }">
      <!-- Skeleton -->
      <div v-if="!imageLoaded[url]" class="absolute inset-0 bg-amoled-gray animate-pulse"></div>
      <!-- Image -->
      <img :src="url" @load="onImageLoad(url)" :alt="url.split('/').pop()" class="w-full h-full object-cover transition-all duration-500 group-hover:scale-110" :class="{ 'opacity-0': !imageLoaded[url] }" loading="lazy" />
      <!-- Overlay -->
      <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-all duration-300">
        <div class="absolute bottom-0 left-0 right-0 p-3">
          <p class="text-xs font-mono text-white/90 truncate">{{ url.split('/').pop() }}</p>
        </div>
      </div>
      <!-- Corner accent -->
      <div class="absolute top-0 right-0 w-8 h-8 opacity-0 group-hover:opacity-100 transition-opacity">
        <div class="absolute top-0 right-0 w-full h-px" :style="{ backgroundColor: accentHex }"></div>
        <div class="absolute top-0 right-0 h-full w-px" :style="{ backgroundColor: accentHex }"></div>
      </div>
    </div>
  </TransitionGroup>

  <!-- Empty State -->
  <div v-else class="flex flex-col items-center justify-center py-20 text-gray-500 font-mono">
    <div class="relative">
      <span class="text-6xl animate-float block">◇</span>
      <div class="absolute inset-0 blur-xl opacity-20" :style="{ backgroundColor: accentHex }"></div>
    </div>
    <p class="text-sm uppercase tracking-wider mt-6">No photos yet</p>
    <p class="text-xs mt-2" :style="{ color: accentHex }">Upload your first image</p>
  </div>

  <!-- Lightbox -->
  <Teleport to="body">
    <Transition name="lightbox">
      <div v-if="lightbox" class="fixed inset-0 z-50 bg-black/98 flex items-center justify-center" @click.self="closeLightbox">
        <!-- Close -->
        <button @click="closeLightbox" class="absolute top-6 right-6 w-10 h-10 flex items-center justify-center text-white/60 hover:text-white text-xl font-mono rounded-full border border-white/20 hover:border-white/40 transition-all hover:rotate-90">✕</button>
        
        <!-- Nav -->
        <button v-if="photos.indexOf(lightbox) > 0" @click="prevImage" class="absolute left-6 w-12 h-12 flex items-center justify-center text-white/60 hover:text-white text-2xl font-mono rounded-full border border-white/20 hover:border-white/40 transition-all hover:-translate-x-1">‹</button>
        <button v-if="photos.indexOf(lightbox) < photos.length - 1" @click="nextImage" class="absolute right-6 w-12 h-12 flex items-center justify-center text-white/60 hover:text-white text-2xl font-mono rounded-full border border-white/20 hover:border-white/40 transition-all hover:translate-x-1">›</button>
        
        <!-- Image -->
        <img :src="lightbox" :alt="lightbox.split('/').pop()" class="max-w-[90vw] max-h-[85vh] object-contain rounded-lg shadow-2xl" />
        
        <!-- Info -->
        <div class="absolute bottom-6 left-1/2 -translate-x-1/2 flex items-center gap-4 px-4 py-2 bg-black/50 backdrop-blur rounded-full border border-white/10">
          <span class="text-xs font-mono text-white/60">{{ photos.indexOf(lightbox) + 1 }} / {{ photos.length }}</span>
          <span class="text-xs font-mono text-white/40 truncate max-w-[200px]">{{ lightbox.split('/').pop() }}</span>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.gallery-enter-active { animation: fade-in-up 0.4s ease forwards; }
.gallery-leave-active { animation: fade-in-up 0.3s ease reverse forwards; }
.gallery-move { transition: transform 0.3s ease; }

.lightbox-enter-active, .lightbox-leave-active { transition: all 0.3s ease; }
.lightbox-enter-from, .lightbox-leave-to { opacity: 0; }
.lightbox-enter-from img, .lightbox-leave-to img { transform: scale(0.9); }

@keyframes fade-in-up {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
