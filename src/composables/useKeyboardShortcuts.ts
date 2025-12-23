import { ref, onUnmounted } from 'vue'
import { useSelection } from './useSelection'
import { useFavorites } from './useFavorites'
import { SHORTCUTS } from '../config'
import type { Photo } from './usePhotoUpload'
import type { ContextMenuItem } from '../components/ContextMenu.vue'

export function useKeyboardShortcuts(
  filteredPhotos: { value: Photo[] },
  photos: { value: Photo[] }
) {
  const { clearSelection, selectAll, getSelected, selectedCount } = useSelection()
  const { toggleFavorite } = useFavorites()
  const contextMenu = ref<{ x: number; y: number; items: ContextMenuItem[] } | null>(null)

  function handleKeydown(e: KeyboardEvent) {
    const { selectAll: selectAllKey, favorite: favKey, escape: escKey } = SHORTCUTS
    
    if ((e.ctrlKey || e.metaKey) && e.key === selectAllKey.key) {
      e.preventDefault()
      selectAll(filteredPhotos.value.map(p => p.sha))
    }
    
    if (e.key === 'Delete' && selectedCount.value > 0) {
      e.preventDefault()
      // TODO: Implement delete
    }
    
    if (e.key === favKey.key && selectedCount.value > 0) {
      e.preventDefault()
      const selectedIds = getSelected()
      for (const id of selectedIds) {
        const photo = photos.value.find(p => p.sha === id)
        if (photo) toggleFavorite({ type: 'photo', id: photo.sha, path: photo.name })
      }
    }
    
    if (e.key === escKey.key) {
      clearSelection()
      contextMenu.value = null
    }
  }

  let mounted = false

  function mount() {
    if (!mounted) {
      document.addEventListener('keydown', handleKeydown)
      mounted = true
    }
  }

  function unmount() {
    if (mounted) {
      document.removeEventListener('keydown', handleKeydown)
      mounted = false
    }
  }

  onUnmounted(unmount)

  return {
    contextMenu,
    mount,
    unmount
  }
}
