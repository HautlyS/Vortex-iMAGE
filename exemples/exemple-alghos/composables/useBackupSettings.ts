import { ref, computed } from 'vue'
import { load } from '@tauri-apps/plugin-store'

export interface BackupConfig {
  enabled: boolean
  watchFolder: string | null
  autoUpload: boolean
  deleteAfterUpload: boolean
  syncDeletions: boolean // Delete from local when deleted from remote
  targetDriverId: string | null
  fileTypes: string[]
  excludePatterns: string[]
  minFileSize: number // bytes, 0 = no minimum
  maxFileSize: number // bytes, 0 = no maximum
  lastBackupAt?: number
  totalBackedUp: number
}

export interface BackupStats {
  pendingFiles: number
  totalSize: number
  lastSync: number | null
  errors: string[]
}

const DEFAULT_CONFIG: BackupConfig = {
  enabled: false,
  watchFolder: null,
  autoUpload: false,
  deleteAfterUpload: false,
  syncDeletions: false,
  targetDriverId: null,
  fileTypes: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'heic', 'heif', 'avif', 'bmp', 'tiff'],
  excludePatterns: ['.*', '_*', 'Thumbs.db', '.DS_Store'],
  minFileSize: 0,
  maxFileSize: 0,
  totalBackedUp: 0
}

const config = ref<BackupConfig>({ ...DEFAULT_CONFIG })
const stats = ref<BackupStats>({
  pendingFiles: 0,
  totalSize: 0,
  lastSync: null,
  errors: []
})
const isWatching = ref(false)
let initialized = false

export function useBackupSettings() {
  const isConfigured = computed(() => 
    config.value.watchFolder !== null && config.value.targetDriverId !== null
  )

  const canAutoBackup = computed(() =>
    config.value.enabled && isConfigured.value && config.value.autoUpload
  )

  async function loadConfig(): Promise<void> {
    if (initialized) return
    try {
      const store = await load('settings.json')
      const saved = await store.get<BackupConfig>('backupConfig')
      if (saved) {
        config.value = { ...DEFAULT_CONFIG, ...saved }
      }
      initialized = true
    } catch (e) {
      console.error('Failed to load backup config:', e)
    }
  }

  async function saveConfig(): Promise<void> {
    try {
      const store = await load('settings.json')
      await store.set('backupConfig', config.value)
      await store.save()
    } catch (e) {
      console.error('Failed to save backup config:', e)
    }
  }

  async function setWatchFolder(path: string | null): Promise<void> {
    config.value.watchFolder = path
    await saveConfig()
  }

  async function setTargetDriver(driverId: string | null): Promise<void> {
    config.value.targetDriverId = driverId
    await saveConfig()
  }

  async function setEnabled(enabled: boolean): Promise<void> {
    config.value.enabled = enabled
    await saveConfig()
  }

  async function setAutoUpload(enabled: boolean): Promise<void> {
    config.value.autoUpload = enabled
    await saveConfig()
  }

  async function setDeleteAfterUpload(enabled: boolean): Promise<void> {
    config.value.deleteAfterUpload = enabled
    await saveConfig()
  }

  async function setSyncDeletions(enabled: boolean): Promise<void> {
    config.value.syncDeletions = enabled
    await saveConfig()
  }

  async function setFileTypes(types: string[]): Promise<void> {
    config.value.fileTypes = types
    await saveConfig()
  }

  async function addExcludePattern(pattern: string): Promise<void> {
    if (!config.value.excludePatterns.includes(pattern)) {
      config.value.excludePatterns.push(pattern)
      await saveConfig()
    }
  }

  async function removeExcludePattern(pattern: string): Promise<void> {
    const index = config.value.excludePatterns.indexOf(pattern)
    if (index !== -1) {
      config.value.excludePatterns.splice(index, 1)
      await saveConfig()
    }
  }

  async function updateStats(newStats: Partial<BackupStats>): Promise<void> {
    Object.assign(stats.value, newStats)
  }

  async function recordBackup(count: number): Promise<void> {
    config.value.totalBackedUp += count
    config.value.lastBackupAt = Date.now()
    stats.value.lastSync = Date.now()
    await saveConfig()
  }

  async function resetConfig(): Promise<void> {
    config.value = { ...DEFAULT_CONFIG }
    await saveConfig()
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return 'Sem limite'
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  function formatLastBackup(): string {
    if (!config.value.lastBackupAt) return 'Nunca'
    const diff = Date.now() - config.value.lastBackupAt
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)
    
    if (minutes < 1) return 'Agora mesmo'
    if (minutes < 60) return `${minutes} min atrás`
    if (hours < 24) return `${hours}h atrás`
    return `${days} dias atrás`
  }

  return {
    config,
    stats,
    isWatching,
    isConfigured,
    canAutoBackup,
    loadConfig,
    setWatchFolder,
    setTargetDriver,
    setEnabled,
    setAutoUpload,
    setDeleteAfterUpload,
    setSyncDeletions,
    setFileTypes,
    addExcludePattern,
    removeExcludePattern,
    updateStats,
    recordBackup,
    resetConfig,
    formatFileSize,
    formatLastBackup
  }
}
