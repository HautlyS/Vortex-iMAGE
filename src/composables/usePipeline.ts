/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 2 modules
 */

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'
import type { PublicBundle } from './useCrypto'

export type OperationType = 'compress' | 'encrypt_password' | 'encrypt_hybrid_pq' | 'hash' | 'base64_encode'

export interface CompressOperation {
  type: 'compress'
  algorithm: string
  level: number
}

export interface EncryptPasswordOperation {
  type: 'encrypt_password'
}

export interface EncryptHybridPQOperation {
  type: 'encrypt_hybrid_pq'
  recipient_bundle: PublicBundle | null
}

export interface HashOperation {
  type: 'hash'
}

export interface Base64EncodeOperation {
  type: 'base64_encode'
}

export type PipelineOperation = 
  | CompressOperation 
  | EncryptPasswordOperation 
  | EncryptHybridPQOperation 
  | HashOperation 
  | Base64EncodeOperation

export interface PipelineLayer {
  id: string
  operation: PipelineOperation
  enabled: boolean
  order: number
}

export interface PipelineConfig {
  id: string
  name: string
  description: string
  layers: PipelineLayer[]
  created_at: number
  updated_at: number
}

export interface LayerResult {
  layer_id: string
  operation_type: string
  input_size: number
  output_size: number
  success: boolean
  error: string | null
}

export interface PipelineResult {
  data: number[]
  original_size: number
  final_size: number
  layers_applied: LayerResult[]
  checksum: number[]
}

export interface PipelineEstimate {
  original_size: number
  estimated_final_size: number
  overall_ratio: number
  operations: {
    operation: string
    ratio: number
    estimated_size_after: number
  }[]
}

const pipelines = ref<PipelineConfig[]>([])
const activePipelineId = ref<string | null>(null)
const presets = ref<PipelineConfig[]>([])
const processing = ref(false)
let initialized = false

export function usePipeline() {
  const activePipeline = computed(() => 
    pipelines.value.find(p => p.id === activePipelineId.value) || null
  )

  async function initialize(): Promise<void> {
    if (initialized) return
    try {
      
      presets.value = await invoke<PipelineConfig[]>('pipeline_get_presets')

      const store = await load('settings.json')
      const saved = await store.get<PipelineConfig[]>('pipelines')
      if (saved) {
        pipelines.value = saved
      }
      
      const activeId = await store.get<string>('activePipelineId')
      if (activeId) {
        activePipelineId.value = activeId
      }
      
      initialized = true
    } catch (e) {
      console.error('Failed to initialize pipeline:', e)
    }
  }

  async function savePipelines(): Promise<void> {
    try {
      const store = await load('settings.json')
      await store.set('pipelines', pipelines.value)
      await store.set('activePipelineId', activePipelineId.value)
      await store.save()
    } catch (e) {
      console.error('Failed to save pipelines:', e)
    }
  }

  function createPipeline(name: string, description = ''): PipelineConfig {
    const now = Date.now()
    const pipeline: PipelineConfig = {
      id: `pipeline-${now}-${Math.random().toString(36).slice(2, 8)}`,
      name,
      description,
      layers: [],
      created_at: Math.floor(now / 1000),
      updated_at: Math.floor(now / 1000)
    }
    pipelines.value.push(pipeline)
    savePipelines()
    return pipeline
  }

  function clonePreset(presetId: string, newName?: string): PipelineConfig | null {
    const preset = presets.value.find(p => p.id === presetId)
    if (!preset) return null
    
    const now = Date.now()
    const pipeline: PipelineConfig = {
      ...JSON.parse(JSON.stringify(preset)),
      id: `pipeline-${now}-${Math.random().toString(36).slice(2, 8)}`,
      name: newName || `${preset.name} (Copy)`,
      created_at: Math.floor(now / 1000),
      updated_at: Math.floor(now / 1000)
    }
    pipelines.value.push(pipeline)
    savePipelines()
    return pipeline
  }

  function deletePipeline(pipelineId: string): void {
    const index = pipelines.value.findIndex(p => p.id === pipelineId)
    if (index !== -1) {
      pipelines.value.splice(index, 1)
      if (activePipelineId.value === pipelineId) {
        activePipelineId.value = pipelines.value[0]?.id || null
      }
      savePipelines()
    }
  }

  function setActivePipeline(pipelineId: string | null): void {
    activePipelineId.value = pipelineId
    savePipelines()
  }

  function addLayer(
    pipelineId: string,
    operation: PipelineOperation
  ): PipelineLayer | null {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return null
    
    const layer: PipelineLayer = {
      id: `layer-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      operation,
      enabled: true,
      order: pipeline.layers.length
    }
    
    pipeline.layers.push(layer)
    pipeline.updated_at = Math.floor(Date.now() / 1000)
    savePipelines()
    return layer
  }

  function removeLayer(pipelineId: string, layerId: string): void {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return
    
    const index = pipeline.layers.findIndex(l => l.id === layerId)
    if (index !== -1) {
      pipeline.layers.splice(index, 1)
      
      pipeline.layers.forEach((l, i) => l.order = i)
      pipeline.updated_at = Math.floor(Date.now() / 1000)
      savePipelines()
    }
  }

  function updateLayer(
    pipelineId: string,
    layerId: string,
    updates: Partial<PipelineLayer>
  ): void {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return
    
    const layer = pipeline.layers.find(l => l.id === layerId)
    if (layer) {
      Object.assign(layer, updates)
      pipeline.updated_at = Math.floor(Date.now() / 1000)
      savePipelines()
    }
  }

  function reorderLayers(pipelineId: string, layerIds: string[]): void {
    const pipeline = pipelines.value.find(p => p.id === pipelineId)
    if (!pipeline) return
    
    layerIds.forEach((id, index) => {
      const layer = pipeline.layers.find(l => l.id === id)
      if (layer) layer.order = index
    })
    
    pipeline.layers.sort((a, b) => a.order - b.order)
    pipeline.updated_at = Math.floor(Date.now() / 1000)
    savePipelines()
  }

  async function processPipeline(
    data: Uint8Array,
    config: PipelineConfig,
    passwords: Record<string, string> = {},
    keypairBytes?: number[]
  ): Promise<PipelineResult> {
    processing.value = true
    try {
      return await invoke<PipelineResult>('pipeline_process', {
        data: Array.from(data),
        config,
        passwords,
        keypairBytes: keypairBytes || null
      })
    } finally {
      processing.value = false
    }
  }

  async function reversePipeline(
    data: Uint8Array,
    passwords: Record<string, string> = {},
    keypairBytes?: number[]
  ): Promise<PipelineResult> {
    processing.value = true
    try {
      return await invoke<PipelineResult>('pipeline_reverse', {
        data: Array.from(data),
        passwords,
        keypairBytes: keypairBytes || null
      })
    } finally {
      processing.value = false
    }
  }

  async function validatePipeline(config: PipelineConfig): Promise<boolean> {
    try {
      return await invoke<boolean>('pipeline_validate', { config })
    } catch {
      return false
    }
  }

  async function estimatePipeline(
    originalSize: number,
    config: PipelineConfig
  ): Promise<PipelineEstimate> {
    return await invoke<PipelineEstimate>('pipeline_estimate', {
      originalSize,
      config
    })
  }

  function createCompressOperation(algorithm = 'zstd', level = 3): CompressOperation {
    return { type: 'compress', algorithm, level }
  }

  function createPasswordEncryptOperation(): EncryptPasswordOperation {
    return { type: 'encrypt_password' }
  }

  function createHybridPQEncryptOperation(recipientBundle: PublicBundle | null = null): EncryptHybridPQOperation {
    return { type: 'encrypt_hybrid_pq', recipient_bundle: recipientBundle }
  }

  function createHashOperation(): HashOperation {
    return { type: 'hash' }
  }

  function createBase64Operation(): Base64EncodeOperation {
    return { type: 'base64_encode' }
  }

  function getOperationLabel(op: PipelineOperation): string {
    switch (op.type) {
      case 'compress':
        return `Compress (${op.algorithm.toUpperCase()} L${op.level})`
      case 'encrypt_password':
        return 'Password Encryption'
      case 'encrypt_hybrid_pq':
        return 'Post-Quantum Encryption'
      case 'hash':
        return 'BLAKE3 Hash'
      case 'base64_encode':
        return 'Base64 Encode'
    }
  }

  function getOperationIcon(op: PipelineOperation): string {
    switch (op.type) {
      case 'compress':
        return '📦'
      case 'encrypt_password':
        return '🔐'
      case 'encrypt_hybrid_pq':
        return '🛡️'
      case 'hash':
        return '#️⃣'
      case 'base64_encode':
        return '📝'
    }
  }

  return {
    pipelines,
    presets,
    activePipelineId,
    activePipeline,
    processing,
    initialize,
    createPipeline,
    clonePreset,
    deletePipeline,
    setActivePipeline,
    addLayer,
    removeLayer,
    updateLayer,
    reorderLayers,
    processPipeline,
    reversePipeline,
    validatePipeline,
    estimatePipeline,
    createCompressOperation,
    createPasswordEncryptOperation,
    createHybridPQEncryptOperation,
    createHashOperation,
    createBase64Operation,
    getOperationLabel,
    getOperationIcon
  }
}