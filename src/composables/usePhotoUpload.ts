/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useGitHubAuth, isDevMode } from './useGitHubAuth'

interface UploadResult {
  url: string
  sha: string
}

interface UploadProgress {
  id: string
  bytes_sent: number
  total_bytes: number
  percent: number
}

export type UploadStatus = 'pending' | 'uploading' | 'success' | 'failed'

export interface UploadItem {
  id: string
  path: string
  name: string
  status: UploadStatus
  progress: number
  error?: string
  url?: string
}

export interface Photo {
  name: string
  url: string
  sha: string
  path?: string
  size?: number
}

const MOCK_PHOTOS: Photo[] = Array.from({ length: 50 }, (_, i) => {
  const heights = [400, 500, 600, 700, 800]
  const h = heights[i % heights.length]
  const names = ['sunset', 'mountain', 'city', 'forest', 'ocean', 'desert', 'autumn', 'snow', 'beach', 'night', 'waterfall', 'flower', 'lake', 'canyon', 'aurora', 'wildlife', 'architecture', 'portrait', 'street', 'abstract']
  return {
    name: `${names[i % names.length]}-${i + 1}.jpg`,
    url: `https://picsum.photos/seed/${i}/${h}/${h}`,
    sha: `mock-sha-${i + 1}`,
    size: 150000 + Math.floor(Math.random() * 200000),
    path: i < 10 ? 'viagens' : i < 20 ? 'viagens/2024' : i < 30 ? 'familia' : undefined
  }
})

const queue = ref<UploadItem[]>([])
const photos = ref<Photo[]>(isDevMode ? [...MOCK_PHOTOS] : [])
const isUploading = ref(false)
const loadingPhotos = ref(false)
let initialized = false

export function usePhotoUpload() {
  const { token, repo, publicBundle } = useGitHubAuth()

  let unlisten: (() => void) | null = null
  let isProcessing = false

  const pendingCount = computed(() => queue.value.filter(i => i.status === 'pending').length)
  const failedCount = computed(() => queue.value.filter(i => i.status === 'failed').length)
  const successCount = computed(() => queue.value.filter(i => i.status === 'success').length)
  const currentUpload = computed(() => queue.value.find(i => i.status === 'uploading'))

  watch([token, repo], ([t, r]) => {
    if (t && r && !isDevMode) loadPhotos()
  }, { immediate: true })

  onMounted(async () => {
    if (isDevMode) {
      
      return
    }

    try {
      const { listen } = await import('@tauri-apps/api/event')
      unlisten = await listen<UploadProgress>('upload-progress', (event) => {
        const item = queue.value.find(i => i.id === event.payload.id)
        if (item) item.progress = event.payload.percent
      })
    } catch { }

    if (!initialized && token.value && repo.value) {
      initialized = true
      loadPhotos()
    }
  })

  onUnmounted(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })

  async function selectFiles() {
    if (isDevMode) {
      
      const mockFiles = [
        `photo-${Date.now()}-1.jpg`,
        `photo-${Date.now()}-2.jpg`,
      ]
      addToQueue(mockFiles)
      return
    }

    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const files = await open({
        multiple: true,
        filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
      })
      if (files) addToQueue(files as string[])
    } catch (e) {
      console.error('Failed to open file dialog:', e)
    }
  }

  function addToQueue(paths: string[]) {
    for (const path of paths) {
      const name = path.split('/').pop() || 'photo'
      queue.value.push({
        id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
        path,
        name,
        status: 'pending',
        progress: 0
      })
    }
    if (!isUploading.value && !isProcessing) processQueue()
  }

  async function processQueue() {
    if (!token.value || !repo.value || isProcessing) return

    isProcessing = true
    isUploading.value = true

    try {
      while (true) {
        const next = queue.value.find(i => i.status === 'pending')
        if (!next) break

        next.status = 'uploading'
        next.progress = 0

        if (isDevMode) {
          
          for (let p = 0; p <= 100; p += 20) {
            next.progress = p
            await new Promise(r => setTimeout(r, 200))
          }
          next.status = 'success'
          next.progress = 100

          const seed = Math.floor(Math.random() * 1000)
          photos.value.unshift({
            name: next.name,
            url: `https://picsum.photos/seed/${seed}/600/600`,
            sha: `mock-sha-${seed}`,
            size: Math.floor(Math.random() * 200000) + 150000
          })
          continue
        }

        try {
          const { invoke } = await import('@tauri-apps/api/core')
          const filename = `${Date.now()}-${next.name}`

          if (!publicBundle.value) throw new Error("Encryption keys not available")

          const result = await invoke<UploadResult>('upload_photo', {
            path: next.path,
            repo: repo.value,
            token: token.value,
            filename,
            uploadId: next.id,
            publicBundle: publicBundle.value
          })

          next.status = 'success'
          next.progress = 100
          next.url = result.url

          photos.value.unshift({
            name: filename,
            url: result.url,
            sha: result.sha,
            path: `photos/${filename}`
          })
        } catch (e) {
          next.status = 'failed'
          next.progress = 0
          next.error = String(e)
        }
      }
    } finally {
      isUploading.value = false
      isProcessing = false
    }

    if (!isDevMode) await loadPhotos()
  }

  function retryFailed() {
    queue.value.filter(i => i.status === 'failed').forEach(i => {
      i.status = 'pending'
      i.progress = 0
      i.error = undefined
    })
    if (!isUploading.value && !isProcessing) processQueue()
  }

  function removeFromQueue(id: string) {
    const idx = queue.value.findIndex(i => i.id === id)
    if (idx !== -1 && queue.value[idx].status !== 'uploading') {
      queue.value.splice(idx, 1)
    }
  }

  function clearCompleted() {
    queue.value = queue.value.filter(i => i.status !== 'success')
  }

  function clearAll() {
    if (!isUploading.value) {
      queue.value = []
    } else {
      queue.value = queue.value.filter(i => i.status === 'uploading')
    }
  }

  async function loadPhotos(folder?: string) {
    if (isDevMode) {
      
      loadingPhotos.value = true
      await new Promise(r => setTimeout(r, 300))
      loadingPhotos.value = false
      return
    }

    if (!token.value || !repo.value || loadingPhotos.value) return

    loadingPhotos.value = true
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const photoUrls = await invoke<string[]>('list_photos', {
        repo: repo.value,
        token: token.value,
        folder: folder || undefined
      })

      photos.value = photoUrls.map(url => {
        const filename = url.split('/').pop() || 'photo'
        return {
          name: filename,
          url,
          sha: filename.replace(/\.[^.]+$/, ''),
          path: url
        }
      })
    } catch (e) {
      console.error('Failed to load photos:', e)
    } finally {
      loadingPhotos.value = false
    }
  }

  return {
    queue,
    photos,
    isUploading,
    loadingPhotos,
    pendingCount,
    failedCount,
    successCount,
    currentUpload,
    selectFiles,
    addToQueue,
    retryFailed,
    removeFromQueue,
    clearCompleted,
    clearAll,
    loadPhotos
  }
}