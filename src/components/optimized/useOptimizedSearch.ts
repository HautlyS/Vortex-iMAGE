/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, watch, onUnmounted, type Ref } from 'vue'

export function useOptimizedSearch<T>(
  items: Ref<T[]>,
  searchFields: (keyof T)[],
  options: {
    debounceMs?: number
    minLength?: number
    caseSensitive?: boolean
  } = {}
) {
  const {
    debounceMs = 300,
    minLength = 2,
    caseSensitive = false
  } = options

  const searchQuery = ref('')
  const isSearching = ref(false)
  const searchResults = ref<T[]>([])

  let debounceTimer: number | null = null
  let searchWorker: Worker | null = null

  const createSearchWorker = () => {
    const workerCode = `
      self.onmessage = function(e) {
        const { items, query, fields, caseSensitive } = e.data;
        
        if (!query || query.length < 2) {
          self.postMessage({ results: items, isComplete: true });
          return;
        }
        
        const searchTerm = caseSensitive ? query : query.toLowerCase();
        const results = [];
        
        for (let i = 0; i < items.length; i++) {
          const item = items[i];
          let matches = false;
          
          for (const field of fields) {
            const value = item[field];
            if (value) {
              const fieldValue = caseSensitive ? String(value) : String(value).toLowerCase();
              if (fieldValue.includes(searchTerm)) {
                matches = true;
                break;
              }
            }
          }
          
          if (matches) {
            results.push(item);
          }

          if (i % 1000 === 0) {
            self.postMessage({ results: results.slice(), isComplete: false });
          }
        }
        
        self.postMessage({ results, isComplete: true });
      };
    `

    const blob = new Blob([workerCode], { type: 'application/javascript' })
    return new Worker(URL.createObjectURL(blob))
  }

  const performSearch = (query: string) => {
    if (!query || query.length < minLength) {
      searchResults.value = items.value
      isSearching.value = false
      return
    }

    isSearching.value = true

    if (items.value.length < 1000) {
      const searchTerm = caseSensitive ? query : query.toLowerCase()

      searchResults.value = items.value.filter(item => {
        return searchFields.some(field => {
          const value = item[field]
          if (!value) return false

          const fieldValue = caseSensitive ? String(value) : String(value).toLowerCase()
          return fieldValue.includes(searchTerm)
        })
      })

      isSearching.value = false
      return
    }

    if (!searchWorker) {
      searchWorker = createSearchWorker()

      searchWorker.onmessage = (e) => {
        const { results, isComplete } = e.data
        searchResults.value = results

        if (isComplete) {
          isSearching.value = false
        }
      }
    }

    searchWorker.postMessage({
      items: items.value,
      query,
      fields: searchFields,
      caseSensitive
    })
  }

  const debouncedSearch = (query: string) => {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
    }

    debounceTimer = window.setTimeout(() => {
      performSearch(query)
    }, debounceMs)
  }

  watch(searchQuery, (newQuery) => {
    debouncedSearch(newQuery)
  })

  watch(items, () => {
    if (searchQuery.value) {
      debouncedSearch(searchQuery.value)
    } else {
      searchResults.value = items.value
    }
  }, { deep: true })

  searchResults.value = items.value

  onUnmounted(() => {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
    }

    if (searchWorker) {
      searchWorker.terminate()
      searchWorker = null
    }
  })

  const clearSearch = () => {
    searchQuery.value = ''
    searchResults.value = items.value
    isSearching.value = false
  }

  const highlightMatches = (text: string, query: string) => {
    if (!query || query.length < minLength) return text

    const searchTerm = caseSensitive ? query : query.toLowerCase()
    const textToSearch = caseSensitive ? text : text.toLowerCase()

    const index = textToSearch.indexOf(searchTerm)
    if (index === -1) return text

    return text.substring(0, index) +
      `<mark>${text.substring(index, index + query.length)}</mark>` +
      text.substring(index + query.length)
  }

  return {
    searchQuery,
    searchResults,
    isSearching,
    clearSearch,
    highlightMatches
  }
}