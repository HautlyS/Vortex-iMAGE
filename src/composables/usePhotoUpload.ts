/**
 * Photo Upload Composable
 * 
 * Handles file uploads with automatic compression and encryption.
 * Uses the best available technology automatically.
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useGitHubAuth, isDevMode, isWebMode } from './useGitHubAuth'
import { useMediaSettings, toBackendSettings, type SimpleSettings } from './useMediaSettings'

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
  folderPath?: string
  settings: SimpleSettings
  password?: string
  // Web mode: store File object for browser uploads
  file?: File
  base64?: string
}

export interface Photo {
  name: string
  url: string
  sha: string
  path?: string
  size?: number
}

// Mock data for dev mode
const MOCK_PHOTOS: Photo[] = Array.from({ length: 30 }, (_, i) => ({
  name: `photo-${i + 1}.jpg`,
  url: `https://picsum.photos/seed/${i}/600/600`,
  sha: `mock-sha-${i + 1}`,
  size: 150000 + Math.floor(Math.random() * 200000)
}))

const queue = ref<UploadItem[]>([])
const photos = ref<Photo[]>(isDevMode ? [...MOCK_PHOTOS] : [])
const isUploading = ref(false)
const loadingPhotos = ref(false)
let initialized = false

export function usePhotoUpload() {
  const { token, repo, publicBundle } = useGitHubAuth()
  const { getFolderSettings, initialize: initMediaSettings } = useMediaSettings()

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
    if (isDevMode) return

    await initMediaSettings()

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

  /**
   * Add files to upload queue (Tauri mode - file paths)
   */
  function addToQueue(
    paths: string[], 
    folderPath?: string, 
    customSettings?: SimpleSettings,
    password?: string
  ) {
    // Get settings (custom or folder-based or global)
    const settings = customSettings || getFolderSettings(folderPath || '')

    for (const path of paths) {
      const name = path.split('/').pop() || 'file'
      queue.value.push({
        id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
        path,
        name,
        status: 'pending',
        progress: 0,
        folderPath,
        settings,
        password
      })
    }
    
    if (!isUploading.value && !isProcessing) processQueue()
  }

  /**
   * Add File objects to upload queue (Web mode - browser File API)
   */
  function addFilesToQueue(
    files: File[],
    folderPath?: string,
    customSettings?: SimpleSettings,
    password?: string
  ) {
    const settings = customSettings || getFolderSettings(folderPath || '')

    for (const file of files) {
      queue.value.push({
        id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
        path: file.name, // Use name as path for web mode
        name: file.name,
        status: 'pending',
        progress: 0,
        folderPath,
        settings,
        password,
        file // Store File object for web upload
      })
    }
    
    if (!isUploading.value && !isProcessing) processQueue()
  }

  /**
   * Process upload queue
   */
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

        // Dev mode simulation
        if (isDevMode) {
          for (let p = 0; p <= 100; p += 20) {
            next.progress = p
            await new Promise(r => setTimeout(r, 150))
          }
          next.status = 'success'
          next.progress = 100
          photos.value.unshift({
            name: next.name,
            url: `https://picsum.photos/seed/${Date.now()}/600/600`,
            sha: `mock-sha-${Date.now()}`,
            size: Math.floor(Math.random() * 200000) + 150000
          })
          continue
        }

        // Web mode: upload via GitHub API
        if (isWebMode) {
          try {
            const filename = `${Date.now()}-${next.name}`
            let base64Content: string

            // Get base64 content from File object
            if (next.file) {
              base64Content = await fileToBase64(next.file)
            } else {
              throw new Error('No file data available for web upload')
            }

            next.progress = 30

            // Upload to GitHub
            const response = await fetch(
              `https://api.github.com/repos/${repo.value}/contents/photos/${filename}`,
              {
                method: 'PUT',
                headers: {
                  'Authorization': `Bearer ${token.value}`,
                  'Content-Type': 'application/json',
                  'User-Agent': 'vortex-image'
                },
                body: JSON.stringify({
                  message: `Upload ${filename}`,
                  content: base64Content
                })
              }
            )

            next.progress = 80

            if (!response.ok) {
              const error = await response.json()
              throw new Error(error.message || 'Upload failed')
            }

            const result = await response.json()
            next.status = 'success'
            next.progress = 100
            next.url = result.content?.html_url

            photos.value.unshift({
              name: filename,
              url: result.content?.download_url || `https://raw.githubusercontent.com/${repo.value}/main/photos/${filename}`,
              sha: result.content?.sha || filename,
              path: `photos/${filename}`
            })
          } catch (e) {
            next.status = 'failed'
            next.progress = 0
            next.error = String(e).replace('Error: ', '')
          }
          continue
        }

        // Tauri mode: use invoke
        try {
          const { invoke } = await import('@tauri-apps/api/core')
          const filename = `${Date.now()}-${next.name}`

          // Convert simple settings to backend format
          const backendSettings = toBackendSettings(
            next.settings,
            publicBundle.value,
            next.password
          )

          // Validate encryption requirements
          if (backendSettings.encryption.enabled) {
            if (backendSettings.encryption.use_keypair && !publicBundle.value) {
              throw new Error("Keypair not available. Unlock your keypair in Security Settings.")
            }
            if (backendSettings.encryption.use_password && !next.password) {
              throw new Error("Password required for encryption")
            }
          }

          const result = await invoke<UploadResult>('upload_photo', {
            path: next.path,
            repo: repo.value,
            token: token.value,
            filename,
            uploadId: next.id,
            publicBundle: backendSettings.encryption.use_keypair ? publicBundle.value : null,
            password: backendSettings.encryption.use_password ? next.password : null,
            settings: backendSettings
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
          next.error = String(e).replace('Error: ', '')
        }
      }
    } finally {
      isUploading.value = false
      isProcessing = false
    }

    if (!isDevMode) await loadPhotos()
  }

  /**
   * Convert File to base64 string (without data URL prefix)
   */
  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const result = reader.result as string
        // Remove data URL prefix (e.g., "data:image/png;base64,")
        const base64 = result.split(',')[1]
        resolve(base64)
      }
      reader.onerror = reject
      reader.readAsDataURL(file)
    })
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
      // Web mode: use GitHub API directly
      if (isWebMode) {
        const path = folder ? `photos/${folder}` : 'photos'
        const response = await fetch(
          `https://api.github.com/repos/${repo.value}/contents/${path}`,
          {
            headers: {
              'Authorization': `Bearer ${token.value}`,
              'Accept': 'application/vnd.github.v3+json'
            }
          }
        )
        
        if (response.ok) {
          const contents = await response.json()
          const imageFiles = Array.isArray(contents) 
            ? contents.filter((f: any) => f.type === 'file' && /\.(jpg|jpeg|png|gif|webp)$/i.test(f.name))
            : []
          
          photos.value = imageFiles.map((f: any) => ({
            name: f.name,
            url: f.download_url,
            sha: f.sha,
            path: f.path,
            size: f.size
          }))
        } else {
          photos.value = []
        }
        return
      }

      // Tauri mode: use invoke
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
    addToQueue,
    addFilesToQueue,
    retryFailed,
    removeFromQueue,
    clearCompleted,
    clearAll,
    loadPhotos
  }
}
