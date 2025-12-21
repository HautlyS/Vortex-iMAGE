<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useGitHubAuth } from './composables/useGitHubAuth'
import { usePhotoUpload } from './composables/usePhotoUpload'
import { useAccentColor } from './composables/useAccentColor'
import SpaceLoader from './components/SpaceLoader.vue'
import AuthButton from './components/AuthButton.vue'
import PhotoUploader from './components/PhotoUploader.vue'
import PhotoGallery from './components/PhotoGallery.vue'
import AccentPicker from './components/AccentPicker.vue'

const { token, repo, init, setRepo } = useGitHubAuth()
const { photos, loadingPhotos, loadPhotos, addToQueue } = usePhotoUpload()
const { accentHex, init: initAccent } = useAccentColor()

const loading = ref(true)
const repoInput = ref('')
const isDragging = ref(false)

onMounted(async () => {
  await Promise.all([init(), initAccent()])
  repoInput.value = repo.value
})

watch(repo, (v) => { repoInput.value = v })
watch([token, repo], () => { if (token.value && repo.value) loadPhotos() })

function handleRepoChange() {
  const trimmed = repoInput.value.trim()
  if (trimmed && /^[\w-]+\/[\w.-]+$/.test(trimmed)) setRepo(trimmed)
}

function onDrop(e: DragEvent) {
  isDragging.value = false
  if (!e.dataTransfer?.files.length) return
  const paths: string[] = []
  for (const file of Array.from(e.dataTransfer.files)) {
    if (!/\.(png|jpe?g|gif|webp)$/i.test(file.name)) continue
    // Tauri provides path property, web doesn't
    const path = (file as any).path
    if (path && typeof path === 'string') {
      paths.push(path)
    }
  }
  if (paths.length) addToQueue(paths)
}
</script>

<template>
  <SpaceLoader v-if="loading" @complete="loading = false" />

  <div v-else class="min-h-screen bg-amoled text-white pb-12 noise" @dragover.prevent="isDragging = true" @dragleave.prevent="isDragging = false" @drop.prevent="onDrop">
    <!-- Ambient glow -->
    <div class="fixed top-0 left-1/4 w-96 h-96 rounded-full blur-[150px] opacity-10 pointer-events-none" :style="{ backgroundColor: accentHex }"></div>
    <div class="fixed bottom-0 right-1/4 w-96 h-96 rounded-full blur-[150px] opacity-10 pointer-events-none" :style="{ backgroundColor: accentHex }"></div>

    <!-- Drag Overlay -->
    <Transition name="fade">
      <div v-if="isDragging && token && repo" class="fixed inset-0 z-40 bg-amoled/95 flex items-center justify-center">
        <div class="text-center animate-float">
          <div class="w-32 h-32 mx-auto mb-6 rounded-full border-2 border-dashed flex items-center justify-center animate-pulse-glow" :style="{ borderColor: accentHex, '--accent': accentHex }">
            <span class="text-5xl">⬆</span>
          </div>
          <span class="text-xl font-mono glow-text" :style="{ '--accent': accentHex, color: accentHex }">DROP PHOTOS HERE</span>
        </div>
      </div>
    </Transition>

    <!-- Header -->
    <header class="border-b border-amoled-gray/50 backdrop-blur-sm bg-amoled/80 sticky top-0 z-30">
      <div class="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
        <div class="flex items-center gap-3 group">
          <span class="text-2xl animate-spin-slow" :style="{ color: accentHex }">◈</span>
          <h1 class="font-mono text-xl tracking-wider group-hover:animate-glitch" :style="{ color: accentHex }">iMAGE</h1>
        </div>
        <div class="flex items-center gap-4">
          <AccentPicker />
          <AuthButton />
        </div>
      </div>
    </header>

    <main class="max-w-7xl mx-auto px-4 py-8">
      <template v-if="token">
        <div class="grid grid-cols-1 lg:grid-cols-4 gap-6 mb-8">
          <!-- Sidebar -->
          <div class="lg:col-span-1 space-y-4 animate-fade-in-up" style="animation-delay: 0.1s">
            <div class="p-4 bg-amoled-dark/80 backdrop-blur rounded-lg border border-amoled-gray/50 relative overflow-hidden">
              <div class="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-current to-transparent opacity-50" :style="{ color: accentHex }"></div>
              <label for="repo-input" class="block text-xs text-gray-500 uppercase tracking-wider mb-2 font-mono">◇ Repository</label>
              <input id="repo-input" v-model="repoInput" @blur="handleRepoChange" @keyup.enter="handleRepoChange" placeholder="owner/repo" class="w-full px-3 py-2 bg-amoled border border-amoled-light rounded font-mono text-sm focus:outline-none transition-all focus:border-current" :style="{ '--tw-border-opacity': 1 }" />
            </div>
            <PhotoUploader v-if="repo" />
          </div>

          <!-- Gallery -->
          <div class="lg:col-span-3 animate-fade-in-up" style="animation-delay: 0.2s">
            <PhotoGallery :photos="photos" :loading="loadingPhotos" />
          </div>
        </div>
      </template>

      <!-- Not logged in -->
      <div v-else class="flex flex-col items-center justify-center py-32 text-center animate-fade-in">
        <div class="relative mb-8">
          <div class="text-8xl animate-float" :style="{ color: accentHex }">◇</div>
          <div class="absolute inset-0 blur-2xl opacity-30" :style="{ backgroundColor: accentHex }"></div>
        </div>
        <h2 class="text-3xl font-mono tracking-wider mb-3 glow-text" :style="{ '--accent': accentHex }">WELCOME, TRAVELER</h2>
        <p class="text-gray-500 font-mono text-sm mb-8 max-w-md">Initialize connection to sync your visual data across the galaxy</p>
        <AuthButton />
        <div class="mt-12 flex gap-8 text-xs font-mono text-gray-600">
          <span>◈ SECURE</span>
          <span>◈ ENCRYPTED</span>
          <span>◈ DECENTRALIZED</span>
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="fixed bottom-0 left-0 right-0 border-t border-amoled-gray/50 bg-amoled/80 backdrop-blur-sm">
      <div class="max-w-7xl mx-auto px-4 py-2 flex justify-between items-center text-xs text-gray-600 font-mono">
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full animate-pulse" :style="{ backgroundColor: accentHex }"></span>
          <span>SYSTEM ACTIVE</span>
        </div>
        <span :style="{ color: accentHex }">{{ photos.length }} FILES SYNCED</span>
      </div>
    </footer>
  </div>
</template>

<style>
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
