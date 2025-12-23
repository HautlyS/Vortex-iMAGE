/**
 * TypeScript Module - 4 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed, watch } from 'vue'

export interface ColorTag {
  id: string
  name: string
  color: string
}

export type ColorTagDefinition = ColorTag

export const PREDEFINED_COLORS: ColorTag[] = [
  { id: 'red', name: 'Vermelho', color: '#ef4444' },
  { id: 'orange', name: 'Laranja', color: '#f97316' },
  { id: 'yellow', name: 'Amarelo', color: '#eab308' },
  { id: 'green', name: 'Verde', color: '#22c55e' },
  { id: 'blue', name: 'Azul', color: '#3b82f6' },
  { id: 'purple', name: 'Roxo', color: '#a855f7' },
  { id: 'pink', name: 'Rosa', color: '#ec4899' },
  { id: 'gray', name: 'Cinza', color: '#6b7280' },
]

export const COLORS = PREDEFINED_COLORS

const photoTags = ref<Record<string, string>>({})
const isLoaded = ref(false)
let store: any = null

export function useColorTagStore() {
  const tagCounts = computed(() => {
    const counts: Record<string, number> = {}
    for (const colorId of Object.values(photoTags.value)) {
      counts[colorId] = (counts[colorId] || 0) + 1
    }
    return counts
  })

  const usedTags = computed(() => {
    const usedIds = new Set(Object.values(photoTags.value))
    return COLORS.filter(c => usedIds.has(c.id))
  })

  watch(photoTags, async () => {
    if (isLoaded.value && store) {
      await store.set('photoTags', photoTags.value)
      await store.save()
    }
  }, { deep: true })

  async function init() {
    if (isLoaded.value) return
    
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      store = await Store.load('color-tags.json')
      const saved = await store.get('photoTags') as Record<string, string> | null
      if (saved) photoTags.value = saved
    } catch (e) {
      console.error('Failed to load color tags store:', e)
    }
    isLoaded.value = true
  }

  const tagPhoto = (photoId: string, colorId: string) => { photoTags.value[photoId] = colorId }
  const removeTag = (photoId: string) => { delete photoTags.value[photoId] }
  const getPhotoTag = (photoId: string) => COLORS.find(c => c.id === photoTags.value[photoId]) || null
  const getPhotosByTag = (tagId: string) => Object.entries(photoTags.value).filter(([, c]) => c === tagId).map(([id]) => id)
  const clearAllTags = () => { photoTags.value = {} }
  const renameTag = () => {} 
  const tagItems = (ids: string[], colorId: string) => ids.forEach(id => { photoTags.value[id] = colorId })
  const getItemsByTag = getPhotosByTag
  const loadTags = init

  return {
    photoTags,
    isLoaded,
    tagCounts,
    usedTags,
    init,
    loadTags,
    tagPhoto,
    tagItems,
    removeTag,
    getPhotoTag,
    getPhotosByTag,
    getItemsByTag,
    clearAllTags,
    renameTag,
    COLORS
  }
}

export const useColorTags = useColorTagStore