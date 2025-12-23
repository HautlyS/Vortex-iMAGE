/**
 * TypeScript Module - 4 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { BREAKPOINTS, LAYOUT, ANIMATION, TOUCH, Z_INDEX, GALLERY } from './constants'

export function injectCSSVariables(): void {
  const root = document.documentElement

  Object.entries(BREAKPOINTS).forEach(([key, value]) => {
    root.style.setProperty(`--breakpoint-${key}`, `${value}px`)
  })

  root.style.setProperty('--sidebar-width', `${LAYOUT.sidebar.width}px`)
  root.style.setProperty('--sidebar-collapsed', `${LAYOUT.sidebar.collapsedWidth}px`)
  root.style.setProperty('--header-height', `${LAYOUT.header.height}px`)
  root.style.setProperty('--header-mobile', `${LAYOUT.header.mobileHeight}px`)
  root.style.setProperty('--mobile-nav-height', `${LAYOUT.mobileNav.height}px`)
  root.style.setProperty('--mobile-nav-gap', `${LAYOUT.mobileNav.gap}px`)
  root.style.setProperty('--content-max-width', `${LAYOUT.content.maxWidth}px`)
  root.style.setProperty('--content-padding', `${LAYOUT.content.padding}px`)
  root.style.setProperty('--content-padding-mobile', `${LAYOUT.content.mobilePadding}px`)

  root.style.setProperty('--preview-min-size', `${GALLERY.preview.minSize}px`)
  root.style.setProperty('--preview-default-size', `${GALLERY.preview.defaultSize}px`)
  root.style.setProperty('--preview-max-size', `${GALLERY.preview.maxSize}px`)
  root.style.setProperty('--grid-gap', `${GALLERY.grid.gap}px`)
  root.style.setProperty('--grid-gap-mobile', `${GALLERY.grid.mobileGap}px`)

  root.style.setProperty('--duration-fast', `${ANIMATION.fast}ms`)
  root.style.setProperty('--duration-normal', `${ANIMATION.normal}ms`)
  root.style.setProperty('--duration-slow', `${ANIMATION.slow}ms`)
  root.style.setProperty('--ease-spring', ANIMATION.spring)
  root.style.setProperty('--ease-out', ANIMATION.easeOut)
  root.style.setProperty('--ease-in-out', ANIMATION.easeInOut)

  root.style.setProperty('--touch-target-min', `${TOUCH.minTarget}px`)
  root.style.setProperty('--touch-target-comfortable', `${TOUCH.comfortableTarget}px`)

  Object.entries(Z_INDEX).forEach(([key, value]) => {
    root.style.setProperty(`--z-${key}`, String(value))
  })
}

export function matchesBreakpoint(breakpoint: keyof typeof BREAKPOINTS, direction: 'up' | 'down' = 'up'): boolean {
  const value = BREAKPOINTS[breakpoint]
  const query = direction === 'up' 
    ? `(min-width: ${value}px)` 
    : `(max-width: ${value - 1}px)`
  return window.matchMedia(query).matches
}

export function prefersReducedMotion(): boolean {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

export function isTouchDevice(): boolean {
  return window.matchMedia('(hover: none) and (pointer: coarse)').matches
}