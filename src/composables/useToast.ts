/**
 * Toast Notification System
 * Modern toast notifications with retro styling
 */

import { ref, readonly } from 'vue'

export interface Toast {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  message?: string
  duration?: number
  action?: { label: string; onClick: () => void }
}

const toasts = ref<Toast[]>([])
let toastId = 0

const generateId = () => `toast-${++toastId}-${Date.now()}`

export function useToast() {
  const add = (toast: Omit<Toast, 'id'>): string => {
    const id = generateId()
    toasts.value.push({ ...toast, id })
    return id
  }

  const remove = (id: string) => {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index > -1) toasts.value.splice(index, 1)
  }

  const success = (title: string, message?: string, duration = 4000) => 
    add({ type: 'success', title, message, duration })

  const error = (title: string, message?: string, duration = 6000) => 
    add({ type: 'error', title, message, duration })

  const warning = (title: string, message?: string, duration = 5000) => 
    add({ type: 'warning', title, message, duration })

  const info = (title: string, message?: string, duration = 4000) => 
    add({ type: 'info', title, message, duration })

  const clear = () => { toasts.value = [] }

  return {
    toasts: readonly(toasts),
    add,
    remove,
    success,
    error,
    warning,
    info,
    clear
  }
}
