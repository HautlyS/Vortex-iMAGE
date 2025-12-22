import { ref, onMounted } from 'vue'

export type Platform = 'desktop' | 'android' | 'ios' | 'web'

const platform = ref<Platform>('desktop')
const isMobile = ref(false)
const isTouch = ref(false)

export function usePlatform() {
  onMounted(async () => {
    // Check user agent for platform detection
    const ua = navigator.userAgent.toLowerCase()
    
    if (/android/.test(ua)) {
      platform.value = 'android'
      isMobile.value = true
    } else if (/iphone|ipad|ipod/.test(ua)) {
      platform.value = 'ios'
      isMobile.value = true
    } else if (window.innerWidth <= 768) {
      platform.value = 'web'
      isMobile.value = true
    } else {
      platform.value = 'desktop'
      isMobile.value = false
    }

    // Check for touch support
    isTouch.value = 'ontouchstart' in window || navigator.maxTouchPoints > 0

    // Listen for resize to update mobile state
    const handleResize = () => {
      isMobile.value = window.innerWidth <= 768 || /android|iphone|ipad|ipod/.test(ua)
    }
    window.addEventListener('resize', handleResize)
  })

  return {
    platform,
    isMobile,
    isTouch
  }
}
