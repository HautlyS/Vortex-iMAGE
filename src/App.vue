<script setup lang="ts">
import { ref, onMounted, watch, computed, onUnmounted } from 'vue'
import { useGitHubAuth, isWebMode } from './composables/useGitHubAuth'
import { usePhotoUpload, type Photo, type UploadStatus } from './composables/usePhotoUpload'
import { useUploadToast } from './composables/useUploadToast'
import { useAccentColor } from './composables/useAccentColor'
import { useTheme } from './composables/useTheme'
import { useSelection } from './composables/useSelection'
import { useFavorites } from './composables/useFavorites'
import { useColorTags, PREDEFINED_COLORS } from './composables/useColorTags'
import { usePhotoPreviewSize } from './composables/usePhotoPreviewSize'
import { useDataDriver, type DataDriver } from './composables/useDataDriver'
import { useBackupSettings } from './composables/useBackupSettings'
import { useSmartAlbums } from './composables/useSmartAlbums'
import { useTimeout } from './composables/useTimeout'
import { useDockApps, type DockView } from './composables/useDockApps'
import { useToast } from './composables/useToast'
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts'
import { 
  UPLOAD, SHORTCUTS, TIMING,
  injectCSSVariables 
} from './config'
import SpaceLoader from './components/SpaceLoader.vue'
import AuthButton from './components/AuthButton.vue'
import VoxelBackground from './components/retro/VoxelBackground.vue'
import PixelCard from './components/retro/PixelCard.vue'
import PhotoGallery from './components/PhotoGallery.vue'
import PrivacySettings from './components/PrivacySettings.vue'
import FolderUploadDialog from './components/FolderUploadDialog.vue'
import FolderCreator from './components/FolderCreator.vue'
import RepoCreator from './components/RepoCreator.vue'
import AlbumTree from './components/AlbumTree.vue'
import SmartAlbumList from './components/SmartAlbumList.vue'
import ContextMenu, { type ContextMenuItem } from './components/ContextMenu.vue'
import UploadToast from './components/UploadToast.vue'
import ThemeSettings from './components/ThemeSettings.vue'
import DataDriverManager from './components/DataDriverManager.vue'
import BackupSettings from './components/BackupSettings.vue'
import LocalImageBrowser from './components/LocalImageBrowser.vue'
import SecuritySettings from './components/SecuritySettings.vue'
import MediaSettingsPanel from './components/MediaSettingsPanel.vue'
import UploadSettingsDialog from './components/UploadSettingsDialog.vue'
import MacOSDock from './components/MacOSDock.vue'
import ErrorBoundary from './components/ErrorBoundary.vue'
import AsyncState from './components/AsyncState.vue'
import ToastContainer from './components/ui/ToastContainer.vue'
import WebLoginStepper from './components/WebLoginStepper.vue'
import BrowserFilePicker from './components/BrowserFilePicker.vue'
import type { ProcessingSettings } from './composables/useMediaSettings'

// Initialize toast system
useToast()

// Initialize global keyboard shortcuts system
useKeyboardShortcuts()

interface Album {
  name: string
  path: string
  photo_count: number
  children: Album[]
}

const { token, repo, init, setRepo } = useGitHubAuth()
const { photos, loadingPhotos, loadPhotos, addToQueue, addFilesToQueue, queue } = usePhotoUpload()
const { addTransfer, updateProgress, setStatus: setTransferStatus } = useUploadToast()
const { init: initAccent } = useAccentColor()
const { loadTheme } = useTheme()
const { selected, clearSelection, selectAll, getSelected } = useSelection()
const { loadFavorites, isFavorite, toggleFavorite } = useFavorites()
const { loadTags, tagItems } = useColorTags()
const { size: previewSize, setSize: setPreviewSize, loadSize: loadPreviewSize } = usePhotoPreviewSize()
const { loadDrivers } = useDataDriver()
const { loadConfig: loadBackupConfig } = useBackupSettings()
const { albums: smartAlbums, generateAlbums: generateSmartAlbums } = useSmartAlbums()
const { createTimeout } = useTimeout()
const { dockApps, activeView, setActiveView } = useDockApps()

// App state
const loading = ref(true)
const repoInput = ref('')
const isDragging = ref(false)
const showSettings = ref(false)
const showPrivacy = ref(false)
const showRepoCreator = ref(false)
const showFolderCreator = ref(false)
const showThemeSettings = ref(false)
const showDataDrivers = ref(false)
const showBackupSettings = ref(false)
const showLocalBrowser = ref(false)
const showSecuritySettings = ref(false)
const showMediaSettings = ref(false)
const mediaSettingsMode = ref<'global' | 'album' | 'item'>('global')
const mediaSettingsAlbumPath = ref<string | undefined>(undefined)
const mediaSettingsAlbumName = ref<string | undefined>(undefined)
const showAlbumPanel = ref(false)
const searchQuery = ref('')
const debouncedSearchQuery = ref('')

// Debounce search to improve performance - instance-specific
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
const viewMode = ref<'grid' | 'list'>('grid')
const uploadError = ref<string | null>(null)

// Navigation state - now driven by dock
const selectedAlbumPath = ref<string | null>(null)
const selectedSmartAlbumId = ref<string | null>(null)
const selectedTagId = ref<string | null>(null)

// Folder upload dialog
const showFolderDialog = ref(false)
const pendingFolderPath = ref<string | null>(null)

// Upload settings dialog
const showUploadSettings = ref(false)
const pendingUploadFiles = ref<string[]>([])

// Web mode state
const showWebLoginStepper = ref(false)
const showBrowserFilePicker = ref(false)

// Context menu
const contextMenu = ref<{ x: number; y: number; items: ContextMenuItem[] } | null>(null)

// Albums
const albums = ref<Album[]>([])
const loadingAlbums = ref(false)

// Filtered photos based on current view (dock navigation)
const filteredPhotos = computed(() => {
  let result = photos.value as Photo[]
  
  // Filter by search (debounced)
  if (debouncedSearchQuery.value) {
    const q = debouncedSearchQuery.value.toLowerCase()
    result = result.filter(p => p.name.toLowerCase().includes(q))
  }
  
  // Filter by dock view
  switch (activeView.value) {
    case 'favorites':
      result = result.filter(p => isFavorite(p.sha))
      break
    case 'albums':
      // Show photos from selected album only
      if (selectedAlbumPath.value) {
        result = result.filter(p => p.path?.startsWith(selectedAlbumPath.value!))
      } else if (selectedSmartAlbumId.value) {
        const album = smartAlbums.value.find(a => a.id === selectedSmartAlbumId.value)
        if (album) result = result.filter(p => album.photoIds.includes(p.name))
      } else {
        // No album selected - show empty until user selects one
        result = []
      }
      break
    case 'all-albums':
      // Show all photos organized by albums (flat view)
      // For now, show all photos - the gallery will show album structure
      break
    case 'photos':
    default:
      // Show all photos
      break
  }
  
  return result
})

const uploadProgress = computed(() => {
  return queue.value.filter(u => u.status === 'uploading' || u.status === 'pending').length
})

const viewTitle = computed(() => {
  switch (activeView.value) {
    case 'favorites': return 'FAVORITOS'
    case 'albums': 
      if (selectedAlbumPath.value) return selectedAlbumPath.value.split('/').pop()?.toUpperCase()
      if (selectedSmartAlbumId.value) return smartAlbums.value.find(a => a.id === selectedSmartAlbumId.value)?.title
      return 'ÁLBUNS'
    case 'all-albums': return 'TODOS ÁLBUNS'
    default: return 'TODAS FOTOS'
  }
})

// Load albums from GitHub
async function loadAlbums() {
  if (!token.value || !repo.value) return
  
  // Web mode: albums not supported yet
  if (isWebMode) {
    albums.value = []
    return
  }
  
  loadingAlbums.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    albums.value = await invoke<Album[]>('list_albums', { token: token.value, repo: repo.value })
  } catch {
    albums.value = []
  } finally {
    loadingAlbums.value = false
  }
}

// Handle browser file selection (web mode)
async function handleBrowserFileSelect(files: File[]) {
  showBrowserFilePicker.value = false
  
  if (files.length === 0) return
  
  // Use the queue system for web uploads
  addFilesToQueue(files, selectedAlbumPath.value || undefined)
}

onMounted(async () => {
  // Inject CSS variables from config
  injectCSSVariables()
  
  // Add base effect classes to html element
  // document.documentElement.classList.add('retro-mode')
  
  await Promise.all([init(), initAccent(), loadTheme(), loadFavorites(), loadTags(), loadPreviewSize(), loadDrivers(), loadBackupConfig()])
  repoInput.value = repo.value
  
  // Load view mode from storage (skip in web mode)
  if (!isWebMode) {
    try {
      const { load } = await import('@tauri-apps/plugin-store')
      const store = await load('settings.json')
      const savedViewMode = await store.get<'grid' | 'list'>('viewMode')
      if (savedViewMode) viewMode.value = savedViewMode
    } catch {}
  }
})

watch(repo, (v) => { repoInput.value = v })
watch([token, repo], () => { 
  if (token.value && repo.value) {
    loadPhotos()
    loadAlbums()
    // Smart albums will be generated via watch(photos)
  }
})

// Sync upload queue with toast notifications - optimized with computed tracking
const queueStatusMap = new Map<string, UploadStatus>()

// Use computed for better performance tracking
const queueItems = computed(() => 
  queue.value.map(i => ({ id: i.id, status: i.status, progress: i.progress, name: i.name, error: i.error }))
)

watch(photos, () => {
    generateSmartAlbums(photos.value)
}, { deep: true })

watch(queueItems, (items) => {
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

// Save view mode when changed
watch(viewMode, async (mode) => {
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    await store.set('viewMode', mode)
    await store.save()
  } catch {}
})

// Keyboard shortcuts using config
function handleKeydown(e: KeyboardEvent) {
  const { selectAll: selectAllKey, favorite: favKey, escape: escKey } = SHORTCUTS
  
  // Ctrl+A - Select all
  if ((e.ctrlKey || e.metaKey) && e.key === selectAllKey.key) {
    e.preventDefault()
    selectAll(filteredPhotos.value.map(p => p.sha))
  }
  // Delete - Remove selected (placeholder)
  if (e.key === 'Delete' && selected.value.size > 0) {
    e.preventDefault()
    // TODO: Implement delete
  }
  // F - Toggle favorite
  if (e.key === favKey.key && selected.value.size > 0) {
    e.preventDefault()
    const selectedIds = getSelected()
    for (const id of selectedIds) {
      const photo = photos.value.find(p => p.sha === id)
      if (photo) toggleFavorite({ type: 'photo', id: photo.sha, path: photo.name })
    }
  }
  // Escape - Clear selection / close menus
  if (e.key === escKey.key) {
    clearSelection()
    contextMenu.value = null
  }
}

onMounted(() => document.addEventListener('keydown', handleKeydown))
onUnmounted(() => document.removeEventListener('keydown', handleKeydown))

async function handleUploadClick() {
  uploadError.value = null
  
  // Web mode: use browser file picker
  if (isWebMode) {
    showBrowserFilePicker.value = true
    return
  }
  
  // Tauri mode: use native dialog
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const files = await open({ 
      multiple: true, 
      directory: false, 
      filters: [{ name: 'Images', extensions: [...UPLOAD.supportedFormats] }] 
    })
    if (files) {
      const fileList = Array.isArray(files) ? files : [files]
      pendingUploadFiles.value = fileList
      showUploadSettings.value = true
    }
  } catch (e) {
    uploadError.value = e instanceof Error ? e.message : 'Erro ao selecionar arquivos'
  }
}

function handleUploadConfirm(settings: ProcessingSettings, password?: string) {
  showUploadSettings.value = false
  addToQueue(pendingUploadFiles.value, selectedAlbumPath.value || undefined, settings, password)
  pendingUploadFiles.value = []
}

function handleUploadCancel() {
  showUploadSettings.value = false
  pendingUploadFiles.value = []
}

async function handleFolderClick() {
  uploadError.value = null
  
  // Web mode: use browser folder picker
  if (isWebMode) {
    showBrowserFilePicker.value = true
    return
  }
  
  // Tauri mode: use native dialog
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const folder = await open({ multiple: false, directory: true })
    if (folder && typeof folder === 'string') {
      pendingFolderPath.value = folder
      showFolderDialog.value = true
    }
  } catch (e) {
    uploadError.value = e instanceof Error ? e.message : 'Erro ao selecionar pasta'
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
    // Check if it's a folder (simple heuristic: no extension)
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
  
  // Web mode: folder upload not supported
  if (isWebMode) {
    uploadError.value = 'Folder upload is only available in the desktop app'
    pendingFolderPath.value = null
    return
  }
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
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

function handleFolderCreated(_path: string) {
  showFolderCreator.value = false
  loadAlbums()
}

// Data Driver handlers
function handleDriverChanged(driver: DataDriver) {
  showDataDrivers.value = false
  if (driver.type === 'github') {
    setRepo(driver.path)
  }
}

// Local image import handler
async function handleLocalImport(imagePaths: string[], _targetDriverId: string) {
  showLocalBrowser.value = false
  void _targetDriverId // suppress unused warning
  // Add images to upload queue
  addToQueue(imagePaths)
}

// Navigation handlers - dock based
function handleDockClick(appId: string) {
  setActiveView(appId as DockView)
  selectedAlbumPath.value = null
  selectedSmartAlbumId.value = null
  selectedTagId.value = null
  clearSelection()
  
  // Show album panel when albums view is selected
  if (appId === 'albums' || appId === 'all-albums') {
    showAlbumPanel.value = true
  } else {
    showAlbumPanel.value = false
  }
}

function handleAlbumSelect(albumOrPath: Album | string | null) {
  const path = typeof albumOrPath === 'string' || albumOrPath === null 
    ? albumOrPath 
    : albumOrPath.path || null
  setActiveView('albums')
  selectedAlbumPath.value = path
  selectedSmartAlbumId.value = null
  clearSelection()
}

function handleOpenAlbumSettings(albumPath: string, albumName: string) {
  mediaSettingsMode.value = 'album'
  mediaSettingsAlbumPath.value = albumPath
  mediaSettingsAlbumName.value = albumName
  showMediaSettings.value = true
}

function handleSmartAlbumSelect(id: string) {
  setActiveView('albums')
  selectedSmartAlbumId.value = id
  selectedAlbumPath.value = null
  clearSelection()
}

// Context menu
function showContextMenu(e: MouseEvent) {
  const selectedIds = getSelected()
  if (selectedIds.length === 0) return
  
  const colorSubmenu: ContextMenuItem[] = PREDEFINED_COLORS.map(color => ({
    id: `color-${color.id}`,
    label: color.name,
    color: color.color,
    action: () => tagItems(selectedIds, color.id)
  }))
  
  const items: ContextMenuItem[] = [
    {
      id: 'favorite',
      label: 'Favoritar',
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>',
      action: () => {
        for (const id of selectedIds) {
          const photo = photos.value.find(p => p.sha === id)
          if (photo) toggleFavorite({ type: 'photo', id: photo.sha, path: photo.name })
        }
      }
    },
    {
      id: 'color-tag',
      label: 'Etiqueta de Cor',
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/></svg>',
      submenu: colorSubmenu
    },
    { id: 'divider-1', label: '', divider: true },
    {
      id: 'copy-url',
      label: 'Copiar URL',
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>',
      action: () => {
        const urls = selectedIds.map(id => photos.value.find(p => p.sha === id)?.url).filter(Boolean)
        navigator.clipboard.writeText(urls.join('\n'))
      }
    },
    { id: 'divider-2', label: '', divider: true },
    {
      id: 'delete',
      label: 'Excluir',
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>',
      disabled: true
    }
  ]
  
  contextMenu.value = { x: e.clientX, y: e.clientY, items }
}

function handlePhotoContextMenu(e: MouseEvent) {
  showContextMenu(e)
}

// Preview size handler
function handlePreviewResize(newSize: number) {
  setPreviewSize(newSize)
}

// Dismiss error
function dismissError() {
  uploadError.value = null
}

// Photo loading error state
const photoLoadError = ref<string | null>(null)

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
    <SpaceLoader v-if="loading" @complete="loading = false" />
    
    <div v-else class="app retro-app" @dragover.prevent="isDragging = true" @dragleave.prevent="isDragging = false" @drop.prevent="onDrop">
      <!-- Retro Voxel Background -->
      <VoxelBackground :count="30" />
      
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

    <!-- Main Content - Full Width (no sidebar) -->
    <main class="main full-width">
      <!-- Header with all controls -->
      <header class="header">
        <div class="header-left">
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
          <h1 class="view-title">{{ viewTitle }}</h1>
          <span v-if="filteredPhotos.length > 0" class="photo-count">{{ filteredPhotos.length }} fotos</span>
        </div>

        <div class="search-bar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
          <input v-model="searchQuery" type="text" placeholder="Pesquisar fotos..." aria-label="Pesquisar fotos" />
          <button v-if="searchQuery" class="search-clear" @click="searchQuery = ''" aria-label="Limpar pesquisa">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
          </button>
        </div>

        <div class="header-actions">
          <!-- View Mode Toggle -->
          <div class="view-toggle">
            <button :class="{ active: viewMode === 'grid' }" @click="viewMode = 'grid'" title="Grade">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
            </button>
            <button :class="{ active: viewMode === 'list' }" @click="viewMode = 'list'" title="Lista">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
            </button>
          </div>

          <!-- Action Buttons -->
          <div class="action-group">
            <button class="btn-action create" @click="showFolderCreator = true" :disabled="!token || !repo" title="Criar pasta">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14m-7-7h14" /></svg>
              <span>Criar</span>
            </button>
            <button class="btn-action upload" @click="handleUploadClick" :disabled="!token || !repo" title="Upload fotos">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17,8 12,3 7,8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
              <span>Upload</span>
              <span v-if="uploadProgress" class="action-badge">{{ uploadProgress }}</span>
            </button>
            <button class="btn-action-icon" @click="handleFolderClick" :disabled="!token || !repo" title="Upload pasta">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
            </button>
            <button class="btn-action-icon" @click="isWebMode ? (showBrowserFilePicker = true) : (showLocalBrowser = true)" :disabled="!token" title="Importar">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>
            </button>
          </div>

          <!-- Settings buttons -->
          <button class="btn-icon" @click="showMediaSettings = true" title="Compressão e Criptografia">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          </button>
          <button class="btn-icon" @click="showDataDrivers = true" title="Fontes de dados">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
          </button>
          <button class="btn-icon" @click="showThemeSettings = true" title="Tema">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
          </button>
          <button class="btn-icon" @click="showSettings = !showSettings" title="Configurações">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          </button>
          <AuthButton class="header-auth" />
        </div>
      </header>

      <!-- Settings Panel -->
      <Transition name="slide">
        <div v-if="showSettings" class="settings-panel">
          <div class="setting-item">
            <label>Repositório GitHub</label>
            <div class="repo-input">
              <input v-model="repoInput" type="text" placeholder="usuario/repositorio" @keyup.enter="saveRepo" />
              <button @click="saveRepo" class="btn-save">Salvar</button>
              <button @click="showRepoCreator = true" class="btn-new" title="Criar novo repositório">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              </button>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Content -->
      <div class="content">
        <template v-if="!token">
          <div class="login-container">
            <div class="empty-state login-state">
              <div class="empty-icon retro-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
              </div>
              <h2>CONECTE SUA CONTA</h2>
              <p v-if="isWebMode">WEB MODE - TOKEN REQUIRED</p>
              <p v-else>LOGIN TO START GAME</p>
              
              <!-- Web mode: show stepper button -->
              <button v-if="isWebMode" class="btn-primary web-login-btn" @click="showWebLoginStepper = true">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="2" y1="12" x2="22" y2="12"/>
                  <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                </svg>
                Setup Web Access
              </button>
              
              <!-- Tauri mode: show auth button -->
              <AuthButton v-else class="login-auth-btn" />
            </div>
          </div>
        </template>

        <template v-else-if="!repo">
          <PixelCard title="MISSION SETUP" :no-padding="false">
            <div class="empty-state">
              <div class="empty-icon retro-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
              </div>
              <h2>SELECT SAVE PATH</h2>
              <p>CHOOSE A REPOSITORY</p>
              <div class="repo-setup">
                <input v-model="repoInput" type="text" placeholder="user/repo" @keyup.enter="saveRepo" />
                <button @click="saveRepo" class="btn-primary">START</button>
              </div>
              <p class="or-divider">OR</p>
              <button @click="showRepoCreator = true" class="btn-secondary">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                NEW SAVE FILE (CREATE)
              </button>
            </div>
          </PixelCard>
        </template>

        <template v-else>
          <AsyncState 
            :loading="loadingPhotos" 
            :error="photoLoadError"
            :empty="!loadingPhotos && filteredPhotos.length === 0 && activeView !== 'albums'"
            empty-message="Nenhuma foto encontrada"
            @retry="retryLoadPhotos"
          >
            <PhotoGallery 
              :photos="filteredPhotos" 
              :loading="false" 
              :view-mode="viewMode"
              :preview-size="previewSize"
              :albums="albums"
              :show-albums="activeView === 'albums' || activeView === 'all-albums'"
              :current-album-path="selectedAlbumPath"
              @refresh="loadPhotos"
              @contextmenu="handlePhotoContextMenu"
              @resize="handlePreviewResize"
              @album-click="handleAlbumSelect"
              @open-album-settings="handleOpenAlbumSettings"
            />
            <template #empty-action>
              <button class="btn-secondary" @click="handleUploadClick">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17,8 12,3 7,8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                Fazer Upload
              </button>
            </template>
          </AsyncState>
        </template>
      </div>
    </main>

    <!-- Modals -->
    <PrivacySettings v-if="showPrivacy" @close="showPrivacy = false" />
    <RepoCreator v-if="showRepoCreator" @created="handleRepoCreated" @close="showRepoCreator = false" />
    <ThemeSettings v-if="showThemeSettings" @close="showThemeSettings = false" />
    <DataDriverManager v-if="showDataDrivers" @close="showDataDrivers = false" @driver-changed="handleDriverChanged" />
    <BackupSettings v-if="showBackupSettings" @close="showBackupSettings = false" />
    <SecuritySettings v-if="showSecuritySettings" @close="showSecuritySettings = false" />
    
    <!-- Web Mode Modals -->
    <WebLoginStepper 
      v-if="showWebLoginStepper" 
      @complete="showWebLoginStepper = false" 
      @close="showWebLoginStepper = false" 
    />
    <BrowserFilePicker 
      v-if="showBrowserFilePicker" 
      @select="handleBrowserFileSelect" 
      @cancel="showBrowserFilePicker = false" 
    />
    
    <MediaSettingsPanel 
      v-if="showMediaSettings" 
      :mode="mediaSettingsMode" 
      :album-path="mediaSettingsAlbumPath"
      :album-name="mediaSettingsAlbumName"
      @close="showMediaSettings = false; mediaSettingsMode = 'global'; mediaSettingsAlbumPath = undefined; mediaSettingsAlbumName = undefined" 
    />
    <LocalImageBrowser v-if="showLocalBrowser" @close="showLocalBrowser = false" @import="handleLocalImport" />
    <SettingsPanel v-if="showSettings" @close="showSettings = false" />
    <FolderCreator 
      v-if="showFolderCreator" 
      @created="handleFolderCreated" 
      @close="showFolderCreator = false" 
    />
    <FolderUploadDialog 
      v-if="showFolderDialog && pendingFolderPath" 
      :folder-path="pendingFolderPath" 
      @confirm="handleFolderUpload" 
      @cancel="showFolderDialog = false; pendingFolderPath = null" 
    />
    
    <!-- Upload Settings Dialog -->
    <UploadSettingsDialog
      v-if="showUploadSettings && pendingUploadFiles.length > 0"
      :files="pendingUploadFiles"
      :album-path="selectedAlbumPath || undefined"
      @confirm="handleUploadConfirm"
      @cancel="handleUploadCancel"
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

    <!-- Toast Notifications -->
    <ToastContainer />

    <!-- MacOS Dock Navigation -->
    <MacOSDock 
      :apps="dockApps"
      :open-apps="[activeView]"
      @app-click="handleDockClick"
    />

    <!-- Album Panel (slides in when albums view is active) -->
    <Transition name="slide-panel">
      <aside v-if="showAlbumPanel && (activeView === 'albums' || activeView === 'all-albums')" class="album-panel">
        <div class="panel-header">
          <h3>{{ activeView === 'all-albums' ? 'Todos Álbuns' : 'Álbuns' }}</h3>
          <button class="panel-close" @click="showAlbumPanel = false">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
          </button>
        </div>
        <AlbumTree 
          :albums="albums" 
          :selected-path="selectedAlbumPath" 
          :loading="loadingAlbums"
          @select="handleAlbumSelect" 
          @refresh="loadAlbums"
        />
        <SmartAlbumList 
          :albums="smartAlbums"
          :selected-id="selectedSmartAlbumId"
          @select="handleSmartAlbumSelect"
        />
      </aside>
    </Transition>
    </div>
  </ErrorBoundary>
</template>

<style scoped>
/* === 8-BIT RETRO APP LAYOUT === */
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  max-height: 100vh;
  background: var(--retro-bg-dark, #0f0a1e);
  color: var(--retro-text-main, #fff);
  position: relative;
  font-family: 'VT323', monospace;
  overflow: hidden;
}

/* === MAIN CONTENT (Full Width) === */
.main.full-width {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background: transparent;
  overflow: hidden;
  position: relative;
}

/* === HEADER === */
.header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 24px;
  background: rgba(26, 16, 48, 0.95);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  min-height: 64px;
  flex-shrink: 0;
  position: sticky;
  top: 0;
  z-index: 50;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 18px;
  font-weight: 700;
  color: #fff;
  letter-spacing: -0.5px;
}

.logo-icon {
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #ff3b30, #ff2d95);
  padding: 7px;
  border-radius: 10px;
  color: #fff;
  box-shadow: 0 4px 12px rgba(255, 45, 149, 0.4);
}

.logo-icon svg {
  width: 100%;
  height: 100%;
}

.view-title {
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  padding: 6px 14px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  white-space: nowrap;
}

.photo-count {
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  font-weight: 500;
}

/* === SEARCH BAR === */
.search-bar {
  flex: 1;
  max-width: 400px;
  min-width: 200px;
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 8px 14px;
  transition: all 0.2s ease;
}

.search-bar:focus-within {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 45, 149, 0.5);
  box-shadow: 0 0 0 3px rgba(255, 45, 149, 0.15);
}

.search-bar svg {
  width: 18px;
  height: 18px;
  color: rgba(255, 255, 255, 0.4);
  flex-shrink: 0;
}

.search-bar input {
  flex: 1;
  background: transparent;
  border: none;
  color: #fff;
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 14px;
  padding: 0;
}

.search-bar input:focus { outline: none; }
.search-bar input::placeholder { color: rgba(255, 255, 255, 0.4); }

.search-clear {
  width: 20px;
  height: 20px;
  padding: 2px;
  background: transparent;
  border: none;
  border-radius: 4px;
  box-shadow: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  transition: all 0.15s ease;
}

.search-clear:hover { 
  color: #ff3b30;
  background: rgba(255, 59, 48, 0.1);
}

/* === HEADER ACTIONS === */
.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: auto;
}

.header-auth { margin-left: 8px; }

/* View Toggle - Modern Pills */
.view-toggle {
  display: flex;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 3px;
  gap: 2px;
}

.view-toggle button {
  width: 34px;
  height: 34px;
  padding: 7px;
  background: transparent;
  border: none;
  border-radius: 6px;
  box-shadow: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: all 0.15s ease;
}

.view-toggle button:hover { 
  color: rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.05);
}

.view-toggle button.active { 
  background: rgba(255, 45, 149, 0.2);
  color: #ff2d95;
}

.view-toggle button svg { width: 100%; height: 100%; }

/* Action Group - Primary Actions */
.action-group {
  display: flex;
  gap: 6px;
  padding-left: 8px;
  margin-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
}

.btn-action {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  cursor: pointer;
  transition: all 0.15s ease;
  box-shadow: none;
}

.btn-action svg { width: 16px; height: 16px; }

.btn-action:hover {
  background: rgba(255, 255, 255, 0.1);
  transform: translateY(-1px);
}

.btn-action.upload { 
  background: linear-gradient(135deg, #00ff87, #00cc6a);
  border-color: transparent;
  color: #000;
  font-weight: 600;
}

.btn-action.upload:hover {
  box-shadow: 0 4px 16px rgba(0, 255, 135, 0.4);
}

.btn-action.create { 
  background: linear-gradient(135deg, #00d4ff, #00a8cc);
  border-color: transparent;
  color: #000;
  font-weight: 600;
}

.btn-action.create:hover {
  box-shadow: 0 4px 16px rgba(0, 212, 255, 0.4);
}

.btn-action:disabled { 
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}

.btn-action-icon {
  width: 36px;
  height: 36px;
  padding: 8px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.15s ease;
  box-shadow: none;
}

.btn-action-icon svg { width: 100%; height: 100%; }

.btn-action-icon:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
  transform: translateY(-1px);
}

.action-badge {
  background: #ff3b30;
  color: #fff;
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 10px;
  margin-left: 4px;
}

/* Icon Buttons - Settings Group */
.btn-icon {
  width: 36px;
  height: 36px;
  padding: 8px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid transparent;
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition: all 0.15s ease;
  box-shadow: none;
  position: relative;
}

.btn-icon svg { width: 100%; height: 100%; }

.btn-icon:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
  transform: translateY(-1px);
}

.btn-icon:active {
  transform: translateY(0);
}

/* === SETTINGS PANEL === */
.settings-panel {
  background: rgba(37, 24, 66, 0.95);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  padding: 20px 24px;
}

.setting-item label {
  display: block;
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
}

.repo-input {
  display: flex;
  gap: 10px;
}

.repo-input input {
  flex: 1;
  padding: 10px 14px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #fff;
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 14px;
}

.repo-input input:focus {
  outline: none;
  border-color: rgba(255, 45, 149, 0.5);
  box-shadow: 0 0 0 3px rgba(255, 45, 149, 0.15);
}

.btn-save {
  padding: 10px 18px;
  background: linear-gradient(135deg, #00ff87, #00cc6a);
  border: none;
  border-radius: 8px;
  color: #000;
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-save:hover {
  box-shadow: 0 4px 16px rgba(0, 255, 135, 0.4);
  transform: translateY(-1px);
}

.btn-new {
  width: 42px;
  padding: 10px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-new:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

/* === CONTENT AREA === */
.content {
  flex: 1;
  padding: 0;
  padding-bottom: 100px; /* Space for dock */
  overflow-y: auto;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

/* === EMPTY STATES === */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 40px 20px;
  color: var(--retro-text-main, #fff);
}

.empty-state h2 {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  margin-bottom: 12px;
}

.empty-state p {
  font-size: 18px;
  color: var(--retro-text-muted, #9d8ec2);
  margin-bottom: 20px;
}

.empty-icon.retro-icon {
  width: 64px;
  height: 64px;
  color: var(--retro-accent-green, #00ff87);
  margin-bottom: 20px;
  filter: drop-shadow(3px 3px 0 #000);
}

.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  padding: 20px;
}

.login-state {
  background: var(--retro-bg-panel, #1a1030);
  border: 3px solid #000;
  box-shadow: 6px 6px 0 #000;
  padding: 40px;
  max-width: 400px;
}

.login-auth-btn {
  margin-top: 20px;
}

.web-login-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 20px;
  padding: 14px 24px;
  background: linear-gradient(135deg, #00d4ff, #0066ff);
  border: 3px solid #000;
  box-shadow: 4px 4px 0 #000;
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: #fff;
  cursor: pointer;
  transition: all 0.1s;
}

.web-login-btn:hover {
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000;
}

.web-login-btn svg {
  width: 18px;
  height: 18px;
}

.repo-setup {
  display: flex;
  gap: 8px;
  width: 100%;
  max-width: 300px;
  margin-bottom: 16px;
}

.repo-setup input {
  flex: 1;
}

.or-divider {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--retro-text-muted, #9d8ec2);
  margin: 16px 0;
}

/* === ALBUMS SECTION === */
.albums-section {
  flex: 1;
  overflow-y: auto;
  margin-top: 8px;
}

/* === DRAG OVERLAY === */
.drag-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.drag-content {
  text-align: center;
  padding: 40px;
  border: 4px dashed var(--retro-accent-green, #00ff87);
  background: var(--retro-bg-panel, #1a1030);
}

.drag-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 20px;
  color: var(--retro-accent-green, #00ff87);
  animation: bounce 0.5s ease infinite alternate;
}

.drag-icon svg {
  width: 100%;
  height: 100%;
}

@keyframes bounce {
  from { transform: translateY(0); }
  to { transform: translateY(-10px); }
}

.drag-content h3 {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  color: var(--retro-accent-green, #00ff87);
  margin-bottom: 8px;
}

.drag-content p {
  font-size: 16px;
  color: var(--retro-text-muted, #9d8ec2);
}

/* === ERROR TOAST === */
.error-toast {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--retro-accent-red, #ff3b30);
  border: 3px solid #000;
  box-shadow: 4px 4px 0 #000;
  color: #fff;
  z-index: 1000;
}

.error-toast svg {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.error-toast button {
  width: 24px;
  height: 24px;
  padding: 4px;
  background: transparent;
  border: none;
  box-shadow: none;
  color: #fff;
}

/* === TOAST TRANSITION === */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}

/* === ALBUM PANEL (Slide-in) === */
.album-panel {
  position: fixed;
  top: 64px;
  left: 0;
  bottom: 100px;
  width: 300px;
  background: rgba(26, 16, 48, 0.98);
  backdrop-filter: blur(16px);
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 8px 0 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: linear-gradient(135deg, rgba(255, 208, 0, 0.15), rgba(255, 149, 0, 0.15));
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.panel-header h3 {
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
}

.panel-close {
  width: 32px;
  height: 32px;
  padding: 6px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.15s ease;
}

.panel-close:hover {
  background: rgba(255, 59, 48, 0.2);
  border-color: rgba(255, 59, 48, 0.5);
  color: #ff3b30;
}

.panel-close svg {
  width: 100%;
  height: 100%;
}

/* Panel slide transition */
.slide-panel-enter-active,
.slide-panel-leave-active {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-panel-enter-from,
.slide-panel-leave-to {
  transform: translateX(-100%);
}

/* === RESPONSIVE === */
@media (max-width: 768px) {
  .header {
    padding: 10px;
    gap: 8px;
  }
  
  .header-left {
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .logo { display: none; }
  
  .view-title { font-size: 10px; }
  
  .search-bar {
    order: 10;
    width: 100%;
    max-width: none;
    min-width: auto;
  }
  
  .action-group { display: none; }
  .view-toggle { display: none; }
  
  .album-panel {
    width: 100%;
    top: 50px;
  }
}

@media (max-width: 480px) {
  .btn-action span { display: none; }
  .header-auth { display: none; }
}
</style>
