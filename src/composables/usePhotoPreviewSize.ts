/**
 * TypeScript Module - 2 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref } from 'vue'

const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI__

const MIN_SIZE = 80
const MAX_SIZE = 400
const DEFAULT_SIZE = 180

const size = ref(DEFAULT_SIZE)
let initialized = false

export function clampSize(value: number): number {
  if (!Number.isFinite(value)) return DEFAULT_SIZE
  return Math.max(MIN_SIZE, Math.min(MAX_SIZE, Math.round(value)))
}

export function usePhotoPreviewSize() {
  async function loadSize(): Promise<void> {
    if (initialized) return
    try {
      if (isTauri) {
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('settings.json')
        const savedSize = await store.get<number>('previewSize')
        if (savedSize !== null && savedSize !== undefined) {
          size.value = clampSize(savedSize)
        }
      } else {
        const savedSize = localStorage.getItem('previewSize')
        if (savedSize) {
          size.value = clampSize(Number(savedSize))
        }
      }
      initialized = true
    } catch {
      
    }
  }

  async function saveSize(): Promise<void> {
    try {
      if (isTauri) {
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('settings.json')
        await store.set('previewSize', size.value)
        await store.save()
      } else {
        localStorage.setItem('previewSize', String(size.value))
      }
    } catch {
      
    }
  }

  function setSize(newSize: number): void {
    size.value = clampSize(newSize)
    saveSize()
  }

  return {
    size,
    MIN_SIZE,
    MAX_SIZE,
    setSize,
    loadSize,
    saveSize,
    clampSize,
  }
}