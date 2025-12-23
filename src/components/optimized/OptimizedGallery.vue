<template>
  <div class="optimized-gallery">
    <!-- Dynamic Masonry Gallery -->
    <VirtualMasonry
      :items="enhancedItems"
      :height="galleryHeight"
      :baseWidth="baseItemWidth"
      :baseHeight="baseItemHeight"
      @item-click="handleItemClick"
      @item-dbl-click="handleItemDblClick"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import VirtualMasonry from './VirtualMasonry.vue'

interface GalleryItem {
  id: string
  img: string
  url: string
  height: number
  width?: number
  isFolder: boolean
  folderName?: string
  photoCount?: number
  photo?: any
  album?: any
  name?: string
  date?: Date
  size?: number
}

interface Props {
  items: GalleryItem[]
  galleryHeight?: number
}

const props = withDefaults(defineProps<Props>(), {
  galleryHeight: 600
})
const emit = defineEmits<{
  'item-click': [item: GalleryItem]
  'item-dbl-click': [item: GalleryItem]
}>()

const handleItemClick = (item: any) => {
  emit('item-click', item as GalleryItem)
}

const handleItemDblClick = (item: any) => {
  emit('item-dbl-click', item as GalleryItem)
}

// Layout settings
const baseSizeMode = ref<'compact' | 'normal' | 'large'>('normal')
const layoutSeed = ref(Date.now())

const baseSizes = {
  compact: { width: 240, height: 280 },
  normal: { width: 280, height: 320 },
  large: { width: 320, height: 380 }
}

const baseItemWidth = computed(() => baseSizes[baseSizeMode.value].width)
const baseItemHeight = computed(() => baseSizes[baseSizeMode.value].height)

const enhancedItems = computed(() => {
  return props.items.map((item, index) => {
    const hash = (item.id + layoutSeed.value.toString()).split('').reduce((a, c) => a + c.charCodeAt(0), 0)
    const random = (hash + index) % 1000 / 1000
    
    let aspectRatio = 1
    if (item.width && item.height) {
      aspectRatio = item.width / item.height
    } else {
      aspectRatio = 0.7 + (random * 0.8)
    }
    
    const sizeVariation = 0.7 + (random * 0.3)
    
    return {
      ...item,
      width: Math.round(baseItemWidth.value * aspectRatio * sizeVariation),
      height: Math.round(baseItemHeight.value * sizeVariation),
    }
  })
})
</script>

<style scoped>
.optimized-gallery {
  position: relative;
  width: 100%;
  height: 100%;
}
</style>
