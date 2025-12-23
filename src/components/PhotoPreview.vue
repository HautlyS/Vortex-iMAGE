/**
 * Vue Component - 2 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: SyncStatusIndicator, DecayCard
 */

<script setup lang="ts">
import { ref } from 'vue'
import SyncStatusIndicator from './SyncStatusIndicator.vue'
import DecayCard from './DecayCard.vue'
import type { SyncStatus } from '../composables/useSyncStatus'

interface Photo {
  sha: string
  name: string
  url: string
  path?: string
}

const props = defineProps<{
  photo: Photo
  size: number
  selected: boolean
  favorited: boolean
  colorTag?: string
  syncStatus?: SyncStatus
}>()

const emit = defineEmits<{
  (e: 'select', photo: Photo, additive: boolean, range: boolean): void
  (e: 'resize', newSize: number): void
  (e: 'favorite', photo: Photo): void
  (e: 'contextmenu', event: MouseEvent, photo: Photo): void
  (e: 'sync-action', action: string, photoId: string): void
}>()

const isResizing = ref(false)
const startX = ref(0)
const startSize = ref(0)

function handleClick(e: MouseEvent) {
  emit('select', props.photo, e.ctrlKey || e.metaKey, e.shiftKey)
}

function handleContextMenu(e: MouseEvent) {
  e.preventDefault()
  emit('contextmenu', e, props.photo)
}

function handleFavoriteClick(e: MouseEvent) {
  e.stopPropagation()
  emit('favorite', props.photo)
}

function startResize(e: MouseEvent) {
  e.stopPropagation()
  e.preventDefault()
  isResizing.value = true
  startX.value = e.clientX
  startSize.value = props.size
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
}

function onResize(e: MouseEvent) {
  if (!isResizing.value) return
  const delta = e.clientX - startX.value
  const newSize = Math.max(80, Math.min(400, startSize.value + delta))
  emit('resize', newSize)
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
}

function handleSyncAction(action: string, photoId: string) {
  emit('sync-action', action, photoId)
}
</script>

<template>
  <DecayCard
    :src="photo.url"
    :alt="photo.name"
    :size="size"
    :selected="selected"
    :color-tag="colorTag"
    class="photo-preview"
    :class="{ resizing: isResizing }"
    @click="handleClick"
    @contextmenu="handleContextMenu"
  >
    <!-- Actions (Favorite & Resize) -->
    <template #actions>
      <!-- Favorite Button -->
      <button class="favorite-btn" :class="{ active: favorited }" @click="handleFavoriteClick">
        <svg viewBox="0 0 24 24" :fill="favorited ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
          <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
        </svg>
      </button>

      <!-- Resize Handle -->
      <div class="resize-handle" @mousedown="startResize">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 21l-6-6m6 6v-6m0 6h-6" />
        </svg>
      </div>
    </template>

    <!-- Sync Status -->
    <template #status>
      <div class="sync-status-wrapper">
        <SyncStatusIndicator 
          v-if="syncStatus"
          :photo-id="photo.sha"
          :status="syncStatus"
          @action="handleSyncAction"
        />
      </div>
    </template>

    <!-- Default Slot (Overlays) -->
    <!-- Selection Indicator -->
    <div v-if="selected" class="selection-indicator">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
      </svg>
    </div>

    <!-- Hover Overlay -->
    <div class="hover-overlay">
      <span class="photo-name">{{ photo.name }}</span>
    </div>
  </DecayCard>
</template>

<style scoped>

.photo-preview.resizing { 
  cursor: nwse-resize; 
  transform: none; 
}

.favorite-btn {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  width: 1.75rem;
  height: 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.7);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 50%;
  color: #fff;
  cursor: pointer;
  opacity: 0;
  transition: all 0.2s ease;
  z-index: 5;
}
:deep(.decay-card:hover) .favorite-btn { opacity: 1; }
.favorite-btn:hover { background: rgba(0, 0, 0, 0.9); transform: scale(1.1); }
.favorite-btn.active { opacity: 1; color: #ff2d6a; }
.favorite-btn svg { width: 0.875rem; height: 0.875rem; }

.resize-handle {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 1.25rem;
  height: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 0.25rem 0 0.5rem 0;
  color: #fff;
  cursor: nwse-resize;
  opacity: 0;
  transition: opacity 0.2s ease;
  z-index: 5;
}
:deep(.decay-card:hover) .resize-handle { opacity: 1; }
.resize-handle:hover { background: var(--cyber-cyan, #00f0ff); color: #000; }
.resize-handle svg { width: 0.75rem; height: 0.75rem; }

.sync-status-wrapper {
  position: absolute;
  bottom: 0.5rem;
  left: 0.5rem;
  z-index: 5;
}
.sync-status-wrapper :deep(.indicator-btn) { opacity: 0; }
:deep(.decay-card:hover) .sync-status-wrapper :deep(.indicator-btn) { opacity: 1; }

.selection-indicator {
  position: absolute;
  top: 0.5rem;
  left: 0.5rem;
  width: 1.25rem;
  height: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--dynamic-border-color, var(--accent-color, #6366f1));
  border-radius: 50%;
  color: #fff;
  box-shadow: 0 0 10px rgba(var(--dynamic-border-rgb, var(--accent-rgb, 99, 102, 241)), 0.6);
  z-index: 6;
  animation: scaleIn 0.15s ease-out;
}
@keyframes scaleIn { from { transform: scale(0); } to { transform: scale(1); } }
.selection-indicator svg { width: 0.875rem; height: 0.875rem; }

.hover-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.5rem;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.9));
  opacity: 0;
  transition: opacity 0.2s ease;
  z-index: 3;
  pointer-events: none;
}
:deep(.decay-card:hover) .hover-overlay { opacity: 1; }
.photo-name {
  font-size: 0.6875rem;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
  font-family: 'JetBrains Mono', monospace;
  letter-spacing: 0.02em;
}
</style>