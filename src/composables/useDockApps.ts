import { ref, computed } from 'vue'

export function useDockApps() {
  const activeApps = ref<string[]>(['photos'])
  
  const dockApps = computed(() => [
    {
      id: 'photos',
      name: 'Fotos',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="photoFlat" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ff4757"/><stop offset="100%" style="stop-color:#c44569"/></linearGradient></defs><rect x="2" y="4" width="20" height="16" rx="5" fill="url(#photoFlat)"/><circle cx="7.5" cy="8.5" r="1.8" fill="#fff" opacity="0.95"/><path d="M20 16l-3.5-3.5-4.5 4.5-2.5-2.5-7 7h17.5z" fill="#000" opacity="0.08"/></svg>`
    },
    {
      id: 'favorites',
      name: 'Favoritas',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="starFlat" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ff3838"/><stop offset="100%" style="stop-color:#ff6b9d"/></linearGradient></defs><path d="M12 2l3.2 6.5h7.3l-5.9 4.3 2.3 7.1L12 16.4l-6.9 3.5 2.3-7.1L1.5 8.5h7.3L12 2z" fill="url(#starFlat)"/></svg>`
    },
    {
      id: 'albums',
      name: 'Álbuns',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="folderFlat" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#3742fa"/><stop offset="100%" style="stop-color:#2f3542"/></linearGradient></defs><path d="M3 6a3 3 0 0 1 3-3h4.5l2.5 2.5h6a3 3 0 0 1 3 3v9a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6z" fill="url(#folderFlat)"/></svg>`
    },
    {
      id: 'trash',
      name: 'Lixeira',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="trashFlat" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ff6348"/><stop offset="100%" style="stop-color:#e55039"/></linearGradient></defs><path d="M8.5 6V4.5a2.5 2.5 0 0 1 2.5-2.5h2a2.5 2.5 0 0 1 2.5 2.5V6" stroke="#555" stroke-width="2" fill="none"/><path d="M6 8h12l-1.2 12a2.5 2.5 0 0 1-2.5 2.5h-4.6a2.5 2.5 0 0 1-2.5-2.5L6 8z" fill="url(#trashFlat)"/><line x1="10.5" y1="11.5" x2="10.5" y2="16.5" stroke="#fff" stroke-width="2.2" opacity="0.9"/><line x1="13.5" y1="11.5" x2="13.5" y2="16.5" stroke="#fff" stroke-width="2.2" opacity="0.9"/></svg>`
    },
    {
      id: 'settings',
      name: 'Config',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="settingsFlat" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ffa502"/><stop offset="100%" style="stop-color:#ff6348"/></linearGradient></defs><circle cx="12" cy="12" r="3.5" fill="url(#settingsFlat)"/><path d="M12 1.5l1.8 4.2L18.5 4l-1.8 4.2L21 12l-4.3 1.8L18.5 20l-4.2-1.8L12 22.5l-1.8-4.2L5.5 20l1.8-4.2L3 12l4.3-1.8L5.5 4l4.2 1.8L12 1.5z" stroke="url(#settingsFlat)" stroke-width="1.8" fill="none"/></svg>`
    }
  ])

  function toggleApp(appId: string) {
    if (activeApps.value.includes(appId)) {
      activeApps.value = activeApps.value.filter(id => id !== appId)
    } else {
      activeApps.value = [...activeApps.value, appId]
    }
  }

  function setActiveApp(appId: string) {
    if (!activeApps.value.includes(appId)) {
      activeApps.value = [...activeApps.value, appId]
    }
  }

  return {
    dockApps,
    activeApps,
    toggleApp,
    setActiveApp
  }
}
