import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { load } from '@tauri-apps/plugin-store'
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

export type UploadStatus = 'pending' | 'uploading' | 'success' | 'failed' | 'retrying'

export interface UploadItem {
  id: string
  path: string
  name: string
  status: UploadStatus
  progress: number
  error?: string
  url?: string
  retryCount: number
  maxRetries: number
}

export interface Photo {
  name: string
  url: string
  sha: string
  path?: string
  size?: number
}

interface PrivacySettings {
  stripMetadata: boolean
  compressImages: boolean
}

const MAX_RETRIES = 3
const RETRY_DELAY_MS = 2000

const queue = ref<UploadItem[]>([])
const photos = ref<Photo[]>([])
const isUploading = ref(false)
const loadingPhotos = ref(false)
const privacySettings = ref<PrivacySettings>({ stripMetadata: true, compressImages: false })
const uploadStats = ref({ total: 0, succeeded: 0, failed: 0, retried: 0 })

let unlisten: UnlistenFn | null = null

async function loadPrivacySettings(): Promise<void> {
  try {
    const store = await load('settings.json')
    const stripMetadata = await store.get<boolean>('stripMetadata')
    const compressImages = await store.get<boolean>('compressImages')
    privacySettings.value = {
      stripMetadata: stripMetadata ?? true,
      compressImages: compressImages ?? false
    }
  } catch {
    // Use defaults
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function isRetryableError(error: string): boolean {
  const retryablePatterns = [
    'rate limit',
    'timeout',
    '429',
    '500',
    '502',
    '503',
    '504',
    'network',
    'connection',
    'ECONNRESET',
    'ETIMEDOUT'
  ]
  const lowerError = error.toLowerCase()
  return retryablePatterns.some(pattern => lowerError.includes(pattern.toLowerCase()))
}

export function usePhotoUpload() {
  const { token, repo } = useGitHubAuth()

  const pendingCount = computed(() => queue.value.filter(i => i.status === 'pending' || i.status === 'retrying').length)
  const failedCount = computed(() => queue.value.filter(i => i.status === 'failed').length)
  const successCount = computed(() => queue.value.filter(i => i.status === 'success').length)
  const currentUpload = computed(() => queue.value.find(i => i.status === 'uploading'))
  const retryingCount = computed(() => queue.value.filter(i => i.status === 'retrying').length)

  onMounted(async () => {
    await loadPrivacySettings()
    unlisten = await listen<UploadProgress>('upload-progress', (event) => {
      const item = queue.value.find(i => i.id === event.payload.id)
      if (item) {
        item.progress = event.payload.percent
      }
    })
  })

  onUnmounted(() => {
    if (unlisten) unlisten()
  })

  async function selectFiles() {
    const files = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (files) addToQueue(files as string[])
  }

  function addToQueue(paths: string[]) {
    for (const path of paths) {
      const name = path.split('/').pop() || 'photo'
      queue.value.push({
        id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
        path,
        name,
        status: 'pending',
        progress: 0,
        retryCount: 0,
        maxRetries: MAX_RETRIES
      })
    }
    uploadStats.value.total += paths.length
    if (!isUploading.value) processQueue()
  }

  async function processQueue() {
    if (!token.value || !repo.value) return

    isUploading.value = true
    
    // Reload privacy settings before processing
    await loadPrivacySettings()

    while (true) {
      const next = queue.value.find(i => i.status === 'pending' || i.status === 'retrying')
      if (!next) break

      next.status = 'uploading'
      next.progress = 0

      try {
        const filename = `${Date.now()}-${next.name}`
        
        // Use processed upload if privacy settings require it
        const needsProcessing = privacySettings.value.stripMetadata || privacySettings.value.compressImages
        
        const result = needsProcessing 
          ? await invoke<UploadResult>('upload_photo_processed', {
              path: next.path,
              repo: repo.value,
              token: token.value,
              filename,
              uploadId: next.id,
              stripExif: privacySettings.value.stripMetadata,
              compress: privacySettings.value.compressImages,
              quality: 85
            })
          : await invoke<UploadResult>('upload_photo', {
              path: next.path,
              repo: repo.value,
              token: token.value,
              filename,
              uploadId: next.id
            })
            
        next.status = 'success'
        next.progress = 100
        next.url = result.url
        uploadStats.value.succeeded++
      } catch (e) {
        const errorMsg = String(e)
        
        // Check if we should retry
        if (next.retryCount < next.maxRetries && isRetryableError(errorMsg)) {
          next.retryCount++
          next.status = 'retrying'
          next.error = `Retry ${next.retryCount}/${next.maxRetries}: ${errorMsg}`
          uploadStats.value.retried++
          
          // Exponential backoff
          const delay = RETRY_DELAY_MS * Math.pow(2, next.retryCount - 1)
          await sleep(delay)
        } else {
          next.status = 'failed'
          next.progress = 0
          next.error = errorMsg
          uploadStats.value.failed++
        }
      }
    }

    isUploading.value = false
    await loadPhotos()
  }

  function retryFailed() {
    queue.value.filter(i => i.status === 'failed').forEach(i => {
      i.status = 'pending'
      i.progress = 0
      i.error = undefined
      i.retryCount = 0
    })
    if (!isUploading.value) processQueue()
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

  function resetStats() {
    uploadStats.value = { total: 0, succeeded: 0, failed: 0, retried: 0 }
  }

  async function loadPhotos() {
    if (!token.value || !repo.value) return
    loadingPhotos.value = true
    try {
      photos.value = await invoke<Photo[]>('list_photos', {
        repo: repo.value,
        token: token.value
      })
    } catch {
      // Silent fail
    }
    loadingPhotos.value = false
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
    retryingCount,
    uploadStats,
    selectFiles,
    addToQueue,
    retryFailed,
    removeFromQueue,
    clearCompleted,
    clearAll,
    resetStats,
    loadPhotos
  }
}
