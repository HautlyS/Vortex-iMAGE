/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed } from 'vue'

export type TransferStatus = 'pending' | 'uploading' | 'downloading' | 'completed' | 'failed'
export type TransferType = 'upload' | 'download'

export interface FileTransfer {
  id: string
  fileName: string
  progress: number
  status: TransferStatus
  error?: string
  type: TransferType
}

const transfers = ref<FileTransfer[]>([])
const isExpanded = ref(false)
let autoDismissTimer: ReturnType<typeof setTimeout> | null = null

export function useUploadToast() {
  const isVisible = computed(() => transfers.value.length > 0)
  
  const activeTransfers = computed(() => 
    transfers.value.filter(t => t.status === 'uploading' || t.status === 'downloading' || t.status === 'pending')
  )
  
  const hasFailures = computed(() => 
    transfers.value.some(t => t.status === 'failed')
  )
  
  const allCompleted = computed(() => 
    transfers.value.length > 0 && 
    transfers.value.every(t => t.status === 'completed' || t.status === 'failed')
  )

  function addTransfer(id: string, fileName: string, type: TransferType): void {
    
    if (autoDismissTimer) {
      clearTimeout(autoDismissTimer)
      autoDismissTimer = null
    }
    
    const existing = transfers.value.find(t => t.id === id)
    if (existing) return
    
    transfers.value.push({
      id,
      fileName,
      progress: 0,
      status: type === 'upload' ? 'uploading' : 'downloading',
      type
    })
  }

  function updateProgress(id: string, progress: number): void {
    const transfer = transfers.value.find(t => t.id === id)
    if (transfer) {
      transfer.progress = Math.min(100, Math.max(0, progress))
    }
  }

  function setStatus(id: string, status: TransferStatus, error?: string): void {
    const transfer = transfers.value.find(t => t.id === id)
    if (transfer) {
      transfer.status = status
      if (error) transfer.error = error
      if (status === 'completed') transfer.progress = 100
    }

    checkAutoDismiss()
  }

  function removeTransfer(id: string): void {
    const idx = transfers.value.findIndex(t => t.id === id)
    if (idx !== -1) {
      transfers.value.splice(idx, 1)
    }
  }

  function clearCompleted(): void {
    transfers.value = transfers.value.filter(t => t.status !== 'completed')
  }

  function clearAll(): void {
    transfers.value = []
    isExpanded.value = false
    if (autoDismissTimer) {
      clearTimeout(autoDismissTimer)
      autoDismissTimer = null
    }
  }

  function getActiveCount(): number {
    return activeTransfers.value.length
  }

  function getSummaryText(): string {
    const uploading = transfers.value.filter(t => t.type === 'upload' && (t.status === 'uploading' || t.status === 'pending')).length
    const downloading = transfers.value.filter(t => t.type === 'download' && (t.status === 'downloading' || t.status === 'pending')).length
    
    const parts: string[] = []
    if (uploading > 0) parts.push(`${uploading} uploading`)
    if (downloading > 0) parts.push(`${downloading} downloading`)
    
    if (parts.length === 0) {
      const failed = transfers.value.filter(t => t.status === 'failed').length
      if (failed > 0) return `${failed} failed`
      return 'All complete'
    }
    
    return parts.join(', ')
  }

  function toggleExpanded(): void {
    isExpanded.value = !isExpanded.value
  }

  function checkAutoDismiss(): void {
    if (allCompleted.value && !hasFailures.value) {
      
      autoDismissTimer = setTimeout(() => {
        clearAll()
      }, 3000)
    }
  }

  function retryTransfer(id: string): void {
    const transfer = transfers.value.find(t => t.id === id)
    if (transfer && transfer.status === 'failed') {
      transfer.status = transfer.type === 'upload' ? 'uploading' : 'downloading'
      transfer.progress = 0
      transfer.error = undefined
    }
  }

  return {
    transfers,
    isExpanded,
    isVisible,
    activeTransfers,
    hasFailures,
    allCompleted,
    addTransfer,
    updateProgress,
    setStatus,
    removeTransfer,
    clearCompleted,
    clearAll,
    getActiveCount,
    getSummaryText,
    toggleExpanded,
    retryTransfer
  }
}