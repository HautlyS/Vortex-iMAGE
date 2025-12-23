<script setup lang="ts">
import { computed } from 'vue'
import { useTheme } from '../composables/useTheme'
import { TOUCH } from '../config'

defineProps<{
  currentView: 'photos' | 'favorites' | 'albums' | 'tags' | 'trash'
}>()

const emit = defineEmits<{
  navigate: [view: 'photos' | 'favorites' | 'albums' | 'tags' | 'trash']
  settings: []
}>()

const { accentColor } = useTheme()

// Touch feedback helper
function triggerHaptic() {
  if ('vibrate' in navigator) {
    navigator.vibrate(10)
  }
}

const navItems = computed(() => [
  {
    id: 'photos' as const,
    label: 'Fotos',
    icon: `<svg viewBox="0 0 24 24" fill="none">
      <rect x="2" y="4" width="20" height="16" rx="4" fill="#333" stroke="#555" stroke-width="1"/>
      <circle cx="8" cy="9" r="1.5" fill="#666"/>
      <path d="M18 15l-3-3-4 4-2-2-5 5h14z" fill="#555"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="none">
      <defs><linearGradient id="photoGradMobile" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ef4444"/><stop offset="100%" style="stop-color:#dc2626"/></linearGradient></defs>
      <rect x="2" y="4" width="20" height="16" rx="4" fill="url(#photoGradMobile)"/>
      <circle cx="8" cy="9" r="1.5" fill="#fff" opacity="0.9"/>
      <path d="M18 15l-3-3-4 4-2-2-5 5h14z" fill="#000" opacity="0.2"/>
    </svg>`
  },
  {
    id: 'favorites' as const,
    label: 'Favoritos',
    icon: `<svg viewBox="0 0 24 24" fill="none">
      <path d="M12 2l2.4 7.2h7.6l-6 4.8 2.4 7.2-6-4.8-6 4.8 2.4-7.2-6-4.8h7.6z" fill="#333" stroke="#555" stroke-width="1"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="none">
      <defs><linearGradient id="starGradMobile" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ef4444"/><stop offset="100%" style="stop-color:#dc2626"/></linearGradient></defs>
      <path d="M12 2l2.4 7.2h7.6l-6 4.8 2.4 7.2-6-4.8-6 4.8 2.4-7.2-6-4.8h7.6z" fill="url(#starGradMobile)"/>
    </svg>`
  },
  {
    id: 'albums' as const,
    label: 'Álbuns',
    icon: `<svg viewBox="0 0 24 24" fill="none">
      <path d="M3 6a3 3 0 0 1 3-3h4l2 2h7a3 3 0 0 1 3 3v10a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6z" fill="#333" stroke="#555" stroke-width="1"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="none">
      <defs><linearGradient id="folderGradMobile" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ef4444"/><stop offset="100%" style="stop-color:#dc2626"/></linearGradient></defs>
      <path d="M3 6a3 3 0 0 1 3-3h4l2 2h7a3 3 0 0 1 3 3v10a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6z" fill="url(#folderGradMobile)"/>
    </svg>`
  },
  {
    id: 'trash' as const,
    label: 'Lixeira',
    icon: `<svg viewBox="0 0 24 24" fill="none">
      <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" stroke="#555" stroke-width="2" fill="none"/>
      <path d="M6 8h12l-1 12a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2L6 8z" fill="#333"/>
      <line x1="10" y1="12" x2="10" y2="16" stroke="#666" stroke-width="2"/>
      <line x1="14" y1="12" x2="14" y2="16" stroke="#666" stroke-width="2"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="none">
      <defs><linearGradient id="trashGradMobile" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ef4444"/><stop offset="100%" style="stop-color:#dc2626"/></linearGradient></defs>
      <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" stroke="#666" stroke-width="2" fill="none"/>
      <path d="M6 8h12l-1 12a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2L6 8z" fill="url(#trashGradMobile)"/>
      <line x1="10" y1="12" x2="10" y2="16" stroke="#fff" stroke-width="2" opacity="0.8"/>
      <line x1="14" y1="12" x2="14" y2="16" stroke="#fff" stroke-width="2" opacity="0.8"/>
    </svg>`
  },
  {
    id: 'settings' as const,
    label: 'Config',
    icon: `<svg viewBox="0 0 24 24" fill="none">
      <path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z" fill="#333"/>
      <path d="M12 1l1.5 4.5L18 4l-1.5 4.5L21 12l-4.5 1.5L18 20l-4.5-1.5L12 23l-1.5-4.5L6 20l1.5-4.5L3 12l4.5-1.5L6 4l4.5 1.5L12 1z" stroke="#555" stroke-width="1.5" fill="none"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="none">
      <defs><linearGradient id="settingsGradMobile" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#ef4444"/><stop offset="100%" style="stop-color:#dc2626"/></linearGradient></defs>
      <path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z" fill="url(#settingsGradMobile)"/>
      <path d="M12 1l1.5 4.5L18 4l-1.5 4.5L21 12l-4.5 1.5L18 20l-4.5-1.5L12 23l-1.5-4.5L6 20l1.5-4.5L3 12l4.5-1.5L6 4l4.5 1.5L12 1z" stroke="url(#settingsGradMobile)" stroke-width="1.5" fill="none"/>
    </svg>`
  }
])

function handleClick(item: typeof navItems.value[0]) {
  triggerHaptic()
  if (item.id === 'settings') {
    emit('settings')
  } else {
    emit('navigate', item.id)
  }
}
</script>

<template>
  <nav 
    class="mobile-nav" 
    :style="{ '--nav-accent': accentColor, '--touch-min': `${TOUCH.minTarget}px` }"
    role="navigation"
    aria-label="Navegação principal"
  >
    <button 
      v-for="item in navItems"
      :key="item.id"
      class="mobile-nav-item" 
      :class="{ active: currentView === item.id }"
      :aria-current="currentView === item.id ? 'page' : undefined"
      :aria-label="item.label"
      @click="handleClick(item)"
    >
      <span 
        class="nav-icon" 
        v-html="currentView === item.id ? item.iconFilled : item.icon"
        aria-hidden="true"
      />
      <span class="nav-label">{{ item.label }}</span>
      <span v-if="currentView === item.id" class="active-indicator" aria-hidden="true" />
    </button>
  </nav>
</template>

<style scoped>
.mobile-nav {
  display: none;
}

@media (max-width: 768px) {
  .mobile-nav {
    display: flex;
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    padding-bottom: env(safe-area-inset-bottom, 0px);
    height: calc(49px + env(safe-area-inset-bottom, 0px));
    background: var(--systemStandardThickMaterialSover);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border-top: 0.5px solid var(--labelDivider);
    z-index: 100;
    justify-content: space-around;
    align-items: flex-start;
    padding-top: 6px;
  }

  .mobile-nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-width: 64px;
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: var(--systemSecondary);
    font-size: 10px;
    font-weight: 500;
    cursor: pointer;
    transition: color 0.15s ease;
    -webkit-tap-highlight-color: transparent;
    position: relative;
    touch-action: manipulation;
  }

  .nav-icon {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s ease;
  }

  .nav-icon :deep(svg) {
    width: 100%;
    height: 100%;
  }

  .nav-label {
    transition: color 0.15s ease;
  }

  .mobile-nav-item.active {
    color: var(--keyColor);
  }

  .mobile-nav-item.active .nav-icon {
    transform: scale(1.05);
  }

  .active-indicator {
    display: none;
  }

  .mobile-nav-item:active {
    opacity: 0.7;
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .mobile-nav-item,
    .nav-icon {
      transition: none;
    }
  }
}
</style>
