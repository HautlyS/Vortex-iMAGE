import { ref, onErrorCaptured, onUnmounted } from 'vue'

export interface ErrorInfo {
  message: string
  stack?: string
  timestamp: number
  component?: string
}

export function useErrorBoundary() {
  const errors = ref<ErrorInfo[]>([])
  const hasError = ref(false)

  function captureError(error: Error, component?: string) {
    const errorInfo: ErrorInfo = {
      message: error.message,
      stack: error.stack,
      timestamp: Date.now(),
      component
    }
    
    errors.value.push(errorInfo)
    hasError.value = true
    
    // Log to console for debugging
    console.error('Error captured:', errorInfo)
  }

  function clearErrors() {
    errors.value = []
    hasError.value = false
  }

  function clearError(index: number) {
    errors.value.splice(index, 1)
    hasError.value = errors.value.length > 0
  }

  // Capture Vue component errors
  onErrorCaptured((error, instance) => {
    const componentName = instance?.$options.name || instance?.$options.__name || 'Unknown'
    captureError(error, componentName)
    return false // Prevent error from propagating
  })

  // Global error handlers with cleanup
  const handleError = (event: ErrorEvent) => {
    captureError(new Error(event.message), 'Global')
  }

  const handleRejection = (event: PromiseRejectionEvent) => {
    captureError(new Error(event.reason), 'Promise')
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('error', handleError)
    window.addEventListener('unhandledrejection', handleRejection)
  }

  // Cleanup on unmount
  onUnmounted(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('error', handleError)
      window.removeEventListener('unhandledrejection', handleRejection)
    }
  })

  return {
    errors,
    hasError,
    captureError,
    clearErrors,
    clearError
  }
}
