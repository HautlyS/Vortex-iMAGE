import { ref, computed } from 'vue'

export type DockView = 'photos' | 'favorites' | 'albums' | 'all-albums'

export function useDockApps() {
  const activeView = ref<DockView>('photos')
  
  const dockApps = computed(() => [
    {
      id: 'photos',
      name: 'Todas Fotos',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="photoGrad" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#00ff87"/><stop offset="100%" style="stop-color:#00cc6a"/></linearGradient></defs><rect x="2" y="4" width="20" height="16" rx="3" fill="url(#photoGrad)"/><circle cx="7.5" cy="9" r="2" fill="#fff" opacity="0.9"/><path d="M22 16l-5-5-6 6-3-3-6 6v1a3 3 0 003 3h14a3 3 0 003-3v-5z" fill="#000" opacity="0.15"/></svg>`
    },
    {
      id: 'favorites',
      name: 'Favoritos',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="heartGrad" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ff2d95"/><stop offset="100%" style="stop-color:#ff6b9d"/></linearGradient></defs><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" fill="url(#heartGrad)"/></svg>`
    },
    {
      id: 'albums',
      name: 'Álbuns',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="folderGrad" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ffd000"/><stop offset="100%" style="stop-color:#ff9500"/></linearGradient></defs><path d="M3 6a3 3 0 0 1 3-3h4l2 2h8a3 3 0 0 1 3 3v10a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6z" fill="url(#folderGrad)"/></svg>`
    },
    {
      id: 'all-albums',
      name: 'Todos Álbuns',
      icon: `<svg viewBox="0 0 24 24" fill="none"><defs><linearGradient id="stackGrad" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#00d4ff"/><stop offset="100%" style="stop-color:#0099cc"/></linearGradient></defs><rect x="4" y="2" width="16" height="12" rx="2" fill="#666" opacity="0.4"/><rect x="2" y="5" width="16" height="12" rx="2" fill="#888" opacity="0.6"/><rect x="4" y="8" width="16" height="12" rx="2" fill="url(#stackGrad)"/></svg>`
    }
  ])

  function setActiveView(view: DockView) {
    activeView.value = view
  }

  return {
    dockApps,
    activeView,
    setActiveView
  }
}
