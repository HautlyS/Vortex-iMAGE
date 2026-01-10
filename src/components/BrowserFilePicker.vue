<script setup lang="ts">
import { ref, computed } from 'vue'
import PixelCard from './retro/PixelCard.vue'
import PixelButton from './retro/PixelButton.vue'

interface SelectedFile {
  file: File
  preview: string
  name: string
  size: number
  type: string
}

const emit = defineEmits<{
  select: [files: File[]]
  cancel: []
}>()

const props = withDefaults(defineProps<{
  multiple?: boolean
  accept?: string
  maxSize?: number // in MB
}>(), {
  multiple: true,
  accept: 'image/*',
  maxSize: 50
})

const selectedFiles = ref<SelectedFile[]>([])
const isDragging = ref(false)
const error = ref<string | null>(null)

const fileInput = ref<HTMLInputElement | null>(null)
const folderInput = ref<HTMLInputElement | null>(null)

const totalSize = computed(() => {
  return selectedFiles.value.reduce((acc, f) => acc + f.size, 0)
})

const formattedTotalSize = computed(() => {
  const bytes = totalSize.value
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
})

function handleFileSelect(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files) {
    processFiles(Array.from(input.files))
  }
}

function handleDrop(e: DragEvent) {
  isDragging.value = false
  const items = e.dataTransfer?.items
  
  if (items) {
    const files: File[] = []
    for (let i = 0; i < items.length; i++) {
      const item = items[i]
      if (item.kind === 'file') {
        const file = item.getAsFile()
        if (file) files.push(file)
      }
    }
    processFiles(files)
  }
}

async function processFiles(files: File[]) {
  error.value = null
  
  const imageFiles = files.filter(f => f.type.startsWith('image/'))
  
  if (imageFiles.length === 0) {
    error.value = 'No valid image files selected'
    return
  }

  for (const file of imageFiles) {
    // Check file size
    if (file.size > props.maxSize * 1024 * 1024) {
      error.value = `File "${file.name}" exceeds ${props.maxSize}MB limit`
      continue
    }

    // Check if already selected
    if (selectedFiles.value.some(f => f.name === file.name && f.size === file.size)) {
      continue
    }

    // Create preview
    const preview = await createPreview(file)
    
    selectedFiles.value.push({
      file,
      preview,
      name: file.name,
      size: file.size,
      type: file.type
    })
  }
}

function createPreview(file: File): Promise<string> {
  return new Promise((resolve) => {
    const reader = new FileReader()
    reader.onload = (e) => resolve(e.target?.result as string)
    reader.readAsDataURL(file)
  })
}

function removeFile(index: number) {
  const file = selectedFiles.value[index]
  if (file.preview.startsWith('blob:')) {
    URL.revokeObjectURL(file.preview)
  }
  selectedFiles.value.splice(index, 1)
}

function clearAll() {
  selectedFiles.value.forEach(f => {
    if (f.preview.startsWith('blob:')) {
      URL.revokeObjectURL(f.preview)
    }
  })
  selectedFiles.value = []
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function handleConfirm() {
  if (selectedFiles.value.length > 0) {
    emit('select', selectedFiles.value.map(f => f.file))
  }
}

function openFilePicker() {
  fileInput.value?.click()
}

function openFolderPicker() {
  folderInput.value?.click()
}
</script>

<template>
  <div class="browser-file-picker">
    <PixelCard title="SELECT FILES" :glow="true">
      <!-- Hidden inputs -->
      <input
        ref="fileInput"
        type="file"
        :accept="accept"
        :multiple="multiple"
        style="display: none"
        @change="handleFileSelect"
      />
      <input
        ref="folderInput"
        type="file"
        :accept="accept"
        webkitdirectory
        directory
        multiple
        style="display: none"
        @change="handleFileSelect"
      />

      <!-- Drop zone -->
      <div 
        class="drop-zone"
        :class="{ dragging: isDragging }"
        @dragover.prevent="isDragging = true"
        @dragleave.prevent="isDragging = false"
        @drop.prevent="handleDrop"
      >
        <div class="drop-content">
          <div class="drop-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17,8 12,3 7,8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
          </div>
          <p class="drop-text">Drag & drop images here</p>
          <p class="drop-subtext">or use the buttons below</p>
        </div>
      </div>

      <!-- Action buttons -->
      <div class="picker-actions">
        <PixelButton @click="openFilePicker">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14,2 14,8 20,8"/>
          </svg>
          Select Files
        </PixelButton>
        <PixelButton @click="openFolderPicker">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
          Select Folder
        </PixelButton>
      </div>

      <!-- Error message -->
      <div v-if="error" class="error-message">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="15" y1="9" x2="9" y2="15"/>
          <line x1="9" y1="9" x2="15" y2="15"/>
        </svg>
        {{ error }}
      </div>

      <!-- Selected files preview -->
      <div v-if="selectedFiles.length > 0" class="selected-files">
        <div class="files-header">
          <span class="files-count">{{ selectedFiles.length }} files selected</span>
          <span class="files-size">{{ formattedTotalSize }}</span>
          <button class="clear-btn" @click="clearAll">Clear all</button>
        </div>
        
        <div class="files-grid">
          <div 
            v-for="(file, index) in selectedFiles" 
            :key="`${file.name}-${index}`"
            class="file-item"
          >
            <img :src="file.preview" :alt="file.name" class="file-preview" />
            <div class="file-info">
              <span class="file-name">{{ file.name }}</span>
              <span class="file-size">{{ formatSize(file.size) }}</span>
            </div>
            <button class="remove-btn" @click="removeFile(index)" aria-label="Remove">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Footer actions -->
      <div class="footer-actions">
        <PixelButton @click="emit('cancel')">
          Cancel
        </PixelButton>
        <PixelButton 
          variant="primary" 
          :disabled="selectedFiles.length === 0"
          @click="handleConfirm"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17,8 12,3 7,8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          Upload {{ selectedFiles.length }} {{ selectedFiles.length === 1 ? 'file' : 'files' }}
        </PixelButton>
      </div>
    </PixelCard>
  </div>
</template>

<style scoped>
.browser-file-picker {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  z-index: 1000;
  padding: 1rem;
}

.browser-file-picker :deep(.pixel-card) {
  max-width: 600px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
}

.drop-zone {
  border: 3px dashed var(--retro-border, #3d2a6d);
  background: var(--retro-bg-dark, #0f0a1e);
  padding: 2rem;
  text-align: center;
  transition: all 0.2s;
  margin-bottom: 1rem;
}

.drop-zone.dragging {
  border-color: var(--retro-accent-cyan, #00d4ff);
  background: rgba(0, 212, 255, 0.1);
}

.drop-content {
  pointer-events: none;
}

.drop-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--retro-bg-panel, #1a1030);
  border: 2px solid var(--retro-border, #3d2a6d);
}

.drop-icon svg {
  width: 32px;
  height: 32px;
  stroke: var(--retro-accent-cyan, #00d4ff);
}

.drop-text {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: var(--retro-text-main, #fff);
  margin-bottom: 0.5rem;
}

.drop-subtext {
  font-size: 11px;
  color: var(--retro-text-muted, #9d8ec2);
}

.picker-actions {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.picker-actions :deep(button) {
  flex: 1;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  background: rgba(255, 45, 85, 0.1);
  border: 2px solid var(--retro-accent-pink, #ff2d95);
  color: var(--retro-accent-pink, #ff2d95);
  font-size: 11px;
  margin-bottom: 1rem;
}

.error-message svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.selected-files {
  margin-bottom: 1rem;
}

.files-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 2px solid var(--retro-border, #3d2a6d);
  margin-bottom: 0.75rem;
}

.files-count {
  font-family: 'Press Start 2P', monospace;
  font-size: 9px;
  color: var(--retro-accent-green, #00ff87);
}

.files-size {
  font-size: 11px;
  color: var(--retro-text-muted, #9d8ec2);
  margin-left: auto;
}

.clear-btn {
  font-size: 10px;
  color: var(--retro-accent-pink, #ff2d95);
  background: none;
  border: none;
  cursor: pointer;
  text-decoration: underline;
}

.files-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 0.75rem;
  max-height: 300px;
  overflow-y: auto;
  padding: 0.5rem;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 2px solid var(--retro-border, #3d2a6d);
}

.file-item {
  position: relative;
  aspect-ratio: 1;
  border: 2px solid var(--retro-border, #3d2a6d);
  overflow: hidden;
  background: var(--retro-bg-panel, #1a1030);
}

.file-preview {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.file-info {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.5rem;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.9));
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.file-name {
  font-size: 8px;
  color: var(--retro-text-main, #fff);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 8px;
  color: var(--retro-text-muted, #9d8ec2);
}

.remove-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--retro-accent-pink, #ff2d95);
  border: 2px solid #000;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
}

.file-item:hover .remove-btn {
  opacity: 1;
}

.remove-btn svg {
  width: 12px;
  height: 12px;
  stroke: #fff;
}

.footer-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding-top: 1rem;
  border-top: 2px solid var(--retro-border, #3d2a6d);
}
</style>
