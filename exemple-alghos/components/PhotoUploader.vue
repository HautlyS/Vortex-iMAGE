<script setup lang="ts">
import { usePhotoUpload } from '../composables/usePhotoUpload'

const { 
  queue, failedCount, successCount,
  selectFiles, retryFailed, removeFromQueue, clearCompleted, clearAll 
} = usePhotoUpload()
</script>

<template>
  <div class="uploader">
    <!-- Upload Button -->
    <button @click="selectFiles" class="upload-btn">
      <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      <span>Select Photos</span>
    </button>

    <!-- Queue -->
    <Transition name="slide">
      <div v-if="queue.length" class="queue">
        <div class="queue-header">
          <span class="queue-title">Queue ({{ queue.length }})</span>
          <div class="queue-actions">
            <button v-if="failedCount" @click="retryFailed" class="queue-btn">Retry</button>
            <button v-if="successCount" @click="clearCompleted" class="queue-btn">Clear</button>
            <button @click="clearAll" class="queue-btn">×</button>
          </div>
        </div>

        <div class="queue-list">
          <div v-for="item in queue" :key="item.id" class="queue-item">
            <div class="queue-status">
              <div v-if="item.status === 'pending'" class="dot gray"></div>
              <div v-else-if="item.status === 'uploading'" class="spinner"></div>
              <svg v-else-if="item.status === 'success'" class="icon success" viewBox="0 0 24 24" fill="none" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/></svg>
              <svg v-else class="icon error" viewBox="0 0 24 24" fill="none" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12"/></svg>
            </div>
            <div class="queue-info">
              <p class="queue-name">{{ item.name }}</p>
              <div v-if="item.status === 'uploading'" class="progress-bar">
                <div class="progress-fill" :style="{ width: `${item.progress}%` }"></div>
              </div>
            </div>
            <button v-if="item.status !== 'uploading'" @click="removeFromQueue(item.id)" class="queue-remove">×</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.uploader { display: flex; flex-direction: column; gap: 0.75rem; }

.upload-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.75rem;
  background: var(--surface-2);
  border: 1px dashed var(--border);
  border-radius: 8px;
  color: var(--text-2);
  font-size: 0.8rem;
  cursor: pointer;
  transition: all var(--duration) var(--ease);
}

.upload-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: rgba(34,211,238,0.05);
}

.upload-btn svg { width: 18px; height: 18px; }

.queue {
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}

.queue-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--border);
}

.queue-title {
  font-size: 0.7rem;
  color: var(--text-3);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.queue-actions { display: flex; gap: 0.25rem; }

.queue-btn {
  background: transparent;
  border: none;
  color: var(--text-3);
  font-size: 0.7rem;
  padding: 0.25rem 0.5rem;
  cursor: pointer;
  border-radius: 4px;
  transition: all var(--duration) var(--ease);
}

.queue-btn:hover { color: var(--accent); background: rgba(34,211,238,0.1); }

.queue-list {
  max-height: 150px;
  overflow-y: auto;
}

.queue-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--border);
}

.queue-item:last-child { border-bottom: none; }

.queue-status { width: 16px; height: 16px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; }

.dot { width: 6px; height: 6px; border-radius: 50%; }
.dot.gray { background: var(--text-3); }

.spinner {
  width: 12px;
  height: 12px;
  border: 2px solid var(--accent);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.icon { width: 14px; height: 14px; }
.icon.success { color: #10b981; }
.icon.error { color: #f87171; }

.queue-info { flex: 1; min-width: 0; }

.queue-name {
  font-size: 0.75rem;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.progress-bar {
  height: 3px;
  background: var(--surface-3);
  border-radius: 2px;
  margin-top: 0.25rem;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.2s ease;
}

.queue-remove {
  background: transparent;
  border: none;
  color: var(--text-3);
  font-size: 1rem;
  cursor: pointer;
  padding: 0.25rem;
  line-height: 1;
  border-radius: 4px;
  transition: all var(--duration) var(--ease);
}

.queue-remove:hover { color: #f87171; background: rgba(248,113,113,0.1); }

.slide-enter-active, .slide-leave-active { transition: all var(--duration) var(--ease); }
.slide-enter-from, .slide-leave-to { opacity: 0; transform: translateY(-8px); }
</style>
