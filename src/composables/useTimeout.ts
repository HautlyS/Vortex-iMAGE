import { ref, onUnmounted } from 'vue'

export function useTimeout() {
  const timeouts = ref(new Set<ReturnType<typeof setTimeout>>())
  const intervals = ref(new Set<ReturnType<typeof setInterval>>())

  function createTimeout(callback: () => void, delay: number): ReturnType<typeof setTimeout> {
    const id = setTimeout(() => {
      timeouts.value.delete(id)
      callback()
    }, delay)
    
    timeouts.value.add(id)
    return id
  }

  function createInterval(callback: () => void, delay: number): ReturnType<typeof setInterval> {
    const id = setInterval(callback, delay)
    intervals.value.add(id)
    return id
  }

  function clearTimeoutById(id: ReturnType<typeof setTimeout>) {
    clearTimeout(id)
    timeouts.value.delete(id)
  }

  function clearIntervalById(id: ReturnType<typeof setInterval>) {
    clearInterval(id)
    intervals.value.delete(id)
  }

  function clearAllTimeouts() {
    timeouts.value.forEach(id => clearTimeout(id))
    timeouts.value.clear()
  }

  function clearAllIntervals() {
    intervals.value.forEach(id => clearInterval(id))
    intervals.value.clear()
  }

  function clearAll() {
    clearAllTimeouts()
    clearAllIntervals()
  }

  // Auto-cleanup on unmount
  onUnmounted(clearAll)

  return {
    createTimeout,
    createInterval,
    clearTimeoutById,
    clearIntervalById,
    clearAllTimeouts,
    clearAllIntervals,
    clearAll
  }
}
