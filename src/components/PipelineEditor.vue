<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { usePipeline, type PipelineConfig, type PipelineLayer, type PipelineOperation } from '../composables/usePipeline'
import { useCrypto } from '../composables/useCrypto'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', pipeline: PipelineConfig): void
}>()

const {
  pipelines,
  presets,
  activePipeline,
  initialize,
  createPipeline,
  clonePreset,
  deletePipeline,
  setActivePipeline,
  addLayer,
  removeLayer,
  updateLayer,
  reorderLayers,
  estimatePipeline,
  createCompressOperation,
  createPasswordEncryptOperation,
  createHybridPQEncryptOperation,
  createHashOperation,
  createBase64Operation,
  getOperationLabel,
  getOperationIcon
} = usePipeline()

const { publicBundle, hasKeypair, isUnlocked } = useCrypto()

const activeTab = ref<'pipelines' | 'presets'>('pipelines')
const editingPipeline = ref<PipelineConfig | null>(null)
const newPipelineName = ref('')
const showAddLayer = ref(false)
const selectedLayerType = ref<string>('compress')
const estimate = ref<any>(null)

// Compression settings for new layer
const compressAlgorithm = ref('zstd')
const compressLevel = ref(3)

const compressionAlgorithms = [
  { value: 'zstd', label: 'Zstandard', levels: '1-22', default: 3 },
  { value: 'lz4', label: 'LZ4', levels: '1', default: 1 },
  { value: 'snap', label: 'Snappy', levels: '1', default: 1 },
  { value: 'brotli', label: 'Brotli', levels: '0-11', default: 6 },
  { value: 'gzip', label: 'Gzip', levels: '0-9', default: 6 },
]

const layerTypes = [
  { value: 'compress', label: 'Compression', icon: '📦', description: 'Reduce file size' },
  { value: 'encrypt_password', label: 'Password Encryption', icon: '🔐', description: 'Protect with password' },
  { value: 'encrypt_hybrid_pq', label: 'Post-Quantum Encryption', icon: '🛡️', description: 'Future-proof encryption' },
  { value: 'hash', label: 'Integrity Hash', icon: '#️⃣', description: 'BLAKE3 checksum' },
  { value: 'base64_encode', label: 'Base64 Encode', icon: '📝', description: 'Text-safe encoding' },
]

onMounted(async () => {
  await initialize()
  if (activePipeline.value) {
    editingPipeline.value = activePipeline.value
    await updateEstimate()
  }
})

watch(editingPipeline, async () => {
  if (editingPipeline.value) {
    await updateEstimate()
  }
})

async function updateEstimate() {
  if (!editingPipeline.value) return
  try {
    estimate.value = await estimatePipeline(1024 * 1024, editingPipeline.value) // 1MB sample
  } catch (e) {
    console.error('Failed to estimate:', e)
  }
}

function handleCreatePipeline() {
  if (!newPipelineName.value.trim()) return
  const pipeline = createPipeline(newPipelineName.value.trim())
  editingPipeline.value = pipeline
  setActivePipeline(pipeline.id)
  newPipelineName.value = ''
}

function handleClonePreset(presetId: string) {
  const pipeline = clonePreset(presetId)
  if (pipeline) {
    editingPipeline.value = pipeline
    setActivePipeline(pipeline.id)
    activeTab.value = 'pipelines'
  }
}

function handleSelectPipeline(pipeline: PipelineConfig) {
  editingPipeline.value = pipeline
  setActivePipeline(pipeline.id)
}

function handleDeletePipeline(pipelineId: string) {
  if (confirm('Delete this pipeline?')) {
    deletePipeline(pipelineId)
    if (editingPipeline.value?.id === pipelineId) {
      editingPipeline.value = pipelines.value[0] || null
    }
  }
}

function handleAddLayer() {
  if (!editingPipeline.value) return
  
  let operation: PipelineOperation
  
  switch (selectedLayerType.value) {
    case 'compress':
      operation = createCompressOperation(compressAlgorithm.value, compressLevel.value)
      break
    case 'encrypt_password':
      operation = createPasswordEncryptOperation()
      break
    case 'encrypt_hybrid_pq':
      operation = createHybridPQEncryptOperation(publicBundle.value)
      break
    case 'hash':
      operation = createHashOperation()
      break
    case 'base64_encode':
      operation = createBase64Operation()
      break
    default:
      return
  }
  
  addLayer(editingPipeline.value.id, operation)
  showAddLayer.value = false
  updateEstimate()
}

function handleRemoveLayer(layerId: string) {
  if (!editingPipeline.value) return
  removeLayer(editingPipeline.value.id, layerId)
  updateEstimate()
}

function handleToggleLayer(layer: PipelineLayer) {
  if (!editingPipeline.value) return
  updateLayer(editingPipeline.value.id, layer.id, { enabled: !layer.enabled })
  updateEstimate()
}

function handleMoveLayer(layerId: string, direction: 'up' | 'down') {
  if (!editingPipeline.value) return
  
  const layers = [...editingPipeline.value.layers]
  const index = layers.findIndex(l => l.id === layerId)
  if (index === -1) return
  
  const newIndex = direction === 'up' ? index - 1 : index + 1
  if (newIndex < 0 || newIndex >= layers.length) return
  
  const layerIds = layers.map(l => l.id)
  ;[layerIds[index], layerIds[newIndex]] = [layerIds[newIndex], layerIds[index]]
  
  reorderLayers(editingPipeline.value.id, layerIds)
  updateEstimate()
}

function handleApply() {
  if (editingPipeline.value) {
    emit('select', editingPipeline.value)
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function formatRatio(ratio: number): string {
  if (ratio >= 1) return `+${((ratio - 1) * 100).toFixed(0)}%`
  return `-${((1 - ratio) * 100).toFixed(0)}%`
}
</script>

<template>
  <div class="pipeline-editor">
    <div class="editor-header">
      <h2>🔧 Processing Pipeline</h2>
      <button class="close-btn" @click="emit('close')">✕</button>
    </div>

    <div class="editor-tabs">
      <button 
        :class="{ active: activeTab === 'pipelines' }"
        @click="activeTab = 'pipelines'"
      >
        My Pipelines ({{ pipelines.length }})
      </button>
      <button 
        :class="{ active: activeTab === 'presets' }"
        @click="activeTab = 'presets'"
      >
        Presets ({{ presets.length }})
      </button>
    </div>

    <div class="editor-content">
      <!-- Pipeline List -->
      <div class="pipeline-list">
        <template v-if="activeTab === 'pipelines'">
          <div class="create-pipeline">
            <input 
              v-model="newPipelineName"
              placeholder="New pipeline name..."
              @keyup.enter="handleCreatePipeline"
            />
            <button @click="handleCreatePipeline" :disabled="!newPipelineName.trim()">
              + Create
            </button>
          </div>
          
          <div 
            v-for="pipeline in pipelines" 
            :key="pipeline.id"
            class="pipeline-item"
            :class="{ active: editingPipeline?.id === pipeline.id }"
            @click="handleSelectPipeline(pipeline)"
          >
            <div class="pipeline-info">
              <span class="pipeline-name">{{ pipeline.name }}</span>
              <span class="pipeline-layers">{{ pipeline.layers.length }} layers</span>
            </div>
            <button 
              class="delete-btn"
              @click.stop="handleDeletePipeline(pipeline.id)"
            >
              🗑️
            </button>
          </div>
          
          <div v-if="pipelines.length === 0" class="empty-state">
            No pipelines yet. Create one or use a preset.
          </div>
        </template>

        <template v-else>
          <div 
            v-for="preset in presets" 
            :key="preset.id"
            class="pipeline-item preset"
            @click="handleClonePreset(preset.id)"
          >
            <div class="pipeline-info">
              <span class="pipeline-name">{{ preset.name }}</span>
              <span class="pipeline-desc">{{ preset.description }}</span>
            </div>
            <button class="use-btn">Use →</button>
          </div>
        </template>
      </div>

      <!-- Layer Editor -->
      <div class="layer-editor" v-if="editingPipeline">
        <div class="editor-title">
          <h3>{{ editingPipeline.name }}</h3>
          <p>{{ editingPipeline.description || 'Configure processing layers' }}</p>
        </div>

        <div class="layers-container">
          <div class="layers-header">
            <span>Processing Layers</span>
            <button class="add-layer-btn" @click="showAddLayer = true">
              + Add Layer
            </button>
          </div>

          <div class="layers-list">
            <div 
              v-for="(layer, index) in editingPipeline.layers" 
              :key="layer.id"
              class="layer-item"
              :class="{ disabled: !layer.enabled }"
            >
              <div class="layer-order">{{ index + 1 }}</div>
              <div class="layer-icon">{{ getOperationIcon(layer.operation) }}</div>
              <div class="layer-info">
                <span class="layer-label">{{ getOperationLabel(layer.operation) }}</span>
                <span class="layer-type">{{ layer.operation.type }}</span>
              </div>
              <div class="layer-actions">
                <button 
                  @click="handleMoveLayer(layer.id, 'up')"
                  :disabled="index === 0"
                  title="Move up"
                >↑</button>
                <button 
                  @click="handleMoveLayer(layer.id, 'down')"
                  :disabled="index === editingPipeline.layers.length - 1"
                  title="Move down"
                >↓</button>
                <button 
                  @click="handleToggleLayer(layer)"
                  :title="layer.enabled ? 'Disable' : 'Enable'"
                >
                  {{ layer.enabled ? '✓' : '○' }}
                </button>
                <button 
                  @click="handleRemoveLayer(layer.id)"
                  class="remove-btn"
                  title="Remove"
                >✕</button>
              </div>
            </div>

            <div v-if="editingPipeline.layers.length === 0" class="empty-layers">
              No layers configured. Add layers to build your pipeline.
            </div>
          </div>

          <!-- Estimate -->
          <div class="estimate-panel" v-if="estimate">
            <h4>📊 Estimated Results (1MB sample)</h4>
            <div class="estimate-stats">
              <div class="stat">
                <span class="stat-label">Final Size</span>
                <span class="stat-value">{{ formatSize(estimate.estimated_final_size) }}</span>
              </div>
              <div class="stat">
                <span class="stat-label">Ratio</span>
                <span class="stat-value" :class="{ positive: estimate.overall_ratio < 1 }">
                  {{ formatRatio(estimate.overall_ratio) }}
                </span>
              </div>
            </div>
            <div class="estimate-breakdown">
              <div v-for="op in estimate.operations" :key="op.operation" class="breakdown-item">
                <span>{{ op.operation }}</span>
                <span>→ {{ formatSize(op.estimated_size_after) }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="editor-actions">
          <button class="apply-btn" @click="handleApply" :disabled="editingPipeline.layers.length === 0">
            ✓ Apply Pipeline
          </button>
        </div>
      </div>

      <div v-else class="no-selection">
        <p>Select a pipeline to edit or create a new one</p>
      </div>
    </div>

    <!-- Add Layer Modal -->
    <div v-if="showAddLayer" class="modal-overlay" @click.self="showAddLayer = false">
      <div class="add-layer-modal">
        <h3>Add Processing Layer</h3>
        
        <div class="layer-type-selector">
          <div 
            v-for="type in layerTypes" 
            :key="type.value"
            class="layer-type-option"
            :class="{ selected: selectedLayerType === type.value }"
            @click="selectedLayerType = type.value"
          >
            <span class="type-icon">{{ type.icon }}</span>
            <span class="type-label">{{ type.label }}</span>
            <span class="type-desc">{{ type.description }}</span>
          </div>
        </div>

        <!-- Compression Options -->
        <div v-if="selectedLayerType === 'compress'" class="layer-options">
          <label>
            Algorithm
            <select v-model="compressAlgorithm">
              <option v-for="alg in compressionAlgorithms" :key="alg.value" :value="alg.value">
                {{ alg.label }} (levels {{ alg.levels }})
              </option>
            </select>
          </label>
          <label>
            Level
            <input 
              type="range" 
              v-model.number="compressLevel" 
              :min="1" 
              :max="compressAlgorithm === 'zstd' ? 22 : compressAlgorithm === 'brotli' ? 11 : 9"
            />
            <span>{{ compressLevel }}</span>
          </label>
        </div>

        <!-- PQ Encryption Warning -->
        <div v-if="selectedLayerType === 'encrypt_hybrid_pq'" class="layer-options">
          <div v-if="!hasKeypair" class="warning">
            ⚠️ You need to generate a keypair first to use PQ encryption.
          </div>
          <div v-else-if="!isUnlocked" class="warning">
            🔒 Unlock your keypair to enable PQ encryption.
          </div>
          <div v-else class="success">
            ✓ Your public key will be used for encryption.
          </div>
        </div>

        <div class="modal-actions">
          <button @click="showAddLayer = false">Cancel</button>
          <button 
            class="primary" 
            @click="handleAddLayer"
            :disabled="selectedLayerType === 'encrypt_hybrid_pq' && (!hasKeypair || !isUnlocked)"
          >
            Add Layer
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pipeline-editor {
  background: var(--bg-secondary, #1a1a2e);
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  max-height: 80vh;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, #333);
}

.editor-header h2 {
  margin: 0;
  font-size: 1.2rem;
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  opacity: 0.7;
}

.close-btn:hover {
  opacity: 1;
}

.editor-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color, #333);
}

.editor-tabs button {
  flex: 1;
  padding: 12px;
  background: none;
  border: none;
  cursor: pointer;
  opacity: 0.6;
  transition: all 0.2s;
}

.editor-tabs button.active {
  opacity: 1;
  border-bottom: 2px solid var(--accent-color, #00ff88);
}

.editor-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.pipeline-list {
  width: 280px;
  border-right: 1px solid var(--border-color, #333);
  overflow-y: auto;
  padding: 12px;
}

.create-pipeline {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.create-pipeline input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  background: var(--bg-primary, #0f0f1a);
}

.create-pipeline button {
  padding: 8px 12px;
  background: var(--accent-color, #00ff88);
  color: #000;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.create-pipeline button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pipeline-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  margin-bottom: 8px;
  background: var(--bg-primary, #0f0f1a);
  transition: all 0.2s;
}

.pipeline-item:hover {
  background: var(--bg-hover, #252540);
}

.pipeline-item.active {
  border: 1px solid var(--accent-color, #00ff88);
}

.pipeline-item.preset {
  border: 1px dashed var(--border-color, #333);
}

.pipeline-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pipeline-name {
  font-weight: 500;
}

.pipeline-layers,
.pipeline-desc {
  font-size: 0.8rem;
  opacity: 0.6;
}

.delete-btn,
.use-btn {
  padding: 4px 8px;
  background: none;
  border: none;
  cursor: pointer;
  opacity: 0.6;
}

.delete-btn:hover {
  opacity: 1;
  color: #ff4444;
}

.use-btn {
  color: var(--accent-color, #00ff88);
}

.empty-state {
  text-align: center;
  padding: 24px;
  opacity: 0.6;
}

.layer-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px;
  overflow-y: auto;
}

.editor-title h3 {
  margin: 0 0 4px 0;
}

.editor-title p {
  margin: 0;
  opacity: 0.6;
  font-size: 0.9rem;
}

.layers-container {
  flex: 1;
  margin-top: 16px;
}

.layers-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.add-layer-btn {
  padding: 6px 12px;
  background: var(--accent-color, #00ff88);
  color: #000;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
}

.layers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.layer-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  transition: all 0.2s;
}

.layer-item.disabled {
  opacity: 0.5;
}

.layer-order {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-color, #00ff88);
  color: #000;
  border-radius: 50%;
  font-size: 0.8rem;
  font-weight: bold;
}

.layer-icon {
  font-size: 1.2rem;
}

.layer-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.layer-label {
  font-weight: 500;
}

.layer-type {
  font-size: 0.75rem;
  opacity: 0.5;
}

.layer-actions {
  display: flex;
  gap: 4px;
}

.layer-actions button {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
}

.layer-actions button:hover {
  background: var(--bg-hover, #252540);
}

.layer-actions button:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.layer-actions .remove-btn:hover {
  background: #ff4444;
  border-color: #ff4444;
}

.empty-layers {
  text-align: center;
  padding: 32px;
  opacity: 0.5;
  border: 2px dashed var(--border-color, #333);
  border-radius: 8px;
}

.estimate-panel {
  margin-top: 20px;
  padding: 16px;
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
}

.estimate-panel h4 {
  margin: 0 0 12px 0;
  font-size: 0.9rem;
}

.estimate-stats {
  display: flex;
  gap: 24px;
  margin-bottom: 12px;
}

.stat {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 0.75rem;
  opacity: 0.6;
}

.stat-value {
  font-size: 1.2rem;
  font-weight: bold;
}

.stat-value.positive {
  color: var(--accent-color, #00ff88);
}

.estimate-breakdown {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 0.8rem;
  opacity: 0.7;
}

.breakdown-item {
  display: flex;
  justify-content: space-between;
}

.editor-actions {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color, #333);
}

.apply-btn {
  width: 100%;
  padding: 12px;
  background: var(--accent-color, #00ff88);
  color: #000;
  border: none;
  border-radius: 8px;
  font-weight: bold;
  cursor: pointer;
}

.apply-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.no-selection {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.5;
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.add-layer-modal {
  background: var(--bg-secondary, #1a1a2e);
  border-radius: 12px;
  padding: 24px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow-y: auto;
}

.add-layer-modal h3 {
  margin: 0 0 16px 0;
}

.layer-type-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.layer-type-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-primary, #0f0f1a);
  border: 2px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.layer-type-option:hover {
  background: var(--bg-hover, #252540);
}

.layer-type-option.selected {
  border-color: var(--accent-color, #00ff88);
}

.type-icon {
  font-size: 1.5rem;
}

.type-label {
  font-weight: 500;
}

.type-desc {
  margin-left: auto;
  font-size: 0.8rem;
  opacity: 0.6;
}

.layer-options {
  padding: 16px;
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  margin-bottom: 16px;
}

.layer-options label {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.layer-options select,
.layer-options input[type="range"] {
  flex: 1;
}

.layer-options .warning {
  color: #ffaa00;
}

.layer-options .success {
  color: var(--accent-color, #00ff88);
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.modal-actions button {
  padding: 10px 20px;
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  background: none;
  cursor: pointer;
}

.modal-actions button.primary {
  background: var(--accent-color, #00ff88);
  color: #000;
  border: none;
}

.modal-actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
