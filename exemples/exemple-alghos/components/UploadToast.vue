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
  bottom: 1.5rem;
  right: 1.5rem;
  width: 320px;
  background: rgba(17, 17, 19, 0.95);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.75rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.05);
  z-index: 1000;
  overflow: hidden;
}

.upload-toast.has-failures {
  border-color: rgba(239, 68, 68, 0.3);
}

/* Header */
.toast-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.875rem 1rem;
  cursor: pointer;
  transition: background 0.2s;
}

.toast-header:hover {
  background: rgba(255, 255, 255, 0.03);
}

.toast-icon {
  width: 1.5rem;
  height: 1.5rem;
  color: var(--accent-color, #6366f1);
  flex-shrink: 0;
}

.toast-icon svg {
  width: 100%;
  height: 100%;
}

.toast-icon .spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.has-failures .toast-icon {
  color: #ef4444;
}

.toast-info {
  flex: 1;
  min-width: 0;
}

.toast-summary {
  font-size: 0.8125rem;
  font-weight: 500;
  color: #fafafa;
  display: block;
  margin-bottom: 0.375rem;
}

.progress-bar {
  height: 3px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-color, #6366f1);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.has-failures .progress-fill {
  background: #ef4444;
}

.toast-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.progress-text {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--accent-color, #818cf8);
  min-width: 2.5rem;
  text-align: right;
}

.expand-btn {
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #71717a;
  cursor: pointer;
  transition: all 0.2s;
}

.expand-btn:hover {
  color: #fafafa;
}

.expand-btn.rotated {
  transform: rotate(180deg);
}

.expand-btn svg {
  width: 1rem;
  height: 1rem;
}

/* File List */
.toast-files {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  max-height: 280px;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 1rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.file-item:last-child {
  border-bottom: none;
}

.file-item.status-failed {
  background: rgba(239, 68, 68, 0.08);
}

.file-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: #71717a;
  flex-shrink: 0;
}

.file-icon svg {
  width: 100%;
  height: 100%;
}

.file-icon.status-uploading,
.file-icon.status-downloading {
  color: var(--accent-color, #818cf8);
}

.file-icon.status-completed {
  color: #22c55e;
}

.file-icon.status-failed {
  color: #ef4444;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 0.75rem;
  color: #e4e4e7;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 0.25rem;
}

.file-error {
  font-size: 0.6875rem;
  color: #fca5a5;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-progress {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.file-progress-bar {
  flex: 1;
  height: 2px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 1px;
  overflow: hidden;
}

.file-progress-fill {
  height: 100%;
  background: var(--accent-color, #6366f1);
  border-radius: 1px;
  transition: width 0.2s ease;
}

.file-percent {
  font-size: 0.625rem;
  color: #71717a;
  min-width: 2rem;
  text-align: right;
}

.file-actions {
  display: flex;
  gap: 0.25rem;
}

.retry-btn,
.remove-btn {
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border: none;
  border-radius: 0.25rem;
  color: #71717a;
  cursor: pointer;
  transition: all 0.2s;
}

.retry-btn:hover {
  background: rgba(var(--accent-rgb, 99, 102, 241), 0.2);
  color: var(--accent-color, #818cf8);
}

.remove-btn:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.retry-btn svg,
.remove-btn svg {
  width: 0.875rem;
  height: 0.875rem;
}

.clear-all-btn {
  width: 100%;
  padding: 0.625rem;
  background: transparent;
  border: none;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  color: #71717a;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-all-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fafafa;
}

/* Transitions */
.toast-slide-enter-active {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.toast-slide-leave-active {
  transition: all 0.2s ease;
}

.toast-slide-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

.toast-slide-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}

.expand-enter-active,
.expand-leave-active {
  transition: all 0.2s ease;
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
