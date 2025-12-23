/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import MacOSDock from './components/MacOSDock.vue'
import { useDockApps } from './composables/useDockApps'

const { dockApps, activeApps, toggleApp, setActiveApp } = useDockApps()

function handleDockAppClick(appId: string) {
  switch (appId) {
    case 'photos':
      navigateTo('photos')
      setActiveApp('photos')
      break
    case 'favorites':
      navigateTo('favorites')
      setActiveApp('favorites')
      break
    case 'albums':
      navigateTo('albums')
      setActiveApp('albums')
      break
    case 'upload':
      handleUploadClick()
      break
    case 'import':
      showLocalBrowser.value = true
      break
    case 'search':
      
      const searchInput = document.querySelector('.search-bar input') as HTMLInputElement
      if (searchInput) searchInput.focus()
      toggleApp('search')
      break
    case 'theme':
      showThemeSettings.value = true
      break
    case 'settings':
      showSettings.value = !showSettings.value
      toggleApp('settings')
      break
    case 'backup':
      showBackupSettings.value = true
      break
    case 'security':
      showSecuritySettings.value = true
      break
  }
}

const dockAppsWithBadges = computed(() => {
  return dockApps.value.map(app => ({
    ...app,
    badge: app.id === 'upload' && uploadProgress.value > 0 ? uploadProgress.value : undefined
  }))
})