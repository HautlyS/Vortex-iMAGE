<script setup lang="ts">
import { computed } from 'vue'
import { useTheme } from '../composables/useTheme'

defineProps<{
  currentView: 'photos' | 'favorites' | 'albums' | 'tags'
}>()

const emit = defineEmits<{
  navigate: [view: 'photos' | 'favorites' | 'albums' | 'tags']
  settings: []
}>()

const { accentColor } = useTheme()

const navItems = computed(() => [
  {
    id: 'photos' as const,
    label: 'Fotos',
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="7" height="7" rx="1"/>
      <rect x="14" y="3" width="7" height="7" rx="1"/>
      <rect x="14" y="14" width="7" height="7" rx="1"/>
      <rect x="3" y="14" width="7" height="7" rx="1"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="currentColor">
      <rect x="3" y="3" width="7" height="7" rx="1"/>
      <rect x="14" y="3" width="7" height="7" rx="1"/>
      <rect x="14" y="14" width="7" height="7" rx="1"/>
      <rect x="3" y="14" width="7" height="7" rx="1"/>
    </svg>`
  },
  {
    id: 'favorites' as const,
    label: 'Favoritos',
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
      <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
    </svg>`
  },
  {
    id: 'albums' as const,
    label: 'Álbuns',
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
    </svg>`
  },
  {
    id: 'settings' as const,
    label: 'Config',
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
    </svg>`,
    iconFilled: `<svg viewBox="0 0 24 24" fill="currentColor">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
    </svg>`
  }
])

function handleClick(item: typeof navItems.value[0]) {
  if (item.id === 'settings') {
    emit('settings')
  } else {
    emit('navigate', item.id)
  }
}
</script>

<template>
  <nav class="mobile-nav" :style="{ '--nav-accent': accentColor }">
    <button 
      v-for="item in navItems"
      :key="item.id"
      class="mobile-nav-item" 
      :class="{ active: currentView === item.id }"
      @click="handleClick(item)"
    >
      <span 
        class="nav-icon" 
        v-html="currentView === item.id ? item.iconFilled : item.icon"
      />
      <span class="nav-label">{{ item.label }}</span>
      <span v-if="currentView === item.id" class="active-indicator" />
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
    bottom: var(--mobile-gap-md, 12px);
    left: var(--mobile-gap-md, 12px);
    right: var(--mobile-gap-md, 12px);
    bottom: calc(var(--mobile-gap-md, 12px) + env(safe-area-inset-bottom, 0px));
    height: var(--mobile-nav-height, 64px);
    background: var(--amoled-surface-2, #121212);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-xl, 20px);
    z-index: 100;
    justify-content: space-around;
    align-items: center;
    box-shadow: 
      0 4px 24px rgba(0, 0, 0, 0.5),
      0 0 0 1px rgba(255, 255, 255, 0.05) inset;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .mobile-nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: var(--mobile-gap-sm, 8px) var(--mobile-gap-lg, 16px);
    background: transparent;
    border: none;
    color: var(--text-muted, #71717a);
    font-size: 0.625rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
    -webkit-tap-highlight-color: transparent;
    border-radius: var(--radius-lg, 14px);
    position: relative;
  }

  .nav-icon {
    width: 1.375rem;
    height: 1.375rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .nav-icon :deep(svg) {
    width: 100%;
    height: 100%;
  }

  .nav-label {
    transition: all 0.2s ease;
  }

  .mobile-nav-item.active {
    color: var(--nav-accent, var(--accent-color, #00ff41));
  }

  .mobile-nav-item.active .nav-icon {
    transform: scale(1.15);
    filter: drop-shadow(0 0 8px var(--nav-accent, var(--accent-color, #00ff41)));
  }

  .active-indicator {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 24px;
    height: 3px;
    background: var(--nav-accent, var(--accent-color, #00ff41));
    border-radius: var(--radius-full, 9999px);
    box-shadow: 0 0 12px var(--nav-accent, var(--accent-color, #00ff41));
    animation: indicator-appear 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  @keyframes indicator-appear {
    from {
      transform: translateX(-50%) scaleX(0);
      opacity: 0;
    }
    to {
      transform: translateX(-50%) scaleX(1);
      opacity: 1;
    }
  }

  .mobile-nav-item:active {
    transform: scale(0.92);
  }
}
</style>
