/**
 * Vue Component - 2 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: PipelineEditor, ConfirmDialog
 */

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useCrypto } from '../composables/useCrypto'
import { useCompression } from '../composables/useCompression'
import { usePipeline } from '../composables/usePipeline'
import PipelineEditor from './PipelineEditor.vue'
import ConfirmDialog from './ui/ConfirmDialog.vue'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  hasStoredKeypair,
  isUnlocked,
  publicBundle,
  cryptoInfo,
  initialize: initCrypto,
  generateKeypair,
  saveKeypair,
  unlockKeypair,
  lockKeypair,
  deleteKeypair
} = useCrypto()

const {
  availableAlgorithms,
  initialize: initCompression,
  getAlgorithmInfo
} = useCompression()

const {
  pipelines,
  activePipeline,
  initialize: initPipeline
} = usePipeline()

const activeTab = ref<'keypair' | 'compression' | 'pipeline'>('keypair')
const showPipelineEditor = ref(false)
const showDeleteConfirm = ref(false)

const keypairPassword = ref('')
const confirmPassword = ref('')
const unlockPassword = ref('')
const generating = ref(false)
const unlocking = ref(false)
const error = ref('')
const success = ref('')

onMounted(async () => {
  await Promise.all([
    initCrypto(),
    initCompression(),
    initPipeline()
  ])
})

async function handleGenerateKeypair() {
  if (!keypairPassword.value || keypairPassword.value !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }
  
  if (keypairPassword.value.length < 12) {
    error.value = 'Password must be at least 12 characters'
    return
  }
  
  generating.value = true
  error.value = ''
  
  try {
    await generateKeypair()
    await saveKeypair(keypairPassword.value)
    success.value = 'Keypair generated and saved successfully!'
    keypairPassword.value = ''
    confirmPassword.value = ''
  } catch (e) {
    error.value = String(e)
  } finally {
    generating.value = false
  }
}

async function handleUnlockKeypair() {
  if (!unlockPassword.value) {
    error.value = 'Please enter your password'
    return
  }
  
  unlocking.value = true
  error.value = ''
  
  try {
    const success = await unlockKeypair(unlockPassword.value)
    if (success) {
      unlockPassword.value = ''
    } else {
      error.value = 'Invalid password'
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    unlocking.value = false
  }
}

function handleLockKeypair() {
  lockKeypair()
  success.value = 'Keypair locked'
}

async function confirmDeleteKeypair() {
  showDeleteConfirm.value = false
  await deleteKeypair()
  success.value = 'Keypair deleted'
}

function copyPublicBundle() {
  if (publicBundle.value) {
    navigator.clipboard.writeText(JSON.stringify(publicBundle.value))
    success.value = 'Public bundle copied to clipboard!'
  }
}

function formatBytes(arr: number[]): string {
  return `${arr.length} bytes`
}
</script>

<template>
  <div class="security-overlay" @click.self="emit('close')">
    <div class="security-settings">
      <div class="settings-header">
        <h2>🔐 Security & Processing</h2>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>

    <div class="settings-tabs">
      <button 
        :class="{ active: activeTab === 'keypair' }"
        @click="activeTab = 'keypair'"
      >
        🔑 Keypair
      </button>
      <button 
        :class="{ active: activeTab === 'compression' }"
        @click="activeTab = 'compression'"
      >
        📦 Compression
      </button>
      <button 
        :class="{ active: activeTab === 'pipeline' }"
        @click="activeTab = 'pipeline'"
      >
        🔧 Pipeline
      </button>
    </div>

    <div class="settings-content">
      <!-- Keypair Tab -->
      <div v-if="activeTab === 'keypair'" class="tab-content">
        <div class="crypto-info" v-if="cryptoInfo">
          <h3>🛡️ Post-Quantum Cryptography</h3>
          <div class="info-grid">
            <div class="info-item">
              <span class="label">Key Exchange</span>
              <span class="value">{{ cryptoInfo.key_exchange }}</span>
            </div>
            <div class="info-item">
              <span class="label">Signatures</span>
              <span class="value">{{ cryptoInfo.signatures }}</span>
            </div>
            <div class="info-item">
              <span class="label">Symmetric</span>
              <span class="value">{{ cryptoInfo.symmetric }}</span>
            </div>
            <div class="info-item">
              <span class="label">Security Level</span>
              <span class="value highlight">{{ cryptoInfo.pq_security_level }}</span>
            </div>
          </div>
        </div>

        <div class="keypair-section">
          <!-- No keypair yet -->
          <div v-if="!hasStoredKeypair" class="keypair-create">
            <h3>Generate New Keypair</h3>
            <p>Create a post-quantum secure keypair for encryption and signing.</p>
            
            <div class="form-group">
              <label>Password</label>
              <input 
                type="password" 
                v-model="keypairPassword"
                placeholder="Enter a strong password"
              />
            </div>
            
            <div class="form-group">
              <label>Confirm Password</label>
              <input 
                type="password" 
                v-model="confirmPassword"
                placeholder="Confirm your password"
              />
            </div>
            
            <button 
              class="primary-btn"
              @click="handleGenerateKeypair"
              :disabled="generating || !keypairPassword || keypairPassword !== confirmPassword"
            >
              {{ generating ? 'Generating...' : '🔑 Generate Keypair' }}
            </button>
          </div>

          <!-- Has keypair but locked -->
          <div v-else-if="!isUnlocked" class="keypair-unlock">
            <h3>🔒 Keypair Locked</h3>
            <p>Enter your password to unlock your keypair.</p>
            
            <div class="form-group">
              <label>Password</label>
              <input 
                type="password" 
                v-model="unlockPassword"
                placeholder="Enter your password"
                @keyup.enter="handleUnlockKeypair"
              />
            </div>
            
            <div class="button-group">
              <button 
                class="primary-btn"
                @click="handleUnlockKeypair"
                :disabled="unlocking || !unlockPassword"
              >
                {{ unlocking ? 'Unlocking...' : '🔓 Unlock' }}
              </button>
              <button 
                class="danger-btn"
                @click="showDeleteConfirm = true"
              >
                🗑️ Delete Keypair
              </button>
            </div>
          </div>

          <!-- Keypair unlocked -->
          <div v-else class="keypair-unlocked">
            <h3>✅ Keypair Unlocked</h3>
            
            <div class="public-bundle">
              <h4>Your Public Bundle</h4>
              <p>Share this with others so they can encrypt data for you.</p>
              
              <div class="bundle-info" v-if="publicBundle">
                <div class="bundle-item">
                  <span class="label">PQ Key (ML-KEM-1024)</span>
                  <span class="value">{{ formatBytes(publicBundle.pq_encap) }}</span>
                </div>
                <div class="bundle-item">
                  <span class="label">X25519 Key</span>
                  <span class="value">{{ formatBytes(publicBundle.x25519) }}</span>
                </div>
                <div class="bundle-item">
                  <span class="label">Signing Key (ML-DSA-65)</span>
                  <span class="value">{{ formatBytes(publicBundle.pq_verify) }}</span>
                </div>
              </div>
              
              <button class="secondary-btn" @click="copyPublicBundle">
                📋 Copy Public Bundle
              </button>
            </div>
            
            <div class="button-group">
              <button class="secondary-btn" @click="handleLockKeypair">
                🔒 Lock Keypair
              </button>
              <button class="danger-btn" @click="showDeleteConfirm = true">
                🗑️ Delete Keypair
              </button>
            </div>
          </div>
        </div>

        <div v-if="error" class="error-message">{{ error }}</div>
        <div v-if="success" class="success-message">{{ success }}</div>
      </div>

      <!-- Compression Tab -->
      <div v-if="activeTab === 'compression'" class="tab-content">
        <h3>📦 Compression Algorithms</h3>
        <p>Available compression methods for your data.</p>
        
        <div class="algorithm-list">
          <div 
            v-for="alg in availableAlgorithms" 
            :key="alg"
            class="algorithm-card"
          >
            <div class="alg-header">
              <span class="alg-name">{{ getAlgorithmInfo(alg as any).name }}</span>
              <span class="alg-id">{{ alg }}</span>
            </div>
            <p class="alg-desc">{{ getAlgorithmInfo(alg as any).description }}</p>
            <div class="alg-stats">
              <span class="stat">
                <span class="stat-label">Speed</span>
                <span class="stat-value">{{ getAlgorithmInfo(alg as any).speed }}</span>
              </span>
              <span class="stat">
                <span class="stat-label">Ratio</span>
                <span class="stat-value">{{ getAlgorithmInfo(alg as any).ratio }}</span>
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Pipeline Tab -->
      <div v-if="activeTab === 'pipeline'" class="tab-content">
        <h3>🔧 Processing Pipelines</h3>
        <p>Configure layered compression and encryption for your files.</p>
        
        <div class="pipeline-summary" v-if="activePipeline">
          <div class="active-pipeline">
            <span class="label">Active Pipeline</span>
            <span class="name">{{ activePipeline.name }}</span>
            <span class="layers">{{ activePipeline.layers.length }} layers</span>
          </div>
        </div>
        
        <div class="pipeline-list-preview">
          <div 
            v-for="pipeline in pipelines.slice(0, 3)" 
            :key="pipeline.id"
            class="pipeline-preview"
          >
            <span class="name">{{ pipeline.name }}</span>
            <span class="layers">{{ pipeline.layers.length }} layers</span>
          </div>
          <div v-if="pipelines.length > 3" class="more-pipelines">
            +{{ pipelines.length - 3 }} more
          </div>
        </div>
        
        <button class="primary-btn" @click="showPipelineEditor = true">
          🔧 Open Pipeline Editor
        </button>
      </div>
    </div>

    <!-- Pipeline Editor Modal -->
    <div v-if="showPipelineEditor" class="modal-overlay" @click.self="showPipelineEditor = false">
      <PipelineEditor 
        @close="showPipelineEditor = false"
        @select="showPipelineEditor = false"
      />
    </div>

    <!-- Delete Keypair Confirm -->
    <ConfirmDialog
      v-if="showDeleteConfirm"
      title="Delete Keypair?"
      message="This action cannot be undone. Your encrypted data will become inaccessible."
      confirm-text="Delete"
      variant="danger"
      @confirm="confirmDeleteKeypair"
      @cancel="showDeleteConfirm = false"
    />
    </div>
  </div>
</template>

<style scoped>
.security-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1rem;
}

.security-settings {
  width: 100%;
  max-width: 600px;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  overflow: hidden;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, #333);
}

.settings-header h2 {
  margin: 0;
  font-size: 1.2rem;
}

.close-btn {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary, #fafafa);
}

.close-btn svg {
  width: 1.25rem;
  height: 1.25rem;
}

.settings-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color, #333);
}

.settings-tabs button {
  flex: 1;
  padding: 12px;
  background: none;
  border: none;
  cursor: pointer;
  opacity: 0.6;
  transition: all 0.2s;
}

.settings-tabs button.active {
  opacity: 1;
  border-bottom: 2px solid var(--accent-color, #00ff88);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.tab-content h3 {
  margin: 0 0 8px 0;
}

.tab-content > p {
  margin: 0 0 20px 0;
  opacity: 0.7;
}

.crypto-info {
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.crypto-info h3 {
  margin-bottom: 12px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-item .label {
  font-size: 0.75rem;
  opacity: 0.6;
}

.info-item .value {
  font-size: 0.85rem;
}

.info-item .value.highlight {
  color: var(--accent-color, #00ff88);
  font-weight: bold;
}

.keypair-section {
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  padding: 20px;
}

.keypair-section h3 {
  margin-bottom: 8px;
}

.keypair-section > p {
  margin-bottom: 16px;
  opacity: 0.7;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 0.85rem;
  opacity: 0.8;
}

.form-group input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  background: var(--bg-secondary, #1a1a2e);
}

.primary-btn {
  width: 100%;
  padding: 12px;
  background: var(--accent-color, #00ff88);
  color: #000;
  border: none;
  border-radius: 8px;
  font-weight: bold;
  cursor: pointer;
}

.primary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.secondary-btn {
  padding: 10px 16px;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  cursor: pointer;
}

.danger-btn {
  padding: 10px 16px;
  background: transparent;
  border: 1px solid #ff4444;
  color: #ff4444;
  border-radius: 6px;
  cursor: pointer;
}

.button-group {
  display: flex;
  gap: 12px;
  margin-top: 16px;
}

.public-bundle {
  background: var(--bg-secondary, #1a1a2e);
  border-radius: 8px;
  padding: 16px;
  margin: 16px 0;
}

.public-bundle h4 {
  margin: 0 0 8px 0;
}

.public-bundle > p {
  margin: 0 0 12px 0;
  font-size: 0.85rem;
  opacity: 0.7;
}

.bundle-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.bundle-item {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
}

.bundle-item .label {
  opacity: 0.6;
}

.error-message {
  margin-top: 16px;
  padding: 12px;
  background: rgba(255, 68, 68, 0.1);
  border: 1px solid #ff4444;
  border-radius: 6px;
  color: #ff4444;
}

.success-message {
  margin-top: 16px;
  padding: 12px;
  background: rgba(0, 255, 136, 0.1);
  border: 1px solid var(--accent-color, #00ff88);
  border-radius: 6px;
  color: var(--accent-color, #00ff88);
}

.algorithm-list {
  display: grid;
  gap: 12px;
}

.algorithm-card {
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  padding: 16px;
}

.alg-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.alg-name {
  font-weight: bold;
}

.alg-id {
  font-size: 0.75rem;
  opacity: 0.5;
  font-family: monospace;
}

.alg-desc {
  margin: 0 0 12px 0;
  font-size: 0.85rem;
  opacity: 0.7;
}

.alg-stats {
  display: flex;
  gap: 24px;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 0.7rem;
  opacity: 0.5;
}

.stat-value {
  font-size: 0.85rem;
  color: var(--accent-color, #00ff88);
}

.pipeline-summary {
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.active-pipeline {
  display: flex;
  align-items: center;
  gap: 12px;
}

.active-pipeline .label {
  font-size: 0.75rem;
  opacity: 0.5;
}

.active-pipeline .name {
  font-weight: bold;
  flex: 1;
}

.active-pipeline .layers {
  font-size: 0.85rem;
  opacity: 0.6;
}

.pipeline-list-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.pipeline-preview {
  display: flex;
  justify-content: space-between;
  padding: 12px;
  background: var(--bg-primary, #0f0f1a);
  border-radius: 6px;
}

.pipeline-preview .name {
  font-weight: 500;
}

.pipeline-preview .layers {
  font-size: 0.85rem;
  opacity: 0.6;
}

.more-pipelines {
  text-align: center;
  font-size: 0.85rem;
  opacity: 0.5;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-overlay > * {
  width: 100%;
  max-width: 900px;
}
</style>