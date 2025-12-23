import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type CompressionAlgorithm = 'zstd' | 'lz4' | 'snap' | 'brotli' | 'gzip' | 'none'

export interface CompressionResult {
  data: number[]
  algorithm: CompressionAlgorithm
  original_size: number
  compressed_size: number
  ratio: number
}

export interface ItemCompressionSettings {
  enabled: boolean
  algorithm: CompressionAlgorithm
  level: number
  prefer_speed: boolean
  min_size_threshold: number
  skip_already_compressed: boolean
}

export interface CompressedFileData {
  data: number[]
  compressed: boolean
  algorithm: CompressionAlgorithm
  original_size: number
  compressed_size: number
  ratio: number
  checksum: number[]
}

export interface CompressionRecommendation {
  algorithm: string
  level: number
  reason: string
  estimated_ratio: number
}

const availableAlgorithms = ref<CompressionAlgorithm[]>([])
let initialized = false

export function useCompression() {
  async function initialize(): Promise<void> {
    if (initialized) return
    try {
      availableAlgorithms.value = await invoke<CompressionAlgorithm[]>('list_compression_algorithms')
      initialized = true
    } catch (e) {
      console.error('Failed to load compression algorithms:', e)
      availableAlgorithms.value = ['zstd', 'lz4', 'snap', 'brotli', 'gzip', 'none']
    }
  }

  async function compress(
    data: Uint8Array,
    algorithm: CompressionAlgorithm = 'zstd',
    level?: number
  ): Promise<CompressionResult> {
    return await invoke<CompressionResult>('compress_data', {
      data: Array.from(data),
      algorithm,
      level
    })
  }

  async function compressAuto(data: Uint8Array, preferSpeed = false): Promise<CompressionResult> {
    return await invoke<CompressionResult>('compress_data_auto', {
      data: Array.from(data),
      preferSpeed
    })
  }

  async function decompress(data: Uint8Array, algorithm: CompressionAlgorithm): Promise<Uint8Array> {
    const result = await invoke<number[]>('decompress_data', {
      data: Array.from(data),
      algorithm
    })
    return new Uint8Array(result)
  }

  async function estimate(data: Uint8Array, algorithm: CompressionAlgorithm = 'zstd'): Promise<CompressionResult> {
    return await invoke<CompressionResult>('estimate_compression', {
      data: Array.from(data),
      algorithm
    })
  }

  // File compression with settings
  async function compressFile(
    data: Uint8Array,
    filename: string,
    settings: ItemCompressionSettings
  ): Promise<CompressedFileData> {
    return await invoke<CompressedFileData>('compress_file', {
      data: Array.from(data),
      filename,
      settings
    })
  }

  async function decompressFile(compressed: CompressedFileData): Promise<Uint8Array> {
    const result = await invoke<number[]>('decompress_file', { compressed })
    return new Uint8Array(result)
  }

  async function getRecommendation(filename: string, fileSize: number): Promise<CompressionRecommendation> {
    return await invoke<CompressionRecommendation>('get_compression_recommendation', {
      filename,
      fileSize
    })
  }

  // Create compression settings helper
  function createCompressionSettings(options: {
    enabled?: boolean
    algorithm?: CompressionAlgorithm
    level?: number
    preferSpeed?: boolean
    minSizeThreshold?: number
    skipAlreadyCompressed?: boolean
  } = {}): ItemCompressionSettings {
    return {
      enabled: options.enabled ?? true,
      algorithm: options.algorithm ?? 'zstd',
      level: options.level ?? 3,
      prefer_speed: options.preferSpeed ?? false,
      min_size_threshold: options.minSizeThreshold ?? 1024,
      skip_already_compressed: options.skipAlreadyCompressed ?? true
    }
  }

  function formatRatio(ratio: number): string {
    if (ratio >= 1) return `${(ratio * 100).toFixed(0)}% (no savings)`
    const savings = (1 - ratio) * 100
    return `${savings.toFixed(1)}% smaller`
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  function getAlgorithmInfo(algorithm: CompressionAlgorithm): { name: string; description: string; speed: string; ratio: string } {
    const info: Record<CompressionAlgorithm, { name: string; description: string; speed: string; ratio: string }> = {
      zstd: { name: 'Zstandard', description: 'Best balance of speed and compression', speed: 'Fast', ratio: 'Excellent' },
      lz4: { name: 'LZ4', description: 'Extremely fast, lower compression', speed: 'Very Fast', ratio: 'Good' },
      snap: { name: 'Snappy', description: 'Very fast, moderate compression', speed: 'Very Fast', ratio: 'Moderate' },
      brotli: { name: 'Brotli', description: 'High compression, slower', speed: 'Slow', ratio: 'Excellent' },
      gzip: { name: 'Gzip', description: 'Universal compatibility', speed: 'Moderate', ratio: 'Good' },
      none: { name: 'None', description: 'No compression', speed: 'Instant', ratio: 'None' }
    }
    return info[algorithm]
  }

  return {
    availableAlgorithms,
    initialize,
    compress,
    compressAuto,
    decompress,
    estimate,
    compressFile,
    decompressFile,
    getRecommendation,
    createCompressionSettings,
    formatRatio,
    formatSize,
    getAlgorithmInfo
  }
}
