/**
 * Simplified Media Settings Composable
 * 
 * Simple toggles for compression and encryption.
 * Technology choices are automatic - users just decide WHAT to protect, not HOW.
 */

import { ref, watch } from 'vue'
import type { PublicBundle } from './useCrypto'

const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI__

// ============================================================================
// Types - Simplified
// ============================================================================

export interface SimpleSettings {
  /** Compress files to save space (auto-selects best algorithm) */
  compress: boolean
  /** Encrypt files for privacy (uses post-quantum encryption when keypair available) */
  encrypt: boolean
}

export interface FolderSettings {
  path: string
  name: string
  settings: SimpleSettings
  createdAt: number
}

// ============================================================================
// State
// ============================================================================

/** Global default settings */
const globalSettings = ref<SimpleSettings>({
  compress: true,
  encrypt: true
})

/** Per-folder overrides */
const folderSettings = ref<Map<string, FolderSettings>>(new Map())

const initialized = ref(false)

// ============================================================================
// Composable
// ============================================================================

export function useMediaSettings() {
  
  /**
   * Initialize settings from storage
   */
  async function initialize(): Promise<void> {
    if (initialized.value) return

    try {
      if (isTauri) {
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('media-settings.json')
        
        const savedGlobal = await store.get<SimpleSettings>('globalSettings')
        if (savedGlobal) {
          globalSettings.value = { ...globalSettings.value, ...savedGlobal }
        }

        const savedFolders = await store.get<[string, FolderSettings][]>('folderSettings')
        if (savedFolders) {
          folderSettings.value = new Map(savedFolders)
        }
      } else {
        const savedGlobal = localStorage.getItem('mediaGlobalSettings')
        if (savedGlobal) {
          globalSettings.value = { ...globalSettings.value, ...JSON.parse(savedGlobal) }
        }

        const savedFolders = localStorage.getItem('mediaFolderSettings')
        if (savedFolders) {
          folderSettings.value = new Map(JSON.parse(savedFolders))
        }
      }

      initialized.value = true
    } catch (e) {
      console.error('Failed to initialize media settings:', e)
      initialized.value = true
    }
  }

  /**
   * Save settings to storage
   */
  async function saveSettings(): Promise<void> {
    try {
      if (isTauri) {
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('media-settings.json')
        await store.set('globalSettings', globalSettings.value)
        await store.set('folderSettings', Array.from(folderSettings.value.entries()))
        await store.save()
      } else {
        localStorage.setItem('mediaGlobalSettings', JSON.stringify(globalSettings.value))
        localStorage.setItem('mediaFolderSettings', JSON.stringify(Array.from(folderSettings.value.entries())))
      }
    } catch (e) {
      console.error('Failed to save media settings:', e)
    }
  }

  // Auto-save on changes
  watch([globalSettings, folderSettings], saveSettings, { deep: true })

  /**
   * Get global settings
   */
  function getGlobalSettings(): SimpleSettings {
    return { ...globalSettings.value }
  }

  /**
   * Update global settings
   */
  function setGlobalSettings(settings: Partial<SimpleSettings>): void {
    globalSettings.value = { ...globalSettings.value, ...settings }
  }

  /**
   * Get settings for a folder (returns folder-specific or global)
   */
  function getFolderSettings(folderPath: string): SimpleSettings {
    const folder = folderSettings.value.get(folderPath)
    return folder ? { ...folder.settings } : { ...globalSettings.value }
  }

  /**
   * Set custom settings for a folder
   */
  function setFolderSettings(folderPath: string, name: string, settings: SimpleSettings): void {
    folderSettings.value.set(folderPath, {
      path: folderPath,
      name,
      settings,
      createdAt: Date.now()
    })
  }

  /**
   * Remove folder-specific settings (revert to global)
   */
  function removeFolderSettings(folderPath: string): void {
    folderSettings.value.delete(folderPath)
  }

  /**
   * Check if folder has custom settings
   */
  function hasFolderSettings(folderPath: string): boolean {
    return folderSettings.value.has(folderPath)
  }

  /**
   * Get all folders with custom settings
   */
  function getAllFolderSettings(): FolderSettings[] {
    return Array.from(folderSettings.value.values())
  }

  /**
   * Reset all settings to defaults
   */
  function resetToDefaults(): void {
    globalSettings.value = { compress: true, encrypt: true }
    folderSettings.value.clear()
  }

  return {
    // State
    globalSettings,
    folderSettings,
    initialized,

    // Lifecycle
    initialize,

    // Global settings
    getGlobalSettings,
    setGlobalSettings,

    // Folder settings
    getFolderSettings,
    setFolderSettings,
    removeFolderSettings,
    hasFolderSettings,
    getAllFolderSettings,

    // Utils
    resetToDefaults
  }
}

// ============================================================================
// Backend Settings Converter
// ============================================================================

/**
 * Convert simple user settings to backend processing settings.
 * Automatically selects the best compression algorithm and encryption method.
 */
export function toBackendSettings(
  settings: SimpleSettings,
  publicBundle: PublicBundle | null,
  password?: string
): {
  compression: {
    enabled: boolean
    algorithm: string
    level: number
    prefer_speed: boolean
    min_size_threshold: number
    skip_already_compressed: boolean
  }
  encryption: {
    enabled: boolean
    use_password: boolean
    use_keypair: boolean
  }
} {
  return {
    compression: {
      enabled: settings.compress,
      // Auto-select best algorithm: Zstd for best balance
      algorithm: 'zstd',
      level: 3, // Good balance of speed and ratio
      prefer_speed: false,
      min_size_threshold: 1024, // Don't compress tiny files
      skip_already_compressed: true // Skip JPG, PNG, MP4, etc.
    },
    encryption: {
      enabled: settings.encrypt,
      // Prefer keypair (post-quantum) if available, otherwise password
      use_keypair: settings.encrypt && publicBundle !== null,
      use_password: settings.encrypt && publicBundle === null && !!password
    }
  }
}
