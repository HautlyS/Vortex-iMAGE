import { ref, computed } from 'vue'

// Types
export interface PipelineOperation {
  type: 'compress' | 'encrypt_password' | 'encrypt_hybrid_pq' | 'hash' | 'base64_encode' | 'base64_decode'
  algorithm?: string
  level?: number
  public_bundle?: any
}

export interface PipelineLayer {
  id: string
  operation: PipelineOperation
  enabled: boolean
}

export interface PipelineConfig {
  id: string
  name: string
  description?: string
  layers: PipelineLayer[]
  createdAt: number
  updatedAt: number
}

export interface PipelinePreset {
  id: string
  name: string
  description: string
  layers: PipelineLayer[]
}

export interface PipelineEstimate {
  estimated_final_size: number
  overall_ratio: number
  operations: Array<{
    operation: string
    estimated_size_after: number
  }>
}

// State
const pipelines = ref<PipelineConfig[]>([])
const activePipelineId = ref<string | null>(null)
const initialized = ref(false)

// Presets
const presets = ref<PipelinePreset[]>([
  {
    id: 'preset-fast-compress',
    name: 'Fast Compression',
    description: 'Quick LZ4 compression for speed',
    layers: [
      { id: 'l1', operation: { type: 'compress', algorithm: 'lz4', level: 1 }, enabled: true }
    ]
  },
  {
    id: 'preset-max-compress',
    name: 'Maximum Compression',
    description: 'Zstd level 19 for best ratio',
    layers: [
      { id: 'l1', operation: { type: 'compress', algorithm: 'zstd', level: 19 }, enabled: true }
    ]
  },
  {
    id: 'preset-secure',
    name: 'Secure Storage',
    description: 'Compress then encrypt with password',
    layers: [
      { id: 'l1', operation: { type: 'compress', algorithm: 'zstd', level: 3 }, enabled: true },
      { id: 'l2', operation: { type: 'encrypt_password' }, enabled: true }
    ]
  },
  {
    id: 'preset-pq-secure',
    name: 'Post-Quantum Secure',
    description: 'Future-proof hybrid encryption',
    layers: [
      { id: 'l1', operation: { type: 'compress', algorithm: 'zstd', level: 3 }, enabled: true },
      { id: 'l2', operation: { type: 'encrypt_hybrid_pq' }, enabled: true },
      { id: 'l3', operation: { type: 'hash' }, enabled: true }
    ]
  },
  {
    id: 'preset-archive',
    name: 'Archive Ready',
    description: 'Compress, hash, and base64 encode',
    layers: [
      { id: 'l1', operation: { type: 'compress', algorithm: 'brotli', level: 6 }, enabled: true },
      { id: 'l2', operation: { type: 'hash' }, enabled: true },
      { id: 'l3', operation: { type: 'base64_encode' }, enabled: true }
    ]
  }
])

// Computed
const activePipeline = computed(() => {
  if (!activePipelineId.value) return null
  return pipelines.value.find(p => p.id === activePipelineId.value) || null
})

// Helper functions
function generateId(): string {
  return `pipeline-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

function generateLayerId(): string {
  return `layer-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

// Storage
const STORAGE_KEY = 'image-pipelines'
const ACTIVE_KEY = 'image-active-pipeline'

function loadFromStorage(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      pipelines.value = JSON.parse(stored)
    }
    const activeId = localStorage.getItem(ACTIVE_KEY)
    if (activeId) {
      activePipelineId.value = activeId
    }
  } catch (e) {
    console.error('Failed to load pipelines from storage:', e)
  }
}

function saveToStorage(): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(pipelines.value))
    if (activePipelineId.value) {
      localStorage.setItem(ACTIVE_KEY, activePipelineId.value)
    }
  } catch (e) {
    console.error('Failed to save pipelines to storage:', e)
  }
}

export function usePipeline() {
  // Initialize
  async function initialize(): Promise<void> {
    if (initialized.value) return
    loadFromStorage()
    initialized.value = true
  }

  // Pipeline CRUD
  function createPipeline(name: string, description?: string): PipelineConfig {
    const pipeline: PipelineConfig = {
      id: generateId(),
      name,
      description,
      layers: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    }
    pipelines.value.push(pipeline)
    saveToStorage()
    return pipeline
  }

  function clonePreset(presetId: string): PipelineConfig | null {
    const preset = presets.value.find(p => p.id === presetId)
    if (!preset) return null

    const pipeline: PipelineConfig = {
      id: generateId(),
      name: `${preset.name} (Copy)`,
      description: preset.description,
      layers: preset.layers.map(l => ({
        ...l,
        id: generateLayerId()
      })),
      createdAt: Date.now(),
      updatedAt: Date.now()
    }
    pipelines.value.push(pipeline)
    saveToStorage()
    return pipeline
  }

  function deletePipeline(pipelineId: string): void {
    const index = pipelines.value.findIndex(p => p.id === pipelineId)
    if (index !== -1) {
      pipelines.value.splice(index, 1)
      if (activePipelineId.value === pipelineId) {
        activePipelineId.value = pipelines.value[0]?.id || null
      }
      saveToStorage()
    }
  }

  function setActivePipeline(pipelineId: string): void {
    activePipelineId.value = pipelineId
    saveToStorage()
  }

  // Layer management
  function addLayer(pipelineId: string, operation: PipelineOperation): PipelineLayer | null {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return null

    const layer: PipelineLayer = {
      id: generateLayerId(),
      operation,
      enabled: true
    }
    pipeline.layers.push(layer)
    pipeline.updatedAt = Date.now()
    saveToStorage()
    return layer
  }

  function removeLayer(pipelineId: string, layerId: string): void {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return

    const index = pipeline.layers.findIndex(l => l.id === layerId)
    if (index !== -1) {
      pipeline.layers.splice(index, 1)
      pipeline.updatedAt = Date.now()
      saveToStorage()
    }
  }

  function updateLayer(pipelineId: string, layerId: string, updates: Partial<PipelineLayer>): void {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return

    const layer = pipeline.layers.find(l => l.id === layerId)
    if (layer) {
      Object.assign(layer, updates)
      pipeline.updatedAt = Date.now()
      saveToStorage()
    }
  }

  function reorderLayers(pipelineId: string, layerIds: string[]): void {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return

    const reordered: PipelineLayer[] = []
    for (const id of layerIds) {
      const layer = pipeline.layers.find(l => l.id === id)
      if (layer) reordered.push(layer)
    }
    pipeline.layers = reordered
    pipeline.updatedAt = Date.now()
    saveToStorage()
  }

  // Estimation
  async function estimatePipeline(inputSize: number, pipeline: PipelineConfig): Promise<PipelineEstimate> {
    let currentSize = inputSize
    const operations: PipelineEstimate['operations'] = []

    for (const layer of pipeline.layers) {
      if (!layer.enabled) continue

      let ratio = 1.0
      switch (layer.operation.type) {
        case 'compress':
          // Estimate compression ratio based on algorithm
          const alg = layer.operation.algorithm || 'zstd'
          const level = layer.operation.level || 3
          if (alg === 'zstd') ratio = 0.3 + (0.4 * (1 - level / 22))
          else if (alg === 'lz4') ratio = 0.5
          else if (alg === 'brotli') ratio = 0.25 + (0.35 * (1 - level / 11))
          else if (alg === 'gzip') ratio = 0.35 + (0.3 * (1 - level / 9))
          else ratio = 0.5
          break
        case 'encrypt_password':
        case 'encrypt_hybrid_pq':
          ratio = 1.02 // Small overhead for encryption
          break
        case 'hash':
          ratio = 1.0 // Hash doesn't change size (stored separately)
          break
        case 'base64_encode':
          ratio = 1.37 // Base64 increases size by ~37%
          break
        case 'base64_decode':
          ratio = 0.75
          break
      }

      currentSize = Math.round(currentSize * ratio)
      operations.push({
        operation: getOperationLabel(layer.operation),
        estimated_size_after: currentSize
      })
    }

    return {
      estimated_final_size: currentSize,
      overall_ratio: currentSize / inputSize,
      operations
    }
  }

  // Operation factories
  function createCompressOperation(algorithm: string, level: number): PipelineOperation {
    return { type: 'compress', algorithm, level }
  }

  function createPasswordEncryptOperation(): PipelineOperation {
    return { type: 'encrypt_password' }
  }

  function createHybridPQEncryptOperation(publicBundle?: any): PipelineOperation {
    return { type: 'encrypt_hybrid_pq', public_bundle: publicBundle }
  }

  function createHashOperation(): PipelineOperation {
    return { type: 'hash' }
  }

  function createBase64Operation(): PipelineOperation {
    return { type: 'base64_encode' }
  }

  // UI helpers
  function getOperationLabel(operation: PipelineOperation): string {
    switch (operation.type) {
      case 'compress':
        return `${(operation.algorithm || 'zstd').toUpperCase()} L${operation.level || 3}`
      case 'encrypt_password':
        return 'Password Encryption'
      case 'encrypt_hybrid_pq':
        return 'PQ Hybrid Encryption'
      case 'hash':
        return 'BLAKE3 Hash'
      case 'base64_encode':
        return 'Base64 Encode'
      case 'base64_decode':
        return 'Base64 Decode'
      default:
        return operation.type
    }
  }

  function getOperationIcon(operation: PipelineOperation): string {
    switch (operation.type) {
      case 'compress':
        return '📦'
      case 'encrypt_password':
        return '🔐'
      case 'encrypt_hybrid_pq':
        return '🛡️'
      case 'hash':
        return '#️⃣'
      case 'base64_encode':
      case 'base64_decode':
        return '📝'
      default:
        return '⚙️'
    }
  }

  return {
    // State
    pipelines,
    presets,
    activePipeline,
    
    // Lifecycle
    initialize,
    
    // Pipeline CRUD
    createPipeline,
    clonePreset,
    deletePipeline,
    setActivePipeline,
    
    // Layer management
    addLayer,
    removeLayer,
    updateLayer,
    reorderLayers,
    
    // Estimation
    estimatePipeline,
    
    // Operation factories
    createCompressOperation,
    createPasswordEncryptOperation,
    createHybridPQEncryptOperation,
    createHashOperation,
    createBase64Operation,
    
    // UI helpers
    getOperationLabel,
    getOperationIcon
  }
}
