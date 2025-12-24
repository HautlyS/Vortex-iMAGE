<script setup lang="ts">
import { ref, onMounted, watch, computed, onUnmounted } from 'vue'
import { useGitHubAuth } from './composables/useGitHubAuth'
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
import { useTimeout } from './composables/useTimeout'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { 
  UPLOAD, SHORTCUTS, TIMING,
  injectCSSVariables 
} from './config'
import SpaceLoader from './components/SpaceLoader.vue'
import AuthButton from './components/AuthButton.vue'
import { Vortex } from './components/ui/vortex'
import { NeuralBg } from './components/ui/bg-neural'
import PhotoGallery from './components/PhotoGallery.vue'
import PrivacySettings from './components/PrivacySettings.vue'
import FolderUploadDialog from './components/FolderUploadDialog.vue'
import RepoCreator from './components/RepoCreator.vue'
import AlbumTree from './components/AlbumTree.vue'
import ColorTagPanel from './components/ColorTagPanel.vue'
import ContextMenu, { type ContextMenuItem } from './components/ContextMenu.vue'
import UploadToast from './components/UploadToast.vue'
import ThemeSettings from './components/ThemeSettings.vue'
import DataDriverManager from './components/DataDriverManager.vue'
import BackupSettings from './components/BackupSettings.vue'
import LocalImageBrowser from './components/LocalImageBrowser.vue'
import SecuritySettings from './components/SecuritySettings.vue'
import MobileNav from './components/MobileNav.vue'
import ErrorBoundary from './components/ErrorBoundary.vue'
import AsyncState from './components/AsyncState.vue'

interface Album {
  name: string
  path: string
  photo_count: number
  children: Album[]
}

const { token, repo, init, setRepo } = useGitHubAuth()
const { photos, loadingPhotos, loadPhotos, addToQueue, queue } = usePhotoUpload()
const { addTransfer, updateProgress, setStatus: setTransferStatus } = useUploadToast()
const { init: initAccent } = useAccentColor()
const { loadTheme } = useTheme()
const { selected, clearSelection, selectAll, getSelected } = useSelection()
const { loadFavorites, isFavorite, toggleFavorite } = useFavorites()
const { loadTags, tagItems, getItemsByTag } = useColorTags()
const { size: previewSize, setSize: setPreviewSize, loadSize: loadPreviewSize } = usePhotoPreviewSize()
const { activeDriver, loadDrivers, setActiveDriver: _setActiveDriver } = useDataDriver()
const { loadConfig: loadBackupConfig } = useBackupSettings()
const { createTimeout } = useTimeout()
void _setActiveDriver // suppress unused warning

// App state
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

// Navigation state
type NavView = 'photos' | 'favorites' | 'albums' | 'tags' | 'trash'
const currentView = ref<NavView>('photos')
const selectedAlbumPath = ref<string | null>(null)
const selectedTagId = ref<string | null>(null)

// Folder upload dialog
const showFolderDialog = ref(false)
const pendingFolderPath = ref<string | null>(null)

// Context menu
const contextMenu = ref<{ x: number; y: number; items: ContextMenuItem[] } | null>(null)

// Albums
const albums = ref<Album[]>([])
const loadingAlbums = ref(false)

// Filtered photos based on current view
const filteredPhotos = computed(() => {
  let result = photos.value as Photo[]
  
  // Filter by search (debounced)
  if (debouncedSearchQuery.value) {
    const q = debouncedSearchQuery.value.toLowerCase()
    result = result.filter(p => p.name.toLowerCase().includes(q))
  }
  
  // Filter by view
  if (currentView.value === 'favorites') {
    result = result.filter(p => isFavorite(p.sha))
  } else if (currentView.value === 'tags' && selectedTagId.value) {
    const taggedIds = getItemsByTag(selectedTagId.value)
    result = result.filter(p => taggedIds.includes(p.sha))
  } else if (currentView.value === 'albums' && selectedAlbumPath.value) {
    result = result.filter(p => p.path?.startsWith(selectedAlbumPath.value!))
  }
  
  return result
})

const uploadProgress = computed(() => {
  return queue.value.filter(u => u.status === 'uploading' || u.status === 'pending').length
})

const viewTitle = computed(() => {
  switch (currentView.value) {
    case 'favorites': return 'Favoritos'
    case 'albums': return selectedAlbumPath.value ? selectedAlbumPath.value.split('/').pop() : 'Álbuns'
    case 'tags': return 'Etiquetas'
    default: return 'Fotos'
  }
})

// Load albums from GitHub
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
  // Inject CSS variables from config
  injectCSSVariables()
  
  // Add base effect classes to html element
  document.documentElement.classList.add('matrix-effects', 'glass-morphism', 'glow-effects')
  
  await Promise.all([init(), initAccent(), loadTheme(), loadFavorites(), loadTags(), loadPreviewSize(), loadDrivers(), loadBackupConfig()])
  repoInput.value = repo.value
  
  // Load view mode from storage
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    const savedViewMode = await store.get<'grid' | 'list'>('viewMode')
    if (savedViewMode) viewMode.value = savedViewMode
  } catch {}
})

watch(repo, (v) => { repoInput.value = v })
watch([token, repo], () => { 
  if (token.value && repo.value) {
    loadPhotos()
    loadAlbums()
  }
})

// Sync upload queue with toast notifications - optimized with computed tracking
const queueStatusMap = new Map<string, UploadStatus>()

// Use computed for better performance tracking
const queueItems = computed(() => 
  queue.value.map(i => ({ id: i.id, status: i.status, progress: i.progress, name: i.name, error: i.error }))
)

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

async function handleFolderClick() {
  uploadError.value = null
  try {
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

// Navigation handlers
function navigateTo(view: NavView) {
  currentView.value = view
  selectedAlbumPath.value = null
  selectedTagId.value = null
  clearSelection()
}

function handleAlbumSelect(path: string | null) {
  currentView.value = 'albums'
  selectedAlbumPath.value = path
  clearSelection()
}

function handleTagSelect(tagId: string | null) {
  currentView.value = 'tags'
  selectedTagId.value = tagId
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
    
    <div v-else class="app" @dragover.prevent="isDragging = true" @dragleave.prevent="isDragging = false" @drop.prevent="onDrop">
      <!-- Neural Background (logged in state) -->
      <NeuralBg v-if="token" class="app-neural-bg" :hue="999" :saturation="5" :chroma="1.1" />
      
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
    <aside class="sidebar">
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

      <!-- Data Sources Section -->
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

      <!-- Backup Section -->
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

      <!-- Albums Section -->
      <div v-if="currentView === 'albums'" class="sidebar-section">
        <div class="section-header">
          <span>Álbuns</span>
        </div>
        <AlbumTree :albums="albums" :selected-path="selectedAlbumPath" @select="handleAlbumSelect" />
      </div>

      <!-- Color Tags Section -->
      <div v-if="currentView === 'tags' || currentView === 'photos'" class="sidebar-section">
        <ColorTagPanel :selected-tag-id="selectedTagId" @select="handleTagSelect" />
      </div>

      <div class="sidebar-footer">
        <AuthButton />
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main">
      <!-- Header -->
      <header class="header">
        <div class="header-left">
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

            <!-- Upload Button -->
            <div class="upload-group">
              <button class="btn-upload" @click="handleUploadClick" :disabled="!token || !repo">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17,8 12,3 7,8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                <span>Upload</span>
                <span v-if="uploadProgress" class="upload-badge">{{ uploadProgress }}</span>
              </button>
              <button class="btn-folder" @click="handleFolderClick" :disabled="!token || !repo" title="Upload pasta">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
              </button>
            </div>

          <button class="btn-icon" @click="showThemeSettings = true" title="Tema">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
          </button>

          <button class="btn-icon" @click="showSettings = !showSettings" title="Configurações">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          </button>
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
            :empty="!loadingPhotos && filteredPhotos.length === 0"
            empty-message="Nenhuma foto encontrada"
            @retry="retryLoadPhotos"
          >
            <PhotoGallery 
              :photos="filteredPhotos" 
              :loading="false" 
              :view-mode="viewMode"
              :preview-size="previewSize"
              @refresh="loadPhotos"
              @contextmenu="handlePhotoContextMenu"
              @resize="handlePreviewResize"
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

    <!-- Mobile Bottom Navigation -->
    <MobileNav 
      :current-view="currentView"
      @navigate="navigateTo"
      @settings="showSettings = !showSettings"
    />
    </div>
  </ErrorBoundary>
</template>

<style scoped>
.app {
  display: flex;
  min-height: 100vh;
  background: var(--void);
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

/* Drag Overlay */
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

/* Sidebar */
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

/* Main */
.main { flex: 1; display: flex; flex-direction: column; min-width: 0; }

/* Header */
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

.header-actions { display: flex; align-items: center; gap: 0.5rem; margin-left: auto; }

/* View Toggle */
.view-toggle {
  display: flex;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.6);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: 0.25rem;
}
.view-toggle button {
  width: 2.25rem;
  height: 2.25rem;
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
.view-toggle button:hover { color: var(--text-primary); }
.view-toggle button.active { 
  background: linear-gradient(135deg, var(--accent-color), var(--accent-secondary)); 
  color: #000;
  box-shadow: 0 2px 12px rgba(var(--accent-rgb), 0.4);
}
.view-toggle button svg { width: 1.125rem; height: 1.125rem; }

/* Apple Store style buttons */
.upload-group { display: flex; gap: 0.5rem; }
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

/* Gray variant button (folder) */
.btn-folder {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 10px 14px;
  background: rgba(var(--accent-rgb), 0.15);
  border: none;
  border-radius: 1000px;
  color: var(--accent-color);
  cursor: pointer;
  transition: background-color 0.14s ease-out;
}
.btn-folder:hover { 
  background: rgba(var(--accent-rgb), 0.25);
  transition: background-color 0.21s ease-out;
}
.btn-folder:active {
  background: rgba(var(--accent-rgb), 0.2);
}
.btn-folder:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-folder svg { width: 1.125rem; height: 1.125rem; }

.btn-icon {
  width: 2.75rem;
  height: 2.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.6);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
}
.btn-icon:hover { 
  background: rgba(var(--surface-2-rgb, 22, 22, 32), 0.8); 
  color: var(--text-primary);
  border-color: var(--border-strong);
}
.btn-icon svg { width: 1.375rem; height: 1.375rem; }

/* Settings Panel */
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

/* Content */
.content { flex: 1; overflow-y: auto; padding: 1.75rem; background: var(--void); }

/* Login Vortex */
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

/* Empty State */
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

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity var(--duration-normal) var(--ease-out); }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.slide-enter-active, .slide-leave-active { transition: all var(--duration-normal) var(--ease-out); }
.slide-enter-from, .slide-leave-to { opacity: 0; transform: translateY(-10px); }
.toast-enter-active { transition: all 0.3s var(--ease-spring); }
.toast-leave-active { transition: all var(--duration-fast); }
.toast-enter-from { opacity: 0; transform: translate(-50%, 20px); }
.toast-leave-to { opacity: 0; transform: translate(-50%, 10px); }

/* Error Toast */
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
</style>
