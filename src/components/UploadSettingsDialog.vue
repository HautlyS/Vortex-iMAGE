<script setup lang="ts">
/**
 * Simple Upload Settings Dialog
 * 
 * Two simple toggles: Compress and Encrypt.
 * Technology is automatic - we use the best available.
 */
import { ref, computed, onMounted } from 'vue'
import { useMediaSettings, type SimpleSettings } from '../composables/useMediaSettings'
import { useCrypto } from '../composables/useCrypto'

const props = defineProps<{
  files: string[]
  folderPath?: string
}>()

const emit = defineEmits<{
  (e: 'confirm', settings: SimpleSettings, password?: string): void
  (e: 'cancel'): void
}>()

const { getFolderSettings, initialize } = useMediaSettings()
const { isUnlocked, publicBundle } = useCrypto()

// Simple toggles
const compress = ref(true)
const encrypt = ref(true)

// Password (only needed if no keypair)
const password = ref('')
const confirmPassword = ref('')

const needsPassword = computed(() => encrypt.value && !isUnlocked.value)
const passwordsMatch = computed(() => password.value === confirmPassword.value)
const passwordValid = computed(() => !needsPassword.value || (password.value.length >= 8 && passwordsMatch.value))

const canConfirm = computed(() => {
  if (encrypt.value && !isUnlocked.value && !passwordValid.value) return false
  return true
})

// File info
const fileCount = computed(() => props.files.length)
const fileNames = computed(() => props.files.map(f => f.split('/').pop() || f).slice(0, 3))

onMounted(async () => {
  await initialize()
  // Load settings for this folder (or global)
  const settings = getFolderSettings(props.folderPath || '')
  compress.value = settings.compress
  encrypt.value = settings.encrypt
})

function handleConfirm() {
  const settings: SimpleSettings = {
    compress: compress.value,
    encrypt: encrypt.value
  }
  emit('confirm', settings, needsPassword.value ? password.value : undefined)
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('cancel')">
    <div class="upload-dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h2>📤 Upload {{ fileCount }} file{{ fileCount > 1 ? 's' : '' }}</h2>
      </div>

      <!-- File Preview -->
      <div class="file-preview">
        <div v-for="name in fileNames" :key="name" class="file-item">
          <span class="file-icon">📄</span>
          <span class="file-name">{{ name }}</span>
        </div>
        <div v-if="fileCount > 3" class="more-files">+{{ fileCount - 3 }} more</div>
      </div>

      <!-- Simple Settings -->
      <div class="settings-section">
        <!-- Compress Toggle -->
        <label class="setting-row">
          <div class="setting-info">
            <span class="setting-icon">📦</span>
            <div>
              <strong>Compress</strong>
              <p>Reduce file size to save storage space</p>
            </div>
          </div>
          <input type="checkbox" v-model="compress" class="toggle" />
        </label>

        <!-- Encrypt Toggle -->
        <label class="setting-row">
          <div class="setting-info">
            <span class="setting-icon">🔒</span>
            <div>
              <strong>Encrypt</strong>
              <p>Protect files with military-grade encryption</p>
            </div>
          </div>
          <input type="checkbox" v-model="encrypt" class="toggle" />
        </label>
      </div>

      <!-- Encryption Info -->
      <div v-if="encrypt" class="encryption-info">
        <div v-if="isUnlocked" class="info-box success">
          <span>🛡️</span>
          <div>
            <strong>Post-Quantum Protection</strong>
            <p>Your files will be encrypted with quantum-resistant cryptography using your keypair.</p>
          </div>
        </div>
        
        <div v-else class="info-box warning">
          <span>🔑</span>
          <div>
            <strong>Password Protection</strong>
            <p>No keypair found. Files will be encrypted with a password.</p>
            <p class="hint">For stronger security, set up a keypair in Security Settings.</p>
          </div>
        </div>

        <!-- Password Fields (only if no keypair) -->
        <div v-if="needsPassword" class="password-section">
          <div class="form-row">
            <label>Password (min 8 characters)</label>
            <input 
              type="password" 
              v-model="password" 
              placeholder="Enter password"
              :class="{ error: password && password.length < 8 }"
            />
          </div>
          <div class="form-row">
            <label>Confirm Password</label>
            <input 
              type="password" 
              v-model="confirmPassword" 
              placeholder="Confirm password"
              :class="{ error: confirmPassword && !passwordsMatch }"
            />
          </div>
          <p v-if="confirmPassword && !passwordsMatch" class="error-text">
            Passwords don't match
          </p>
        </div>
      </div>

      <!-- Compression Info -->
      <div v-if="compress && !encrypt" class="info-box neutral">
        <span>💡</span>
        <p>Already compressed files (JPG, PNG, MP4) will be skipped automatically.</p>
      </div>

      <!-- Footer -->
      <div class="dialog-footer">
        <button class="btn-secondary" @click="emit('cancel')">Cancel</button>
        <button class="btn-primary" :disabled="!canConfirm" @click="handleConfirm">
          Upload
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  padding: 1rem;
}

.upload-dialog {
  width: 100%;
  max-width: 420px;
  background: var(--bg-secondary, #1a1a2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 16px;
  overflow: hidden;
}

.dialog-header {
  padding: 20px 24px 16px;
  text-align: center;
}

.dialog-header h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
}

.file-preview {
  margin: 0 24px 16px;
  padding: 12px;
  background: var(--bg-primary, #0f0f1a);
  border-radius: 10px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
  font-size: 0.85rem;
}

.file-icon { opacity: 0.6; }

.file-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-secondary, #a1a1aa);
}

.more-files {
  font-size: 0.8rem;
  color: var(--text-secondary, #a1a1aa);
  padding-top: 8px;
  margin-top: 8px;
  border-top: 1px solid var(--border-color, #333);
}

.settings-section {
  padding: 0 24px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  margin-bottom: 8px;
  background: var(--bg-primary, #0f0f1a);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.setting-row:hover {
  background: rgba(255, 255, 255, 0.03);
}

.setting-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.setting-icon {
  font-size: 1.5rem;
}

.setting-info strong {
  display: block;
  margin-bottom: 2px;
}

.setting-info p {
  margin: 0;
  font-size: 0.8rem;
  color: var(--text-secondary, #a1a1aa);
}

.toggle {
  width: 48px;
  height: 26px;
  appearance: none;
  background: var(--bg-secondary, #1a1a2e);
  border: 2px solid var(--border-color, #333);
  border-radius: 13px;
  cursor: pointer;
  position: relative;
  transition: all 0.2s;
}

.toggle::before {
  content: '';
  position: absolute;
  width: 18px;
  height: 18px;
  top: 2px;
  left: 2px;
  background: var(--text-secondary, #a1a1aa);
  border-radius: 50%;
  transition: all 0.2s;
}

.toggle:checked {
  background: var(--accent-color, #00ff88);
  border-color: var(--accent-color, #00ff88);
}

.toggle:checked::before {
  transform: translateX(22px);
  background: #fff;
}

.encryption-info {
  padding: 0 24px 16px;
}

.info-box {
  display: flex;
  gap: 12px;
  padding: 14px;
  border-radius: 10px;
  font-size: 0.85rem;
  margin-bottom: 12px;
}

.info-box span {
  font-size: 1.3rem;
  flex-shrink: 0;
}

.info-box strong {
  display: block;
  margin-bottom: 4px;
}

.info-box p {
  margin: 0;
  color: var(--text-secondary, #a1a1aa);
  line-height: 1.4;
}

.info-box .hint {
  margin-top: 8px;
  font-size: 0.75rem;
  opacity: 0.8;
}

.info-box.success {
  background: rgba(0, 255, 136, 0.08);
  border: 1px solid rgba(0, 255, 136, 0.2);
}

.info-box.success strong {
  color: var(--accent-color, #00ff88);
}

.info-box.warning {
  background: rgba(255, 200, 0, 0.08);
  border: 1px solid rgba(255, 200, 0, 0.2);
}

.info-box.warning strong {
  color: #ffc800;
}

.info-box.neutral {
  background: rgba(100, 150, 255, 0.08);
  border: 1px solid rgba(100, 150, 255, 0.2);
  align-items: center;
}

.password-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-row label {
  font-size: 0.8rem;
  color: var(--text-secondary, #a1a1aa);
}

.form-row input {
  padding: 12px;
  background: var(--bg-primary, #0f0f1a);
  border: 1px solid var(--border-color, #333);
  border-radius: 8px;
  color: inherit;
  font-size: 0.9rem;
}

.form-row input:focus {
  outline: none;
  border-color: var(--accent-color, #00ff88);
}

.form-row input.error {
  border-color: #ff4444;
}

.error-text {
  font-size: 0.8rem;
  color: #ff4444;
  margin: 0;
}

.dialog-footer {
  display: flex;
  gap: 12px;
  padding: 16px 24px 24px;
}

.btn-primary, .btn-secondary {
  flex: 1;
  padding: 14px;
  border-radius: 10px;
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: var(--accent-color, #00ff88);
  border: none;
  color: #000;
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-secondary {
  background: transparent;
  border: 1px solid var(--border-color, #333);
  color: inherit;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.05);
}
</style>
