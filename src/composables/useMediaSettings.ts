/**
 * Media Settings Composable
 * 
 * Provides per-album, per-photo, and per-video compression/encryption settings.
 * Users can configure what processing applies to each type of content.
 */

import { ref, computed, watch } from 'vue'
import { load } from '@tauri-apps/plugin-store'
import type { PublicBundle } from './useCrypto'

// ============================================================================
// Types
// ============================================================================

export type MediaType = 'photo' | 'video' | 'document' | 'other'

export type CompressionAlgorithm = 'zstd' | 'lz4' | 'snap' | 'brotli' | 'gzip' | 'none'

export interface CompressionConfig {
  enabled: boolean
  algorithm: CompressionAlgorithm
  level: number
  preferSpeed: boolean
  minSizeThreshold: number
  skipAlreadyCompressed: boolean
}

export interface EncryptionConfig {
  enabled: boolean
  usePassword: boolean
  useKeypair: boolean
  recipientBundle: PublicBundle | null
}

export interface ProcessingSettings {
  compression: CompressionConfig
  encryption: EncryptionConfig
}

export interface MediaTypeSettings {
  photo: ProcessingSettings
  video: ProcessingSettings
  document: ProcessingSettings
  other: ProcessingSettings
}

export interface AlbumSettings {
  albumId: string
  albumPath: string
  name: string
  useCustomSettings: boolean
  settings: ProcessingSettings
  createdAt: number
  updatedAt: number
}

export interface ItemOverride {
  itemId: string
  itemPath: string
  useCustomSettings: boolean
  settings: ProcessingSettings
  createdAt: number
}

// ============================================================================
// Default Settings
// ============================================================================

const DEFAULT_COMPRESSION: CompressionConfig = {
  enabled: true,
  algorithm: 'zstd',
  level: 3,
  preferSpeed: false,
  minSizeThreshold: 1024,
  skipAlreadyCompressed: true
}

const DEFAULT_ENCRYPTION: EncryptionConfig = {
  enabled: true,
  usePassword: false,
  useKeypair: true,
  recipientBundle: null
}

const DEFAULT_PROCESSING: ProcessingSettings = {
  compression: { ...DEFAULT_COMPRESSION },
  encryption: { ...DEFAULT_ENCRYPTION }
}

const DEFAULT_MEDIA_SETTINGS: MediaTypeSettings = {
  photo: {
    compression: { ...DEFAULT_COMPRESSION, skipAlreadyCompressed: true },
    encryption: { ...DEFAULT_ENCRYPTION }
  },
  video: {
    compression: { ...DEFAULT_COMPRESSION, algorithm: 'lz4', preferSpeed: true, skipAlreadyCompressed: true },
    encryption: { ...DEFAULT_ENCRYPTION }
  },
  document: {
    compression: { ...DEFAULT_COMPRESSION, level: 6 },
    encryption: { ...DEFAULT_ENCRYPTION }
  },
  other: {
    compression: { ...DEFAULT_COMPRESSION },
    encryption: { ...DEFAULT_ENCRYPTION }
  }
}

// ============================================================================
// State
// ============================================================================

const globalSettings = ref<MediaTypeSettings>({ ...DEFAULT_MEDIA_SETTINGS })
const albumSettings = ref<Map<string, AlbumSettings>>(new Map())
const itemOverrides = ref<Map<string, ItemOverride>>(new Map())
const initialized = ref(false)

// ============================================================================
// Composable
// ============================================================================

export function useMediaSettings() {
  // Computed
  const hasCustomAlbumSettings = computed(() => albumSettings.value.size > 0)
  const hasItemOverrides = computed(() => itemOverrides.value.size > 0)

  /**
   * Initialize settings from storage
   */
  async function initialize(): Promise<void> {
    if (initialized.value) return

    try {
      const store = await load('media-settings.json')
      
      const savedGlobal = await store.get<MediaTypeSettings>('globalSettings')
      if (savedGlobal) {
        globalSettings.value = mergeSettings(DEFAULT_MEDIA_SETTINGS, savedGlobal)
      }

      const savedAlbums = await store.get<[string, AlbumSettings][]>('albumSettings')
      if (savedAlbums) {
        albumSettings.value = new Map(savedAlbums)
      }

      const savedOverrides = await store.get<[string, ItemOverride][]>('itemOverrides')
      if (savedOverrides) {
        itemOverrides.value = new Map(savedOverrides)
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
      const store = await load('media-settings.json')
      await store.set('globalSettings', globalSettings.value)
      await store.set('albumSettings', Array.from(albumSettings.value.entries()))
      await store.set('itemOverrides', Array.from(itemOverrides.value.entries()))
      await store.save()
    } catch (e) {
      console.error('Failed to save media settings:', e)
    }
  }

  // Auto-save on changes
  watch([globalSettings, albumSettings, itemOverrides], saveSettings, { deep: true })

  /**
   * Get settings for a specific media type
   */
  function getMediaTypeSettings(type: MediaType): ProcessingSettings {
    return globalSettings.value[type] || DEFAULT_PROCESSING
  }

  /**
   * Update global settings for a media type
   */
  function updateMediaTypeSettings(type: MediaType, settings: Partial<ProcessingSettings>): void {
    globalSettings.value[type] = {
      ...globalSettings.value[type],
      ...settings,
      compression: {
        ...globalSettings.value[type].compression,
        ...(settings.compression || {})
      },
      encryption: {
        ...globalSettings.value[type].encryption,
        ...(settings.encryption || {})
      }
    }
  }

  /**
   * Get or create album settings
   */
  function getAlbumSettings(albumPath: string): AlbumSettings | null {
    return albumSettings.value.get(albumPath) || null
  }

  /**
   * Set custom settings for an album
   */
  function setAlbumSettings(
    albumPath: string,
    name: string,
    settings: ProcessingSettings,
    useCustom = true
  ): AlbumSettings {
    const now = Date.now()
    const existing = albumSettings.value.get(albumPath)
    
    const albumSetting: AlbumSettings = {
      albumId: existing?.albumId || `album-${now}-${Math.random().toString(36).slice(2, 8)}`,
      albumPath,
      name,
      useCustomSettings: useCustom,
      settings,
      createdAt: existing?.createdAt || now,
      updatedAt: now
    }

    albumSettings.value.set(albumPath, albumSetting)
    return albumSetting
  }

  /**
   * Remove custom album settings (revert to global)
   */
  function removeAlbumSettings(albumPath: string): void {
    albumSettings.value.delete(albumPath)
  }

  /**
   * Get item-specific override
   */
  function getItemOverride(itemPath: string): ItemOverride | null {
    return itemOverrides.value.get(itemPath) || null
  }

  /**
   * Set item-specific override
   */
  function setItemOverride(
    itemPath: string,
    settings: ProcessingSettings,
    useCustom = true
  ): ItemOverride {
    const now = Date.now()
    const existing = itemOverrides.value.get(itemPath)

    const override: ItemOverride = {
      itemId: existing?.itemId || `item-${now}-${Math.random().toString(36).slice(2, 8)}`,
      itemPath,
      useCustomSettings: useCustom,
      settings,
      createdAt: existing?.createdAt || now
    }

    itemOverrides.value.set(itemPath, override)
    return override
  }

  /**
   * Remove item override
   */
  function removeItemOverride(itemPath: string): void {
    itemOverrides.value.delete(itemPath)
  }

  /**
   * Get effective settings for an item (respects hierarchy: item > album > global)
   */
  function getEffectiveSettings(
    itemPath: string,
    albumPath: string | null,
    mediaType: MediaType
  ): ProcessingSettings {
    // Check item override first
    const itemOverride = itemOverrides.value.get(itemPath)
    if (itemOverride?.useCustomSettings) {
      return itemOverride.settings
    }

    // Check album settings
    if (albumPath) {
      const album = albumSettings.value.get(albumPath)
      if (album?.useCustomSettings) {
        return album.settings
      }
    }

    // Fall back to global media type settings
    return globalSettings.value[mediaType] || DEFAULT_PROCESSING
  }

  /**
   * Detect media type from filename
   */
  function detectMediaType(filename: string): MediaType {
    const ext = filename.split('.').pop()?.toLowerCase() || ''
    
    const photoExts = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'avif', 'heic', 'heif', 'bmp', 'tiff', 'tif', 'raw']
    const videoExts = ['mp4', 'mkv', 'avi', 'mov', 'webm', 'wmv', 'flv', 'm4v', '3gp']
    const docExts = ['pdf', 'doc', 'docx', 'txt', 'md', 'json', 'xml', 'html', 'css', 'js', 'ts']

    if (photoExts.includes(ext)) return 'photo'
    if (videoExts.includes(ext)) return 'video'
    if (docExts.includes(ext)) return 'document'
    return 'other'
  }

  /**
   * Check if file is already compressed format
   */
  function isAlreadyCompressed(filename: string): boolean {
    const ext = filename.split('.').pop()?.toLowerCase() || ''
    const compressedExts = [
      'jpg', 'jpeg', 'png', 'gif', 'webp', 'avif', 'heic', 'heif',
      'mp4', 'mkv', 'avi', 'mov', 'webm',
      'mp3', 'aac', 'ogg', 'flac',
      'zip', 'gz', 'bz2', 'xz', '7z', 'rar', 'zst', 'lz4', 'br'
    ]
    return compressedExts.includes(ext)
  }

  /**
   * Get compression recommendation for a file
   */
  function getCompressionRecommendation(filename: string, fileSize: number): {
    algorithm: CompressionAlgorithm
    level: number
    reason: string
    shouldCompress: boolean
  } {
    const ext = filename.split('.').pop()?.toLowerCase() || ''
    const mediaType = detectMediaType(filename)

    if (isAlreadyCompressed(filename)) {
      return {
        algorithm: 'none',
        level: 0,
        reason: 'File is already in a compressed format',
        shouldCompress: false
      }
    }

    if (fileSize < 1024) {
      return {
        algorithm: 'none',
        level: 0,
        reason: 'File too small to benefit from compression',
        shouldCompress: false
      }
    }

    if (fileSize > 100 * 1024 * 1024) {
      return {
        algorithm: 'lz4',
        level: 1,
        reason: 'Large file - using fast compression',
        shouldCompress: true
      }
    }

    if (['txt', 'json', 'xml', 'html', 'css', 'js', 'ts', 'md'].includes(ext)) {
      return {
        algorithm: 'zstd',
        level: 6,
        reason: 'Text file - high compression ratio recommended',
        shouldCompress: true
      }
    }

    if (['bmp', 'tiff', 'tif', 'raw'].includes(ext)) {
      return {
        algorithm: 'zstd',
        level: 3,
        reason: 'Uncompressed image - good compression potential',
        shouldCompress: true
      }
    }

    return {
      algorithm: 'zstd',
      level: 3,
      reason: 'Default balanced compression',
      shouldCompress: true
    }
  }

  /**
   * Reset all settings to defaults
   */
  function resetToDefaults(): void {
    globalSettings.value = { ...DEFAULT_MEDIA_SETTINGS }
    albumSettings.value.clear()
    itemOverrides.value.clear()
  }

  /**
   * Export settings for backup
   */
  function exportSettings(): string {
    return JSON.stringify({
      globalSettings: globalSettings.value,
      albumSettings: Array.from(albumSettings.value.entries()),
      itemOverrides: Array.from(itemOverrides.value.entries()),
      exportedAt: Date.now()
    }, null, 2)
  }

  /**
   * Import settings from backup
   */
  function importSettings(json: string): boolean {
    try {
      const data = JSON.parse(json)
      if (data.globalSettings) {
        globalSettings.value = mergeSettings(DEFAULT_MEDIA_SETTINGS, data.globalSettings)
      }
      if (data.albumSettings) {
        albumSettings.value = new Map(data.albumSettings)
      }
      if (data.itemOverrides) {
        itemOverrides.value = new Map(data.itemOverrides)
      }
      return true
    } catch {
      return false
    }
  }

  return {
    // State
    globalSettings,
    albumSettings,
    itemOverrides,
    initialized,
    hasCustomAlbumSettings,
    hasItemOverrides,

    // Lifecycle
    initialize,

    // Global settings
    getMediaTypeSettings,
    updateMediaTypeSettings,

    // Album settings
    getAlbumSettings,
    setAlbumSettings,
    removeAlbumSettings,

    // Item overrides
    getItemOverride,
    setItemOverride,
    removeItemOverride,

    // Effective settings
    getEffectiveSettings,

    // Utilities
    detectMediaType,
    isAlreadyCompressed,
    getCompressionRecommendation,
    resetToDefaults,
    exportSettings,
    importSettings,

    // Constants
    DEFAULT_COMPRESSION,
    DEFAULT_ENCRYPTION,
    DEFAULT_PROCESSING
  }
}

// ============================================================================
// Helpers
// ============================================================================

function mergeSettings(defaults: MediaTypeSettings, saved: Partial<MediaTypeSettings>): MediaTypeSettings {
  return {
    photo: { ...defaults.photo, ...saved.photo },
    video: { ...defaults.video, ...saved.video },
    document: { ...defaults.document, ...saved.document },
    other: { ...defaults.other, ...saved.other }
  }
}
