/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed } from 'vue'
import { useSelection } from './useSelection'

export type NavView = 'photos' | 'favorites' | 'albums' | 'tags' | 'trash'

interface NavState {
  view: NavView
  albumPath: string | null
  tagId: string | null
}

const MAX_HISTORY = 50

export function useNavigation() {
  const { clearSelection } = useSelection()
  
  const currentView = ref<NavView>('photos')
  const selectedAlbumPath = ref<string | null>(null)
  const selectedTagId = ref<string | null>(null)
  const currentPhotoName = ref<string | null>(null)
  
  const navigationHistory = ref<NavState[]>([{ view: 'photos', albumPath: null, tagId: null }])
  const currentHistoryIndex = ref(0)

  const canGoBack = computed(() => currentHistoryIndex.value > 0)
  const canGoForward = computed(() => currentHistoryIndex.value < navigationHistory.value.length - 1)
  const canGoUp = computed(() => currentView.value === 'albums' && selectedAlbumPath.value !== null)

  const breadcrumbs = computed(() => {
    const crumbs: { label: string; path: string | null }[] = []
    
    if (currentView.value === 'photos') {
      crumbs.push({ label: 'Fotos', path: null })
    } else if (currentView.value === 'albums') {
      crumbs.push({ label: 'Álbuns', path: null })
      if (selectedAlbumPath.value) {
        const parts = selectedAlbumPath.value.split('/')
        let accumulated = ''
        parts.forEach((part, i) => {
          accumulated = i === 0 ? part : `${accumulated}/${part}`
          crumbs.push({ label: part, path: accumulated })
        })
      }
    } else if (currentView.value === 'favorites') {
      crumbs.push({ label: 'Favoritos', path: null })
    } else if (currentView.value === 'tags') {
      crumbs.push({ label: 'Etiquetas', path: null })
    } else if (currentView.value === 'trash') {
      crumbs.push({ label: 'Lixeira', path: null })
    }
    
    if (currentPhotoName.value) {
      crumbs.push({ label: currentPhotoName.value, path: '__photo__' })
    }
    
    return crumbs
  })

  function isSameState(a: NavState, b: NavState): boolean {
    return a.view === b.view && a.albumPath === b.albumPath && a.tagId === b.tagId
  }

  function applyNavState(state: NavState) {
    currentView.value = state.view
    selectedAlbumPath.value = state.albumPath
    selectedTagId.value = state.tagId
    clearSelection()
  }

  function pushNavState() {
    const state: NavState = { 
      view: currentView.value, 
      albumPath: selectedAlbumPath.value, 
      tagId: selectedTagId.value 
    }
    
    const current = navigationHistory.value[currentHistoryIndex.value]
    if (current && isSameState(current, state)) return
    
    navigationHistory.value = navigationHistory.value.slice(0, currentHistoryIndex.value + 1)
    navigationHistory.value.push(state)
    
    if (navigationHistory.value.length > MAX_HISTORY) {
      navigationHistory.value = navigationHistory.value.slice(-MAX_HISTORY)
    }
    
    currentHistoryIndex.value = navigationHistory.value.length - 1
  }

  function goBack() {
    if (!canGoBack.value) return
    currentHistoryIndex.value--
    applyNavState(navigationHistory.value[currentHistoryIndex.value])
  }

  function goForward() {
    if (!canGoForward.value) return
    currentHistoryIndex.value++
    applyNavState(navigationHistory.value[currentHistoryIndex.value])
  }

  function goUp() {
    if (!selectedAlbumPath.value) return
    const parts = selectedAlbumPath.value.split('/')
    parts.pop()
    navigateToAlbum(parts.length > 0 ? parts.join('/') : null)
  }

  function navigateToAlbum(path: string | null) {
    currentView.value = 'albums'
    selectedAlbumPath.value = path
    selectedTagId.value = null
    pushNavState()
    clearSelection()
  }

  function navigateToView(view: NavView, addToHistory = true) {
    currentView.value = view
    selectedAlbumPath.value = null
    selectedTagId.value = null
    if (addToHistory) pushNavState()
    clearSelection()
  }

  function setCurrentPhotoName(name: string | null) {
    currentPhotoName.value = name
  }

  return {
    currentView,
    selectedAlbumPath,
    selectedTagId,
    currentPhotoName,
    breadcrumbs,
    canGoBack,
    canGoForward,
    canGoUp,
    goBack,
    goForward,
    goUp,
    navigateToAlbum,
    navigateToView,
    setCurrentPhotoName
  }
}