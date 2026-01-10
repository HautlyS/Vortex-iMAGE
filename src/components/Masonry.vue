<template>
  <div 
    ref="containerRef" 
    class="masonry-container"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  >
    <!-- Selection Menu -->
    <Transition name="slide-up">
      <div v-if="selected.length > 0" class="selection-menu">
        <div class="selection-info">
          <span class="selection-count">{{ selected.length }}</span>
          <span class="selection-label">selecionados</span>
        </div>
        <div class="selection-actions">
          <button class="sel-btn" @click="selectAllVisible" title="Selecionar tudo">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
              <rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
            </svg>
          </button>
          <button class="sel-btn danger" @click="clearSelection" title="Limpar">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>
    </Transition>

    <!-- Masonry Grid -->
    <div class="masonry-grid" :style="gridStyle">
      <div
        v-for="item in visibleItems"
        :key="item.id"
        class="masonry-item"
        :class="{ 
          'selected': isSelected(item.id),
          'is-folder': item.isFolder,
          'in-drag-select': isInDragSelection(item),
          'is-resizing': resizingItemId === item.id
        }"
        :style="getItemStyle(item)"
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
          @load="(e) => handleImageLoad(item.id, e)"
        />
        
        <!-- Resize Handle (per item) -->
        <div 
          class="item-resize"
          @mousedown.stop.prevent="startItemResize($event, item)"
          @touchstart.stop.prevent="startItemResizeTouch($event, item)"
        >
          <svg viewBox="0 0 10 10" fill="currentColor">
            <circle cx="8" cy="8" r="1.5"/>
            <circle cx="4" cy="8" r="1.5"/>
            <circle cx="8" cy="4" r="1.5"/>
          </svg>
        </div>
        
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
          <span class="folder-count">{{ item.photoCount }} fotos</span>
        </div>

        <!-- Tag Indicator -->
        <div v-if="item.tagColor" class="tag-indicator" :style="{ backgroundColor: item.tagColor }"></div>

        <!-- Hover Info -->
        <div class="item-info">
          <span class="item-name">{{ item.folderName || item.name || 'Photo' }}</span>
        </div>
      </div>
    </div>

    <!-- Global Resize Control -->
    <div class="global-resize">
      <button @click="gridSize = Math.max(120, gridSize - 30)">−</button>
      <input type="range" v-model.number="gridSize" min="120" max="400" step="10" />
      <button @click="gridSize = Math.min(400, gridSize + 30)">+</button>
      <span>{{ gridSize }}px</span>
    </div>

    <!-- Drag Selection Box -->
    <div 
      v-if="isDragSelecting"
      class="selection-box"
      :style="selectionBoxStyle"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, reactive } from 'vue';

interface Item {
  id: string;
  img: string;
  url: string;
  height: number;
  isFolder?: boolean;
  folderName?: string;
  photoCount?: number;
  tagColor?: string;
  name?: string;
}

const props = withDefaults(defineProps<{
  items: Item[];
  initialSize?: number;
}>(), {
  initialSize: 200
});

const emit = defineEmits<{
  itemClick: [item: Item];
  itemDblClick: [item: Item];
  contextMenu: [item: Item, event: MouseEvent];
  select: [ids: Set<string>];
  resize: [size: number];
}>();

// State
const gridSize = ref(props.initialSize);
const selected = ref<string[]>([]);
const containerRef = ref<HTMLElement | null>(null);

// Image aspect ratios
const imageRatios = reactive<Record<string, number>>({});

// Item custom widths
const itemWidths = reactive<Record<string, number>>({});

// Resize state
const resizingItemId = ref<string | null>(null);
const resizeStartX = ref(0);
const resizeStartWidth = ref(0);

// Drag selection
const isDragSelecting = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const dragCurrent = ref({ x: 0, y: 0 });
const itemsInDragSelection = ref<Set<string>>(new Set());

// Computed
const visibleItems = computed(() => props.items);

const gridStyle = computed(() => ({
  '--grid-size': `${gridSize.value}px`
}));

const selectionBoxStyle = computed(() => {
  const x = Math.min(dragStart.value.x, dragCurrent.value.x);
  const y = Math.min(dragStart.value.y, dragCurrent.value.y);
  const w = Math.abs(dragCurrent.value.x - dragStart.value.x);
  const h = Math.abs(dragCurrent.value.y - dragStart.value.y);
  return {
    left: `${x}px`,
    top: `${y}px`,
    width: `${w}px`,
    height: `${h}px`
  };
});

// Load/save grid size
onMounted(async () => {
  try {
    const { load } = await import('@tauri-apps/plugin-store');
    const store = await load('settings.json');
    const saved = await store.get<number>('gridSize');
    if (saved && saved >= 120 && saved <= 400) gridSize.value = saved;
  } catch {}
});

watch(gridSize, async (size) => {
  try {
    const { load } = await import('@tauri-apps/plugin-store');
    const store = await load('settings.json');
    await store.set('gridSize', size);
    await store.save();
    emit('resize', size);
  } catch {}
});

// Image load handler
function handleImageLoad(id: string, e: Event) {
  const img = e.target as HTMLImageElement;
  if (img.naturalWidth && img.naturalHeight) {
    imageRatios[id] = img.naturalWidth / img.naturalHeight;
  }
}

// Get item style
function getItemStyle(item: Item) {
  const customWidth = itemWidths[item.id];
  const ratio = imageRatios[item.id] || 1;
  const width = customWidth || gridSize.value;
  const height = Math.round(width / ratio);
  
  return {
    width: `${width}px`,
    height: `${Math.max(80, height)}px`
  };
}

// Item resize
function startItemResize(e: MouseEvent, item: Item) {
  e.preventDefault();
  e.stopPropagation();
  resizingItemId.value = item.id;
  resizeStartX.value = e.clientX;
  resizeStartWidth.value = itemWidths[item.id] || gridSize.value;
  
  document.addEventListener('mousemove', onItemResize);
  document.addEventListener('mouseup', stopItemResize);
  document.body.classList.add('resizing');
}

function startItemResizeTouch(e: TouchEvent, item: Item) {
  e.preventDefault();
  e.stopPropagation();
  if (e.touches.length !== 1) return;
  
  resizingItemId.value = item.id;
  resizeStartX.value = e.touches[0].clientX;
  resizeStartWidth.value = itemWidths[item.id] || gridSize.value;
  
  document.addEventListener('touchmove', onItemResizeTouch, { passive: false });
  document.addEventListener('touchend', stopItemResizeTouch);
  document.body.classList.add('resizing');
}

function onItemResize(e: MouseEvent) {
  if (!resizingItemId.value) return;
  e.preventDefault();
  const delta = e.clientX - resizeStartX.value;
  const newWidth = Math.max(100, Math.min(600, resizeStartWidth.value + delta));
  itemWidths[resizingItemId.value] = newWidth;
}

function onItemResizeTouch(e: TouchEvent) {
  if (!resizingItemId.value || e.touches.length !== 1) return;
  e.preventDefault();
  const delta = e.touches[0].clientX - resizeStartX.value;
  const newWidth = Math.max(100, Math.min(600, resizeStartWidth.value + delta));
  itemWidths[resizingItemId.value] = newWidth;
}

function stopItemResize() {
  resizingItemId.value = null;
  document.removeEventListener('mousemove', onItemResize);
  document.removeEventListener('mouseup', stopItemResize);
  document.body.classList.remove('resizing');
}

function stopItemResizeTouch() {
  resizingItemId.value = null;
  document.removeEventListener('touchmove', onItemResizeTouch);
  document.removeEventListener('touchend', stopItemResizeTouch);
  document.body.classList.remove('resizing');
}

// Selection
const isSelected = (id: string) => selected.value.includes(id);
const isInDragSelection = (item: Item) => itemsInDragSelection.value.has(item.id);

function toggleSelect(id: string) {
  const idx = selected.value.indexOf(id);
  if (idx >= 0) selected.value.splice(idx, 1);
  else selected.value.push(id);
  emit('select', new Set(selected.value));
}

function selectAllVisible() {
  selected.value = props.items.filter(i => !i.isFolder).map(i => i.id);
  emit('select', new Set(selected.value));
}

function clearSelection() {
  selected.value = [];
  emit('select', new Set());
}

// Drag selection
function handleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement;
  if (target.closest('.masonry-item') || target.closest('.selection-menu') || target.closest('.global-resize')) return;
  
  isDragSelecting.value = true;
  const rect = containerRef.value?.getBoundingClientRect();
  if (!rect) return;
  
  const x = e.clientX - rect.left + (containerRef.value?.scrollLeft || 0);
  const y = e.clientY - rect.top + (containerRef.value?.scrollTop || 0);
  dragStart.value = { x, y };
  dragCurrent.value = { x, y };
  itemsInDragSelection.value.clear();
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragSelecting.value || !containerRef.value) return;
  
  const rect = containerRef.value.getBoundingClientRect();
  dragCurrent.value = {
    x: e.clientX - rect.left + containerRef.value.scrollLeft,
    y: e.clientY - rect.top + containerRef.value.scrollTop
  };
  
  // Find items in selection
  const x1 = Math.min(dragStart.value.x, dragCurrent.value.x);
  const y1 = Math.min(dragStart.value.y, dragCurrent.value.y);
  const x2 = Math.max(dragStart.value.x, dragCurrent.value.x);
  const y2 = Math.max(dragStart.value.y, dragCurrent.value.y);
  
  const newSelection = new Set<string>();
  containerRef.value.querySelectorAll('.masonry-item').forEach(el => {
    const itemRect = el.getBoundingClientRect();
    const itemX = itemRect.left - rect.left + containerRef.value!.scrollLeft;
    const itemY = itemRect.top - rect.top + containerRef.value!.scrollTop;
    
    if (itemX < x2 && itemX + itemRect.width > x1 && itemY < y2 && itemY + itemRect.height > y1) {
      const id = el.getAttribute('data-id');
      if (id) newSelection.add(id);
    }
  });
  itemsInDragSelection.value = newSelection;
}

function handleMouseUp() {
  if (isDragSelecting.value && itemsInDragSelection.value.size > 0) {
    itemsInDragSelection.value.forEach(id => {
      if (!selected.value.includes(id)) selected.value.push(id);
    });
    emit('select', new Set(selected.value));
  }
  isDragSelecting.value = false;
  itemsInDragSelection.value.clear();
}

// Item click
function handleItemClick(item: Item, e: MouseEvent) {
  if (e.ctrlKey || e.metaKey) {
    toggleSelect(item.id);
    return;
  }
  if (e.shiftKey && selected.value.length > 0) {
    const lastIdx = props.items.findIndex(i => i.id === selected.value[selected.value.length - 1]);
    const currIdx = props.items.findIndex(i => i.id === item.id);
    if (lastIdx !== -1 && currIdx !== -1) {
      const [start, end] = [Math.min(lastIdx, currIdx), Math.max(lastIdx, currIdx)];
      for (let i = start; i <= end; i++) {
        if (!selected.value.includes(props.items[i].id)) {
          selected.value.push(props.items[i].id);
        }
      }
      emit('select', new Set(selected.value));
      return;
    }
  }
  emit('itemClick', item);
}

function handleItemDblClick(item: Item) {
  emit('itemDblClick', item);
}

function handleContextMenu(item: Item, e: MouseEvent) {
  emit('contextMenu', item, e);
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onItemResize);
  document.removeEventListener('mouseup', stopItemResize);
  document.removeEventListener('touchmove', onItemResizeTouch);
  document.removeEventListener('touchend', stopItemResizeTouch);
});
</script>

<style scoped>
.masonry-container {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 16px;
  position: relative;
  -webkit-overflow-scrolling: touch;
}

/* Selection Menu */
.selection-menu {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  margin: -16px -16px 16px -16px;
  background: rgba(26, 16, 48, 0.95);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255,255,255,0.1);
}

.selection-info {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.selection-count {
  font-size: 20px;
  font-weight: 700;
  color: #00ff87;
}

.selection-label {
  font-size: 14px;
  color: rgba(255,255,255,0.6);
}

.selection-actions {
  display: flex;
  gap: 8px;
}

.sel-btn {
  width: 36px;
  height: 36px;
  padding: 8px;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 8px;
  color: rgba(255,255,255,0.7);
  cursor: pointer;
}

.sel-btn:hover {
  background: rgba(255,255,255,0.1);
  color: #fff;
}

.sel-btn.danger:hover {
  background: rgba(255,59,48,0.2);
  color: #ff3b30;
}

.sel-btn svg {
  width: 100%;
  height: 100%;
}

/* Masonry Grid */
.masonry-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: flex-start;
  align-content: flex-start;
}

/* Masonry Item */
.masonry-item {
  position: relative;
  border-radius: 12px;
  overflow: hidden;
  background: rgba(37,24,66,0.6);
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  flex-shrink: 0;
}

.masonry-item:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,0.4), 0 0 0 1px rgba(255,255,255,0.1);
  z-index: 5;
}

.masonry-item.selected {
  box-shadow: 0 0 0 3px #00ff87, 0 8px 24px rgba(0,255,135,0.3);
}

.masonry-item.in-drag-select {
  box-shadow: 0 0 0 2px #00d4ff, 0 4px 16px rgba(0,212,255,0.3);
}

.masonry-item.is-resizing {
  z-index: 100;
  box-shadow: 0 0 0 2px #ffd000, 0 8px 32px rgba(255,208,0,0.4);
}

.masonry-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  user-select: none;
  -webkit-user-drag: none;
}

/* Item Resize Handle */
.item-resize {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 28px;
  height: 28px;
  background: rgba(0,0,0,0.7);
  border-top-left-radius: 8px;
  color: rgba(255,255,255,0.6);
  cursor: nwse-resize;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s;
  z-index: 10;
  touch-action: none;
}

.masonry-item:hover .item-resize {
  opacity: 1;
}

.item-resize:hover {
  background: rgba(255,208,0,0.4);
  color: #ffd000;
}

.item-resize:active {
  background: rgba(255,208,0,0.6);
}

.item-resize svg {
  width: 14px;
  height: 14px;
  pointer-events: none;
}

/* Selection Checkbox */
.select-btn {
  position: absolute;
  top: 10px;
  left: 10px;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 2px solid rgba(255,255,255,0.5);
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s;
  padding: 0;
  cursor: pointer;
}

.masonry-item:hover .select-btn,
.select-btn.active {
  opacity: 1;
}

.select-btn.active {
  background: #00ff87;
  border-color: #00ff87;
}

.select-btn svg {
  width: 14px;
  height: 14px;
  color: #fff;
}

/* Folder Badge */
.folder-badge {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: linear-gradient(to top, rgba(0,0,0,0.8), rgba(0,0,0,0.2));
  gap: 8px;
}

.folder-badge svg {
  width: 40px;
  height: 40px;
  color: #ffd000;
}

.folder-name {
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  text-align: center;
}

.folder-count {
  font-size: 12px;
  color: rgba(255,255,255,0.6);
}

/* Tag Indicator */
.tag-indicator {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid rgba(255,255,255,0.3);
}

/* Hover Info */
.item-info {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 12px;
  background: linear-gradient(to top, rgba(0,0,0,0.85), transparent);
  opacity: 0;
  transition: opacity 0.2s;
}

.masonry-item:hover .item-info {
  opacity: 1;
}

.item-name {
  font-size: 13px;
  font-weight: 500;
  color: #fff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Global Resize Control */
.global-resize {
  position: fixed;
  bottom: 100px;
  right: 20px;
  z-index: 30;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: rgba(26,16,48,0.95);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.4);
}

.global-resize button {
  width: 28px;
  height: 28px;
  padding: 0;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 6px;
  color: rgba(255,255,255,0.7);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.global-resize button:hover {
  background: rgba(255,255,255,0.1);
  color: #fff;
}

.global-resize input[type="range"] {
  width: 100px;
  height: 4px;
  -webkit-appearance: none;
  background: rgba(255,255,255,0.1);
  border-radius: 2px;
}

.global-resize input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  background: #00ff87;
  border-radius: 50%;
  cursor: pointer;
}

.global-resize span {
  font-size: 12px;
  color: rgba(255,255,255,0.6);
  min-width: 45px;
  text-align: right;
}

/* Selection Box */
.selection-box {
  position: absolute;
  border: 2px solid #00d4ff;
  background: rgba(0,212,255,0.1);
  border-radius: 4px;
  pointer-events: none;
  z-index: 100;
}

/* Transitions */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.2s;
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}

/* Mobile */
@media (max-width: 768px) {
  .masonry-container {
    padding: 12px;
  }
  
  .global-resize {
    bottom: 110px;
    right: 12px;
    padding: 8px 10px;
  }
  
  .global-resize input[type="range"] {
    width: 60px;
  }
  
  .global-resize span {
    display: none;
  }
}

/* Touch devices */
@media (hover: none) {
  .select-btn {
    opacity: 1;
  }
  
  .item-resize {
    opacity: 0.8;
    width: 32px;
    height: 32px;
  }
  
  .item-info {
    opacity: 1;
  }
}
</style>
