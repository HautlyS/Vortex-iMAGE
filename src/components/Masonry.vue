<template>
  <div 
    ref="containerRef" 
    class="photo-grid-container"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  >
    <!-- Selection Menu (appears when items selected) -->
    <Transition name="slide-up">
      <div v-if="selected.length > 0" class="selection-menu">
        <div class="selection-info">
          <span class="selection-count">{{ selected.length }}</span>
          <span class="selection-label">selecionados</span>
        </div>
        <div class="selection-actions">
          <button class="sel-btn" @click="selectAllVisible" title="Selecionar tudo visível">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
              <rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
            </svg>
            <span>Tudo</span>
          </button>
          <button class="sel-btn" @click="selectAlbum" title="Selecionar álbum">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <span>Álbum</span>
          </button>
          <button class="sel-btn danger" @click="clearSelection" title="Limpar seleção">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
            <span>Limpar</span>
          </button>
        </div>
      </div>
    </Transition>

    <!-- Resize Handle (bottom-right corner) -->
    <div 
      class="resize-handle"
      @mousedown.stop="startResize"
      title="Arrastar para redimensionar"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M22 22H20V20H22V22ZM22 18H20V16H22V18ZM18 22H16V20H18V22ZM22 14H20V12H22V14ZM18 18H16V16H18V18ZM14 22H12V20H14V22ZM18 14H16V12H18V14ZM14 18H12V16H14V18ZM10 22H8V20H10V22Z"/>
      </svg>
      <span class="resize-label">{{ Math.round(gridSize) }}px</span>
    </div>

    <!-- Photo Grid -->
    <div 
      class="photo-grid" 
      :style="{ '--grid-size': `${gridSize}px` }"
    >
      <div
        v-for="item in visibleItems"
        :key="item.id"
        class="photo-grid-item"
        :class="{ 
          'selected': isSelected(item.id),
          'is-folder': item.isFolder,
          'in-drag-select': isInDragSelection(item)
        }"
        :data-id="item.id"
        @click.stop="handleItemClick(item, $event)"
        @dblclick.stop="handleItemDblClick(item)"
        @contextmenu.prevent="handleContextMenu(item, $event)"
      >
        <!-- Image -->
        <img 
          :src="item.img" 
          :alt="item.folderName || 'Photo'"
          loading="lazy"
          draggable="false"
        />
        
        <!-- Selection Checkbox -->
        <button
          v-if="!item.isFolder"
          class="select-btn"
          :class="{ 'active': isSelected(item.id) }"
          @click.stop="toggleSelect(item.id)"
        >
          <svg v-if="isSelected(item.id)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <path d="M5 13l4 4L19 7"/>
          </svg>
        </button>

        <!-- Folder Overlay -->
        <div v-if="item.isFolder" class="folder-badge">
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
          </svg>
          <span class="folder-name">{{ item.folderName }}</span>
          <span class="folder-count">{{ item.photoCount }}</span>
        </div>

        <!-- Tag Indicator -->
        <div v-if="item.tagColor" class="tag-indicator" :style="{ backgroundColor: item.tagColor }"></div>

        <!-- Hover Info -->
        <div class="item-info">
          <span class="item-name">{{ item.folderName || (item as any).name || 'Photo' }}</span>
        </div>
      </div>
    </div>

    <!-- Drag Selection Box -->
    <div 
      v-if="isDragSelecting"
      class="selection-box"
      :style="{
        left: `${selectionBox.x}px`,
        top: `${selectionBox.y}px`,
        width: `${selectionBox.w}px`,
        height: `${selectionBox.h}px`
      }"
    >
      <span class="selection-box-count" v-if="dragSelectCount > 0">
        {{ dragSelectCount }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';

interface Item {
  id: string;
  img: string;
  url: string;
  height: number;
  isFolder?: boolean;
  folderName?: string;
  photoCount?: number;
  tagColor?: string;
  [key: string]: any;
}

const props = withDefaults(defineProps<{
  items: Item[];
  scaleOnHover?: boolean;
  animateFrom?: string;
  initialSize?: number;
}>(), {
  scaleOnHover: true,
  animateFrom: 'bottom',
  initialSize: 180
});

const emit = defineEmits<{
  itemClick: [item: Item];
  itemDblClick: [item: Item];
  contextMenu: [item: Item, event: MouseEvent];
  select: [ids: Set<string>];
  resize: [size: number];
}>()

// Grid size state
const gridSize = ref(props.initialSize)
const isResizing = ref(false)
const resizeStart = ref({ x: 0, size: 0 })

// Selection state
const selected = ref<string[]>([])
const containerRef = ref<HTMLElement | null>(null)

// Drag selection state
const isDragSelecting = ref(false)
const dragSelectStart = ref({ x: 0, y: 0 })
const selectionBox = ref({ x: 0, y: 0, w: 0, h: 0 })
const dragSelectCount = ref(0)
const itemsInDragSelection = ref<Set<string>>(new Set())

// Load saved grid size
onMounted(async () => {
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    const savedSize = await store.get<number>('gridSize')
    if (savedSize && savedSize >= 100 && savedSize <= 400) {
      gridSize.value = savedSize
    }
  } catch {}
})

// Save grid size when changed
watch(gridSize, async (size) => {
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    await store.set('gridSize', size)
    await store.save()
    emit('resize', size)
  } catch {}
})

// Resize handlers
function startResize(e: MouseEvent) {
  isResizing.value = true
  resizeStart.value = { x: e.clientX, size: gridSize.value }
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'nwse-resize'
  document.body.style.userSelect = 'none'
}

function onResize(e: MouseEvent) {
  if (!isResizing.value) return
  const delta = e.clientX - resizeStart.value.x
  const newSize = Math.max(100, Math.min(400, resizeStart.value.size + delta * 0.5))
  gridSize.value = newSize
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

// Simplified items
const visibleItems = computed(() => props.items)

const isSelected = (id: string) => selected.value.includes(id)
const isInDragSelection = (item: Item) => itemsInDragSelection.value.has(item.id)

const toggleSelect = (id: string) => {
  const idx = selected.value.indexOf(id)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(id)
  emit('select', new Set(selected.value))
}

// Mouse handlers for drag selection
function handleMouseDown(e: MouseEvent) {
  // Only start drag select on left click and not on items
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('.photo-grid-item') || target.closest('.selection-menu') || target.closest('.resize-handle')) return
  
  isDragSelecting.value = true
  const rect = containerRef.value?.getBoundingClientRect()
  if (!rect) return
  
  dragSelectStart.value = { 
    x: e.clientX - rect.left + (containerRef.value?.scrollLeft || 0),
    y: e.clientY - rect.top + (containerRef.value?.scrollTop || 0)
  }
  selectionBox.value = { 
    x: dragSelectStart.value.x, 
    y: dragSelectStart.value.y, 
    w: 0, 
    h: 0 
  }
  itemsInDragSelection.value.clear()
  dragSelectCount.value = 0
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragSelecting.value || !containerRef.value) return
  
  const rect = containerRef.value.getBoundingClientRect()
  const currentX = e.clientX - rect.left + containerRef.value.scrollLeft
  const currentY = e.clientY - rect.top + containerRef.value.scrollTop
  
  const x = Math.min(dragSelectStart.value.x, currentX)
  const y = Math.min(dragSelectStart.value.y, currentY)
  const w = Math.abs(currentX - dragSelectStart.value.x)
  const h = Math.abs(currentY - dragSelectStart.value.y)
  
  selectionBox.value = { x, y, w, h }
  
  // Find items in selection box
  const newSelection = new Set<string>()
  const items = containerRef.value.querySelectorAll('.photo-grid-item')
  
  items.forEach((el) => {
    const itemRect = el.getBoundingClientRect()
    const itemX = itemRect.left - rect.left + containerRef.value!.scrollLeft
    const itemY = itemRect.top - rect.top + containerRef.value!.scrollTop
    
    // Check intersection
    const intersects = (
      itemX < x + w &&
      itemX + itemRect.width > x &&
      itemY < y + h &&
      itemY + itemRect.height > y
    )
    
    if (intersects) {
      const id = el.getAttribute('data-id')
      if (id) newSelection.add(id)
    }
  })
  
  itemsInDragSelection.value = newSelection
  dragSelectCount.value = newSelection.size
}

function handleMouseUp() {
  if (isDragSelecting.value && itemsInDragSelection.value.size > 0) {
    // Add drag-selected items to selection
    itemsInDragSelection.value.forEach(id => {
      if (!selected.value.includes(id)) {
        selected.value.push(id)
      }
    })
    emit('select', new Set(selected.value))
  }
  
  isDragSelecting.value = false
  itemsInDragSelection.value.clear()
  dragSelectCount.value = 0
}

// Selection menu actions
function selectAllVisible() {
  const allIds = props.items.filter(i => !i.isFolder).map(i => i.id)
  selected.value = [...allIds]
  emit('select', new Set(selected.value))
}

function selectAlbum() {
  // Select all items (simulating album selection)
  const allIds = props.items.filter(i => !i.isFolder).map(i => i.id)
  selected.value = [...allIds]
  emit('select', new Set(selected.value))
}

function clearSelection() {
  selected.value = []
  emit('select', new Set())
}

// Item click with shift/ctrl support
function handleItemClick(item: Item, e: MouseEvent) {
  if (e.shiftKey && selected.value.length > 0) {
    // Range select
    const lastSelected = selected.value[selected.value.length - 1]
    const lastIndex = props.items.findIndex(i => i.id === lastSelected)
    const currentIndex = props.items.findIndex(i => i.id === item.id)
    
    if (lastIndex !== -1 && currentIndex !== -1) {
      const start = Math.min(lastIndex, currentIndex)
      const end = Math.max(lastIndex, currentIndex)
      
      for (let i = start; i <= end; i++) {
        const id = props.items[i].id
        if (!selected.value.includes(id)) {
          selected.value.push(id)
        }
      }
      emit('select', new Set(selected.value))
      return
    }
  }
  
  if (e.ctrlKey || e.metaKey) {
    // Toggle select
    toggleSelect(item.id)
    return
  }
  
  // Normal click
  emit('itemClick', item)
}

const handleItemDblClick = (item: Item) => emit('itemDblClick', item)
const handleContextMenu = (item: Item, event: MouseEvent) => emit('contextMenu', item, event)

onUnmounted(() => {
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
})
</script>

<style scoped>
.photo-grid-container {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px;
  position: relative;
  user-select: none;
  -webkit-user-select: none;
}

/* === SELECTION MENU === */
.selection-menu {
  position: sticky;
  top: 0;
  left: 0;
  right: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  margin: -8px -8px 8px -8px;
  background: var(--retro-bg-panel, #1a1030);
  border-bottom: 3px solid #000;
  box-shadow: 0 4px 0 rgba(0,0,0,0.3);
}

.selection-info {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.selection-count {
  font-family: 'Press Start 2P', monospace;
  font-size: 14px;
  color: var(--retro-accent-green, #00ff87);
  text-shadow: 0 0 10px var(--retro-accent-green, #00ff87);
}

.selection-label {
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: var(--retro-text-muted, #9d8ec2);
}

.selection-actions {
  display: flex;
  gap: 8px;
}

.sel-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--retro-bg-card, #251842);
  border: 2px solid var(--retro-bg-lighter, #2d1f4d);
  color: var(--retro-text-main, #fff);
  font-family: 'VT323', monospace;
  font-size: 16px;
  box-shadow: 2px 2px 0 #000;
  cursor: pointer;
  transition: all 0.1s;
}

.sel-btn svg {
  width: 16px;
  height: 16px;
}

.sel-btn:hover {
  border-color: var(--retro-accent-green, #00ff87);
  color: var(--retro-accent-green, #00ff87);
}

.sel-btn:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

.sel-btn.danger:hover {
  border-color: var(--retro-accent-red, #ff3b30);
  color: var(--retro-accent-red, #ff3b30);
}

/* === RESIZE HANDLE === */
.resize-handle {
  position: fixed;
  bottom: 80px;
  right: 20px;
  z-index: 30;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px;
  background: var(--retro-bg-panel, #1a1030);
  border: 2px solid var(--retro-accent-yellow, #ffd000);
  box-shadow: 3px 3px 0 #000;
  cursor: nwse-resize;
  transition: all 0.1s;
  user-select: none;
}

.resize-handle:hover {
  background: var(--retro-bg-card, #251842);
  box-shadow: 0 0 15px var(--retro-accent-yellow, #ffd000), 3px 3px 0 #000;
}

.resize-handle svg {
  width: 20px;
  height: 20px;
  color: var(--retro-accent-yellow, #ffd000);
}

.resize-label {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: var(--retro-accent-yellow, #ffd000);
}

/* === PHOTO GRID === */
.photo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--grid-size, 180px), 1fr));
  gap: 4px;
  padding: 4px;
}

/* === GRID ITEM === */
.photo-grid-item {
  aspect-ratio: 1;
  position: relative;
  overflow: hidden;
  background: var(--retro-bg-card, #251842);
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.1s;
  user-select: none;
  -webkit-user-select: none;
}

.photo-grid-item * {
  user-select: none;
  -webkit-user-select: none;
}

.photo-grid-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.15s;
  pointer-events: none;
}

.photo-grid-item:hover {
  border-color: var(--retro-accent-yellow, #ffd000);
  box-shadow: 
    0 0 0 1px var(--retro-accent-yellow, #ffd000),
    0 0 15px rgba(255, 208, 0, 0.4),
    4px 4px 0 #000;
  z-index: 5;
}

.photo-grid-item:hover img {
  transform: scale(1.05);
}

.photo-grid-item.selected {
  border-color: var(--retro-accent-green, #00ff87);
  box-shadow: 
    0 0 0 2px var(--retro-accent-green, #00ff87),
    0 0 20px rgba(0, 255, 135, 0.5),
    inset 0 0 40px rgba(0, 255, 135, 0.15);
}

.photo-grid-item.in-drag-select {
  border-color: var(--retro-accent-blue, #00d4ff);
  box-shadow: 
    0 0 0 2px var(--retro-accent-blue, #00d4ff),
    0 0 15px rgba(0, 212, 255, 0.4);
}

/* === FOLDER STYLE === */
.photo-grid-item.is-folder {
  background: linear-gradient(135deg, var(--retro-bg-lighter, #2d1f4d), var(--retro-bg-card, #251842));
}

.folder-badge {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: linear-gradient(to top, rgba(0,0,0,0.9) 0%, rgba(0,0,0,0.3) 50%, transparent 100%);
  padding: 12px;
  gap: 4px;
}

.folder-badge svg {
  width: 32px;
  height: 32px;
  color: var(--retro-accent-yellow, #ffd000);
  filter: drop-shadow(2px 2px 0 #000);
}

.folder-name {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #fff;
  text-shadow: 1px 1px 0 #000;
  text-align: center;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.folder-count {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: var(--retro-text-muted, #9d8ec2);
}

/* === SELECTION CHECKBOX === */
.select-btn {
  position: absolute;
  top: 8px;
  left: 8px;
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255,255,255,0.6);
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.1s;
  padding: 0;
  box-shadow: none;
}

.select-btn svg {
  width: 14px;
  height: 14px;
  color: #fff;
}

.photo-grid-item:hover .select-btn,
.select-btn.active {
  opacity: 1;
}

.select-btn.active {
  background: var(--retro-accent-green, #00ff87);
  border-color: var(--retro-accent-green, #00ff87);
}

.select-btn:hover {
  transform: none;
  box-shadow: none;
  border-color: var(--retro-accent-green, #00ff87);
}

/* === TAG INDICATOR === */
.tag-indicator {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 12px;
  height: 12px;
  border: 2px solid #000;
  box-shadow: 2px 2px 0 #000;
}

/* === HOVER INFO === */
.item-info {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 8px;
  background: linear-gradient(to top, rgba(0,0,0,0.9), transparent);
  opacity: 0;
  transition: opacity 0.1s;
}

.photo-grid-item:hover .item-info {
  opacity: 1;
}

.item-name {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: #fff;
  text-shadow: 1px 1px 0 #000;
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* === SELECTION BOX === */
.selection-box {
  position: absolute;
  border: 2px solid var(--retro-accent-blue, #00d4ff);
  background: rgba(0, 212, 255, 0.15);
  pointer-events: none;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
}

.selection-box-count {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  color: var(--retro-accent-blue, #00d4ff);
  text-shadow: 0 0 10px var(--retro-accent-blue, #00d4ff);
  background: rgba(0, 0, 0, 0.8);
  padding: 4px 8px;
  border: 2px solid var(--retro-accent-blue, #00d4ff);
}

/* === TRANSITIONS === */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.2s;
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}

/* === MOBILE === */
@media (max-width: 768px) {
  .resize-handle {
    bottom: 100px;
    right: 12px;
    padding: 8px;
  }
  
  .resize-handle svg {
    width: 16px;
    height: 16px;
  }
  
  .resize-label {
    font-size: 12px;
  }
  
  .selection-menu {
    flex-direction: column;
    gap: 10px;
    padding: 12px;
  }
  
  .selection-actions {
    width: 100%;
    justify-content: space-between;
  }
  
  .sel-btn span {
    display: none;
  }
  
  .sel-btn {
    padding: 10px;
  }
}

/* === TOUCH DEVICES === */
@media (hover: none) {
  .select-btn {
    opacity: 1;
  }
  
  .item-info {
    opacity: 1;
    background: linear-gradient(to top, rgba(0,0,0,0.7), transparent);
  }
}
</style>
