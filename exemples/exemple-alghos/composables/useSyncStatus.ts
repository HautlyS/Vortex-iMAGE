import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGitHubAuth } from './useGitHubAuth'
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
      case 'local-only':
        return 'Local only - This photo exists only on your device and has not been uploaded to GitHub'
      case 'remote-only':
        return 'Remote only - This photo is stored on GitHub but not downloaded to your device'
      case 'synced':
        return 'Synced - This photo is available both locally and on GitHub'
      default:
        return 'Unknown status'
    }
  }

  function getAvailableActions(status: SyncStatus, photoId: string): SyncAction[] {
    const actions: SyncAction[] = []
    
    switch (status) {
      case 'local-only':
        actions.push({
          id: 'upload',
          label: 'Upload to GitHub',
          icon: 'upload',
          action: () => uploadPhoto(photoId)
        })
        actions.push({
          id: 'remove-local',
          label: 'Remove local copy',
          icon: 'trash',
          action: () => removeLocalCopy(photoId)
        })
        break
      case 'remote-only':
        actions.push({
          id: 'download',
          label: 'Download from GitHub',
          icon: 'download',
          action: () => downloadPhoto(photoId)
        })
        actions.push({
          id: 'delete-remote',
          label: 'Delete from GitHub',
          icon: 'trash',
          action: () => deleteFromRemote(photoId)
        })
        break
      case 'synced':
        actions.push({
          id: 'remove-local',
          label: 'Remove local copy',
          icon: 'trash-local',
          action: () => removeLocalCopy(photoId)
        })
        actions.push({
          id: 'delete-remote',
          label: 'Delete from GitHub',
          icon: 'trash-remote',
          action: () => deleteFromRemote(photoId)
        })
        break
    }
    
    return actions
  }

  async function uploadPhoto(photoId: string): Promise<void> {
    const state = syncStates.value.get(photoId)
    if (!state?.localPath || !token.value || !repo.value) return
    
    const transferId = `upload-${photoId}-${Date.now()}`
    const fileName = state.localPath.split('/').pop() || 'photo'
    
    addTransfer(transferId, fileName, 'upload')
    
    try {
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
    const state = syncStates.value.get(photoId)
    if (!state?.remotePath || !token.value || !repo.value) return
    
    const transferId = `download-${photoId}-${Date.now()}`
    const fileName = state.remotePath.split('/').pop() || 'photo'
    
    addTransfer(transferId, fileName, 'download')
    
    try {
      const localPath = await invoke<string>('download_photo', {
        remotePath: state.remotePath,
        repo: repo.value,
        token: token.value,
        downloadId: transferId,
        localDir: null // Use default downloads folder
      })
      
      setStatus(transferId, 'completed')
      setStatusState(photoId, 'synced', localPath, state.remotePath)
    } catch (error) {
      setStatus(transferId, 'failed', String(error))
    }
  }

  async function removeLocalCopy(photoId: string): Promise<void> {
    const state = syncStates.value.get(photoId)
    if (!state?.localPath) return
    
    try {
      await invoke('remove_local_file', { path: state.localPath })
      
      if (state.remotePath) {
        setStatusState(photoId, 'remote-only', undefined, state.remotePath)
      } else {
        syncStates.value.delete(photoId)
      }
    } catch (error) {
      console.error('Failed to remove local copy:', error)
    }
  }

  async function deleteFromRemote(photoId: string): Promise<void> {
    const state = syncStates.value.get(photoId)
    if (!state?.remotePath || !token.value || !repo.value) return
    
    try {
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
    } catch (error) {
      console.error('Failed to delete from remote:', error)
    }
  }

  function getStatusIcon(status: SyncStatus): string {
    switch (status) {
      case 'local-only':
        return 'device'
      case 'remote-only':
        return 'cloud'
      case 'synced':
        return 'cloud-check'
      default:
        return 'question'
    }
  }

  function getStatusColor(status: SyncStatus): string {
    switch (status) {
      case 'local-only':
        return '#eab308' // yellow
      case 'remote-only':
        return '#3b82f6' // blue
      case 'synced':
        return '#22c55e' // green
      default:
        return '#71717a' // gray
    }
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
