import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { useGitHubAuth } from './useGitHubAuth'

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

const queue = ref<UploadItem[]>([])
const photos = ref<Photo[]>([])
const isUploading = ref(false)
const loadingPhotos = ref(false)

export function usePhotoUpload() {
  const { token, repo } = useGitHubAuth()
  
  // Instance-specific event listener to prevent conflicts
  let unlisten: UnlistenFn | null = null
  let isProcessing = false // Prevent race conditions

  const pendingCount = computed(() => queue.value.filter(i => i.status === 'pending').length)
  const failedCount = computed(() => queue.value.filter(i => i.status === 'failed').length)
  const successCount = computed(() => queue.value.filter(i => i.status === 'success').length)
  const currentUpload = computed(() => queue.value.find(i => i.status === 'uploading'))

  onMounted(async () => {
    try {
      unlisten = await listen<UploadProgress>('upload-progress', (event) => {
        const item = queue.value.find(i => i.id === event.payload.id)
        if (item) {
          item.progress = event.payload.percent
        }
      })
    } catch (error) {
      console.warn('Failed to setup upload progress listener:', error)
    }
  })

  onUnmounted(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })

  async function selectFiles() {
    try {
      const files = await open({
        multiple: true,
        filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
      })
      if (files) addToQueue(files as string[])
    } catch (error) {
      console.error('File selection failed:', error)
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

        try {
          const filename = `${Date.now()}-${next.name}`
          const result = await invoke<UploadResult>('upload_photo', {
            path: next.path,
            repo: repo.value,
            token: token.value,
            filename,
            uploadId: next.id
          })
          
          next.status = 'success'
          next.progress = 100
          next.url = result.url
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
    
    await loadPhotos()
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

  async function loadPhotos() {
    if (!token.value || !repo.value) return
    
    loadingPhotos.value = true
    try {
      const photoUrls = await invoke<string[]>('list_photos', {
        repo: repo.value,
        token: token.value
      })
      
      // Convert URLs to Photo objects with proper typing
      photos.value = photoUrls.map(url => ({
        name: url.split('/').pop() || 'photo',
        url,
        sha: url.split('/').pop()?.split('.')[0] || Math.random().toString(36),
        path: url
      }))
    } catch (error) {
      console.warn('Failed to load photos:', error)
      photos.value = []
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
