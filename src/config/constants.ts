/**
 * Centralized configuration constants for iMAGE
 * Eliminates magic numbers and hardcoded values throughout the app
 */

// ============================================================================
// Breakpoints - Responsive Design
// ============================================================================
export const BREAKPOINTS = {
  xs: 320,   // Small phones
  sm: 480,   // Large phones
  md: 768,   // Tablets
  lg: 1024,  // Small laptops
  xl: 1280,  // Desktops
  xxl: 1536, // Large screens
} as const

export const MEDIA_QUERIES = {
  mobile: `(max-width: ${BREAKPOINTS.md - 1}px)`,
  tablet: `(min-width: ${BREAKPOINTS.md}px) and (max-width: ${BREAKPOINTS.lg - 1}px)`,
  desktop: `(min-width: ${BREAKPOINTS.lg}px)`,
  touch: '(hover: none) and (pointer: coarse)',
  reducedMotion: '(prefers-reduced-motion: reduce)',
} as const

// ============================================================================
// Layout Dimensions
// ============================================================================
export const LAYOUT = {
  sidebar: {
    width: 240,
    collapsedWidth: 64,
  },
  header: {
    height: 64,
    mobileHeight: 56,
  },
  mobileNav: {
    height: 64,
    gap: 12,
  },
  content: {
    maxWidth: 1400,
    padding: 24,
    mobilePadding: 12,
  },
} as const

// ============================================================================
// Photo Gallery
// ============================================================================
export const GALLERY = {
  preview: {
    minSize: 100,
    defaultSize: 180,
    maxSize: 400,
    step: 20,
  },
  grid: {
    gap: 12,
    mobileGap: 8,
  },
  list: {
    thumbSize: 48,
    rowHeight: 64,
  },
  virtualScroll: {
    overscan: 5,
    threshold: 100,
  },
} as const

// ============================================================================
// Upload Configuration
// ============================================================================
export const UPLOAD = {
  maxRetries: 3,
  retryDelayMs: 2000,
  maxFileSizeMb: 50,
  lfsThresholdMb: 50,
  chunkSizeKb: 512,
  supportedFormats: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'heic', 'raw', 'bmp', 'tiff', 'avif'],
  defaultQuality: 85,
} as const

// ============================================================================
// Animation Timings
// ============================================================================
export const ANIMATION = {
  fast: 150,
  normal: 250,
  slow: 400,
  spring: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
  easeOut: 'cubic-bezier(0.16, 1, 0.3, 1)',
  easeInOut: 'cubic-bezier(0.65, 0, 0.35, 1)',
} as const

// ============================================================================
// Touch & Interaction
// ============================================================================
export const TOUCH = {
  minTarget: 44,
  comfortableTarget: 48,
  longPressMs: 500,
  doubleTapMs: 300,
  swipeThreshold: 50,
} as const

// ============================================================================
// Polling & Timeouts
// ============================================================================
export const TIMING = {
  oauth: {
    pollIntervalMs: 5000,
    expirationMs: 900000, // 15 minutes
  },
  debounce: {
    search: 300,
    resize: 150,
    scroll: 100,
  },
  toast: {
    defaultDurationMs: 5000,
    errorDurationMs: 8000,
  },
  autoSave: {
    intervalMs: 30000,
  },
} as const

// ============================================================================
// Z-Index Layers
// ============================================================================
export const Z_INDEX = {
  base: 0,
  dropdown: 10,
  sticky: 20,
  fixed: 30,
  modalBackdrop: 40,
  modal: 50,
  popover: 60,
  tooltip: 70,
  toast: 80,
  mobileNav: 100,
  lightbox: 300,
  contextMenu: 400,
  max: 9999,
} as const

// ============================================================================
// Color Tags (Predefined)
// ============================================================================
export const COLOR_TAGS = [
  { id: 'red', name: 'Vermelho', color: '#ef4444' },
  { id: 'orange', name: 'Laranja', color: '#f97316' },
  { id: 'yellow', name: 'Amarelo', color: '#eab308' },
  { id: 'green', name: 'Verde', color: '#22c55e' },
  { id: 'blue', name: 'Azul', color: '#3b82f6' },
  { id: 'purple', name: 'Roxo', color: '#a855f7' },
  { id: 'pink', name: 'Rosa', color: '#ec4899' },
  { id: 'gray', name: 'Cinza', color: '#6b7280' },
] as const

// ============================================================================
// Image Extensions
// ============================================================================
export const IMAGE_EXTENSIONS = [
  'jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 
  'tiff', 'tif', 'svg', 'ico', 'heic', 'heif', 'avif'
] as const

// ============================================================================
// GitHub API
// ============================================================================
export const GITHUB = {
  clientId: 'Ov23lijNSMM1i93CQdfQ',
  apiBase: 'https://api.github.com',
  uploadTimeout: 120000,
  lfsUploadTimeout: 300000,
} as const

// ============================================================================
// Keyboard Shortcuts
// ============================================================================
export const SHORTCUTS = {
  selectAll: { key: 'a', ctrl: true },
  delete: { key: 'Delete' },
  favorite: { key: 'f' },
  escape: { key: 'Escape' },
  search: { key: 'k', ctrl: true },
  upload: { key: 'u', ctrl: true },
} as const

// ============================================================================
// Accessibility
// ============================================================================
export const A11Y = {
  focusRingWidth: 2,
  focusRingOffset: 2,
  minContrastRatio: 4.5,
  largeTextContrastRatio: 3,
} as const

// Type exports for TypeScript
export type Breakpoint = keyof typeof BREAKPOINTS
export type ColorTagId = typeof COLOR_TAGS[number]['id']
export type ImageExtension = typeof IMAGE_EXTENSIONS[number]
