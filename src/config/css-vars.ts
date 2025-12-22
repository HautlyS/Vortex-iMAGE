/**
 * CSS Custom Properties configuration
 * Generates CSS variables from constants for use in styles
 */

import { BREAKPOINTS, LAYOUT, ANIMATION, TOUCH, Z_INDEX, GALLERY } from './constants'

/**
 * Inject CSS custom properties into document root
 * Call this on app mount to sync JS constants with CSS
 */
export function injectCSSVariables(): void {
  const root = document.documentElement
  
  // Breakpoints
  Object.entries(BREAKPOINTS).forEach(([key, value]) => {
    root.style.setProperty(`--breakpoint-${key}`, `${value}px`)
  })
  
  // Layout
  root.style.setProperty('--sidebar-width', `${LAYOUT.sidebar.width}px`)
  root.style.setProperty('--sidebar-collapsed', `${LAYOUT.sidebar.collapsedWidth}px`)
  root.style.setProperty('--header-height', `${LAYOUT.header.height}px`)
  root.style.setProperty('--header-mobile', `${LAYOUT.header.mobileHeight}px`)
  root.style.setProperty('--mobile-nav-height', `${LAYOUT.mobileNav.height}px`)
  root.style.setProperty('--mobile-nav-gap', `${LAYOUT.mobileNav.gap}px`)
  root.style.setProperty('--content-max-width', `${LAYOUT.content.maxWidth}px`)
  root.style.setProperty('--content-padding', `${LAYOUT.content.padding}px`)
  root.style.setProperty('--content-padding-mobile', `${LAYOUT.content.mobilePadding}px`)
  
  // Gallery
  root.style.setProperty('--preview-min-size', `${GALLERY.preview.minSize}px`)
  root.style.setProperty('--preview-default-size', `${GALLERY.preview.defaultSize}px`)
  root.style.setProperty('--preview-max-size', `${GALLERY.preview.maxSize}px`)
  root.style.setProperty('--grid-gap', `${GALLERY.grid.gap}px`)
  root.style.setProperty('--grid-gap-mobile', `${GALLERY.grid.mobileGap}px`)
  
  // Animation
  root.style.setProperty('--duration-fast', `${ANIMATION.fast}ms`)
  root.style.setProperty('--duration-normal', `${ANIMATION.normal}ms`)
  root.style.setProperty('--duration-slow', `${ANIMATION.slow}ms`)
  root.style.setProperty('--ease-spring', ANIMATION.spring)
  root.style.setProperty('--ease-out', ANIMATION.easeOut)
  root.style.setProperty('--ease-in-out', ANIMATION.easeInOut)
  
  // Touch
  root.style.setProperty('--touch-target-min', `${TOUCH.minTarget}px`)
  root.style.setProperty('--touch-target-comfortable', `${TOUCH.comfortableTarget}px`)
  
  // Z-Index
  Object.entries(Z_INDEX).forEach(([key, value]) => {
    root.style.setProperty(`--z-${key}`, String(value))
  })
}

/**
 * Media query helper for JS
 */
export function matchesBreakpoint(breakpoint: keyof typeof BREAKPOINTS, direction: 'up' | 'down' = 'up'): boolean {
  const value = BREAKPOINTS[breakpoint]
  const query = direction === 'up' 
    ? `(min-width: ${value}px)` 
    : `(max-width: ${value - 1}px)`
  return window.matchMedia(query).matches
}

/**
 * Check if device prefers reduced motion
 */
export function prefersReducedMotion(): boolean {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

/**
 * Check if device is touch-primary
 */
export function isTouchDevice(): boolean {
  return window.matchMedia('(hover: none) and (pointer: coarse)').matches
}
