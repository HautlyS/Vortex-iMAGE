<script setup lang="ts">
import { ref, computed } from 'vue'
import SyncStatusIndicator from './SyncStatusIndicator.vue'
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

const containerStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
}))

// Dynamic border color: use colorTag if available, otherwise use accent color
const borderColor = computed(() => {
  if (props.colorTag) {
    return props.colorTag
  }
  // Fall back to CSS variable (accent color from theme)
  return 'var(--accent-color, #6366f1)'
})

// Compute RGB values for glow effect
const borderColorRgb = computed(() => {
  if (props.colorTag) {
    // Convert hex to RGB
    const hex = props.colorTag.replace('#', '')
    const r = parseInt(hex.substring(0, 2), 16)
    const g = parseInt(hex.substring(2, 4), 16)
    const b = parseInt(hex.substring(4, 6), 16)
    return `${r}, ${g}, ${b}`
  }
  return 'var(--accent-rgb, 99, 102, 241)'
})

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
  <div
    class="photo-preview"
    :class="{ selected, resizing: isResizing, 'has-color-tag': !!colorTag }"
    :style="[containerStyle, selected ? { 
      '--dynamic-border-color': borderColor,
      '--dynamic-border-rgb': borderColorRgb
    } : {}]"
    @click="handleClick"
    @contextmenu="handleContextMenu"
  >
    <!-- Image -->
    <img :src="photo.url" :alt="photo.name" loading="lazy" />

    <!-- Color Tag Indicator -->
    <div v-if="colorTag" class="color-tag" :style="{ backgroundColor: colorTag }" />

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

    <!-- Sync Status Indicator -->
    <div class="sync-status-wrapper">
      <SyncStatusIndicator 
        v-if="syncStatus"
        :photo-id="photo.sha"
        :status="syncStatus"
        @action="handleSyncAction"
      />
    </div>

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
  </div>
</template>

<style scoped>
.photo-preview {
  position: relative;
  border-radius: 0.625rem;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  background: var(--bg-tertiary, #1a1a1c);
  will-change: transform;
}

.photo-preview:hover {
  transform: scale(1.02);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.08);
}

.photo-preview.selected {
  outline: 2px solid var(--dynamic-border-color, var(--accent-color, #6366f1));
  outline-offset: 2px;
  box-shadow: 0 0 0 4px rgba(var(--dynamic-border-rgb, var(--accent-rgb, 99, 102, 241)), 0.2);
}

.photo-preview.selected.has-color-tag {
  box-shadow: 0 0 0 4px rgba(var(--dynamic-border-rgb), 0.25),
              0 0 16px rgba(var(--dynamic-border-rgb), 0.15);
}

.photo-preview.resizing {
  cursor: nwse-resize;
  transform: none;
}

.photo-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}

.photo-preview:hover img {
  transform: scale(1.05);
}

/* Color Tag */
.color-tag {
  position: absolute;
  top: 0.5rem;
  left: 0.5rem;
  width: 0.875rem;
  height: 0.875rem;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
  z-index: 2;
}

/* Favorite Button */
.favorite-btn {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  border: none;
  border-radius: 50%;
  color: #fff;
  cursor: pointer;
  opacity: 0;
  transition: all 0.2s ease;
  z-index: 2;
}

.photo-preview:hover .favorite-btn {
  opacity: 1;
}

.favorite-btn:hover {
  background: rgba(0, 0, 0, 0.8);
  transform: scale(1.1);
}

.favorite-btn.active {
  opacity: 1;
  color: var(--error, #ef4444);
}

.favorite-btn svg {
  width: 1rem;
  height: 1rem;
}

/* Resize Handle */
.resize-handle {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  border-radius: 0.375rem 0 0.625rem 0;
  color: #fff;
  cursor: nwse-resize;
  opacity: 0;
  transition: opacity 0.2s ease;
  z-index: 2;
}

.photo-preview:hover .resize-handle {
  opacity: 1;
}

.resize-handle:hover {
  background: rgba(var(--accent-rgb, 99, 102, 241), 0.8);
}

.resize-handle svg {
  width: 0.875rem;
  height: 0.875rem;
}

/* Sync Status Wrapper */
.sync-status-wrapper {
  position: absolute;
  bottom: 0.5rem;
  left: 0.5rem;
  z-index: 2;
}

.sync-status-wrapper :deep(.indicator-btn) {
  opacity: 0;
}

.photo-preview:hover .sync-status-wrapper :deep(.indicator-btn) {
  opacity: 1;
}

/* Selection Indicator */
.selection-indicator {
  position: absolute;
  top: 0.5rem;
  left: 0.5rem;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--dynamic-border-color, var(--accent-color, #6366f1));
  border-radius: 50%;
  color: #fff;
  box-shadow: 0 2px 8px rgba(var(--dynamic-border-rgb, var(--accent-rgb, 99, 102, 241)), 0.4);
  z-index: 3;
  animation: scaleIn 0.15s ease-out;
}

@keyframes scaleIn {
  from { transform: scale(0); }
  to { transform: scale(1); }
}

.selection-indicator svg {
  width: 1rem;
  height: 1rem;
}

/* Hover Overlay */
.hover-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.75rem;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.85));
  opacity: 0;
  transition: opacity 0.2s ease;
  z-index: 1;
}

.photo-preview:hover .hover-overlay {
  opacity: 1;
}

.photo-name {
  font-size: 0.75rem;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
  font-weight: 500;
}
</style>
