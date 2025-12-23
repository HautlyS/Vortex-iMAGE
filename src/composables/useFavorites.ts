/**
 * TypeScript Module - 4 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed } from 'vue'

export interface FavoriteItem {
  type: 'photo' | 'album'
  id: string
  path: string
  addedAt: number
}

const favorites = ref<FavoriteItem[]>([])
let initialized = false

export function toggleFavoriteItem(
  item: { type: 'photo' | 'album'; id: string; path: string },
  currentFavorites: FavoriteItem[]
): FavoriteItem[] {
  const idx = currentFavorites.findIndex(f => f.id === item.id)
  if (idx !== -1) return currentFavorites.filter((_, i) => i !== idx)
  return [...currentFavorites, { ...item, addedAt: Date.now() }]
}

export function groupFavoritesByType(items: FavoriteItem[]) {
  const photos: FavoriteItem[] = []
  const albums: FavoriteItem[] = []
  for (const item of items) {
    (item.type === 'photo' ? photos : albums).push(item)
  }
  return { photos, albums }
}

export function isItemFavorite(id: string, list: FavoriteItem[]): boolean {
  return list.some(f => f.id === id)
}

export function useFavorites() {
  const favoritePhotos = computed(() => favorites.value.filter(f => f.type === 'photo'))
  const favoriteAlbums = computed(() => favorites.value.filter(f => f.type === 'album'))
  const favoriteCount = computed(() => favorites.value.length)

  async function loadFavorites(): Promise<void> {
    if (initialized) return
    try {
      const { load } = await import('@tauri-apps/plugin-store')
      const store = await load('settings.json')
      const saved = await store.get<FavoriteItem[]>('favorites')
      if (Array.isArray(saved)) favorites.value = saved
    } catch {}
    initialized = true
  }

  async function saveFavorites(): Promise<void> {
    try {
      const { load } = await import('@tauri-apps/plugin-store')
      const store = await load('settings.json')
      await store.set('favorites', favorites.value)
      await store.save()
    } catch {}
  }

  const isFavorite = (id: string) => isItemFavorite(id, favorites.value)

  function toggleFavorite(item: { type: 'photo' | 'album'; id: string; path: string }): void {
    favorites.value = toggleFavoriteItem(item, favorites.value)
    saveFavorites()
  }

  const getFavorites = () => [...favorites.value]
  const getFavoritesByType = () => groupFavoritesByType(favorites.value)
  
  function clearFavorites(): void {
    favorites.value = []
    saveFavorites()
  }

  return {
    favorites,
    favoritePhotos,
    favoriteAlbums,
    favoriteCount,
    isFavorite,
    toggleFavorite,
    getFavorites,
    getFavoritesByType,
    loadFavorites,
    saveFavorites,
    clearFavorites,
  }
}