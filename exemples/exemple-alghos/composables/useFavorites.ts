import { ref, computed } from 'vue'
import { load } from '@tauri-apps/plugin-store'

export interface FavoriteItem {
  type: 'photo' | 'album'
  id: string
  path: string
  addedAt: number
}

const favorites = ref<FavoriteItem[]>([])
let initialized = false

/**
 * Toggles favorite status for an item
 * Returns the new favorites array
 */
export function toggleFavoriteItem(
  item: { type: 'photo' | 'album'; id: string; path: string },
  currentFavorites: FavoriteItem[]
): FavoriteItem[] {
  const existingIndex = currentFavorites.findIndex((f) => f.id === item.id)

  if (existingIndex !== -1) {
    // Remove from favorites
    return currentFavorites.filter((_, i) => i !== existingIndex)
  } else {
    // Add to favorites
    return [
      ...currentFavorites,
      {
        type: item.type,
        id: item.id,
        path: item.path,
        addedAt: Date.now(),
      },
    ]
  }
}

/**
 * Groups favorites by type
 */
export function groupFavoritesByType(items: FavoriteItem[]): {
  photos: FavoriteItem[]
  albums: FavoriteItem[]
} {
  const photos: FavoriteItem[] = []
  const albums: FavoriteItem[] = []

  for (const item of items) {
    if (item.type === 'photo') {
      photos.push(item)
    } else {
      albums.push(item)
    }
  }

  return { photos, albums }
}

/**
 * Checks if an item is in the favorites list
 */
export function isItemFavorite(id: string, favoritesList: FavoriteItem[]): boolean {
  return favoritesList.some((f) => f.id === id)
}

export function useFavorites() {
  const favoritePhotos = computed(() => favorites.value.filter((f) => f.type === 'photo'))
  const favoriteAlbums = computed(() => favorites.value.filter((f) => f.type === 'album'))
  const favoriteCount = computed(() => favorites.value.length)

  async function loadFavorites(): Promise<void> {
    if (initialized) return
    try {
      const store = await load('settings.json')
      const saved = await store.get<FavoriteItem[]>('favorites')
      if (saved && Array.isArray(saved)) {
        favorites.value = saved
      }
      initialized = true
    } catch {
      // Use empty array on error
    }
  }

  async function saveFavorites(): Promise<void> {
    try {
      const store = await load('settings.json')
      await store.set('favorites', favorites.value)
      await store.save()
    } catch {
      // Silent fail
    }
  }

  function isFavorite(id: string): boolean {
    return isItemFavorite(id, favorites.value)
  }

  function toggleFavorite(item: { type: 'photo' | 'album'; id: string; path: string }): void {
    favorites.value = toggleFavoriteItem(item, favorites.value)
    saveFavorites()
  }

  function getFavorites(): FavoriteItem[] {
    return [...favorites.value]
  }

  function getFavoritesByType(): { photos: FavoriteItem[]; albums: FavoriteItem[] } {
    return groupFavoritesByType(favorites.value)
  }

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
