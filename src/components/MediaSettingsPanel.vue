<script setup lang="ts">
/**
 * Media Settings Panel
 * 
 * Simple settings for compression and encryption.
 * Users choose WHAT to protect, not HOW - we use the best technology automatically.
 */
import { ref, onMounted, computed } from 'vue'
import { useMediaSettings } from '../composables/useMediaSettings'
import { useCrypto } from '../composables/useCrypto'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  globalSettings,
  initialize,
  setGlobalSettings,
  removeFolderSettings,
  getAllFolderSettings,
  resetToDefaults
} = useMediaSettings()

const { isUnlocked, initialize: initCrypto } = useCrypto()

const showResetConfirm = ref(false)
const folders = computed(() => getAllFolderSettings())

onMounted(async () => {
  await Promise.all([initialize(), initCrypto()])
})

function toggleCompress() {
  setGlobalSettings({ compress: !globalSettings.value.compress })
}

function toggleEncrypt() {
  setGlobalSettings({ encrypt: !globalSettings.value.encrypt })
}

function handleReset() {
  resetToDefaults()
  showResetConfirm.value = false
}
</script>

<template>
  <div class="settings-overlay" @click.self="emit('close')">
    <div class="settings-panel">
      <!-- Header -->
      <div class="panel-header">
        <h2>⚙️ File Protection</h2>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>

      <div class="panel-content">
        <!-- Main Settings -->
        <section class="settings-section">
          <h3>Default Settings</h3>
          <p class="section-desc">Applied to all uploads unless overridden per folder.</p>

          <!-- Compress Toggle -->
          <div class="setting-card" @click="toggleCompress">
            <div class="setting-icon">📦</div>
            <div class="setting-info">
              <strong>Compression</strong>
              <p>Reduce file size automatically</p>
              <span class="tech-badge">Zstandard • Auto-skip compressed files</span>
            </div>
            <div class="toggle-wrapper">
              <input type="checkbox" :checked="globalSettings.compress" class="toggle" @click.stop />
            </div>
          </div>

          <!-- Encrypt Toggle -->
          <div class="setting-card" @click="toggleEncrypt">
            <div class="setting-icon">🔒</div>
            <div class="setting-info">
              <strong>Encryption</strong>
              <p>Protect files with military-grade security</p>
              <span v-if="isUnlocked" class="tech-badge success">
                🛡️ Post-Quantum • ML-KEM-1024 + X25519
              </span>
              <span v-else class="tech-badge warning">
                🔑 Password-based • Argon2id + ChaCha20
              </span>
            </div>
            <div class="toggle-wrapper">
              <input type="checkbox" :checked="globalSettings.encrypt" class="toggle" @click.stop />
            </div>
          </div>
        </section>

        <!-- Encryption Status -->
        <section class="settings-section">
          <h3>Security Status</h3>
          
          <div v-if="isUnlocked" class="status-card success">
            <span class="status-icon">✓</span>
            <div>
              <strong>Keypair Active</strong>
              <p>Your files are protected with quantum-resistant encryption. Even future quantum computers won't be able to decrypt them.</p>
            </div>
          </div>

          <div v-else class="status-card warning">
            <span class="status-icon">!</span>
            <div>
              <strong>No Keypair</strong>
              <p>Files will be encrypted with a password. For stronger protection, set up a keypair in Security Settings.</p>
            </div>
          </div>
        </section>

        <!-- Folder Overrides -->
        <section v-if="folders.length > 0" class="settings-section">
          <h3>Folder Overrides</h3>
          <p class="section-desc">These folders have custom settings.</p>

          <div v-for="folder in folders" :key="folder.path" class="folder-item">
            <div class="folder-info">
              <span class="folder-icon">📁</span>
              <span class="folder-name">{{ folder.name || folder.path }}</span>
            </div>
            <div class="folder-badges">
              <span v-if="folder.settings.compress" class="mini-badge">📦</span>
              <span v-if="folder.settings.encrypt" class="mini-badge">🔒</span>
            </div>
            <button class="remove-btn" @click="removeFolderSettings(folder.path)">×</button>
          </div>
        </section>

        <!-- How It Works -->
        <section class="settings-section info-section">
          <h3>💡 How It Works</h3>
          <div class="info-grid">
            <div class="info-item">
              <strong>Compression</strong>
              <p>Uses Zstandard (zstd) - the best balance of speed and size reduction. Already compressed files (JPG, PNG, MP4) are automatically skipped.</p>
            </div>
            <div class="info-item">
              <strong>Encryption</strong>
              <p>With a keypair: Post-quantum hybrid encryption (ML-KEM-1024 + X25519) - secure against future quantum computers.</p>
              <p>Without keypair: Password-based encryption (Argon2id + ChaCha20-Poly1305) - strong but requires remembering the password.</p>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="panel-footer">
        <button class="btn-danger" @click="showResetConfirm = true">Reset to Defaults</button>
      </div>

      <!-- Reset Confirm -->
      <div v-if="showResetConfirm" class="confirm-overlay" @click.self="showResetConfirm = false">
        <div class="confirm-dialog">
          <h3>Reset Settings?</h3>
          <p>This will reset all settings to defaults and remove folder overrides.</p>
          <div class="confirm-actions">
            <button class="btn-secondary" @click="showResetConfirm = false">Cancel</button>
            <button class="btn-danger" @click="handleReset">Reset</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1rem;
}

.settings-panel {
  width: 100%;
  max-width: 520px;
  max-height: 90vh;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color, #333);
}

.panel-header h2 {
  margin: 0;
  font-size: 1.15rem;
}

.close-btn {
  width: 32px;
  height: 32px;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  font-size: 1.5rem;
  cursor: pointer;
  border-radius: 8px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section h3 {
  margin: 0 0 8px;
  font-size: 0.9rem;
  color: var(--text-secondary, #a1a1aa);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.section-desc {
  margin: 0 0 16px;
  font-size: 0.85rem;
  color: var(--text-secondary, #a1a1aa);
}

.setting-card {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 18px;
  background: var(--bg-primary, #0f0f1a);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  margin-bottom: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.setting-card:hover {
  border-color: var(--accent-color, #00ff88);
  background: rgba(0, 255, 136, 0.02);
}

.setting-icon {
  font-size: 1.8rem;
  flex-shrink: 0;
}

.setting-info {
  flex: 1;
}

.setting-info strong {
  display: block;
  margin-bottom: 4px;
  font-size: 1rem;
}

.setting-info p {
  margin: 0 0 8px;
  font-size: 0.85rem;
  color: var(--text-secondary, #a1a1aa);
}

.tech-badge {
  display: inline-block;
  font-size: 0.7rem;
  padding: 4px 8px;
  background: rgba(100, 100, 100, 0.2);
  border-radius: 4px;
  color: var(--text-secondary, #a1a1aa);
}

.tech-badge.success {
  background: rgba(0, 255, 136, 0.1);
  color: var(--accent-color, #00ff88);
}

.tech-badge.warning {
  background: rgba(255, 200, 0, 0.1);
  color: #ffc800;
}

.toggle-wrapper {
  flex-shrink: 0;
  padding-top: 4px;
}

.toggle {
  width: 52px;
  height: 28px;
  appearance: none;
  background: var(--bg-secondary, #1a1a2e);
  border: 2px solid var(--border-color, #444);
  border-radius: 14px;
  cursor: pointer;
  position: relative;
  transition: all 0.2s;
}

.toggle::before {
  content: '';
  position: absolute;
  width: 20px;
  height: 20px;
  top: 2px;
  left: 2px;
  background: var(--text-secondary, #666);
  border-radius: 50%;
  transition: all 0.2s;
}

.toggle:checked {
  background: var(--accent-color, #00ff88);
  border-color: var(--accent-color, #00ff88);
}

.toggle:checked::before {
  transform: translateX(24px);
  background: #fff;
}

.status-card {
  display: flex;
  gap: 14px;
  padding: 16px;
  border-radius: 10px;
}

.status-card.success {
  background: rgba(0, 255, 136, 0.08);
  border: 1px solid rgba(0, 255, 136, 0.2);
}

.status-card.warning {
  background: rgba(255, 200, 0, 0.08);
  border: 1px solid rgba(255, 200, 0, 0.2);
}

.status-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-weight: bold;
  flex-shrink: 0;
}

.status-card.success .status-icon {
  background: var(--accent-color, #00ff88);
  color: #000;
}

.status-card.warning .status-icon {
  background: #ffc800;
  color: #000;
}

.status-card strong {
  display: block;
  margin-bottom: 4px;
}

.status-card p {
  margin: 0;
  font-size: 0.85rem;
  color: var(--text-secondary, #a1a1aa);
  line-height: 1.5;
}

.folder-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-primary, #0f0f1a);
  border-radius: 8px;
  margin-bottom: 8px;
}

.folder-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.folder-icon {
  font-size: 1.2rem;
}

.folder-name {
  font-size: 0.9rem;
}

.folder-badges {
  display: flex;
  gap: 4px;
}

.mini-badge {
  font-size: 0.9rem;
}

.remove-btn {
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  color: var(--text-secondary, #666);
  font-size: 1.2rem;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.remove-btn:hover {
  color: #ff6b6b;
  background: rgba(255, 107, 107, 0.1);
}

.info-section {
  background: var(--bg-primary, #0f0f1a);
  border-radius: 12px;
  padding: 16px;
}

.info-section h3 {
  margin-bottom: 12px;
}

.info-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-item strong {
  display: block;
  margin-bottom: 6px;
  font-size: 0.9rem;
}

.info-item p {
  margin: 0 0 6px;
  font-size: 0.8rem;
  color: var(--text-secondary, #a1a1aa);
  line-height: 1.5;
}

.panel-footer {
  padding: 16px 24px;
  border-top: 1px solid var(--border-color, #333);
  display: flex;
  justify-content: flex-end;
}

.btn-secondary, .btn-danger {
  padding: 10px 18px;
  border-radius: 8px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary {
  background: transparent;
  border: 1px solid var(--border-color, #333);
  color: inherit;
}

.btn-danger {
  background: transparent;
  border: 1px solid rgba(255, 100, 100, 0.5);
  color: #ff6b6b;
}

.btn-danger:hover {
  background: rgba(255, 100, 100, 0.1);
}

.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.confirm-dialog {
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  padding: 24px;
  max-width: 360px;
}

.confirm-dialog h3 {
  margin: 0 0 12px;
}

.confirm-dialog p {
  margin: 0 0 20px;
  color: var(--text-secondary, #a1a1aa);
}

.confirm-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}
</style>
