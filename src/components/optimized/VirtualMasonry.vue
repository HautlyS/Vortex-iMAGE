<template>
  <div class="masonry-wrapper">
    <div 
      ref="containerRef" 
      class="virtual-masonry"
      :style="{ height: height ? `${height}px` : '100%' }"
      @scroll="onScroll"
      @mousemove="onDragMove"
      @mouseup="onDragEnd"
      @mouseleave="onDragEnd"
    >
      <div class="masonry-content" :style="{ height: `${totalHeight}px` }">
        <!-- Drop indicator line -->
        <div v-if="isDragging && dropIndex >= 0" class="drop-indicator" :style="getDropIndicatorStyle()" />
        
        <div
          v-for="item in visibleItems"
          :key="item.id"
          class="masonry-item cursor-target"
          :class="[
            `item-${item.type}`, 
            { 'item-highlight': item.highlight, 'item-grouped': item.group },
            { 'dragging': dragItem?.id === item.id }
          ]"
          :style="getDragStyle(item)"
          @click="onItemClick(item)"
          @dblclick="$emit('item-dbl-click', item.data)"
          @mousedown="onDragStart($event, item)"
        >
          <img
            v-if="item.visible"
            :src="item.data.img"
            :alt="item.data.folderName || 'Image'"
            class="masonry-img"
            loading="lazy"
            decoding="async"
            draggable="false"
          />
          <div v-else class="placeholder" />
          
          <div v-if="item.data.isFolder" class="folder-overlay">
            <svg class="folder-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
            </svg>
            <span class="folder-name">{{ item.data.folderName }}</span>
            <span class="folder-count">{{ item.data.photoCount }} fotos</span>
          </div>
          
          <template v-else>
            <div v-if="item.group" class="group-badge">{{ item.group.split('-')[1] }}</div>
            
            <button class="btn-fav" :class="{ active: favorites.has(item.id) }" @click.stop="toggleFav(item.id)">
              <svg viewBox="0 0 24 24" :fill="favorites.has(item.id) ? '#ef4444' : 'none'" stroke="currentColor" stroke-width="2">
                <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
              </svg>
            </button>
            <button class="btn-sel" :class="{ active: selected.has(item.id) }" @click.stop="toggleSel(item.id)">
              <svg v-if="selected.has(item.id)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <path d="M5 13l4 4L19 7"/>
              </svg>
            </button>
            
            <div class="resize-handle" @mousedown.stop="startResize($event, item)" @touchstart.stop.prevent="startTouchResize($event, item)">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <circle cx="18" cy="18" r="2"/><circle cx="12" cy="18" r="2"/><circle cx="18" cy="12" r="2"/>
              </svg>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, shallowRef, reactive } from 'vue'

interface MasonryItem {
  id: string
  img: string
  width?: number
  url?: string
  height: number
  size?: number
  date?: string | Date
  uploadedAt?: string | Date
  createdAt?: string | Date
  isFolder?: boolean
  folderName?: string
  photoCount?: number
  [key: string]: any
}

type SizeType = 'small' | 'medium' | 'large' | 'hero' | 'wide' | 'tall'
type SortType = 'newest' | 'oldest' | 'uploaded' | 'custom'

interface LayoutItem {
  id: string
  data: MasonryItem
  x: number
  y: number
  w: number
  h: number
  type: SizeType
  highlight: boolean
  group?: string
  visible: boolean
  scale: number
}

const props = withDefaults(defineProps<{
  items: MasonryItem[]
  height?: number
  baseWidth?: number
  baseHeight?: number
  gap?: number
  buffer?: number
}>(), {
  height: 0,
  baseWidth: 260,
  baseHeight: 300,
  gap: 2,
  buffer: 400
})

const emit = defineEmits<{
  'item-click': [item: MasonryItem]
  'item-dbl-click': [item: MasonryItem]
  'sort-change': [sort: SortType, desc: boolean]
  'reorder': [items: MasonryItem[]]
}>()

const containerRef = ref<HTMLElement>()
const scrollTop = ref(0)
const containerWidth = ref(0)
const containerHeight = ref(0)
const selected = ref(new Set<string>())
const favorites = ref(new Set<string>())
const customScales = reactive<Record<string, number>>({})
const customOrder = ref<string[]>([])

// Drag state
const dragItem = ref<LayoutItem | null>(null)
const dropIndex = ref<number>(-1)
const dragOffset = ref({ x: 0, y: 0 })
const dragPos = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const dragStartPos = ref({ x: 0, y: 0 })

const onDragStart = (e: MouseEvent, item: LayoutItem) => {
  if ((e.target as HTMLElement).closest('.resize-handle, .btn-fav, .btn-sel')) return
  
  dragItem.value = item
  dragStartPos.value = { x: e.clientX, y: e.clientY }
  dragOffset.value = { x: e.clientX - item.x, y: e.clientY - item.y + scrollTop.value }
  dragPos.value = { x: item.x, y: item.y }
  isDragging.value = false
}

const onDragMove = (e: MouseEvent) => {
  if (!dragItem.value) return
  
  const dx = Math.abs(e.clientX - dragStartPos.value.x)
  const dy = Math.abs(e.clientY - dragStartPos.value.y)
  
  if (dx > 8 || dy > 8) isDragging.value = true
  if (!isDragging.value) return
  
  dragPos.value = {
    x: e.clientX - dragOffset.value.x,
    y: e.clientY - dragOffset.value.y + scrollTop.value
  }
  
  // Find insertion position based on drag position
  const centerX = dragPos.value.x + dragItem.value.w / 2
  const centerY = dragPos.value.y + dragItem.value.h / 2
  
  // Find closest position in layout order
  let bestIdx = -1
  let minDist = Infinity
  
  for (let i = 0; i < layout.value.length; i++) {
    const item = layout.value[i]
    if (item.id === dragItem.value.id) continue
    
    // Check distance to item center
    const itemCenterX = item.x + item.w / 2
    const itemCenterY = item.y + item.h / 2
    const dist = Math.hypot(centerX - itemCenterX, centerY - itemCenterY)
    
    if (dist < minDist) {
      minDist = dist
      // Insert before or after based on position
      const insertBefore = centerY < itemCenterY || (Math.abs(centerY - itemCenterY) < item.h / 2 && centerX < itemCenterX)
      bestIdx = insertBefore ? i : i + 1
    }
  }
  
  dropIndex.value = bestIdx
}

const onDragEnd = () => {
  if (dragItem.value && isDragging.value && dropIndex.value >= 0) {
    const items = [...sortedItems.value]
    const fromIdx = items.findIndex(i => i.id === dragItem.value!.id)
    
    if (fromIdx !== -1 && dropIndex.value !== fromIdx) {
      // Remove from old position
      const [moved] = items.splice(fromIdx, 1)
      // Adjust target index if needed
      const targetIdx = dropIndex.value > fromIdx ? dropIndex.value - 1 : dropIndex.value
      // Insert at new position
      items.splice(targetIdx, 0, moved)
      
      customOrder.value = items.map(i => i.id)
      sortBy.value = 'custom'
      emit('reorder', items)
      calculateLayout()
    }
  }
  
  dragItem.value = null
  dropIndex.value = -1
  isDragging.value = false
}

const onItemClick = (item: LayoutItem) => {
  if (!isDragging.value) {
    emit('item-click', item.data)
  }
}

const getDragStyle = (item: LayoutItem) => {
  if (dragItem.value?.id === item.id && isDragging.value) {
    return {
      transform: `translate3d(${dragPos.value.x}px, ${dragPos.value.y}px, 0)`,
      width: `${item.w}px`,
      height: `${item.h}px`,
      zIndex: 1000,
      opacity: 0.85,
      pointerEvents: 'none' as const
    }
  }
  return getItemStyle(item)
}

const getDropIndicatorStyle = () => {
  if (dropIndex.value < 0 || !layout.value.length) return { display: 'none' }
  
  const targetItem = layout.value[Math.min(dropIndex.value, layout.value.length - 1)]
  if (!targetItem) return { display: 'none' }
  
  // Show indicator at the target position
  const isEnd = dropIndex.value >= layout.value.length
  return {
    left: `${targetItem.x}px`,
    top: `${isEnd ? targetItem.y + targetItem.h : targetItem.y}px`,
    width: `${targetItem.w}px`
  }
}

// Sorting
const sortBy = ref<SortType>('newest')
const sortDesc = ref(true)



const getDate = (item: MasonryItem, type: 'meta' | 'upload'): number => {
  const field = type === 'upload' ? (item.uploadedAt || item.createdAt) : (item.date || item.createdAt)
  if (!field) return 0
  return new Date(field).getTime()
}

const sortedItems = computed(() => {
  if (sortBy.value === 'custom' && customOrder.value.length) {
    const orderMap = new Map(customOrder.value.map((id, i) => [id, i]))
    return [...props.items].sort((a, b) => {
      const ai = orderMap.get(a.id) ?? Infinity
      const bi = orderMap.get(b.id) ?? Infinity
      return ai - bi
    })
  }
  if (sortBy.value === 'custom') return props.items
  const sorted = [...props.items].sort((a, b) => {
    const type = sortBy.value === 'uploaded' ? 'upload' : 'meta'
    return getDate(b, type) - getDate(a, type)
  })
  return sortDesc.value ? sorted : sorted.reverse()
})

const toggleSel = (id: string) => { const s = new Set(selected.value); s.has(id) ? s.delete(id) : s.add(id); selected.value = s }
const toggleFav = (id: string) => { const f = new Set(favorites.value); f.has(id) ? f.delete(id) : f.add(id); favorites.value = f }

// Size configs with weights
const sizeConfig: Record<SizeType, { scale: number; weight: number }> = {
  small: { scale: 0.8, weight: 0.30 },
  medium: { scale: 1.0, weight: 0.35 },
  large: { scale: 1.3, weight: 0.18 },
  hero: { scale: 1.8, weight: 0.07 },
  wide: { scale: 1.5, weight: 0.05 },
  tall: { scale: 1.3, weight: 0.05 }
}

const getQualityScore = (item: MasonryItem): number => {
  const pixels = (item.width || 800) * (item.height || 600)
  const mp = pixels / 1_000_000
  const fs = (item.size || 100000) / 1_000_000
  return Math.min(1, (mp * 0.3 + fs * 0.7) / 5)
}

const hash = (str: string, seed = 0): number => {
  let h = seed
  for (let i = 0; i < str.length; i++) h = ((h << 5) - h + str.charCodeAt(i)) | 0
  return Math.abs(h)
}

const getSizeType = (item: MasonryItem, index: number): { type: SizeType; highlight: boolean; group?: string } => {
  const quality = getQualityScore(item)
  const h = hash(item.id, index)
  const rand = (h % 1000) / 1000
  const boost = quality > 0.7 ? 0.15 : quality > 0.4 ? 0.05 : 0
  const adjusted = Math.min(1, rand + boost)
  
  let cumulative = 0, type: SizeType = 'medium'
  for (const [t, cfg] of Object.entries(sizeConfig)) {
    cumulative += cfg.weight
    if (adjusted <= cumulative) { type = t as SizeType; break }
  }
  
  if (quality > 0.85 && index % 12 === 0) type = 'hero'
  
  const aspect = (item.width || 800) / (item.height || 600)
  if (aspect > 1.6 && type === 'medium') type = 'wide'
  if (aspect < 0.65 && type === 'medium') type = 'tall'
  
  const highlight = type === 'hero'
  const group = (type === 'small' && index % 6 < 2) ? `g-${Math.floor(index / 6)}` : undefined
  
  return { type, highlight, group }
}

const layout = shallowRef<LayoutItem[]>([])
const totalHeight = ref(0)

const calculateLayout = () => {
  const cw = containerWidth.value
  if (!cw || !sortedItems.value.length) { layout.value = []; totalHeight.value = 0; return }

  const cols = cw >= 1400 ? 5 : cw >= 1000 ? 4 : cw >= 650 ? 3 : 2
  const gap = props.gap
  const baseColW = Math.floor((cw - (cols - 1) * gap) / cols)
  const colHeights = new Array(cols).fill(0)
  const items: LayoutItem[] = []

  for (let i = 0; i < sortedItems.value.length; i++) {
    const item = sortedItems.value[i]
    const { type, highlight, group } = getSizeType(item, i)
    const customScale = customScales[item.id]
    const typeScale = sizeConfig[type].scale
    
    const aspect = (item.width || props.baseWidth) / (item.height || props.baseHeight)
    let span = 1, w: number, h: number

    // Dynamic sizing based on type
    if (type === 'hero') {
      span = Math.min(2, cols)
      w = baseColW * span + (span - 1) * gap
      h = Math.round(w / Math.max(aspect, 1.1))
    } else if (type === 'wide') {
      span = Math.min(2, cols)
      w = baseColW * span + (span - 1) * gap
      h = Math.round(w / Math.max(aspect, 1.3))
    } else if (type === 'tall') {
      w = baseColW
      h = Math.round(w / Math.min(aspect, 0.55))
    } else if (type === 'large') {
      w = baseColW
      h = Math.round((w / aspect) * typeScale)
    } else if (type === 'small') {
      w = baseColW
      h = Math.round((w / aspect) * typeScale)
    } else {
      w = baseColW
      h = Math.round(w / aspect)
    }

    // Apply custom scale if resized
    if (customScale) {
      w = Math.round(baseColW * customScale)
      h = Math.round(w / aspect)
      span = Math.ceil(w / (baseColW + gap))
    }

    w = Math.min(w, cw)
    span = Math.min(span, cols)

    // Find best position for multi-span items
    let col = 0, minH = Infinity
    for (let c = 0; c <= cols - span; c++) {
      const maxH = Math.max(...colHeights.slice(c, c + span))
      if (maxH < minH) { minH = maxH; col = c }
    }

    // Grouped items try to stay together
    if (group && !customScale) {
      const groupCol = hash(group) % (cols - span + 1)
      if (colHeights[groupCol] <= minH + 100) col = groupCol
    }

    const x = col * (baseColW + gap)
    const y = Math.max(...colHeights.slice(col, col + span))

    items.push({ id: item.id, data: item, x, y, w, h, type, highlight, group, visible: false, scale: customScale || typeScale })

    // Update all spanned columns
    const newH = y + h + gap
    for (let c = col; c < col + span; c++) colHeights[c] = newH
  }

  layout.value = items
  totalHeight.value = Math.max(...colHeights) - gap
}

const visibleItems = computed(() => {
  const top = scrollTop.value - props.buffer
  const bottom = scrollTop.value + containerHeight.value + props.buffer
  return layout.value
    .filter(item => (item.y + item.h) >= top && item.y <= bottom)
    .map(item => ({ ...item, visible: true }))
})

const getItemStyle = (item: LayoutItem) => ({
  transform: `translate3d(${item.x}px, ${item.y}px, 0)`,
  width: `${item.w}px`,
  height: `${item.h}px`,
  zIndex: item.highlight ? 5 : item.type === 'hero' ? 4 : item.type === 'large' ? 3 : 1
})

let resizeItem: LayoutItem | null = null
let resizeStart = { x: 0, w: 0 }

const startResize = (e: MouseEvent, item: LayoutItem) => {
  resizeItem = item
  resizeStart = { x: e.clientX, w: item.w }
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'nwse-resize'
}

const startTouchResize = (e: TouchEvent, item: LayoutItem) => {
  resizeItem = item
  resizeStart = { x: e.touches[0].clientX, w: item.w }
  document.addEventListener('touchmove', onTouchResize, { passive: false })
  document.addEventListener('touchend', stopResize)
}

const onResize = (e: MouseEvent) => {
  if (!resizeItem) return
  const dx = e.clientX - resizeStart.x
  customScales[resizeItem.id] = Math.max(0.5, Math.min(2.5, (resizeStart.w + dx) / props.baseWidth))
  sortBy.value = 'custom'
  calculateLayout()
}

const onTouchResize = (e: TouchEvent) => {
  e.preventDefault()
  if (!resizeItem) return
  const dx = e.touches[0].clientX - resizeStart.x
  customScales[resizeItem.id] = Math.max(0.5, Math.min(2.5, (resizeStart.w + dx) / props.baseWidth))
  sortBy.value = 'custom'
  calculateLayout()
}

const stopResize = () => {
  resizeItem = null
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
  document.removeEventListener('touchmove', onTouchResize)
  document.removeEventListener('touchend', stopResize)
  document.body.style.cursor = ''
}

let rafId = 0
const onScroll = () => {
  cancelAnimationFrame(rafId)
  rafId = requestAnimationFrame(() => { scrollTop.value = containerRef.value?.scrollTop || 0 })
}

let ro: ResizeObserver | null = null

onMounted(() => {
  if (!containerRef.value) return
  containerWidth.value = containerRef.value.clientWidth
  containerHeight.value = containerRef.value.clientHeight
  ro = new ResizeObserver(([e]) => {
    const { width, height } = e.contentRect
    if (width !== containerWidth.value || height !== containerHeight.value) {
      containerWidth.value = width
      containerHeight.value = height
      calculateLayout()
    }
  })
  ro.observe(containerRef.value)
  calculateLayout()
})

onUnmounted(() => { ro?.disconnect(); cancelAnimationFrame(rafId); stopResize() })

watch([sortedItems, sortBy, sortDesc], calculateLayout, { deep: true })
</script>

<style scoped>
.masonry-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
}

.virtual-masonry {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: thin;
}

.masonry-content { position: relative; width: 100%; }

.drop-indicator {
  position: absolute;
  height: 4px;
  background: var(--accent-color, #00f0ff);
  border-radius: 2px;
  z-index: 999;
  pointer-events: none;
  box-shadow: 0 0 12px var(--accent-color, #00f0ff), 0 0 24px var(--accent-color, #00f0ff);
  animation: pulse-glow 0.8s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { opacity: 1; transform: scaleX(1); }
  50% { opacity: 0.7; transform: scaleX(0.98); }
}

.masonry-item {
  position: absolute;
  top: 0;
  left: 0;
  will-change: transform;
  overflow: hidden;
  cursor: grab;
  user-select: none;
  -webkit-user-drag: none;
  transition: box-shadow 0.15s, border-color 0.15s, opacity 0.15s;
}

.masonry-item:active { cursor: grabbing; }
.masonry-item:hover { z-index: 20 !important; }

.masonry-item.dragging {
  cursor: grabbing;
  box-shadow: 0 25px 50px rgba(0,0,0,0.6) !important;
  border: 3px solid var(--accent-color, #00f0ff) !important;
  z-index: 1000 !important;
}

/* Dynamic size styles */
.item-small { opacity: 0.95; }

.item-large { box-shadow: 0 4px 20px rgba(0,0,0,0.3); }
.item-hero { 
  box-shadow: 0 6px 30px rgba(0,0,0,0.4), 0 0 15px rgba(var(--accent-rgb, 0,240,255), 0.2);
  border: 2px solid var(--accent-color, #00f0ff);
}
.item-wide { box-shadow: 0 4px 16px rgba(0,0,0,0.25); }
.item-tall { box-shadow: 0 4px 20px rgba(0,0,0,0.25); }

.item-highlight { 
  box-shadow: 0 8px 32px rgba(var(--accent-rgb, 0,240,255), 0.35), 0 4px 16px rgba(0,0,0,0.3) !important;
}

.item-grouped { 
  border: 1px solid rgba(var(--accent-rgb, 0,240,255), 0.25);
}

.masonry-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  pointer-events: none;
  user-select: none;
  -webkit-user-drag: none;
  transition: transform 0.2s ease;
}

.masonry-item:hover .masonry-img { transform: scale(1.03); }

.placeholder {
  width: 100%;
  height: 100%;
  background: rgba(255,255,255,0.05);
}

.group-badge {
  position: absolute;
  bottom: 8px;
  left: 8px;
  width: 20px;
  height: 20px;
  background: rgba(var(--accent-rgb, 0,240,255), 0.9);
  color: #000;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
}

.folder-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.9), rgba(0,0,0,0.2));
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: white;
  gap: 4px;
}

.folder-icon { width: 32px; height: 32px; color: var(--accent-color, #00f0ff); }
.folder-name { font-weight: 600; font-size: 14px; }
.folder-count { font-size: 12px; opacity: 0.7; }

.btn-fav, .btn-sel {
  position: absolute;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 2px solid rgba(255,255,255,0.7);
  background: rgba(0,0,0,0.4);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s, transform 0.15s;
  backdrop-filter: blur(4px);
}

.btn-fav { top: 8px; left: 8px; }
.btn-sel { top: 8px; right: 8px; }
.btn-fav svg, .btn-sel svg { width: 12px; height: 12px; }

.masonry-item:hover .btn-fav,
.masonry-item:hover .btn-sel,
.btn-fav.active, .btn-sel.active { opacity: 1; }

.btn-fav:hover, .btn-sel:hover { transform: scale(1.1); }
.btn-sel.active { background: #3b82f6; border-color: #3b82f6; }

.resize-handle {
  position: absolute;
  bottom: 4px;
  right: 4px;
  width: 18px;
  height: 18px;
  cursor: nwse-resize;
  opacity: 0;
  transition: opacity 0.15s;
  color: rgba(255,255,255,0.8);
  background: rgba(0,0,0,0.5);
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.resize-handle svg { width: 12px; height: 12px; }
.masonry-item:hover .resize-handle { opacity: 0.7; }
.resize-handle:hover { opacity: 1 !important; color: var(--accent-color, #00f0ff); }

</style>
