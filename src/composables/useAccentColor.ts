/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed } from 'vue'
import { load } from '@tauri-apps/plugin-store'

export type AccentColor = 'pink' | 'cyan' | 'purple' | 'green' | 'orange' | 'yellow'

const accent = ref<AccentColor>('cyan')

const colors: Record<AccentColor, string> = {
  pink: '#ff2d6a',
  cyan: '#00f0ff',
  purple: '#b026ff',
  green: '#39ff14',
  orange: '#ff6b35',
  yellow: '#ffd700',
}

export function useAccentColor() {
  const accentHex = computed(() => colors[accent.value])
  const accentClass = computed(() => `text-cyber-${accent.value}`)
  const bgClass = computed(() => `bg-cyber-${accent.value}`)
  const borderClass = computed(() => `border-cyber-${accent.value}`)

  async function init() {
    try {
      const store = await load('settings.json')
      const saved = await store.get<AccentColor>('accent')
      if (saved && colors[saved]) {
        accent.value = saved
        document.documentElement.style.setProperty('--accent', colors[saved])
      }
    } catch {
      
    }
  }

  async function setAccent(color: AccentColor) {
    accent.value = color
    document.documentElement.style.setProperty('--accent', colors[color])
    try {
      const store = await load('settings.json')
      await store.set('accent', color)
      await store.save()
    } catch {
      
    }
  }

  return { accent, accentHex, accentClass, bgClass, borderClass, init, setAccent, colors }
}