<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
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
const decayOffset = ref({ x: 0, y: 0 })

onMounted(() => {
  decayOffset.value = { x: Math.random() * 100, y: Math.random() * 100 }
})

const containerStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  '--decay-x': decayOffset.value.x,
  '--decay-y': decayOffset.value.y,
}))

const borderColor = computed(() => props.colorTag || 'var(--accent-color, #6366f1)')

const borderColorRgb = computed(() => {
  if (props.colorTag) {
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
    class="decay-card"
    :class="{ selected, resizing: isResizing, 'has-color-tag': !!colorTag }"
    :style="[containerStyle, selected ? { 
      '--dynamic-border-color': borderColor,
      '--dynamic-border-rgb': borderColorRgb
    } : {}]"
    @click="handleClick"
    @contextmenu="handleContextMenu"
  >
    <!-- Decay layers -->
    <div class="decay-layer decay-cyan" />
    <div class="decay-layer decay-pink" />
    
    <!-- Scanlines -->
    <div class="decay-scanlines" />

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
    
    <!-- Corner accents -->
    <div class="decay-corner tl" />
    <div class="decay-corner br" />
  </div>
</template>

<style scoped>
.decay-card {
  position: relative;
  border-radius: 0.5rem;
  overflow: hidden;
  cursor: pointer;
  background: var(--bg-tertiary, #0a0a0a);
  will-change: transform;
  transition: transform 0.2s ease;
}

.decay-card:hover {
  transform: scale(1.02);
}

.decay-card:hover .decay-layer { opacity: 1; }
.decay-card:hover .decay-scanlines { opacity: 0.15; }
.decay-card:hover .decay-corner { opacity: 1; }

/* Decay glitch layers */
.decay-layer {
  position: absolute;
  inset: 0;
  opacity: 0;
  pointer-events: none;
  mix-blend-mode: screen;
  transition: opacity 0.2s;
}

.decay-cyan {
  background: linear-gradient(135deg, transparent 40%, rgba(0, 240, 255, 0.15) 50%, transparent 60%);
  animation: decay-shift 3s ease-in-out infinite;
}

.decay-pink {
  background: linear-gradient(225deg, transparent 40%, rgba(255, 45, 106, 0.15) 50%, transparent 60%);
  animation: decay-shift 3s ease-in-out infinite reverse;
}

@keyframes decay-shift {
  0%, 100% { transform: translateX(-2px); }
  50% { transform: translateX(2px); }
}

/* Scanlines */
.decay-scanlines {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.1) 2px, rgba(0,0,0,0.1) 4px);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.2s;
}

/* Corner accents */
.decay-corner {
  position: absolute;
  width: 12px;
  height: 12px;
  border: 1px solid var(--cyber-cyan, #00f0ff);
  opacity: 0;
  transition: opacity 0.2s;
  pointer-events: none;
}
.decay-corner.tl { top: 4px; left: 4px; border-right: none; border-bottom: none; }
.decay-corner.br { bottom: 4px; right: 4px; border-left: none; border-top: none; }

.decay-card.selected {
  outline: 2px solid var(--dynamic-border-color, var(--accent-color, #6366f1));
  outline-offset: 2px;
  box-shadow: 0 0 12px rgba(var(--dynamic-border-rgb, var(--accent-rgb, 99, 102, 241)), 0.4);
}

.decay-card.resizing { cursor: nwse-resize; transform: none; }

.decay-card img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease, filter 0.2s;
}

.decay-card:hover img {
  transform: scale(1.05);
  filter: saturate(1.1) contrast(1.05);
}

/* Color Tag */
.color-tag {
  position: absolute;
  top: 0.5rem;
  left: 0.5rem;
  width: 0.75rem;
  height: 0.75rem;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 0 8px currentColor;
  z-index: 4;
}

/* Favorite Button */
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
  z-index: 4;
}
.decay-card:hover .favorite-btn { opacity: 1; }
.favorite-btn:hover { background: rgba(0, 0, 0, 0.9); transform: scale(1.1); }
.favorite-btn.active { opacity: 1; color: #ff2d6a; }
.favorite-btn svg { width: 0.875rem; height: 0.875rem; }

/* Resize Handle */
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
  z-index: 4;
}
.decay-card:hover .resize-handle { opacity: 1; }
.resize-handle:hover { background: var(--cyber-cyan, #00f0ff); color: #000; }
.resize-handle svg { width: 0.75rem; height: 0.75rem; }

/* Sync Status */
.sync-status-wrapper {
  position: absolute;
  bottom: 0.5rem;
  left: 0.5rem;
  z-index: 4;
}
.sync-status-wrapper :deep(.indicator-btn) { opacity: 0; }
.decay-card:hover .sync-status-wrapper :deep(.indicator-btn) { opacity: 1; }

/* Selection Indicator */
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
  z-index: 5;
  animation: scaleIn 0.15s ease-out;
}
@keyframes scaleIn { from { transform: scale(0); } to { transform: scale(1); } }
.selection-indicator svg { width: 0.875rem; height: 0.875rem; }

/* Hover Overlay */
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
}
.decay-card:hover .hover-overlay { opacity: 1; }
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
