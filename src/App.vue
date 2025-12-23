/**
 * Vue Component - 22 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: SpaceLoader, AuthButton, NeuralBackground...
 */

<script setup lang="ts">
import { ref, onMounted, watch, computed, onUnmounted, nextTick } from 'vue'
import { useGitHubAuth } from './composables/useGitHubAuth'
import { usePhotoUpload, type Photo, type UploadStatus } from './composables/usePhotoUpload'
import { useUploadToast } from './composables/useUploadToast'
import { useAccentColor } from './composables/useAccentColor'
import { useTheme } from './composables/useTheme'
import { useSelection } from './composables/useSelection'
import { useFavorites } from './composables/useFavorites'
import { useColorTags } from './composables/useColorTags'
import { usePhotoPreviewSize } from './composables/usePhotoPreviewSize'
import { useDataDriver, type DataDriver } from './composables/useDataDriver'
import { useBackupSettings } from './composables/useBackupSettings'
import { useTimeout } from './composables/useTimeout'
import { useNavigation, type NavView } from './composables/useNavigation'
import { useMobileSearch, useMobileDetection } from './composables/useMobileSearch'

import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { UPLOAD, TIMING, injectCSSVariables } from './config'
import SpaceLoader from './components/SpaceLoader.vue'
import AuthButton from './components/AuthButton.vue'
import { Vortex } from './components/ui/vortex'
import NeuralBackground from './components/ui/NeuralBackground/NeuralBackground.vue'
import Cursor from './components/ui/cursor/Cursor.vue'
import PhotoGallery from './components/PhotoGallery.vue'
import PrivacySettings from './components/PrivacySettings.vue'
import FolderUploadDialog from './components/FolderUploadDialog.vue'
import RepoCreator from './components/RepoCreator.vue'
import ContextMenu from './components/ContextMenu.vue'
import UploadToast from './components/UploadToast.vue'
import ThemeSettings from './components/ThemeSettings.vue'
import DataDriverManager from './components/DataDriverManager.vue'
import BackupSettings from './components/BackupSettings.vue'
import LocalImageBrowser from './components/LocalImageBrowser.vue'
import SecuritySettings from './components/SecuritySettings.vue'
import ErrorBoundary from './components/ErrorBoundary.vue'
import AsyncState from './components/AsyncState.vue'
import HaloSearch from './components/HaloSearch.vue'
import MacOSDock from './components/MacOSDock.vue'
import GlassSurface from './components/GlassSurface.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import TopHeader from './components/TopHeader.vue'
import { useDockApps } from './composables/useDockApps'

interface Album {
  name: string
  path: string
  photo_count: number
  children: Album[]
  coverUrl?: string
}

const { token, repo, init, setRepo } = useGitHubAuth()
const { photos, loadingPhotos, loadPhotos, addToQueue, queue } = usePhotoUpload()
const { addTransfer, updateProgress, setStatus: setTransferStatus } = useUploadToast()
const { init: initAccent } = useAccentColor()
const { loadTheme } = useTheme()
const { clearSelection, selectAll, getSelected, selectedCount } = useSelection()
const { loadFavorites, isFavorite, toggleFavorite } = useFavorites()
const { loadTags, getItemsByTag } = useColorTags()
const { loadSize: loadPreviewSize } = usePhotoPreviewSize()
const { loadDrivers } = useDataDriver()
const { loadConfig: loadBackupConfig } = useBackupSettings()
const { dockApps, activeApps, toggleApp, setActiveApp } = useDockApps()
const { createTimeout } = useTimeout()

const {
  currentView, selectedAlbumPath, selectedTagId,
  navigateToAlbum, navigateToView, setCurrentPhotoName
} = useNavigation()

const { isMobile } = useMobileDetection()
const {
  mobileSearchOpen, searchPullDistance, searchOpacity, pullDistance, isPulling,
  toggleMobileSearch, handleSearchTouchStart, handleSearchTouchMove: _handleSearchTouchMove,
  handleSearchTouchEnd: _handleSearchTouchEnd, handleSearchScroll: _handleSearchScroll,
  handleTouchStart, handleTouchMove, handleTouchEnd
} = useMobileSearch()

const loading = ref(true)
const repoInput = ref('')
const isDragging = ref(false)
const showSettings = ref(false)
const showPrivacy = ref(false)
const showRepoCreator = ref(false)
const showThemeSettings = ref(false)
const showDataDrivers = ref(false)
const showBackupSettings = ref(false)
const showLocalBrowser = ref(false)
const showSecuritySettings = ref(false)
const searchQuery = ref('')
const debouncedSearchQuery = ref('')
const mobileSearchRef = ref<InstanceType<typeof HaloSearch> | null>(null)

const viewMode = ref<'grid' | 'list'>('grid')
const uploadError = ref<string | null>(null)
const showFolderDialog = ref(false)
const pendingFolderPath = ref<string | null>(null)
const contextMenu = ref<{ x: number; y: number; items: any[] } | null>(null)
const albums = ref<Album[]>([])
const loadingAlbums = ref(false)
const photoLoadError = ref<string | null>(null)

function handleSearchTouchMoveWrapper(e: TouchEvent) {
  const hasResults = !!(searchQuery.value && filteredPhotos.value.length > 0)
  _handleSearchTouchMove(e, hasResults)
}

function handleSearchTouchEndWrapper() {
  _handleSearchTouchEnd(searchQuery)
}

function handleSearchScrollWrapper(e: Event) {
  const hasResults = !!(searchQuery.value && filteredPhotos.value.length > 0)
  _handleSearchScroll(e, hasResults, searchQuery)
}

function setupSearchOverlayListeners() {
  nextTick(() => {
    const input = mobileSearchRef.value?.$el?.querySelector('input')
    input?.focus()
  })
}

const searchTimeout = ref<ReturnType<typeof setTimeout> | null>(null)
watch(searchQuery, (newQuery) => {
  if (searchTimeout.value) clearTimeout(searchTimeout.value)
  searchTimeout.value = createTimeout(() => {
    debouncedSearchQuery.value = newQuery
  }, TIMING.debounce.search)
})

onUnmounted(() => {
  if (searchTimeout.value) clearTimeout(searchTimeout.value)
})

const filteredPhotos = computed(() => {
  let result = photos.value as Photo[]

  if (debouncedSearchQuery.value) {
    const q = debouncedSearchQuery.value.toLowerCase()
    result = result.filter(p => p.name.toLowerCase().includes(q))
  }

  if (currentView.value === 'favorites') {
    
    result = result.filter(p => isFavorite(p.sha))
  } else if (currentView.value === 'trash') {
    
    loadPhotos('trash')
    result = []
  } else if (currentView.value === 'tags' && selectedTagId.value) {
    const taggedIds = getItemsByTag(selectedTagId.value)
    result = result.filter(p => taggedIds.includes(p.sha))
  } else if (currentView.value === 'albums' && selectedAlbumPath.value) {
    
    loadPhotos(`albums/${selectedAlbumPath.value}`)
    result = result.filter(p => {
      if (!p.path) return false
      return p.path.startsWith(selectedAlbumPath.value!)
    })
  } else if (currentView.value === 'photos') {
    
    loadPhotos('photos')
  }
  
  return result
})

const uploadProgress = computed(() => {
  return queue.value.filter(u => u.status === 'uploading' || u.status === 'pending').length
})

const storageStats = computed(() => {
  const totalBytes = photos.value.reduce((acc, p) => acc + (p.size || 0), 0)
  const gb = totalBytes / (1024 * 1024 * 1024)
  const mb = totalBytes / (1024 * 1024)
  const sizeText = gb >= 1 ? `${gb.toFixed(2)} GB` : `${mb.toFixed(1)} MB`
  return {
    size: sizeText,
    mediaCount: photos.value.length,
    albumCount: albums.value.length
  }
})

const dockAppsWithBadges = computed(() => {
  return dockApps.value.map(app => ({
    ...app,
    badge: app.id === 'upload' && uploadProgress.value > 0 ? uploadProgress.value : undefined
  }))
})

async function loadAlbums() {
  if (!token.value || !repo.value) return
  loadingAlbums.value = true
  try {
    albums.value = await invoke<Album[]>('list_albums', { token: token.value, repo: repo.value })
  } catch {
    albums.value = []
  } finally {
    loadingAlbums.value = false
  }
}

onMounted(async () => {
  injectCSSVariables()
  document.documentElement.classList.add('matrix-effects', 'glass-morphism', 'glow-effects')

  document.addEventListener('keydown', handleKeydown)
  
  await Promise.all([init(), initAccent(), loadTheme(), loadFavorites(), loadTags(), loadPreviewSize(), loadDrivers(), loadBackupConfig()])
  repoInput.value = repo.value
  
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    const savedViewMode = await store.get<'grid' | 'list'>('viewMode')
    if (savedViewMode) viewMode.value = savedViewMode
  } catch {}
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

watch(repo, (v) => { repoInput.value = v })
watch([token, repo], () => { 
  if (token.value && repo.value) {
    loadPhotos()
    loadAlbums()
  }
})

const queueStatusMap = new Map<string, UploadStatus>()

const queueItems = computed(() => 
  queue.value.map(i => ({ id: i.id, status: i.status, progress: i.progress, name: i.name, error: i.error }))
)

watch(queueItems, (items) => {
  const activeIds = new Set(items.map(i => i.id))

  for (const [id, status] of queueStatusMap) {
    if (!activeIds.has(id) && (status === 'success' || status === 'failed')) {
      queueStatusMap.delete(id)
    }
  }
  
  for (const item of items) {
    const prevStatus = queueStatusMap.get(item.id)
    if (prevStatus !== item.status) {
      queueStatusMap.set(item.id, item.status)
      
      if (item.status === 'uploading' || item.status === 'pending') {
        addTransfer(item.id, item.name, 'upload')
      } else if (item.status === 'success') {
        setTransferStatus(item.id, 'completed')
      } else if (item.status === 'failed') {
        setTransferStatus(item.id, 'failed', item.error)
      }
    }
    if (item.status === 'uploading') {
      updateProgress(item.id, item.progress)
    }
  }
}, { deep: false })

watch(viewMode, async (mode) => {
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    await store.set('viewMode', mode)
    await store.save()
  } catch {}
})

import { SHORTCUTS } from './config'
function handleKeydown(e: KeyboardEvent) {
  const { selectAll: selectAllKey, favorite: favKey, escape: escKey } = SHORTCUTS
  
  if ((e.ctrlKey || e.metaKey) && e.key === selectAllKey.key) {
    e.preventDefault()
    selectAll(filteredPhotos.value.map(p => p.sha))
  }
  if (e.key === 'Delete' && selectedCount.value > 0) {
    e.preventDefault()
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

async function handleUploadClick() {
  uploadError.value = null
  try {
    const files = await open({ 
      multiple: true, 
      directory: false, 
      filters: [{ name: 'Images', extensions: [...UPLOAD.supportedFormats] }] 
    })
    if (files) addToQueue(Array.isArray(files) ? files : [files])
  } catch (e) {
    uploadError.value = e instanceof Error ? e.message : 'Erro ao selecionar arquivos'
  }
}

function onDrop(e: DragEvent) {
  isDragging.value = false
  const paths: string[] = []
  for (const file of e.dataTransfer?.files || []) {
    const path = (file as File & { path?: string }).path
    if (path) paths.push(path)
  }
  if (paths.length === 1) {
    
    const isFolder = !paths[0].includes('.') || paths[0].split('/').pop()?.indexOf('.') === -1
    if (isFolder) {
      pendingFolderPath.value = paths[0]
      showFolderDialog.value = true
      return
    }
  }
  if (paths.length) addToQueue(paths)
}

async function handleFolderUpload(mode: 'album' | 'recursive') {
  if (!pendingFolderPath.value || !token.value || !repo.value) return
  showFolderDialog.value = false
  uploadError.value = null
  
  try {
    if (mode === 'album') {
      const albumName = pendingFolderPath.value.split('/').pop() || 'album'
      await invoke('upload_folder_as_album', { 
        token: token.value, 
        repo: repo.value, 
        path: pendingFolderPath.value,
        albumName,
        createSubalbums: true
      })
    } else {
      await invoke('upload_folder_recursive', { 
        token: token.value, 
        repo: repo.value, 
        path: pendingFolderPath.value 
      })
    }
    loadPhotos()
    loadAlbums()
  } catch (err) {
    uploadError.value = err instanceof Error ? err.message : 'Erro ao fazer upload da pasta'
    console.error('Folder upload failed:', err)
  }
  pendingFolderPath.value = null
}

function saveRepo() {
  const trimmed = repoInput.value.trim()
  if (trimmed && /^[\w-]+\/[\w.-]+$/.test(trimmed)) setRepo(trimmed)
}

function handleRepoCreated(repoName: string) {
  showRepoCreator.value = false
  setRepo(repoName)
}

function handleDriverChanged(driver: DataDriver) {
  showDataDrivers.value = false
  if (driver.type === 'github') {
    setRepo(driver.path)
  }
}

async function handleLocalImport(imagePaths: string[], _targetDriverId: string) {
  showLocalBrowser.value = false
  void _targetDriverId 
  
  addToQueue(imagePaths)
}

function navigateTo(view: NavView) {
  navigateToView(view, true)

  setActiveApp(view)
}

function handleAlbumSelect(albumOrPath: Album | string | null) {
  const path = typeof albumOrPath === 'string' ? albumOrPath : albumOrPath?.path ?? null
  navigateToAlbum(path)
}

function handlePhotoClick(photo: Photo) {
  setCurrentPhotoName(photo.name)
}

function dismissError() {
  uploadError.value = null
}

function handleDockAppClick(appId: string) {
  console.log('Dock app clicked:', appId)
  
  switch (appId) {
    case 'photos':
      navigateTo('photos')
      setActiveApp('photos')
      break
    case 'favorites':
      navigateTo('favorites')
      setActiveApp('favorites')
      break
    case 'albums':
      navigateTo('albums')
      setActiveApp('albums')
      break
    case 'search':
      
      if (isMobile.value) {
        toggleMobileSearch()
      } else {
        
        const searchInput = document.querySelector('.desktop-search input') as HTMLInputElement
        if (searchInput) {
          searchInput.focus()
        }
      }
      toggleApp('search')
      break
    case 'trash':
      
      navigateToView('trash')
      setActiveApp('trash')
      break
    case 'settings':
      showSettings.value = !showSettings.value
      toggleApp('settings')
      break
    default:
      console.warn('Unknown dock app:', appId)
  }
}

async function retryLoadPhotos() {
  photoLoadError.value = null
  try {
    await loadPhotos()
  } catch (e) {
    photoLoadError.value = e instanceof Error ? e.message : 'Erro ao carregar fotos'
  }
}
</script>

<template>
  <ErrorBoundary>
    <!-- Custom Cursor (desktop only) -->
    <Cursor v-if="!isMobile" target-selector=".cursor-target" :spin-duration="2" />
    
    <SpaceLoader v-if="loading" @complete="loading = false" />
    
    <div v-else class="app-dock" @dragover.prevent="isDragging = true" @dragleave.prevent="isDragging = false" @drop.prevent="onDrop">
      <!-- Neural Background -->
      <NeuralBackground class="fixed inset-0 z-0 opacity-30" :hue="200" :saturation="0.7" :chroma="0.5" />
      
      <!-- Top Header -->
      <TopHeader @create-repo="showRepoCreator = true" />
      
      <!-- Drag Overlay -->
      <Transition name="fade">
      <div v-if="isDragging && token && repo" class="drag-overlay">
        <div class="drag-content">
          <div class="drag-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 16V4m0 0L8 8m4-4l4 4M4 14v4a2 2 0 002 2h12a2 2 0 002-2v-4"/>
            </svg>
          </div>
          <h3>Solte para fazer upload</h3>
          <p>Arraste fotos ou pastas</p>
        </div>
      </div>
      </Transition>

      <!-- Sidebar -->
    <!-- <aside class="sidebar">
      <div class="sidebar-header">
        <div class="logo">
          <div class="logo-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <circle cx="9" cy="9" r="2"/>
              <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
            </svg>
          </div>
          <span>iMAGE</span>
        </div>
      </div>

      <nav class="sidebar-nav">
        <button class="nav-item" :class="{ active: currentView === 'photos' }" @click="navigateTo('photos')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
          <span>Fotos</span>
        </button>
        <button class="nav-item" :class="{ active: currentView === 'favorites' }" @click="navigateTo('favorites')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
          <span>Favoritos</span>
        </button>
        <button class="nav-item" :class="{ active: currentView === 'albums' }" @click="navigateTo('albums')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
          <span>Álbuns</span>
        </button>
        <button class="nav-item" @click="showLocalBrowser = true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>
          <span>Importar</span>
        </button>
      </nav>

      <div class="sidebar-section">
        <div class="section-header">
          <span>Fontes de Dados</span>
          <button class="section-btn" @click="showDataDrivers = true" title="Gerenciar">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          </button>
        </div>
        <div v-if="activeDriver" class="active-driver">
          <div class="driver-icon" :class="activeDriver.type">
            <svg v-if="activeDriver.type === 'github'" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
          </div>
          <div class="driver-details">
            <span class="driver-name">{{ activeDriver.name }}</span>
            <span class="driver-type">{{ activeDriver.type === 'github' ? 'GitHub' : 'Local' }}</span>
          </div>
        </div>
        <button v-else class="add-driver-btn" @click="showDataDrivers = true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          Adicionar Fonte
        </button>
      </div>

      <div class="sidebar-section">
        <div class="section-header">
          <span>Backup</span>
        </div>
        <button class="nav-item compact" @click="showBackupSettings = true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
          <span>Configurar Backup</span>
        </button>
        <button class="nav-item compact" @click="showSecuritySettings = true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          <span>Segurança</span>
        </button>
        <button class="nav-item compact" @click="showPrivacy = true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          <span>Privacidade</span>
        </button>
      </div>

      <div v-if="currentView === 'albums'" class="sidebar-section">
        <div class="section-header">
          <span>Álbuns</span>
        </div>
        <AlbumTree :albums="albums" :selected-path="selectedAlbumPath" @select="handleAlbumSelect" />
      </div>

      <div v-if="currentView === 'tags' || currentView === 'photos'" class="sidebar-section">
        <ColorTagPanel :selected-tag-id="selectedTagId" @select="handleTagSelect" />
      </div>

      <div class="sidebar-footer">
        <AuthButton />
      </div>
    </aside> -->

    <!-- Main Content -->
    <main 
      class="main-dock"
      @touchstart="handleTouchStart"
      @touchmove="handleTouchMove"
      @touchend="handleTouchEnd"
      :style="{ transform: `translateY(${pullDistance}px)`, transition: isPulling ? 'none' : 'transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1)' }"
    >
      <!-- Pull indicator -->
      <div 
        class="pull-indicator" 
        :style="{ 
          transform: `translateY(${pullDistance}px)`,
          opacity: Math.min(pullDistance / 40, 1)
        }"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
        </svg>
        <span>Solte para pesquisar</span>
      </div>

      <!-- Mobile Search Overlay -->
      <Transition name="mobile-search" @after-enter="setupSearchOverlayListeners">
        <div 
          v-if="mobileSearchOpen" 
          class="mobile-search-overlay" 
          @click.self="mobileSearchOpen = false"
          @touchstart="handleSearchTouchStart"
          @touchmove="handleSearchTouchMoveWrapper"
          @touchend="handleSearchTouchEndWrapper"
          :style="{ 
            opacity: searchOpacity,
            transform: `translateY(${searchPullDistance}px)`,
            transition: searchOpacity < 1 ? 'none' : 'all 0.3s ease'
          }"
        >
          <div class="mobile-search-container">
            <HaloSearch 
              ref="mobileSearchRef"
              v-model="searchQuery" 
              placeholder="Pesquisar fotos..." 
              class="mobile-search-input"
            />
            <button class="mobile-search-cancel" @click="mobileSearchOpen = false; searchQuery = ''">
              Cancelar
            </button>
          </div>
          
          <!-- Search Results -->
          <div 
            class="mobile-search-results"
            @scroll="handleSearchScrollWrapper"
          >
            <div v-if="searchQuery && filteredPhotos.length === 0" class="search-empty">
              Nenhum resultado para "{{ searchQuery }}"
            </div>
            <div v-else-if="searchQuery" class="search-results-grid">
              <div 
                v-for="photo in filteredPhotos.slice(0, 20)" 
                :key="photo.sha" 
                class="search-result-item"
                @click="mobileSearchOpen = false"
              >
                <img :src="photo.url" :alt="photo.name" loading="lazy" />
                <span class="search-result-name">{{ photo.name }}</span>
              </div>
            </div>
            <template v-else>
              <div class="search-hint">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="hint-icon">
                  <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
                </svg>
                <span>Digite para pesquisar</span>
              </div>
            </template>
            <!-- Scroll spacer for overscroll detection -->
            <div class="scroll-spacer"></div>
          </div>
        </div>
      </Transition>

      <!-- Header -->
      <!-- Content -->
      <div class="content-dock">
        <template v-if="!token">
          <Vortex
            class="login-vortex"
            :particle-count="400"
            :base-hue="210"
            :range-y="300"
            :base-speed="0.05"
            :range-speed="1"
            background-color="transparent"
          >
            <div class="empty-state login-state">
              <div class="empty-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
              </div>
              <h2>Conecte sua conta GitHub</h2>
              <p>Faça login para começar a armazenar suas fotos com segurança</p>
              <AuthButton class="login-auth-btn" />
            </div>
          </Vortex>
        </template>

        <template v-else-if="!repo">
          <div class="empty-state">
            <div class="empty-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
            </div>
            <h2>Configure seu repositório</h2>
            <p>Defina um repositório GitHub para armazenar suas fotos</p>
            <div class="repo-setup">
              <input v-model="repoInput" type="text" placeholder="usuario/repositorio" @keyup.enter="saveRepo" />
              <button @click="saveRepo" class="btn-primary">Configurar</button>
            </div>
            <p class="or-divider">ou</p>
            <button @click="showRepoCreator = true" class="btn-secondary">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              Criar Novo Repositório
            </button>
          </div>
        </template>

        <template v-else>
          <AsyncState 
            :loading="loadingPhotos" 
            :error="photoLoadError"
            :empty="false"
            @retry="retryLoadPhotos"
          >
            <!-- Trash Empty State -->
            <div v-if="currentView === 'trash' && filteredPhotos.length === 0" class="empty-state">
              <div class="empty-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6h14ZM10 11v6M14 11v6"/>
                </svg>
              </div>
              <h2>Lixeira vazia</h2>
              <p>Itens excluídos aparecerão aqui</p>
            </div>
            
            <!-- Optimized Photo Gallery -->
            <PhotoGallery 
              v-else
              :photos="filteredPhotos"
              :loading="loadingPhotos"
              :albums="albums"
              :show-albums="currentView === 'albums'"
              :view-mode="viewMode"
              :current-album-path="selectedAlbumPath"
              @photo-click="handlePhotoClick"
              @album-click="handleAlbumSelect"
            />
          </AsyncState>
        </template>
      </div>
    </main>

    <!-- Mobile Upload Button (positioned above dock) -->
    <Transition name="fade">
      <div v-if="isMobile && token && repo && !mobileSearchOpen" class="mobile-upload-container">
        <button 
          @click="handleUploadClick" 
          :disabled="!token || !repo" 
          class="mobile-upload-btn"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="w-6 h-6">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17,8 12,3 7,8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
        </button>
      </div>
    </Transition>

    <!-- macOS Dock (All Devices) -->
    <Transition name="dock-fade">
      <MacOSDock 
        v-show="!mobileSearchOpen"
        :apps="dockAppsWithBadges" 
        :open-apps="activeApps"
        :is-mobile="isMobile"
        @app-click="handleDockAppClick"
        :class="isMobile ? 'mobile-dock' : ''"
      />
    </Transition>

    <!-- Modals -->
    <SettingsPanel v-if="showSettings" @close="showSettings = false" />
    <PrivacySettings v-if="showPrivacy" @close="showPrivacy = false" />
    <RepoCreator v-if="showRepoCreator" @created="handleRepoCreated" @close="showRepoCreator = false" />
    <ThemeSettings v-if="showThemeSettings" @close="showThemeSettings = false" />
    <DataDriverManager v-if="showDataDrivers" @close="showDataDrivers = false" @driver-changed="handleDriverChanged" />
    <BackupSettings v-if="showBackupSettings" @close="showBackupSettings = false" />
    <SecuritySettings v-if="showSecuritySettings" @close="showSecuritySettings = false" />
    <LocalImageBrowser v-if="showLocalBrowser" @close="showLocalBrowser = false" @import="handleLocalImport" />
    <FolderUploadDialog 
      v-if="showFolderDialog && pendingFolderPath" 
      :folder-path="pendingFolderPath" 
      @confirm="handleFolderUpload" 
      @cancel="showFolderDialog = false; pendingFolderPath = null" 
    />
    
    <!-- Context Menu -->
    <ContextMenu 
      v-if="contextMenu" 
      :x="contextMenu.x" 
      :y="contextMenu.y" 
      :items="contextMenu.items"
      @close="contextMenu = null"
      @select="contextMenu = null"
    />

    <!-- Error Toast -->
    <Transition name="toast">
      <div v-if="uploadError" class="error-toast" role="alert">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 8v4M12 16h.01"/>
        </svg>
        <span>{{ uploadError }}</span>
        <button @click="dismissError" aria-label="Fechar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>
      </div>
    </Transition>

    <!-- Upload Progress Toast -->
    <UploadToast />

    <!-- Floating Breadcrumb Navigation -->
    <div class="floating-breadcrumb" v-if="token && repo">
      <!-- Left: Stats -->
      <GlassSurface :border-radius="18" :border-width="0.08" :background-opacity="0.4" class="bottom-pill">
        <span class="pill-text">{{ storageStats.mediaCount }} mídias</span>
        <span class="pill-sep">·</span>
        <span class="pill-text">{{ storageStats.albumCount }} álbuns</span>
        <span class="pill-sep">·</span>
        <span class="pill-text">{{ storageStats.size }}</span>
      </GlassSurface>
      <!-- Right: Sort -->
      <GlassSurface :border-radius="18" :border-width="0.08" :background-opacity="0.4" class="bottom-pill bottom-pill-right">
        <span class="pill-text">Date</span>
        <span class="pill-sep">·</span>
        <span class="pill-text">Uploaded</span>
        <span class="pill-sep">·</span>
        <span class="pill-text">Custom</span>
      </GlassSurface>
    </div>

    <!-- Mobile Search FAB -->
    <button 
      class="mobile-search-fab" 
      @click="mobileSearchOpen = true"
      :class="{ hidden: mobileSearchOpen }"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" class="w-6 h-6">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
    </button>

    <!-- Mobile Bottom Navigation -->
    <!-- Removed - using MacOS dock instead -->
    </div>
  </ErrorBoundary>
</template>

<style scoped>
.app-dock {
  position: fixed;
  inset: 0;
  background: transparent;
  color: var(--text-primary);
  overflow: hidden;
}

.faulty-terminal-bg {
  position: fixed;
  inset: 0;
  z-index: 0;
  opacity: 0.15;
  pointer-events: none;
}

.main-dock {
  position: fixed;
  inset: 0;
  background: transparent;
  overflow-y: auto;
  z-index: 1;
}

@media (max-width: 768px) {
  .content-dock {
    padding: 0;
  }
}

.selected-count {
  font-size: 0.8125rem;
  color: var(--accent-color);
  font-weight: 600;
  background: var(--accent-light);
  padding: 0.25rem 0.5rem;
  border-radius: var(--radius-sm);
}

.floating-btn {
  width: 3.5rem;
  height: 3.5rem;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(12px);
}

.search-btn {
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.9);
  color: var(--text-primary);
}

.search-btn:hover {
  transform: scale(1.1);
  box-shadow: 0 6px 25px rgba(0, 0, 0, 0.4);
}

.upload-btn {
  background: linear-gradient(135deg, var(--accent-color), var(--accent-secondary));
  color: #000;
}

.upload-btn:hover {
  transform: scale(1.1);
  box-shadow: 0 6px 25px rgba(var(--accent-rgb), 0.5);
}

.upload-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.floating-btn svg {
  width: 1.25rem;
  height: 1.25rem;
}

.mobile-search-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  z-index: 1000;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 2rem;
}

.mobile-search-container {
  position: relative;
  width: 90%;
  max-width: 400px;
}

.mobile-search-close {
  position: absolute;
  top: 50%;
  right: 1rem;
  transform: translateY(-50%);
  width: 2rem;
  height: 2rem;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.mobile-search-close:hover {
  color: var(--text-primary);
}

.header-dock {
  position: sticky;
  top: 0;
  z-index: 100;
  margin: 1rem;
}

.header-glass {
  width: 100%;
}

.header-inner {
  width: 100%;
  display: flex;
  align-items: center;
}

.mobile-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  min-height: 44px;
}

.mobile-title {
  flex: 1;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (min-width: 768px) {
  .mobile-header { display: none; }
  .desktop-header { display: flex; }
}

.desktop-header {
  display: none;
  align-items: center;
  gap: 1rem;
  width: 100%;
}

.header-section-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
}

.nav-buttons {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.breadcrumbs {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.875rem;
}

.breadcrumb-sep {
  color: var(--text-muted);
}

.breadcrumb-link {
  background: none;
  border: none;
  color: var(--text-secondary);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}

.breadcrumb-link:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.1);
}

.breadcrumb-link.active {
  color: var(--text-primary);
  font-weight: 600;
}

.photo-count {
  color: var(--text-muted);
  font-size: 0.8125rem;
  margin-left: 0.25rem;
}

.header-section-center {
  flex: 1;
  display: flex;
  justify-content: center;
  min-width: 0;
  max-width: 400px;
  margin: 0 1rem;
}

.search-box {
  position: relative;
  width: 100%;
}

.search-box .search-icon {
  position: absolute;
  left: 0.875rem;
  top: 50%;
  transform: translateY(-50%);
  width: 1.125rem;
  height: 1.125rem;
  color: var(--text-muted);
  pointer-events: none;
}

.search-box input {
  width: 100%;
  height: 2.5rem;
  padding: 0 1rem 0 2.5rem;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 9999px;
  color: var(--text-primary);
  font-size: 0.875rem;
  outline: none;
  transition: all 0.2s;
}

.search-box input::placeholder {
  color: var(--text-muted);
}

.search-box input:focus {
  border-color: var(--accent-color);
  background: transparent;
}

.header-section-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.upload-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
}

.upload-btn .badge {
  margin-left: 0.25rem;
  padding: 0.125rem 0.375rem;
  font-size: 0.6875rem;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 9999px;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.nav-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  background: transparent;
  border: none;
  border-radius: 9999px;
  color: var(--accent-color);
  cursor: pointer;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: rgba(var(--accent-rgb), 0.15);
}

.icon-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.view-toggle {
  display: flex;
  background: transparent;
  border: none;
  border-radius: 6px;
  padding: 0.125rem;
  gap: 2px;
}

.view-toggle button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s;
}

.view-toggle button:hover {
  color: var(--text-primary);
}

.view-toggle button.active {
  color: var(--accent-color);
}

@media (max-width: 768px) {
  .header-dock {
    margin: 0 0.5rem;
    padding: 0.5rem 0.75rem;
  }
}

.content-dock {
  position: absolute;
  inset: 0;
  padding: 0;
  background: transparent;
}

.app {
  display: flex;
  min-height: 100vh;
  background: transparent;
  color: var(--text-primary);
  position: relative;
}

.app-neural-bg {
  position: fixed;
  inset: 0;
  z-index: 0;
  opacity: 0.2;
  pointer-events: none;
}

.drag-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(16px);
  z-index: 300;
  display: flex;
  align-items: center;
  justify-content: center;
}
.drag-content {
  text-align: center;
  padding: 3rem 4rem;
  border: 2px dashed rgba(var(--accent-rgb), 0.5);
  border-radius: var(--radius-xl);
  background: rgba(var(--accent-rgb), 0.08);
  animation: pulse-border 2s ease-in-out infinite;
}
@keyframes pulse-border {
  0%, 100% { border-color: rgba(var(--accent-rgb), 0.3); }
  50% { border-color: rgba(var(--accent-rgb), 0.6); }
}
.drag-icon {
  width: 4rem;
  height: 4rem;
  margin: 0 auto 1.5rem;
  color: var(--accent-color);
  animation: bounce 1s ease-in-out infinite;
}
@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}
.drag-icon svg { width: 100%; height: 100%; }
.drag-content h3 { font-size: 1.5rem; font-weight: 600; margin-bottom: 0.5rem; }
.drag-content p { color: var(--text-muted); }

.sidebar {
  width: var(--sidebar-width);
  background: rgba(var(--surface-1-rgb, 14, 14, 20), var(--glass-opacity, 0.8));
  backdrop-filter: blur(20px);
  border-right: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.sidebar-header { padding: 1.5rem; }
.logo {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-weight: 700;
  font-size: 1.375rem;
  letter-spacing: -0.02em;
}
.logo-icon {
  width: 2.5rem;
  height: 2.5rem;
  background: linear-gradient(135deg, var(--accent-color), var(--accent-secondary));
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 20px rgba(var(--accent-rgb), 0.4);
}
.logo-icon svg { width: 1.5rem; height: 1.5rem; stroke: #000; stroke-width: 2.5; }

.sidebar-nav { padding: 0.75rem; }
.nav-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.875rem;
  padding: 0.875rem 1rem;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.9375rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast);
}
.nav-item:hover { 
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.8); 
  color: var(--text-primary); 
}
.nav-item.active { 
  background: var(--accent-light); 
  color: var(--accent-color);
  font-weight: 600;
}
.nav-item svg { width: 1.375rem; height: 1.375rem; }

.sidebar-section {
  padding: 0.75rem;
  border-top: 1px solid var(--border-default);
}
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 0.875rem;
  font-size: 0.6875rem;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.section-btn {
  width: 1.75rem;
  height: 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}
.section-btn:hover { 
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.8); 
  color: var(--text-primary); 
}
.section-btn svg { width: 1rem; height: 1rem; }

.active-driver {
  display: flex;
  align-items: center;
  gap: 0.875rem;
  padding: 0.875rem;
  background: var(--accent-light);
  border: 1px solid var(--border-accent);
  border-radius: var(--radius-md);
  margin: 0 0.25rem;
}
.driver-icon {
  width: 2.25rem;
  height: 2.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.8);
  border-radius: var(--radius-sm);
}
.driver-icon.github { color: var(--text-primary); }
.driver-icon.local { color: var(--accent-color); }
.driver-icon svg { width: 1.125rem; height: 1.125rem; }
.driver-details { display: flex; flex-direction: column; gap: 0.125rem; }
.driver-name { font-size: 0.875rem; font-weight: 600; color: var(--text-primary); }
.driver-type { font-size: 0.75rem; color: var(--text-secondary); }

.add-driver-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  width: calc(100% - 0.5rem);
  margin: 0 0.25rem;
  padding: 0.75rem;
  background: transparent;
  border: 1px dashed var(--border-default);
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast);
}
.add-driver-btn:hover { 
  border-color: var(--accent-color); 
  color: var(--accent-color);
  background: var(--accent-light);
}
.add-driver-btn svg { width: 1rem; height: 1rem; }

.nav-item.compact {
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
}
.nav-item.compact svg { width: 1rem; height: 1rem; }

.sidebar-footer { 
  margin-top: auto;
  padding: 1rem; 
  border-top: 1px solid var(--border-subtle); 
}

.main { flex: 1; display: flex; flex-direction: column; min-width: 0; }

.header {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding: 1.25rem 1.75rem;
  border-bottom: 1px solid var(--border-default);
  min-height: var(--header-height);
  background: rgba(var(--surface-0-rgb, 8, 8, 12), var(--glass-opacity, 0.8));
  backdrop-filter: blur(20px);
}
.header-left { display: flex; align-items: baseline; gap: 1rem; }
.view-title { 
  font-size: 1.5rem; 
  font-weight: 700; 
  margin: 0;
  letter-spacing: -0.02em;
}
.photo-count { 
  font-size: 0.875rem; 
  color: var(--text-secondary);
  font-weight: 500;
}

.search-bar {
  flex: 1;
  max-width: 520px;
  display: flex;
  align-items: center;
  gap: 0.875rem;
  padding: 0.75rem 1.125rem;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.6);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-full);
  backdrop-filter: blur(12px);
  transition: all var(--duration-fast);
}
.search-bar:focus-within {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px var(--accent-light);
}
.search-bar svg { width: 1.25rem; height: 1.25rem; color: var(--text-secondary); flex-shrink: 0; }
.search-bar input {
  flex: 1;
  background: none;
  border: none;
  color: var(--text-primary);
  font-size: 0.9375rem;
  outline: none;
}
.search-bar input::placeholder { color: var(--text-muted); }
.search-clear {
  width: 1.625rem;
  height: 1.625rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(var(--surface-3-rgb, 32, 32, 44), 0.8);
  border: none;
  border-radius: 50%;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
}
.search-clear:hover { background: rgba(var(--surface-4-rgb, 44, 44, 58), 0.9); color: var(--text-primary); }
.search-clear svg { width: 0.875rem; height: 0.875rem; }

.btn-upload {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 10px 20px;
  background: var(--accent-color);
  border: none;
  border-radius: 1000px;
  color: #000;
  font-weight: 700;
  font-size: 0.9375rem;
  line-height: 1.4;
  cursor: pointer;
  transition: background-color 0.14s ease-out;
  position: relative;
}
.btn-upload:hover { 
  background: color-mix(in srgb, var(--accent-color), #000 8%);
  transition: background-color 0.21s ease-out;
}
.btn-upload:active {
  background: color-mix(in srgb, var(--accent-color), #000 12%);
}
.btn-upload:disabled { 
  opacity: 0.5; 
  cursor: not-allowed; 
}
.btn-upload svg { width: 1.125rem; height: 1.125rem; }
.upload-badge {
  position: absolute;
  top: -0.375rem;
  right: -0.375rem;
  min-width: 1.25rem;
  height: 1.25rem;
  background: var(--error);
  border-radius: 1000px;
  font-size: 0.6875rem;
  font-weight: 700;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-panel {
  padding: 1rem 1.5rem;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.5);
  border-bottom: 1px solid var(--border-subtle);
}
.setting-item label { display: block; font-size: 0.75rem; color: var(--text-muted); margin-bottom: 0.5rem; }
.repo-input { display: flex; gap: 0.5rem; }
.repo-input input {
  flex: 1;
  padding: 0.5rem 0.75rem;
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.6);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 0.875rem;
}
.btn-save {
  padding: 0.5rem 1rem;
  background: linear-gradient(135deg, var(--accent-color), var(--accent-secondary));
  border: none;
  color: #000;
  font-size: 0.875rem;
  font-weight: 600;
  border-radius: var(--radius-sm);
  cursor: pointer;
}
.btn-new {
  width: 2.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.6);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
}
.btn-new:hover { background: rgba(var(--surface-3-rgb, 32, 32, 44), 0.8); color: var(--text-primary); }
.btn-new svg { width: 1rem; height: 1rem; }

.content { flex: 1; overflow-y: auto; padding: 1.75rem; background: transparent; }

.login-vortex {
  position: absolute;
  inset: 0;
  background: var(--pageBg);
}

.login-state {
  min-height: 100%;
  padding: 2rem;
}

.login-state .empty-icon {
  background: rgba(var(--keyColor-rgb), 0.15);
  border-radius: 50%;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}

.login-state .empty-icon svg {
  color: var(--keyColor);
}

.login-auth-btn {
  margin-top: 1rem;
  width: 280px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  text-align: center;
}
.empty-icon {
  width: 6rem;
  height: 6rem;
  margin-bottom: 2rem;
  color: var(--text-muted);
}
.empty-icon svg { width: 100%; height: 100%; }
.empty-state h2 { 
  font-size: 1.75rem; 
  font-weight: 700; 
  margin-bottom: 0.75rem;
  letter-spacing: -0.02em;
}
.empty-state p { 
  color: var(--text-secondary); 
  margin-bottom: 2rem;
  font-size: 1.0625rem;
}
.repo-setup { display: flex; gap: 0.75rem; }
.repo-setup input {
  padding: 0.875rem 1.125rem;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.6);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 0.9375rem;
  width: 280px;
}
.repo-setup input:focus {
  outline: none;
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px var(--accent-light);
}
.btn-primary {
  padding: 0.875rem 1.75rem;
  background: linear-gradient(135deg, var(--accent-color), var(--accent-secondary));
  border: none;
  color: #000;
  font-weight: 700;
  font-size: 0.9375rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  box-shadow: 0 4px 20px rgba(var(--accent-rgb), 0.4);
  transition: all var(--duration-fast);
}
.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(var(--accent-rgb), 0.5);
}
.or-divider {
  color: var(--text-muted);
  font-size: 0.9375rem;
  margin: 1.25rem 0;
}
.btn-secondary {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.875rem 1.75rem;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.6);
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
  font-size: 0.9375rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast);
}
.btn-secondary:hover { 
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.8); 
  color: var(--text-primary);
  border-color: var(--border-strong);
}
.btn-secondary svg { width: 1.125rem; height: 1.125rem; }

.floating-breadcrumb {
  position: fixed;
  bottom: 24px;
  left: 24px;
  right: 24px;
  display: flex;
  justify-content: space-between;
  z-index: 80;
  pointer-events: none;
}

.bottom-pill {
  display: inline-flex;
  align-items: center;
  padding: 10px 16px;
  pointer-events: auto;
}

.bottom-pill .pill-text {
  color: rgba(255, 255, 255, 0.85);
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.bottom-pill .pill-sep {
  color: rgba(255, 255, 255, 0.3);
  margin: 0 8px;
}

@media (max-width: 768px) {
  .floating-breadcrumb {
    display: none;
  }
}

.upload-btn-small {
  box-shadow: 0 2px 8px rgba(var(--accent-rgb), 0.25) !important;
}

.upload-btn-small:hover {
  box-shadow: 0 4px 12px rgba(var(--accent-rgb), 0.35) !important;
}

.neural-grid {
  width: 100%;
  height: 100%;
  background-image: 
    linear-gradient(rgba(0, 255, 255, 0.1) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 255, 255, 0.1) 1px, transparent 1px);
  background-size: 50px 50px;
  animation: neuralFlow 20s linear infinite;
}

@keyframes neuralFlow {
  0% { transform: translate(0, 0); }
  100% { transform: translate(50px, 50px); }
}

.fade-enter-active, .fade-leave-active { transition: opacity var(--duration-normal) var(--ease-out); }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.slide-enter-active, .slide-leave-active { transition: all var(--duration-normal) var(--ease-out); }
.slide-enter-from, .slide-leave-to { opacity: 0; transform: translateY(-10px); }
.toast-enter-active { transition: all 0.3s var(--ease-spring); }
.toast-leave-active { transition: all var(--duration-fast); }
.toast-enter-from { opacity: 0; transform: translate(-50%, 20px); }
.toast-leave-to { opacity: 0; transform: translate(-50%, 10px); }

.dock-fade-enter-active { transition: opacity 0.3s ease-out, transform 0.3s ease-out; }
.dock-fade-leave-active { transition: opacity 0.2s ease-out, transform 0.2s ease-out; }
.dock-fade-enter-from { opacity: 0; transform: translateY(20px); }
.dock-fade-leave-to { opacity: 0; transform: translateY(20px); }

.pull-indicator {
  display: none;
}

.mobile-search-fab {
  display: none;
}

.mobile-search-overlay {
  display: none;
}

.desktop-search {
  display: flex;
}

@media (max-width: 768px) {
  
  .pull-indicator {
    display: flex;
    position: fixed;
    top: -80px;
    left: 0;
    right: 0;
    height: 80px;
    padding-top: env(safe-area-inset-top, 0px);
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--accent-color, #007aff);
    font-size: 13px;
    font-weight: 500;
    z-index: 150;
    pointer-events: none;
    transition: opacity 0.15s ease;
  }
  
  .pull-indicator svg {
    width: 18px;
    height: 18px;
  }
  .desktop-search {
    display: none !important;
  }
  
  .mobile-search-fab {
    display: flex;
    position: fixed;
    bottom: 200px;
    right: 16px;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: 
      radial-gradient(circle at 30% 30%, rgba(0, 255, 255, 0.8) 0%, transparent 50%),
      radial-gradient(circle at 70% 70%, rgba(255, 0, 255, 0.8) 0%, transparent 50%),
      radial-gradient(circle at 50% 50%, rgba(0, 255, 127, 0.9) 0%, transparent 70%),
      linear-gradient(45deg, #001122, #003366, #0066cc);
    animation: liquidFlow 6s ease-in-out infinite, fluidPulse 3s ease-in-out infinite;
    border: 2px solid transparent;
    background-clip: padding-box;
    align-items: center;
    justify-content: center;
    box-shadow: 
      0 0 40px rgba(0, 255, 255, 0.4),
      inset 0 0 20px rgba(0, 255, 127, 0.2);
    z-index: 90;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
    position: relative;
    overflow: hidden;
    backdrop-filter: blur(10px);
  }
  
  .mobile-search-fab::before {
    content: '';
    position: absolute;
    inset: -3px;
    border-radius: 50%;
    background: conic-gradient(
      from 0deg,
      #00ffff 0deg,
      #ff00ff 60deg,
      #00ff7f 120deg,
      #ffff00 180deg,
      #ff0080 240deg,
      #0080ff 300deg,
      #00ffff 360deg
    );
    animation: borderSpin 4s linear infinite, borderPulse 2s ease-in-out infinite;
    z-index: -1;
  }
  
  .mobile-search-fab::after {
    content: '';
    position: absolute;
    width: 3px;
    height: 3px;
    background: rgba(255, 255, 255, 0.9);
    border-radius: 50%;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    animation: nanoParticles 5s linear infinite;
    box-shadow: 
      10px 4px 0 0px rgba(0, 255, 255, 0.8),
      -5px 12px 0 -1px rgba(255, 0, 255, 0.7),
      15px -3px 0 0px rgba(0, 255, 127, 0.6),
      -3px -5px 0 -1px rgba(255, 255, 0, 0.5),
      8px 15px 0 -1px rgba(255, 0, 128, 0.4);
  }
  
  @keyframes liquidFlow {
    0% { 
      border-radius: 50%;
      background: 
        radial-gradient(circle at 30% 30%, rgba(0, 255, 255, 0.8) 0%, transparent 50%),
        radial-gradient(circle at 70% 70%, rgba(255, 0, 255, 0.8) 0%, transparent 50%),
        radial-gradient(circle at 50% 50%, rgba(0, 255, 127, 0.9) 0%, transparent 70%),
        linear-gradient(45deg, #001122, #003366, #0066cc);
    }
    16.66% { 
      border-radius: 60% 40% 30% 70% / 60% 30% 70% 40%;
      background: 
        radial-gradient(circle at 70% 20%, rgba(0, 255, 255, 0.9) 0%, transparent 60%),
        radial-gradient(circle at 20% 80%, rgba(255, 0, 255, 0.7) 0%, transparent 45%),
        radial-gradient(circle at 60% 60%, rgba(0, 255, 127, 0.8) 0%, transparent 65%),
        linear-gradient(75deg, #002244, #004488, #0088ff);
    }
    33.33% { 
      border-radius: 30% 70% 70% 30% / 30% 60% 40% 70%;
      background: 
        radial-gradient(circle at 50% 80%, rgba(0, 255, 255, 0.7) 0%, transparent 55%),
        radial-gradient(circle at 80% 20%, rgba(255, 0, 255, 0.9) 0%, transparent 50%),
        radial-gradient(circle at 20% 40%, rgba(0, 255, 127, 0.8) 0%, transparent 60%),
        linear-gradient(105deg, #001133, #0055aa, #00aaff);
    }
    50% { 
      border-radius: 70% 30% 40% 60% / 40% 70% 30% 60%;
      background: 
        radial-gradient(circle at 20% 60%, rgba(0, 255, 255, 0.8) 0%, transparent 50%),
        radial-gradient(circle at 80% 40%, rgba(255, 0, 255, 0.8) 0%, transparent 55%),
        radial-gradient(circle at 40% 20%, rgba(0, 255, 127, 0.9) 0%, transparent 65%),
        linear-gradient(135deg, #003355, #0066bb, #00ccff);
    }
    66.66% { 
      border-radius: 40% 60% 60% 40% / 70% 30% 60% 40%;
      background: 
        radial-gradient(circle at 60% 40%, rgba(0, 255, 255, 0.9) 0%, transparent 60%),
        radial-gradient(circle at 30% 60%, rgba(255, 0, 255, 0.7) 0%, transparent 45%),
        radial-gradient(circle at 80% 80%, rgba(0, 255, 127, 0.8) 0%, transparent 65%),
        linear-gradient(165deg, #002244, #004488, #0088ff);
    }
    83.33% { 
      border-radius: 55% 45% 35% 65% / 45% 65% 35% 55%;
      background: 
        radial-gradient(circle at 40% 70%, rgba(0, 255, 255, 0.8) 0%, transparent 55%),
        radial-gradient(circle at 70% 30%, rgba(255, 0, 255, 0.9) 0%, transparent 50%),
        radial-gradient(circle at 30% 30%, rgba(0, 255, 127, 0.8) 0%, transparent 60%),
        linear-gradient(195deg, #001133, #0055aa, #00aaff);
    }
    100% { 
      border-radius: 50%;
      background: 
        radial-gradient(circle at 30% 30%, rgba(0, 255, 255, 0.8) 0%, transparent 50%),
        radial-gradient(circle at 70% 70%, rgba(255, 0, 255, 0.8) 0%, transparent 50%),
        radial-gradient(circle at 50% 50%, rgba(0, 255, 127, 0.9) 0%, transparent 70%),
        linear-gradient(45deg, #001122, #003366, #0066cc);
    }
  }
  
  @keyframes fluidPulse {
    0%, 100% { 
      transform: scale(1);
      box-shadow: 
        0 0 40px rgba(0, 255, 255, 0.4),
        inset 0 0 20px rgba(0, 255, 127, 0.2);
    }
    50% { 
      transform: scale(1.05);
      box-shadow: 
        0 0 60px rgba(0, 255, 255, 0.6),
        inset 0 0 30px rgba(0, 255, 127, 0.4);
    }
  }
  
  @keyframes borderSpin {
    0% { 
      transform: rotate(0deg) scale(1);
      background: conic-gradient(
        from 0deg,
        #00ffff 0deg,
        #ff00ff 60deg,
        #00ff7f 120deg,
        #ffff00 180deg,
        #ff0080 240deg,
        #0080ff 300deg,
        #00ffff 360deg
      );
    }
    100% { 
      transform: rotate(360deg) scale(1);
      background: conic-gradient(
        from 360deg,
        #00ffff 0deg,
        #ff00ff 60deg,
        #00ff7f 120deg,
        #ffff00 180deg,
        #ff0080 240deg,
        #0080ff 300deg,
        #00ffff 360deg
      );
    }
  }
  
  @keyframes borderPulse {
    0%, 100% { 
      opacity: 0.8;
      filter: blur(0px);
    }
    50% { 
      opacity: 1;
      filter: blur(1px);
    }
  }
  
  @keyframes nanoParticles {
    0% { transform: translate(0, 0) rotate(0deg); opacity: 1; }
    20% { transform: translate(12px, -6px) rotate(72deg); opacity: 0.8; }
    40% { transform: translate(-4px, 15px) rotate(144deg); opacity: 0.6; }
    60% { transform: translate(-15px, -8px) rotate(216deg); opacity: 0.9; }
    80% { transform: translate(8px, -12px) rotate(288deg); opacity: 0.7; }
    100% { transform: translate(0, 0) rotate(360deg); opacity: 1; }
  }
  
  .mobile-search-fab svg {
    width: 24px;
    height: 24px;
    color: white;
  }
  
  .mobile-search-fab:active {
    transform: scale(0.92);
  }
  
  .mobile-search-fab.hidden {
    opacity: 0;
    transform: scale(0.5);
    pointer-events: none;
  }
  
  .mobile-search-overlay {
    display: flex;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(15, 25, 35, 0.75);
    backdrop-filter: blur(40px) saturate(180%) brightness(0.9);
    -webkit-backdrop-filter: blur(40px) saturate(180%) brightness(0.9);
    z-index: 200;
    flex-direction: column;
    padding-top: env(safe-area-inset-top, 0px);
    transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
    overflow: hidden;
  }
  
  .mobile-search-container {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    width: 100%;
    box-sizing: border-box;
  }
  
  .pull-close-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    color: rgba(255, 255, 255, 0.6);
    font-size: 12px;
    transition: opacity 0.15s, transform 0.15s;
  }
  
  .pull-close-bar {
    width: 40px;
    height: 4px;
    background: rgba(255, 255, 255, 0.3);
    border-radius: 2px;
  }
  
  .mobile-search-results {
    flex: 1;
    width: 100%;
    height: 100%;
    overflow-y: scroll;
    -webkit-overflow-scrolling: touch;
  }
  
  .mobile-search-results.no-scroll {
    overflow: hidden;
    touch-action: none;
  }
  
  .mobile-search-input {
    flex: 1;
    min-width: 0;
  }
  
  .mobile-search-input :deep(input) {
    height: 48px !important;
    font-size: 17px !important;
  }
  
  .mobile-search-cancel {
    background: none;
    border: none;
    color: var(--accent-color, #007aff);
    font-size: 17px;
    font-weight: 500;
    padding: 12px 4px;
    cursor: pointer;
    white-space: nowrap;
    transition: opacity 0.15s;
  }
  
  .mobile-search-cancel:active {
    opacity: 0.6;
  }
  
  .search-empty {
    text-align: center;
    color: rgba(255, 255, 255, 0.5);
    font-size: 14px;
    padding: 40px 20px;
  }
  
  .scroll-spacer {
    min-height: 100vh;
    width: 100%;
  }
  
  .search-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 20px 32px;
    margin: 0 auto;
    margin-top: 20vh;
    background: rgba(255, 255, 255, 0.06);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 16px;
    color: rgba(255, 255, 255, 0.5);
    font-size: 14px;
    text-align: center;
    width: fit-content;
    animation: hint-fade-in 0.5s ease-out 0.3s backwards;
  }
  
  .search-hint .hint-icon {
    width: 32px;
    height: 32px;
    opacity: 0.4;
  }
  
  @keyframes hint-fade-in {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  .search-results-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 3px;
  }
  
  .search-result-item {
    position: relative;
    aspect-ratio: 1;
    overflow: hidden;
    border-radius: 4px;
    background: #111;
    animation: result-fade-in 0.3s ease-out backwards;
  }
  
  .search-result-item:nth-child(1) { animation-delay: 0.05s; }
  .search-result-item:nth-child(2) { animation-delay: 0.1s; }
  .search-result-item:nth-child(3) { animation-delay: 0.15s; }
  .search-result-item:nth-child(4) { animation-delay: 0.2s; }
  .search-result-item:nth-child(5) { animation-delay: 0.25s; }
  .search-result-item:nth-child(6) { animation-delay: 0.3s; }
  
  @keyframes result-fade-in {
    from { opacity: 0; transform: scale(0.9); }
    to { opacity: 1; transform: scale(1); }
  }
  
  .search-result-item img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.2s;
  }
  
  .search-result-item:active img {
    transform: scale(0.95);
  }
  
  .search-result-name {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 4px 6px;
    font-size: 10px;
    color: white;
    background: linear-gradient(transparent, rgba(0,0,0,0.8));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mobile-search-enter-active {
    animation: search-overlay-in 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  
  .mobile-search-enter-active .mobile-search-container {
    animation: search-bar-bounce 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  
  .mobile-search-leave-active {
    animation: search-overlay-out 0.25s ease-out forwards;
  }
  
  .mobile-search-leave-active .mobile-search-container {
    animation: search-bar-out 0.2s ease-out forwards;
  }
  
  @keyframes search-overlay-in {
    0% { opacity: 0; }
    100% { opacity: 1; }
  }
  
  @keyframes search-overlay-out {
    0% { opacity: 1; }
    100% { opacity: 0; }
  }
  
  @keyframes search-bar-bounce {
    0% { 
      transform: translateY(-100%) scale(0.95); 
      opacity: 0;
    }
    60% { 
      transform: translateY(8px) scale(1.02); 
    }
    100% { 
      transform: translateY(0) scale(1); 
      opacity: 1;
    }
  }
  
  @keyframes search-bar-out {
    0% { transform: translateY(0); opacity: 1; }
    100% { transform: translateY(-50px); opacity: 0; }
  }
}

.mobile-upload-container {
  position: fixed;
  bottom: 130px;
  right: 16px;
  z-index: 45;
  overflow: visible;
}

.mobile-upload-btn {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: 
    radial-gradient(circle at 30% 30%, rgba(255, 45, 106, 0.8) 0%, transparent 50%),
    radial-gradient(circle at 70% 70%, rgba(176, 38, 255, 0.8) 0%, transparent 50%),
    radial-gradient(circle at 50% 50%, rgba(255, 107, 53, 0.9) 0%, transparent 70%),
    linear-gradient(45deg, #1a0011, #330022, #660044);
  animation: uploadLiquidFlow 6s ease-in-out infinite, uploadFluidPulse 3s ease-in-out infinite;
  border: none;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 
    0 0 20px rgba(255, 45, 106, 0.4),
    inset 0 0 10px rgba(255, 107, 53, 0.2);
  z-index: 90;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  position: relative;
  overflow: hidden;
  backdrop-filter: blur(10px);
}

.mobile-upload-btn::before {
  content: '';
  position: absolute;
  inset: -3px;
  border-radius: 50%;
  background: conic-gradient(
    from 0deg,
    #ff2d6a 0deg,
    #b026ff 60deg,
    #ff6b35 120deg,
    #ffd700 180deg,
    #ff2d6a 240deg,
    #00f0ff 300deg,
    #ff2d6a 360deg
  );
  animation: borderSpin 4s linear infinite, borderPulse 2s ease-in-out infinite;
  z-index: -1;
}

.mobile-upload-btn::after {
  content: '';
  position: absolute;
  width: 3px;
  height: 3px;
  background: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: nanoParticles 5s linear infinite;
  box-shadow: 
    10px 4px 0 0px rgba(255, 45, 106, 0.8),
    -5px 12px 0 -1px rgba(176, 38, 255, 0.7),
    15px -3px 0 0px rgba(255, 107, 53, 0.6),
    -3px -5px 0 -1px rgba(255, 215, 0, 0.5),
    8px 15px 0 -1px rgba(0, 240, 255, 0.4);
}

@keyframes uploadLiquidFlow {
  0%, 100% { 
    border-radius: 50%;
    background: 
      radial-gradient(circle at 30% 30%, rgba(255, 45, 106, 0.8) 0%, transparent 50%),
      radial-gradient(circle at 70% 70%, rgba(176, 38, 255, 0.8) 0%, transparent 50%),
      radial-gradient(circle at 50% 50%, rgba(255, 107, 53, 0.9) 0%, transparent 70%),
      linear-gradient(45deg, #1a0011, #330022, #660044);
  }
  50% { 
    border-radius: 45% 55% 55% 45% / 55% 45% 55% 45%;
    background: 
      radial-gradient(circle at 70% 30%, rgba(255, 45, 106, 0.9) 0%, transparent 55%),
      radial-gradient(circle at 30% 70%, rgba(176, 38, 255, 0.7) 0%, transparent 45%),
      radial-gradient(circle at 50% 50%, rgba(255, 107, 53, 0.8) 0%, transparent 65%),
      linear-gradient(135deg, #220011, #440033, #880055);
  }
}

@keyframes uploadFluidPulse {
  0%, 100% { 
    transform: scale(1);
    box-shadow: 
      0 0 40px rgba(255, 45, 106, 0.4),
      inset 0 0 20px rgba(255, 107, 53, 0.2);
  }
  50% { 
    transform: scale(1.02);
    box-shadow: 
      0 0 60px rgba(255, 45, 106, 0.6),
      inset 0 0 30px rgba(255, 107, 53, 0.3);
  }
}

.mobile-upload-btn:active {
  transform: scale(0.92);
}

.mobile-upload-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  animation: none;
}

.mobile-upload-btn:disabled::before,
.mobile-upload-btn:disabled::after {
  animation: none;
}

.error-toast {
  position: fixed;
  bottom: 1.75rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 0.875rem;
  padding: 1rem 1.5rem;
  background: rgba(var(--error-rgb), 0.15);
  border: 1px solid rgba(var(--error-rgb), 0.4);
  border-radius: var(--radius-lg);
  color: #ffc7c4;
  font-size: 0.9375rem;
  font-weight: 500;
  backdrop-filter: blur(16px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  z-index: 1000;
}
.error-toast svg { width: 1.375rem; height: 1.375rem; flex-shrink: 0; color: var(--error); }
.error-toast button {
  width: 1.75rem;
  height: 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: var(--radius-sm);
  color: #ffc7c4;
  cursor: pointer;
  margin-left: 0.5rem;
  transition: all var(--duration-fast);
}
.error-toast button:hover { background: rgba(255, 255, 255, 0.2); }
.error-toast button svg { width: 1rem; height: 1rem; }

.cursor-area {
  width: 100%;
  min-height: 100vh;
}

.cursor-pointer {
  filter: drop-shadow(0 0 6px rgba(255, 255, 255, 0.6));
}

@media (hover: none), (max-width: 768px) {
  .cursor-pointer {
    display: none !important;
  }
  .cursor-area {
    cursor: auto !important;
  }
}
</style>