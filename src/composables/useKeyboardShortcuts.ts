/**
 * Keyboard Shortcuts Composable
 * Centralized keyboard event handling for the app
 * Supports ESC to close overlays, and other global shortcuts
 */
import { ref, onMounted, onUnmounted } from 'vue'

export interface KeyboardShortcut {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  meta?: boolean
  action: () => void
  description?: string
  preventDefault?: boolean
}

// Stack of active overlays - last one gets ESC priority
const overlayStack = ref<string[]>([])

// Global shortcuts registry
const globalShortcuts = ref<Map<string, KeyboardShortcut>>(new Map())

/**
 * Register an overlay to receive ESC key events
 * Returns a function to unregister
 */
export function registerOverlay(id: string, onClose: () => void): () => void {
  overlayStack.value.push(id)
  
  const shortcutId = `overlay-${id}`
  globalShortcuts.value.set(shortcutId, {
    key: 'Escape',
    action: () => {
      // Only close if this overlay is on top of the stack
      if (overlayStack.value[overlayStack.value.length - 1] === id) {
        onClose()
      }
    },
    preventDefault: true
  })
  
  return () => {
    overlayStack.value = overlayStack.value.filter(o => o !== id)
    globalShortcuts.value.delete(shortcutId)
  }
}

/**
 * Check if click is outside an element (for click-outside-to-close)
 */
export function isClickOutside(event: MouseEvent, element: HTMLElement | null): boolean {
  if (!element) return false
  return !element.contains(event.target as Node)
}

/**
 * Create a click-outside handler
 */
export function useClickOutside(
  elementRef: { value: HTMLElement | null },
  callback: () => void,
  options: { enabled?: boolean } = {}
) {
  const { enabled = true } = options
  
  function handleClick(event: MouseEvent) {
    if (!enabled) return
    if (elementRef.value && isClickOutside(event, elementRef.value)) {
      callback()
    }
  }
  
  onMounted(() => {
    // Use setTimeout to avoid immediate trigger on the same click that opened the overlay
    setTimeout(() => {
      document.addEventListener('click', handleClick)
    }, 0)
  })
  
  onUnmounted(() => {
    document.removeEventListener('click', handleClick)
  })
  
  return { handleClick }
}

/**
 * Main keyboard shortcuts composable
 */
export function useKeyboardShortcuts() {
  function handleKeydown(event: KeyboardEvent) {
    // Check global shortcuts
    for (const shortcut of globalShortcuts.value.values()) {
      const keyMatch = event.key === shortcut.key || event.code === shortcut.key
      const ctrlMatch = !shortcut.ctrl || (event.ctrlKey || event.metaKey)
      const shiftMatch = !shortcut.shift || event.shiftKey
      const altMatch = !shortcut.alt || event.altKey
      
      if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
        if (shortcut.preventDefault) {
          event.preventDefault()
        }
        shortcut.action()
        return
      }
    }
  }
  
  function registerShortcut(id: string, shortcut: KeyboardShortcut) {
    globalShortcuts.value.set(id, shortcut)
    return () => globalShortcuts.value.delete(id)
  }
  
  function unregisterShortcut(id: string) {
    globalShortcuts.value.delete(id)
  }
  
  // Setup global listener
  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })
  
  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })
  
  return {
    registerShortcut,
    unregisterShortcut,
    registerOverlay,
    overlayStack
  }
}

/**
 * Hook for overlay components - handles ESC and click-outside
 */
export function useOverlayClose(
  overlayId: string,
  onClose: () => void,
  options: {
    closeOnEsc?: boolean
    closeOnClickOutside?: boolean
  } = {}
) {
  const { closeOnEsc = true, closeOnClickOutside = true } = options
  
  let unregisterOverlay: (() => void) | null = null
  
  onMounted(() => {
    if (closeOnEsc) {
      unregisterOverlay = registerOverlay(overlayId, onClose)
    }
  })
  
  onUnmounted(() => {
    if (unregisterOverlay) {
      unregisterOverlay()
    }
  })
  
  // Handler for overlay background click
  function handleOverlayClick(event: MouseEvent) {
    if (!closeOnClickOutside) return
    // Only close if clicking directly on the overlay background
    if (event.target === event.currentTarget) {
      onClose()
    }
  }
  
  return {
    handleOverlayClick
  }
}

export default useKeyboardShortcuts
