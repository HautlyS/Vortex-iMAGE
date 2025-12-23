<template>
  <div 
    ref="containerRef" 
    class="relative w-full"
    :style="{ height: `${totalHeight}px` }"
    @mousedown="startDragSelect"
    @touchstart="handleTouchStart"
    @touchmove="handleTouchMove"
  >
    <!-- Visible Items -->
    <div
      v-for="item in visibleGrid"
      :key="item.id"
      :data-key="item.id"
      class="absolute box-content group"
      :style="{ 
        transform: `translate(${item.x}px, ${item.y}px)`,
        width: `${item.w}px`,
        height: `${item.h}px`,
        willChange: 'transform, opacity'
      }"
      @click.stop="handleItemClick(item)"
      @dblclick.stop="handleItemDblClick(item)"
      @touchend="handleTouchEnd(item, $event)"
      @mouseenter="e => handleMouseEnter(item.id, e.currentTarget as HTMLElement)"
      @mouseleave="e => handleMouseLeave(item.id, e.currentTarget as HTMLElement)"
      @contextmenu.prevent="handleContextMenu(item, $event)"
    >
      <div
        class="relative w-full h-full bg-cover bg-center rounded-[12px] overflow-hidden transition-all duration-300"
        :class="isSelected(item.id) ? 'ring-4 ring-offset-2 ring-primary ring-offset-black/50 scale-[0.98]' : 'hover:scale-[1.02] shadow-2xl'"
        :style="{ backgroundImage: `url(${item.img})`, backgroundColor: '#1a1a20' }"
      >
        <!-- Selection Checkbox -->
        <button
          v-if="!item.isFolder"
          class="absolute top-2 right-2 w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all duration-200 z-10"
          :class="[
            isSelected(item.id) 
            ? 'bg-blue-500 border-blue-500' 
            : 'border-white/50 bg-black/20 opacity-0 group-hover:opacity-100 hover:border-white hover:bg-black/40'
          ]"
          @click.stop="toggleSelect(item.id)"
        >
          <svg v-if="isSelected(item.id)" class="w-3.5 h-3.5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <path d="M5 13l4 4L19 7"/>
          </svg>
        </button>

        <!-- Folder overlay -->
        <div v-if="item.isFolder" class="folder-overlay">
          <div class="folder-icon">
             <svg viewBox="0 0 24 24" fill="currentColor">
               <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
             </svg>
          </div>
          <div class="folder-info">
            <span class="folder-name">{{ item.folderName }}</span>
            <span class="folder-count">{{ item.photoCount }} items</span>
          </div>
        </div>
        
        <!-- Tag Glow Overlay -->
        <div v-if="item.tagColor" 
             class="absolute inset-0 rounded-[10px] pointer-events-none transition-all duration-300"
             :style="{ 
               boxShadow: `inset 0 0 0 2px ${item.tagColor}, inset 0 0 20px ${item.tagColor}40`
             }"
        ></div>

        <!-- Hover Overlay -->
        <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none" />
        
        <!-- Name Label -->
        <div v-if="!item.isFolder" class="absolute bottom-3 left-3 right-3 opacity-0 group-hover:opacity-100 transition-opacity duration-300 z-10">
           <span class="text-white text-sm font-medium truncate block drop-shadow-md">{{ (item as any).name || 'Photo' }}</span>
        </div>
      </div>
    </div>

    <!-- Selection Box -->
    <div 
      v-if="isSelecting"
      class="fixed border-2 border-blue-500 bg-blue-500/20 z-50 pointer-events-none rounded-lg backdrop-blur-sm"
      :style="{
        left: `${selectionBox.x}px`,
        top: `${selectionBox.y}px`,
        width: `${selectionBox.w}px`,
        height: `${selectionBox.h}px`
      }"
    />
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

interface MasonryProps {
  items: Item[];
  scaleOnHover?: boolean;
  animateFrom?: string;
}

const props = withDefaults(defineProps<MasonryProps>(), {
  scaleOnHover: true,
  animateFrom: 'bottom'
});

interface GridItem extends Item {
  x: number;
  y: number;
  w: number;
  h: number;
}

const emit = defineEmits<{
  itemClick: [item: GridItem];
  itemDblClick: [item: GridItem];
  contextMenu: [item: GridItem, event: MouseEvent];
  select: [ids: Set<string>];
}>()

const selected = ref<string[]>([])
const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(window.innerHeight)
const containerWidth = ref(0)

// Config
const COLUMN_WIDTH = 300 // Target width
const GAP = 16

// Measure container
const resizeObserver = new ResizeObserver((entries) => {
  for (const entry of entries) {
    containerWidth.value = entry.contentRect.width
  }
})

// Scroll listener
const handleScroll = (e: Event) => {
  scrollTop.value = (e.target as HTMLElement).scrollTop
}

onMounted(() => {
  if (containerRef.value) {
    resizeObserver.observe(containerRef.value)
    // Find scrolling parent
    let parent = containerRef.value.parentElement
    while (parent) {
      if (getComputedStyle(parent).overflowY === 'auto' || getComputedStyle(parent).overflowY === 'scroll') {
        parent.addEventListener('scroll', handleScroll, { passive: true })
        // Initial values
        scrollTop.value = parent.scrollTop
        viewportHeight.value = parent.clientHeight
        break
      }
      parent = parent.parentElement
    }
  }
  window.addEventListener('resize', () => { viewportHeight.value = window.innerHeight })
})

onUnmounted(() => {
    if (containerRef.value) resizeObserver.unobserve(containerRef.value)
})

// Compute Grid Layout (Full Calculation)
const layout = computed(() => {
  const width = containerWidth.value
  if (!width) return { items: [], totalHeight: 0 }

  const numColumns = Math.max(1, Math.floor((width + GAP) / (COLUMN_WIDTH + GAP)))
  const actualColumnWidth = (width - (numColumns - 1) * GAP) / numColumns
  
  const colHeights = new Array(numColumns).fill(0)
  const itemsWithPos = props.items.map(item => {
    const colIndex = colHeights.indexOf(Math.min(...colHeights))
    const finalHeight = item.height
    
    const x = colIndex * (actualColumnWidth + GAP)
    const y = colHeights[colIndex]
    
    colHeights[colIndex] += finalHeight + GAP
    
    return {
      ...item,
      x,
      y,
      w: actualColumnWidth,
      h: finalHeight
    } as GridItem
  })
  
  return {
    items: itemsWithPos,
    totalHeight: Math.max(...colHeights)
  }
})

const totalHeight = computed(() => layout.value.totalHeight)

// Virtualization
const BUFFER = 600
const visibleGrid = computed(() => {
  const start = scrollTop.value - BUFFER
  const end = scrollTop.value + viewportHeight.value + BUFFER
  
  // Also optimize: only slice items that are likely in range if sorted by y?
  // Since items are not strictly sorted by y (columns mix), filter is necessary.
  return layout.value.items.filter(item => {
    return (item.y + item.h > start) && (item.y < end)
  })
})

// Selection Logic
const isSelected = (id: string) => selected.value.includes(id)

const toggleSelect = (id: string) => {
  const idx = selected.value.indexOf(id)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(id)
  emit('select', new Set(selected.value))
}

// Drag Selection
const isSelecting = ref(false)
const selectionStart = ref({ x: 0, y: 0 })
const selectionBox = ref({ x: 0, y: 0, w: 0, h: 0 })

function startDragSelect(e: MouseEvent) {
  if (e.button !== 0) return 
  
  isSelecting.value = true
  selectionStart.value = { x: e.clientX, y: e.clientY }
  selectionBox.value = { x: e.clientX, y: e.clientY, w: 0, h: 0 }
  
  document.addEventListener('mousemove', onDragSelect)
  document.addEventListener('mouseup', stopDragSelect)
  document.body.style.userSelect = 'none'
}

function onDragSelect(e: MouseEvent) {
  const currentX = e.clientX
  const currentY = e.clientY
  
  const x = Math.min(selectionStart.value.x, currentX)
  const y = Math.min(selectionStart.value.y, currentY)
  const w = Math.abs(currentX - selectionStart.value.x)
  const h = Math.abs(currentY - selectionStart.value.y)
  
  selectionBox.value = { x, y, w, h }
  

  const containerRect = containerRef.value?.getBoundingClientRect()
  if (!containerRect) return
  
  const newSelected: string[] = []
  
  const relBoxCorrect = {
      left: x - containerRect.left,
      right: x + w - containerRect.left,
      top: y - containerRect.top,
      bottom: y + h - containerRect.top
  } 

  layout.value.items.forEach(item => {
      const left = item.x
      const right = item.x + item.w
      const top = item.y
      const bottom = item.y + item.h
      
      const intersects = (left < relBoxCorrect.right && right > relBoxCorrect.left && top < relBoxCorrect.bottom && bottom > relBoxCorrect.top)
      
      if (intersects) {
           if (!newSelected.includes(item.id)) newSelected.push(item.id)
      }
  })
  
  if (newSelected.length > 0) {
      selected.value = newSelected
      emit('select', new Set(selected.value))
  }
}

function stopDragSelect() {
  isSelecting.value = false
  document.removeEventListener('mousemove', onDragSelect)
  document.removeEventListener('mouseup', stopDragSelect)
  document.body.style.userSelect = ''
}

// Touch boilerplate
const handleTouchStart = (e: TouchEvent) => { void e }
const handleTouchMove = (e: TouchEvent) => { void e }
const handleTouchEnd = (item: any, e: TouchEvent) => { void item; void e }

// Animations removed to fix lint errors and simplify virtualization behavior
watch(visibleGrid, () => {})

// --- Interaction Handlers ---
const handleItemClick = (item: GridItem) => emit('itemClick', item);
const handleItemDblClick = (item: GridItem) => emit('itemDblClick', item);
const handleContextMenu = (item: GridItem, event: MouseEvent) => emit('contextMenu', item, event);

const handleMouseEnter = (id: string, el: HTMLElement) => {
    void id; void el;
}
const handleMouseLeave = (id: string, el: HTMLElement) => {
    void id; void el;
}
</script>

<style scoped>
.folder-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.8), transparent);
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  padding: 1rem;
  pointer-events: none;
}
.folder-icon {
  position: absolute;
  top: 10px;
  right: 10px;
  background: rgba(255,255,255,0.2);
  backdrop-filter: blur(8px);
  padding: 8px;
  border-radius: 8px;
  color: white;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.folder-info {
  color: white;
}
.folder-name {
  font-weight: 700;
  font-size: 1.1rem;
  text-shadow: 0 2px 4px rgba(0,0,0,0.5);
}
.folder-count {
  font-size: 0.8rem;
  opacity: 0.8;
}
</style>
