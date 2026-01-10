/**
 * Photo Uploader Component
 * 
 * Simple upload with compress/encrypt toggles.
 */

<script setup lang="ts">
import { ref } from 'vue'
import { usePhotoUpload } from '../composables/usePhotoUpload'
import type { SimpleSettings } from '../composables/useMediaSettings'
import UploadSettingsDialog from './UploadSettingsDialog.vue'

const props = defineProps<{
  folderPath?: string
  /** Skip settings dialog and use defaults */
  quickUpload?: boolean
}>()

const { 
  queue, failedCount, successCount,
  retryFailed, removeFromQueue, clearCompleted, clearAll,
  addToQueue
} = usePhotoUpload()

const showSettings = ref(false)
const pendingFiles = ref<string[]>([])

async function handleSelectFiles() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const files = await open({
      multiple: true,
      filters: [
        { name: 'Media', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif', 'heic', 'mp4', 'mkv', 'mov', 'webm'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })
    
    if (files && files.length > 0) {
      if (props.quickUpload) {
        // Quick upload with defaults
        addToQueue(files as string[], props.folderPath)
      } else {
        // Show settings dialog
        pendingFiles.value = files as string[]
        showSettings.value = true
      }
    }
  } catch (e) {
    console.error('Failed to open file dialog:', e)
  }
}

function handleConfirmUpload(settings: SimpleSettings, password?: string) {
  showSettings.value = false
  addToQueue(pendingFiles.value, props.folderPath, settings, password)
  pendingFiles.value = []
}

function handleCancelUpload() {
  showSettings.value = false
  pendingFiles.value = []
}
</script>

<template>
  <div class="uploader">
    <!-- Upload Button -->
    <button @click="handleSelectFiles" class="upload-btn">
      <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      <span>Upload Files</span>
    </button>

    <!-- Queue -->
    <Transition name="slide">
      <div v-if="queue.length" class="queue">
        <div class="queue-header">
          <span class="queue-title">Queue ({{ queue.length }})</span>
          <div class="queue-actions">
            <button v-if="failedCount" @click="retryFailed" class="queue-btn retry">↻ Retry</button>
            <button v-if="successCount" @click="clearCompleted" class="queue-btn">Clear</button>
            <button @click="clearAll" class="queue-btn close">×</button>
          </div>
        </div>

        <div class="queue-list">
          <div v-for="item in queue" :key="item.id" class="queue-item">
            <div class="queue-status">
              <div v-if="item.status === 'pending'" class="dot pending"></div>
              <div v-else-if="item.status === 'uploading'" class="spinner"></div>
              <span v-else-if="item.status === 'success'" class="status-icon success">✓</span>
              <span v-else class="status-icon error">✕</span>
            </div>
            <div class="queue-info">
              <p class="queue-name">{{ item.name }}</p>
              <div v-if="item.status === 'uploading'" class="progress-bar">
                <div class="progress-fill" :style="{ width: `${item.progress}%` }"></div>
              </div>
              <p v-if="item.error" class="queue-error">{{ item.error }}</p>
            </div>
            <button v-if="item.status !== 'uploading'" @click="removeFromQueue(item.id)" class="queue-remove">×</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Upload Settings Dialog -->
    <UploadSettingsDialog
      v-if="showSettings"
      :files="pendingFiles"
      :folder-path="folderPath"
      @confirm="handleConfirmUpload"
      @cancel="handleCancelUpload"
    />
  </div>
</template>

<style scoped>
.uploader { 
  display: flex; 
  flex-direction: column; 
  gap: 0.75rem; 
}

.upload-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.875rem;
  background: var(--surface-2);
  border: 2px dashed var(--border);
  border-radius: 10px;
  color: var(--text-2);
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.upload-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: rgba(0, 255, 136, 0.05);
}

.upload-btn svg { 
  width: 20px; 
  height: 20px; 
}

.queue {
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
}

.queue-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.625rem 0.875rem;
  border-bottom: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.2);
}

.queue-title {
  font-size: 0.75rem;
  color: var(--text-3);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 600;
}

.queue-actions { 
  display: flex; 
  gap: 0.375rem; 
}

.queue-btn {
  background: transparent;
  border: none;
  color: var(--text-3);
  font-size: 0.7rem;
  padding: 0.25rem 0.5rem;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s;
}

.queue-btn:hover { 
  color: var(--accent); 
  background: rgba(0, 255, 136, 0.1); 
}

.queue-btn.retry:hover {
  color: #ffc800;
  background: rgba(255, 200, 0, 0.1);
}

.queue-btn.close:hover { 
  color: #ff6b6b; 
  background: rgba(255, 107, 107, 0.1); 
}

.queue-list {
  max-height: 180px;
  overflow-y: auto;
}

.queue-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 0.875rem;
  border-bottom: 1px solid var(--border);
}

.queue-item:last-child { 
  border-bottom: none; 
}

.queue-status { 
  width: 18px; 
  height: 18px; 
  flex-shrink: 0; 
  display: flex; 
  align-items: center; 
  justify-content: center; 
}

.dot { 
  width: 8px; 
  height: 8px; 
  border-radius: 50%; 
}

.dot.pending { 
  background: var(--text-3); 
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--accent);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { 
  to { transform: rotate(360deg); } 
}

.status-icon { 
  font-size: 0.85rem; 
  font-weight: bold; 
}

.status-icon.success { 
  color: #10b981; 
}

.status-icon.error { 
  color: #ff6b6b; 
}

.queue-info { 
  flex: 1; 
  min-width: 0; 
}

.queue-name {
  font-size: 0.8rem;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin: 0;
}

.progress-bar {
  height: 4px;
  background: var(--surface-3);
  border-radius: 2px;
  margin-top: 0.375rem;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.15s ease;
}

.queue-error {
  font-size: 0.7rem;
  color: #ff6b6b;
  margin: 0.25rem 0 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.queue-remove {
  background: transparent;
  border: none;
  color: var(--text-3);
  font-size: 1.1rem;
  cursor: pointer;
  padding: 0.25rem;
  line-height: 1;
  border-radius: 4px;
  transition: all 0.15s;
  opacity: 0.6;
}

.queue-remove:hover { 
  color: #ff6b6b; 
  background: rgba(255, 107, 107, 0.1); 
  opacity: 1;
}

.slide-enter-active, .slide-leave-active { 
  transition: all 0.2s ease; 
}

.slide-enter-from, .slide-leave-to { 
  opacity: 0; 
  transform: translateY(-8px); 
}
</style>
