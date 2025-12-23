/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { computed } from 'vue'
import { useUploadToast, type FileTransfer } from '../composables/useUploadToast'

const { 
  transfers, 
  isExpanded, 
  isVisible, 
  hasFailures,
  allCompleted,
  getSummaryText, 
  toggleExpanded,
  clearAll,
  retryTransfer,
  removeTransfer
} = useUploadToast()

const overallProgress = computed(() => {
  if (transfers.value.length === 0) return 0
  const total = transfers.value.reduce((sum, t) => sum + t.progress, 0)
  return Math.round(total / transfers.value.length)
})

function getStatusClass(transfer: FileTransfer): string {
  return `status-${transfer.status}`
}
</script>

<template>
  <Transition name="toast-slide">
    <div v-if="isVisible" class="upload-toast" :class="{ expanded: isExpanded, 'has-failures': hasFailures }">
      <!-- Summary Header -->
      <div class="toast-header" @click="toggleExpanded">
        <div class="toast-icon">
          <svg v-if="!allCompleted" class="spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
            <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
          </svg>
          <svg v-else-if="hasFailures" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 8v4M12 16h.01" />
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 6L9 17l-5-5" />
          </svg>
        </div>
        
        <div class="toast-info">
          <span class="toast-summary">{{ getSummaryText() }}</span>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${overallProgress}%` }" />
          </div>
        </div>
        
        <div class="toast-actions">
          <span class="progress-text">{{ overallProgress }}%</span>
          <button class="expand-btn" :class="{ rotated: isExpanded }">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M6 9l6 6 6-6" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Expanded File List -->
      <Transition name="expand">
        <div v-if="isExpanded" class="toast-files">
          <div 
            v-for="transfer in transfers" 
            :key="transfer.id" 
            class="file-item"
            :class="getStatusClass(transfer)"
          >
            <div class="file-icon" :class="getStatusClass(transfer)">
              <!-- Upload icon -->
              <svg v-if="transfer.type === 'upload' && transfer.status !== 'completed' && transfer.status !== 'failed'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17,8 12,3 7,8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              <!-- Download icon -->
              <svg v-else-if="transfer.type === 'download' && transfer.status !== 'completed' && transfer.status !== 'failed'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7,10 12,15 17,10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              <!-- Check icon -->
              <svg v-else-if="transfer.status === 'completed'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20 6L9 17l-5-5" />
              </svg>
              <!-- Error icon -->
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 8v4M12 16h.01" />
              </svg>
            </div>
            
            <div class="file-info">
              <span class="file-name">{{ transfer.fileName }}</span>
              <span v-if="transfer.error" class="file-error">{{ transfer.error }}</span>
              <div v-else class="file-progress">
                <div class="file-progress-bar">
                  <div class="file-progress-fill" :style="{ width: `${transfer.progress}%` }" />
                </div>
                <span class="file-percent">{{ transfer.progress }}%</span>
              </div>
            </div>
            
            <div class="file-actions">
              <button v-if="transfer.status === 'failed'" class="retry-btn" @click.stop="retryTransfer(transfer.id)" title="Retry">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M1 4v6h6M23 20v-6h-6" />
                  <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15" />
                </svg>
              </button>
              <button v-if="transfer.status === 'completed' || transfer.status === 'failed'" class="remove-btn" @click.stop="removeTransfer(transfer.id)" title="Remove">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          
          <!-- Clear All Button -->
          <button v-if="allCompleted" class="clear-all-btn" @click.stop="clearAll">
            Clear All
          </button>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>

.upload-toast {
  position: fixed;
  bottom: 24px;
  right: 24px;
  width: 340px;
  background: var(--systemStandardUltrathickMaterialSover);
  backdrop-filter: blur(40px) saturate(180%);
  -webkit-backdrop-filter: blur(40px) saturate(180%);
  border-radius: var(--global-border-radius-large);
  box-shadow: 
    0 0 0 0.5px rgba(0, 0, 0, 0.1),
    0 10px 40px rgba(0, 0, 0, 0.25);
  z-index: 1000;
  overflow: hidden;
}

.upload-toast.has-failures {
  box-shadow: 
    0 0 0 0.5px rgba(255, 59, 48, 0.3),
    0 10px 40px rgba(0, 0, 0, 0.25);
}

.toast-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  cursor: pointer;
  transition: background 0.15s;
}

.toast-header:hover {
  background: var(--systemQuinary);
}

.toast-icon {
  width: 24px;
  height: 24px;
  color: var(--keyColor);
  flex-shrink: 0;
}

.toast-icon svg {
  width: 100%;
  height: 100%;
}

.toast-icon .spinner {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.has-failures .toast-icon {
  color: var(--systemRed);
}

.toast-info {
  flex: 1;
  min-width: 0;
}

.toast-summary {
  font-size: 15px;
  font-weight: 600;
  color: var(--systemPrimary);
  display: block;
  margin-bottom: 6px;
}

.progress-bar {
  height: 4px;
  background: var(--systemGray5);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--keyColor);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.has-failures .progress-fill {
  background: var(--systemRed);
}

.toast-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-text {
  font-size: 13px;
  font-weight: 600;
  color: var(--keyColor);
  min-width: 40px;
  text-align: right;
}

.expand-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--systemTertiary);
  cursor: pointer;
  border-radius: 50%;
  transition: all 0.2s;
}

.expand-btn:hover {
  background: var(--systemQuinary);
  color: var(--systemPrimary);
}

.expand-btn.rotated {
  transform: rotate(180deg);
}

.expand-btn svg {
  width: 16px;
  height: 16px;
}

.toast-files {
  border-top: 0.5px solid var(--labelDivider);
  max-height: 280px;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-bottom: 0.5px solid var(--labelDivider);
}

.file-item:last-child {
  border-bottom: none;
}

.file-item.status-failed {
  background: rgba(255, 59, 48, 0.08);
}

.file-icon {
  width: 20px;
  height: 20px;
  color: var(--systemTertiary);
  flex-shrink: 0;
}

.file-icon svg {
  width: 100%;
  height: 100%;
}

.file-icon.status-uploading,
.file-icon.status-downloading {
  color: var(--keyColor);
}

.file-icon.status-completed {
  color: var(--systemGreen);
}

.file-icon.status-failed {
  color: var(--systemRed);
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  color: var(--systemPrimary);
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.file-error {
  font-size: 11px;
  color: var(--systemRed);
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-progress {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-progress-bar {
  flex: 1;
  height: 3px;
  background: var(--systemGray5);
  border-radius: 1.5px;
  overflow: hidden;
}

.file-progress-fill {
  height: 100%;
  background: var(--keyColor);
  border-radius: 1.5px;
  transition: width 0.2s ease;
}

.file-percent {
  font-size: 11px;
  font-weight: 500;
  color: var(--systemSecondary);
  min-width: 32px;
  text-align: right;
}

.file-actions {
  display: flex;
  gap: 4px;
}

.retry-btn,
.remove-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.15s;
}

.retry-btn {
  color: var(--keyColor);
}

.retry-btn:hover {
  background: rgba(var(--keyColor-rgb), 0.12);
}

.remove-btn {
  color: var(--systemTertiary);
}

.remove-btn:hover {
  background: var(--systemQuinary);
  color: var(--systemPrimary);
}

.retry-btn svg,
.remove-btn svg {
  width: 14px;
  height: 14px;
}

.clear-all-btn {
  width: 100%;
  padding: 12px;
  background: transparent;
  border: none;
  border-top: 0.5px solid var(--labelDivider);
  color: var(--keyColor);
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}

.clear-all-btn:hover {
  background: var(--systemQuinary);
}

.toast-slide-enter-active,
.toast-slide-leave-active {
  transition: all 0.3s cubic-bezier(0.32, 0.72, 0, 1);
}

.toast-slide-enter-from,
.toast-slide-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

.expand-enter-active,
.expand-leave-active {
  transition: all 0.25s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
}

.expand-enter-to,
.expand-leave-from {
  max-height: 280px;
}
</style>