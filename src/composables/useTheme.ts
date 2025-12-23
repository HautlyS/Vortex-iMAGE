/**
 * TypeScript Module - 9 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed, watch } from 'vue'
import { load } from '@tauri-apps/plugin-store'

export interface ThemeConfig {
  
  accentColor: string
  accentSecondary: string

  matrixEffects: boolean
  scanlines: boolean
  glow: boolean
  glowIntensity: number 

  borderRadius: 'sharp' | 'soft' | 'round'
  glassMorphism: boolean
  glassOpacity: number 

  animationSpeed: 'instant' | 'fast' | 'normal' | 'slow'
  reduceMotion: boolean

  fontFamily: 'inter' | 'system' | 'mono'
  fontSize: 'compact' | 'normal' | 'large'

  anarchyMode: boolean
  anarchyHue: number 
  anarchyNoise: boolean
  anarchyGlitch: boolean
}

export const ACCENT_COLORS = [
  { id: 'cyber', color: '#00f0ff', secondary: '#b026ff', name: 'Cyber' },
  { id: 'neon', color: '#ff2d6a', secondary: '#b026ff', name: 'Neon Pink' },
  { id: 'matrix', color: '#39ff14', secondary: '#00ff88', name: 'Matrix' },
  { id: 'sunset', color: '#ff6b35', secondary: '#ffd700', name: 'Sunset' },
  { id: 'aurora', color: '#00ff88', secondary: '#00f0ff', name: 'Aurora' },
  { id: 'blood', color: '#ff1744', secondary: '#ff2d6a', name: 'Blood' },
  { id: 'gold', color: '#ffd700', secondary: '#ff6b35', name: 'Gold' },
  { id: 'ice', color: '#88ffff', secondary: '#00bfff', name: 'Ice' },
  { id: 'toxic', color: '#39ff14', secondary: '#ccff00', name: 'Toxic' },
  { id: 'void', color: '#b026ff', secondary: '#ff2d6a', name: 'Void' },
]

export const BORDER_RADIUS = {
  sharp: { sm: '4px', md: '6px', lg: '8px', xl: '12px', full: '6px' },
  soft: { sm: '8px', md: '12px', lg: '18px', xl: '24px', full: '16px' },
  round: { sm: '12px', md: '18px', lg: '26px', xl: '36px', full: '9999px' },
}

export const ANIMATION_SPEEDS = {
  instant: { fast: '0ms', normal: '0ms', slow: '0ms' },
  fast: { fast: '100ms', normal: '150ms', slow: '200ms' },
  normal: { fast: '150ms', normal: '250ms', slow: '400ms' },
  slow: { fast: '250ms', normal: '400ms', slow: '600ms' },
}

export const FONT_FAMILIES = {
  inter: "'Inter', system-ui, sans-serif",
  system: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  mono: "'JetBrains Mono', 'Fira Code', monospace",
}

export const FONT_SIZES = {
  compact: { base: '13px', sm: '11px', lg: '15px', xl: '18px' },
  normal: { base: '14px', sm: '12px', lg: '16px', xl: '20px' },
  large: { base: '16px', sm: '14px', lg: '18px', xl: '24px' },
}

const DEFAULT_THEME: ThemeConfig = {
  accentColor: '#00f0ff',
  accentSecondary: '#b026ff',
  matrixEffects: true,
  scanlines: false,
  glow: true,
  glowIntensity: 70,
  borderRadius: 'round',
  glassMorphism: true,
  glassOpacity: 75,
  animationSpeed: 'normal',
  reduceMotion: false,
  fontFamily: 'inter',
  fontSize: 'normal',
  anarchyMode: false,
  anarchyHue: 0,
  anarchyNoise: false,
  anarchyGlitch: false,
}

const theme = ref<ThemeConfig>({ ...DEFAULT_THEME })
let initialized = false

export function getContrastRatio(color1: string, color2: string): number {
  const lum1 = getLuminance(color1)
  const lum2 = getLuminance(color2)
  const lighter = Math.max(lum1, lum2)
  const darker = Math.min(lum1, lum2)
  return (lighter + 0.05) / (darker + 0.05)
}

function getLuminance(hex: string): number {
  const rgb = hexToRgb(hex)
  if (!rgb) return 0
  const [r, g, b] = [rgb.r, rgb.g, rgb.b].map((c) => {
    const sRGB = c / 255
    return sRGB <= 0.03928 ? sRGB / 12.92 : Math.pow((sRGB + 0.055) / 1.055, 2.4)
  })
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}

export function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result
    ? { r: parseInt(result[1], 16), g: parseInt(result[2], 16), b: parseInt(result[3], 16) }
    : null
}

export function meetsWCAGAA(foreground: string, background: string, isLargeText = false): boolean {
  const ratio = getContrastRatio(foreground, background)
  return isLargeText ? ratio >= 3 : ratio >= 4.5
}

function applyTheme(config: ThemeConfig) {
  const root = document.documentElement
  const style = root.style

  style.setProperty('--accent-color', config.accentColor)
  style.setProperty('--accent-secondary', config.accentSecondary)
  
  const rgb = hexToRgb(config.accentColor)
  const rgb2 = hexToRgb(config.accentSecondary)
  if (rgb) {
    style.setProperty('--accent-rgb', `${rgb.r}, ${rgb.g}, ${rgb.b}`)
    style.setProperty('--accent-glow', `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${config.glowIntensity / 100})`)
    style.setProperty('--accent-light', `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.1)`)
    style.setProperty('--accent-medium', `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.2)`)
    style.setProperty('--accent-strong', `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.35)`)
  }
  if (rgb2) {
    style.setProperty('--accent-secondary-rgb', `${rgb2.r}, ${rgb2.g}, ${rgb2.b}`)
  }

  const glassOpacity = config.glassOpacity / 100
  style.setProperty('--glass-opacity', String(glassOpacity))
  style.setProperty('--glass-bg', `rgba(14, 14, 20, ${glassOpacity})`)

  const radius = BORDER_RADIUS[config.borderRadius]
  style.setProperty('--radius-sm', radius.sm)
  style.setProperty('--radius-md', radius.md)
  style.setProperty('--radius-lg', radius.lg)
  style.setProperty('--radius-xl', radius.xl)
  style.setProperty('--radius-full', radius.full)

  const speeds = config.reduceMotion ? ANIMATION_SPEEDS.instant : ANIMATION_SPEEDS[config.animationSpeed]
  style.setProperty('--duration-fast', speeds.fast)
  style.setProperty('--duration-normal', speeds.normal)
  style.setProperty('--duration-slow', speeds.slow)

  style.setProperty('--font-family', FONT_FAMILIES[config.fontFamily])
  const sizes = FONT_SIZES[config.fontSize]
  style.setProperty('--font-size-base', sizes.base)
  style.setProperty('--font-size-sm', sizes.sm)
  style.setProperty('--font-size-lg', sizes.lg)
  style.setProperty('--font-size-xl', sizes.xl)

  style.setProperty('--glow-intensity', `${config.glowIntensity}%`)

  if (config.anarchyMode) {
    style.setProperty('--anarchy-hue', `${config.anarchyHue}deg`)
  }

  root.classList.toggle('matrix-effects', config.matrixEffects)
  root.classList.toggle('scanlines', config.scanlines)
  root.classList.toggle('glow-effects', config.glow)
  root.classList.toggle('glass-morphism', config.glassMorphism)
  root.classList.toggle('reduce-motion', config.reduceMotion)
  root.classList.toggle('anarchy-mode', config.anarchyMode)
  root.classList.toggle('anarchy-noise', config.anarchyNoise)
  root.classList.toggle('anarchy-glitch', config.anarchyGlitch)

  root.classList.remove('font-inter', 'font-system', 'font-mono')
  root.classList.add(`font-${config.fontFamily}`)

  root.classList.remove('size-compact', 'size-normal', 'size-large')
  root.classList.add(`size-${config.fontSize}`)
}

export function useTheme() {
  const accentColor = computed(() => theme.value.accentColor)
  const accentSecondary = computed(() => theme.value.accentSecondary)
  const matrixEffects = computed(() => theme.value.matrixEffects)
  const scanlines = computed(() => theme.value.scanlines)
  const glow = computed(() => theme.value.glow)
  const glowIntensity = computed(() => theme.value.glowIntensity)
  const borderRadius = computed(() => theme.value.borderRadius)
  const glassMorphism = computed(() => theme.value.glassMorphism)
  const glassOpacity = computed(() => theme.value.glassOpacity)
  const animationSpeed = computed(() => theme.value.animationSpeed)
  const reduceMotion = computed(() => theme.value.reduceMotion)
  const fontFamily = computed(() => theme.value.fontFamily)
  const fontSize = computed(() => theme.value.fontSize)
  const anarchyMode = computed(() => theme.value.anarchyMode)

  async function loadTheme(): Promise<void> {
    if (initialized) return
    try {
      const store = await load('settings.json')
      const savedTheme = await store.get<ThemeConfig>('theme')
      if (savedTheme) {
        theme.value = { ...DEFAULT_THEME, ...savedTheme }
      }
    } catch {
      
    }
    
    applyTheme(theme.value)
    initialized = true
  }

  async function saveTheme(): Promise<void> {
    try {
      const store = await load('settings.json')
      await store.set('theme', theme.value)
      await store.save()
    } catch {
      
    }
  }

  function updateTheme(updates: Partial<ThemeConfig>): void {
    theme.value = { ...theme.value, ...updates }
    applyTheme(theme.value)
    saveTheme()
  }

  function setAccentColor(color: string, secondary?: string): void {
    const preset = ACCENT_COLORS.find(c => c.color === color)
    updateTheme({ 
      accentColor: color, 
      accentSecondary: secondary || preset?.secondary || color 
    })
  }

  function setAccentPreset(presetId: string): void {
    const preset = ACCENT_COLORS.find(c => c.id === presetId)
    if (preset) {
      updateTheme({ accentColor: preset.color, accentSecondary: preset.secondary })
    }
  }

  function setMatrixEffects(enabled: boolean): void {
    updateTheme({ matrixEffects: enabled })
  }

  function setScanlines(enabled: boolean): void {
    updateTheme({ scanlines: enabled })
  }

  function setGlow(enabled: boolean): void {
    updateTheme({ glow: enabled })
  }

  function setGlowIntensity(intensity: number): void {
    updateTheme({ glowIntensity: Math.max(0, Math.min(100, intensity)) })
  }

  function setBorderRadius(radius: ThemeConfig['borderRadius']): void {
    updateTheme({ borderRadius: radius })
  }

  function setGlassMorphism(enabled: boolean): void {
    updateTheme({ glassMorphism: enabled })
  }

  function setGlassOpacity(opacity: number): void {
    updateTheme({ glassOpacity: Math.max(0, Math.min(100, opacity)) })
  }

  function setAnimationSpeed(speed: ThemeConfig['animationSpeed']): void {
    updateTheme({ animationSpeed: speed })
  }

  function setReduceMotion(enabled: boolean): void {
    updateTheme({ reduceMotion: enabled })
  }

  function setFontFamily(family: ThemeConfig['fontFamily']): void {
    updateTheme({ fontFamily: family })
  }

  function setFontSize(size: ThemeConfig['fontSize']): void {
    updateTheme({ fontSize: size })
  }

  function setAnarchyMode(enabled: boolean): void {
    updateTheme({ anarchyMode: enabled })
  }

  function setAnarchyHue(hue: number): void {
    updateTheme({ anarchyHue: hue % 360 })
  }

  function setAnarchyNoise(enabled: boolean): void {
    updateTheme({ anarchyNoise: enabled })
  }

  function setAnarchyGlitch(enabled: boolean): void {
    updateTheme({ anarchyGlitch: enabled })
  }

  function resetTheme(): void {
    theme.value = { ...DEFAULT_THEME }
    applyTheme(theme.value)
    saveTheme()
  }

  function exportTheme(): string {
    return JSON.stringify(theme.value, null, 2)
  }

  function importTheme(json: string): boolean {
    try {
      const imported = JSON.parse(json) as Partial<ThemeConfig>
      theme.value = { ...DEFAULT_THEME, ...imported }
      applyTheme(theme.value)
      saveTheme()
      return true
    } catch {
      return false
    }
  }

  watch(theme, applyTheme, { deep: true })

  return {
    theme,
    accentColor,
    accentSecondary,
    matrixEffects,
    scanlines,
    glow,
    glowIntensity,
    borderRadius,
    glassMorphism,
    glassOpacity,
    animationSpeed,
    reduceMotion,
    fontFamily,
    fontSize,
    anarchyMode,
    loadTheme,
    saveTheme,
    updateTheme,
    setAccentColor,
    setAccentPreset,
    setMatrixEffects,
    setScanlines,
    setGlow,
    setGlowIntensity,
    setBorderRadius,
    setGlassMorphism,
    setGlassOpacity,
    setAnimationSpeed,
    setReduceMotion,
    setFontFamily,
    setFontSize,
    setAnarchyMode,
    setAnarchyHue,
    setAnarchyNoise,
    setAnarchyGlitch,
    resetTheme,
    exportTheme,
    importTheme,
    ACCENT_COLORS,
    getContrastRatio,
    meetsWCAGAA,
  }
}