/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 2 modules
 */

import { ref } from 'vue'
import { useGitHubAuth, isDevMode } from './useGitHubAuth'
import { useUploadToast } from './useUploadToast'

export type SyncStatus = 'local-only' | 'remote-only' | 'synced'

export interface PhotoSyncState {
  photoId: string
  status: SyncStatus
  localPath?: string
  remotePath?: string
}

export interface SyncAction {
  id: string
  label: string
  icon: string
  action: () => Promise<void>
}

const syncStates = ref<Map<string, PhotoSyncState>>(new Map())

export function useSyncStatus() {
  const { token, repo } = useGitHubAuth()
  const { addTransfer, setStatus } = useUploadToast()

  function getStatus(photoId: string): SyncStatus {
    return syncStates.value.get(photoId)?.status ?? 'synced'
  }

  function setStatusState(photoId: string, status: SyncStatus, localPath?: string, remotePath?: string): void {
    syncStates.value.set(photoId, { photoId, status, localPath, remotePath })
  }

  function getStatusTooltip(status: SyncStatus): string {
    switch (status) {
      case 'local-only': return 'Local only - Not uploaded to GitHub'
      case 'remote-only': return 'Remote only - Not downloaded locally'
      case 'synced': return 'Synced - Available locally and on GitHub'
      default: return 'Unknown status'
    }
  }

  function getAvailableActions(status: SyncStatus, photoId: string): SyncAction[] {
    const actions: SyncAction[] = []
    
    if (status === 'local-only') {
      actions.push({ id: 'upload', label: 'Upload to GitHub', icon: 'upload', action: () => uploadPhoto(photoId) })
      actions.push({ id: 'remove-local', label: 'Remove local copy', icon: 'trash', action: () => removeLocalCopy(photoId) })
    } else if (status === 'remote-only') {
      actions.push({ id: 'download', label: 'Download from GitHub', icon: 'download', action: () => downloadPhoto(photoId) })
      actions.push({ id: 'delete-remote', label: 'Delete from GitHub', icon: 'trash', action: () => deleteFromRemote(photoId) })
    } else if (status === 'synced') {
      actions.push({ id: 'remove-local', label: 'Remove local copy', icon: 'trash-local', action: () => removeLocalCopy(photoId) })
      actions.push({ id: 'delete-remote', label: 'Delete from GitHub', icon: 'trash-remote', action: () => deleteFromRemote(photoId) })
    }
    
    return actions
  }

  async function uploadPhoto(photoId: string): Promise<void> {
    if (isDevMode) return
    const state = syncStates.value.get(photoId)
    if (!state?.localPath || !token.value || !repo.value) return
    
    const transferId = `upload-${photoId}-${Date.now()}`
    const fileName = state.localPath.split('/').pop() || 'photo'
    
    addTransfer(transferId, fileName, 'upload')
    
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('upload_photo', {
        path: state.localPath,
        repo: repo.value,
        token: token.value,
        filename: fileName,
        uploadId: transferId
      })
      
      setStatus(transferId, 'completed')
      setStatusState(photoId, 'synced', state.localPath, state.remotePath)
    } catch (error) {
      setStatus(transferId, 'failed', String(error))
    }
  }

  async function downloadPhoto(photoId: string): Promise<void> {
    if (isDevMode) return
    const state = syncStates.value.get(photoId)
    if (!state?.remotePath || !token.value || !repo.value) return
    
    const transferId = `download-${photoId}-${Date.now()}`
    const fileName = state.remotePath.split('/').pop() || 'photo'
    
    addTransfer(transferId, fileName, 'download')
    
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const localPath = await invoke<string>('download_photo', {
        remotePath: state.remotePath,
        repo: repo.value,
        token: token.value,
        downloadId: transferId,
        localDir: null
      })
      
      setStatus(transferId, 'completed')
      setStatusState(photoId, 'synced', localPath, state.remotePath)
    } catch (error) {
      setStatus(transferId, 'failed', String(error))
    }
  }

  async function removeLocalCopy(photoId: string): Promise<void> {
    if (isDevMode) return
    const state = syncStates.value.get(photoId)
    if (!state?.localPath) return
    
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('remove_local_file', { path: state.localPath })
      
      if (state.remotePath) {
        setStatusState(photoId, 'remote-only', undefined, state.remotePath)
      } else {
        syncStates.value.delete(photoId)
      }
    } catch {}
  }

  async function deleteFromRemote(photoId: string): Promise<void> {
    if (isDevMode) return
    const state = syncStates.value.get(photoId)
    if (!state?.remotePath || !token.value || !repo.value) return
    
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('delete_photo', {
        path: state.remotePath,
        repo: repo.value,
        token: token.value
      })
      
      if (state.localPath) {
        setStatusState(photoId, 'local-only', state.localPath, undefined)
      } else {
        syncStates.value.delete(photoId)
      }
    } catch {}
  }

  function getStatusIcon(status: SyncStatus): string {
    return status === 'local-only' ? 'device' : status === 'remote-only' ? 'cloud' : 'cloud-check'
  }

  function getStatusColor(status: SyncStatus): string {
    return status === 'local-only' ? '#eab308' : status === 'remote-only' ? '#3b82f6' : '#22c55e'
  }

  return {
    syncStates,
    getStatus,
    setStatus: setStatusState,
    getStatusTooltip,
    getAvailableActions,
    uploadPhoto,
    downloadPhoto,
    removeLocalCopy,
    deleteFromRemote,
    getStatusIcon,
    getStatusColor
  }
}