/**
 * TypeScript Module - 2 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, onMounted, onUnmounted } from 'vue'

export function useMobileSearch() {
  const mobileSearchOpen = ref(false)
  const searchPullDistance = ref(0)
  const searchOpacity = ref(1)
  const pullDistance = ref(0)
  const isPulling = ref(false)
  
  let searchTouchStartY = 0
  let accumulatedScroll = 0
  let touchStartY = 0

  function toggleMobileSearch() {
    mobileSearchOpen.value = !mobileSearchOpen.value
  }

  function handleSearchTouchStart(e: TouchEvent) {
    searchTouchStartY = e.touches[0].clientY
    accumulatedScroll = 0
  }

  function handleSearchTouchMove(e: TouchEvent, hasResults: boolean) {
    if (hasResults) return
    
    const currentY = e.touches[0].clientY
    const deltaY = currentY - searchTouchStartY
    
    if (deltaY > 0) {
      accumulatedScroll = deltaY
      searchOpacity.value = Math.max(0, 1 - (deltaY / 200))
      searchPullDistance.value = deltaY * 0.3
    }
  }

  function handleSearchTouchEnd(searchQuery: { value: string }) {
    if (searchOpacity.value < 0.4 || accumulatedScroll > 150) {
      mobileSearchOpen.value = false
      searchQuery.value = ''
    }
    searchOpacity.value = 1
    searchPullDistance.value = 0
    accumulatedScroll = 0
  }

  function handleSearchScroll(e: Event, hasResults: boolean, searchQuery: { value: string }) {
    const target = e.target as HTMLElement
    const maxScroll = target.scrollHeight - target.clientHeight
    const currentScroll = target.scrollTop
    
    if (hasResults) {
      if (currentScroll > maxScroll + 80) {
        mobileSearchOpen.value = false
        searchQuery.value = ''
      }
    } else {
      if (currentScroll > 50) {
        mobileSearchOpen.value = false
        searchQuery.value = ''
      }
    }
  }

  function handleTouchStart(e: TouchEvent) {
    touchStartY = e.touches[0].clientY
    isPulling.value = false
  }

  function handleTouchMove(e: TouchEvent) {
    if (mobileSearchOpen.value) return
    const scrollTop = document.querySelector('.main-dock')?.scrollTop || 0
    if (scrollTop > 0) {
      isPulling.value = false
      pullDistance.value = 0
      return
    }
    const deltaY = e.touches[0].clientY - touchStartY
    if (deltaY > 0 && scrollTop <= 0) {
      isPulling.value = true
      pullDistance.value = Math.min(deltaY * 0.5, 80)
    }
  }

  function handleTouchEnd() {
    if (pullDistance.value > 50) {
      mobileSearchOpen.value = true
    }
    pullDistance.value = 0
    isPulling.value = false
  }

  return {
    mobileSearchOpen,
    searchPullDistance,
    searchOpacity,
    pullDistance,
    isPulling,
    toggleMobileSearch,
    handleSearchTouchStart,
    handleSearchTouchMove,
    handleSearchTouchEnd,
    handleSearchScroll,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd
  }
}

export function useMobileDetection() {
  const isMobile = ref(false)
  let resizeHandler: (() => void) | null = null

  onMounted(() => {
    isMobile.value = window.innerWidth < 768
    resizeHandler = () => {
      isMobile.value = window.innerWidth < 768
    }
    window.addEventListener('resize', resizeHandler)
  })

  onUnmounted(() => {
    if (resizeHandler) {
      window.removeEventListener('resize', resizeHandler)
    }
  })

  return { isMobile }
}