<script setup lang="ts">
/**
 * Media Settings Panel
 * 
 * Allows users to configure compression and encryption settings
 * per media type (photo, video, document) and per album.
 */
import { ref, onMounted, computed } from 'vue'
import { useMediaSettings, type MediaType, type ProcessingSettings, type CompressionAlgorithm } from '../composables/useMediaSettings'
import { useCrypto } from '../composables/useCrypto'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  globalSettings,
  albumSettings,
  initialized,
  initialize,
  updateMediaTypeSettings,
  getMediaTypeSettings,
  resetToDefaults,
  exportSettings,
  importSettings,
  DEFAULT_COMPRESSION,
  DEFAULT_ENCRYPTION
} = useMediaSettings()

const { publicBundle, isUnlocked } = useCrypto()

const activeTab = ref<MediaType>('photo')
const showExportDialog = ref(false)
const showImportDialog = ref(false)
const importJson = ref('')
const importError = ref('')

const compressionAlgorithms: { id: CompressionAlgorithm; name: string; description: string }[] = [
  { id: 'zstd', name: 'Zstandard', description: 'Best balance of speed and ratio' },
  { id: 'lz4', name: 'LZ4', description: 'Fastest compression' },
  { id: 'brotli', name: 'Brotli', description: 'Best ratio for text' },
  { id: 'gzip', name: 'Gzip', description: 'Universal compatibility' },
  { id: 'snap', name: 'Snappy', description: 'Fast with good ratio' },
  { id: 'none', name: 'None', description: 'No compression' }
]

const mediaTypes: { id: MediaType; name: string; icon: string }[] = [
  { id: 'photo', name: 'Photos', icon: '📷' },
  { id: 'video', name: 'Videos', icon: '🎬' },
  { id: 'document', name: 'Documents', icon: '📄' },
  { id: 'other', name: 'Other', icon: '📁' }
]

const currentSettings = computed(() => getMediaTypeSettings(activeTab.value))

onMounted(async () => {
  await initialize()
})

function updateCompression(key: string, value: unknown) {
  const current = getMediaTypeSettings(activeTab.value)
  updateMediaTypeSettings(activeTab.value, {
    compression: { ...current.compression, [key]: value }
  })
}

function updateEncryption(key: string, value: unknown) {
  const current = getMediaTypeSettings(activeTab.value)
  updateMediaTypeSettings(activeTab.value, {
    encryption: { ...current.encryption, [key]: value }
  })
}

function handleExport() {
  const json = exportSettings()
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `image-settings-${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)
  showExportDialog.value = false
}

function handleImport() {
  importError.value = ''
  if (!importJson.value.trim()) {
    importError.value = 'Please paste settings JSON'
    return
  }
  
  const success = importSettings(importJson.value)
  if (success) {
    showImportDialog.value = false
    importJson.value = ''
  } else {
    importError.value = 'Invalid settings format'
  }
}

function handleReset() {
  if (confirm('Reset all settings to defaults? This cannot be undone.')) {
    resetToDefaults()
  }
}
</script>

<template>
  <div class="media-settings-overlay" @click.self="emit('close')">
    <div class="media-settings-panel">
      <div class="panel-header">
        <h2>⚙️ Media Processing Settings</h2>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>

      <div class="panel-tabs">
        <button
          v-for="type in mediaTypes"
          :key="type.id"
          :class="{ active: activeTab === type.id }"
          @click="activeTab = type.id"
        >
          <span class="tab-icon">{{ type.icon }}</span>
          <span class="tab-name">{{ type.name }}</span>
        </button>
      </div>

      <div class="panel-content" v-if="initialized">
        <!-- Compression Section -->
        <section class="settings-section">
          <div class="section-header">
            <h3>📦 Compression</h3>
            <label class="toggle-switch">
              <input
                type="checkbox"
                :checked="currentSettings.compression.enabled"
                @change="updateCompression('enabled', ($event.target as HTMLInputElement).checked)"
              />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div v-if="currentSettings.compression.enabled" class="section-content">
            <div class="form-group">
              <label>Algorithm</label>
              <select
                :value="currentSettings.compression.algorithm"
                @change="updateCompression('algorithm', ($event.target as HTMLSelectElement).value)"
              >
                <option v-for="alg in compressionAlgorithms" :key="alg.id" :value="alg.id">
                  {{ alg.name }} - {{ alg.description }}
                </option>
              </select>
            </div>

            <div class="form-group">
              <label>Level ({{ currentSettings.compression.level }})</label>
              <input
                type="range"
                min="1"
                max="22"
                :value="currentSettings.compression.level"
                @input="updateCompression('level', parseInt(($event.target as HTMLInputElement).value))"
              />
              <div class="range-labels">
                <span>Fast</span>
                <span>Balanced</span>
                <span>Best</span>
              </div>
            </div>

            <div class="form-group checkbox-group">
              <label>
                <input
                  type="checkbox"
                  :checked="currentSettings.compression.preferSpeed"
                  @change="updateCompression('preferSpeed', ($event.target as HTMLInputElement).checked)"
                />
                Prefer speed over ratio
              </label>
            </div>

            <div class="form-group checkbox-group">
              <label>
                <input
                  type="checkbox"
                  :checked="currentSettings.compression.skipAlreadyCompressed"
                  @change="updateCompression('skipAlreadyCompressed', ($event.target as HTMLInputElement).checked)"
                />
                Skip already compressed files (JPG, PNG, MP4, etc.)
              </label>
            </div>

            <div class="form-group">
              <label>Minimum size threshold</label>
              <div class="input-with-unit">
                <input
                  type="number"
                  min="0"
                  :value="currentSettings.compression.minSizeThreshold"
                  @input="updateCompression('minSizeThreshold', parseInt(($event.target as HTMLInputElement).value) || 0)"
                />
                <span class="unit">bytes</span>
              </div>
              <p class="hint">Files smaller than this won't be compressed</p>
            </div>
          </div>
        </section>

        <!-- Encryption Section -->
        <section class="settings-section">
          <div class="section-header">
            <h3>🔐 Encryption</h3>
            <label class="toggle-switch">
              <input
                type="checkbox"
                :checked="currentSettings.encryption.enabled"
                @change="updateEncryption('enabled', ($event.target as HTMLInputElement).checked)"
              />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div v-if="currentSettings.encryption.enabled" class="section-content">
            <div class="encryption-methods">
              <label class="method-option" :class="{ active: currentSettings.encryption.useKeypair }">
                <input
                  type="radio"
                  name="encryption-method"
                  :checked="currentSettings.encryption.useKeypair"
                  @change="() => { updateEncryption('useKeypair', true); updateEncryption('usePassword', false) }"
                />
                <div class="method-info">
                  <span class="method-icon">🛡️</span>
                  <div>
                    <strong>Post-Quantum Encryption</strong>
                    <p>ML-KEM-1024 + X25519 hybrid (recommended)</p>
                  </div>
                </div>
              </label>

              <label class="method-option" :class="{ active: currentSettings.encryption.usePassword }">
                <input
                  type="radio"
                  name="encryption-method"
                  :checked="currentSettings.encryption.usePassword"
                  @change="() => { updateEncryption('usePassword', true); updateEncryption('useKeypair', false) }"
                />
                <div class="method-info">
                  <span class="method-icon">🔑</span>
                  <div>
                    <strong>Password Encryption</strong>
                    <p>Argon2id + ChaCha20-Poly1305</p>
                  </div>
                </div>
              </label>
            </div>

            <div v-if="currentSettings.encryption.useKeypair && !isUnlocked" class="warning-box">
              ⚠️ Keypair not unlocked. Go to Security Settings to unlock your keypair.
            </div>

            <div v-if="currentSettings.encryption.useKeypair && isUnlocked" class="success-box">
              ✅ Keypair ready for encryption
            </div>
          </div>
        </section>

        <!-- Album Settings Summary -->
        <section class="settings-section">
          <div class="section-header">
            <h3>📁 Album Overrides</h3>
          </div>
          <div class="section-content">
            <p class="hint">
              {{ albumSettings.size }} album(s) with custom settings
            </p>
            <p class="hint">
              Configure per-album settings when viewing an album.
            </p>
          </div>
        </section>
      </div>

      <div class="panel-footer">
        <button class="btn-secondary" @click="showExportDialog = true">
          📤 Export
        </button>
        <button class="btn-secondary" @click="showImportDialog = true">
          📥 Import
        </button>
        <button class="btn-danger" @click="handleReset">
          🔄 Reset
        </button>
      </div>

      <!-- Export Dialog -->
      <div v-if="showExportDialog" class="dialog-overlay" @click.self="showExportDialog = false">
        <div class="dialog">
          <h3>Export Settings</h3>
          <p>Download your media processing settings as a JSON file.</p>
          <div class="dialog-actions">
            <button class="btn-secondary" @click="showExportDialog = false">Cancel</button>
            <button class="btn-primary" @click="handleExport">Download</button>
          </div>
        </div>
      </div>

      <!-- Import Dialog -->
      <div v-if="showImportDialog" class="dialog-overlay" @click.self="showImportDialog = false">
        <div class="dialog">
          <h3>Import Settings</h3>
          <p>Paste your settings JSON below:</p>
          <textarea v-model="importJson" placeholder="Paste JSON here..."></textarea>
          <p v-if="importError" class="error">{{ importError }}</p>
          <div class="dialog-actions">
            <button class="btn-secondary" @click="showImportDialog = false">Cancel</button>
            <button class="btn-primary" @click="handleImport">Import</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.media-settings-overlay {
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

.media-settings-panel {
  width: 100%;
  max-width: 600px;
  max-height: 90vh;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, #333);
}

.panel-header h2 {
  margin: 0;
  font-size: 1.1rem;
}

.close-btn {
  width: 32px;
  height: 32px;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  font-size: 1.5rem;
  cursor: pointer;
  border-radius: 6px;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.panel-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color, #333);
  overflow-x: auto;
}

.panel-tabs button {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 8px;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  cursor: pointer;
  transition: all 0.2s;
}

.panel-tabs button:hover {
  background: rgba(255, 255, 255, 0.05);
}

.panel-tabs button.active {
  color: var(--accent-color, #00ff88);
  border-bottom: 2px solid var(--accent-color, #00ff88);
}

.tab-icon {
  font-size: 1.2rem;
}

.tab-name {
  font-size: 0.75rem;
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.settings-section {
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.02);
}

.section-header h3 {
  margin: 0;
  font-size: 0.9rem;
}

.section-content {
  padding: 16px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 0.85rem;
  color: var(--text-secondary, #a1a1aa);
}

.form-group select,
.form-group input[type="number"] {
  width: 100%;
  padding: 10px 12px;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  color: inherit;
  font-size: 0.9rem;
}

.form-group input[type="range"] {
  width: 100%;
  margin: 8px 0;
}

.range-labels {
  display: flex;
  justify-content: space-between;
  font-size: 0.7rem;
  color: var(--text-secondary, #a1a1aa);
}

.checkbox-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.checkbox-group input[type="checkbox"] {
  width: 16px;
  height: 16px;
}

.input-with-unit {
  display: flex;
  gap: 8px;
  align-items: center;
}

.input-with-unit input {
  flex: 1;
}

.unit {
  font-size: 0.85rem;
  color: var(--text-secondary, #a1a1aa);
}

.hint {
  margin: 4px 0 0;
  font-size: 0.75rem;
  color: var(--text-secondary, #a1a1aa);
}

.toggle-switch {
  position: relative;
  width: 44px;
  height: 24px;
  cursor: pointer;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  transition: all 0.2s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 18px;
  height: 18px;
  left: 2px;
  top: 2px;
  background: var(--text-secondary, #a1a1aa);
  border-radius: 50%;
  transition: all 0.2s;
}

.toggle-switch input:checked + .toggle-slider {
  background: var(--accent-color, #00ff88);
  border-color: var(--accent-color, #00ff88);
}

.toggle-switch input:checked + .toggle-slider::before {
  transform: translateX(20px);
  background: #fff;
}

.encryption-methods {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.method-option {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.method-option:hover {
  border-color: var(--accent-color, #00ff88);
}

.method-option.active {
  border-color: var(--accent-color, #00ff88);
  background: rgba(0, 255, 136, 0.05);
}

.method-option input[type="radio"] {
  margin-top: 4px;
}

.method-info {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.method-icon {
  font-size: 1.5rem;
}

.method-info strong {
  display: block;
  margin-bottom: 4px;
}

.method-info p {
  margin: 0;
  font-size: 0.8rem;
  color: var(--text-secondary, #a1a1aa);
}

.warning-box {
  padding: 12px;
  background: rgba(255, 200, 0, 0.1);
  border: 1px solid rgba(255, 200, 0, 0.3);
  border-radius: 6px;
  color: #ffc800;
  font-size: 0.85rem;
  margin-top: 12px;
}

.success-box {
  padding: 12px;
  background: rgba(0, 255, 136, 0.1);
  border: 1px solid rgba(0, 255, 136, 0.3);
  border-radius: 6px;
  color: var(--accent-color, #00ff88);
  font-size: 0.85rem;
  margin-top: 12px;
}

.panel-footer {
  display: flex;
  gap: 8px;
  padding: 16px;
  border-top: 1px solid var(--border-color, #333);
}

.btn-primary,
.btn-secondary,
.btn-danger {
  padding: 10px 16px;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: var(--accent-color, #00ff88);
  border: none;
  color: #000;
}

.btn-secondary {
  background: transparent;
  border: 1px solid var(--border-color, #333);
  color: inherit;
}

.btn-danger {
  background: transparent;
  border: 1px solid #ff4444;
  color: #ff4444;
  margin-left: auto;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.05);
}

.btn-danger:hover {
  background: rgba(255, 68, 68, 0.1);
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.dialog {
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  padding: 20px;
  width: 90%;
  max-width: 400px;
}

.dialog h3 {
  margin: 0 0 12px;
}

.dialog p {
  margin: 0 0 16px;
  color: var(--text-secondary, #a1a1aa);
}

.dialog textarea {
  width: 100%;
  height: 150px;
  padding: 12px;
  background: var(--bg-primary, #0f0f1a);
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  color: inherit;
  font-family: monospace;
  font-size: 0.85rem;
  resize: vertical;
  margin-bottom: 12px;
}

.dialog .error {
  color: #ff4444;
  font-size: 0.85rem;
}

.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
