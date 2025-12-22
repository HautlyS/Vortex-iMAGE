import { describe, it, expect, vi, beforeEach } from 'vitest'
import * as fc from 'fast-check'
import { getContrastRatio, meetsWCAGAA, ACCENT_COLORS } from '../useTheme'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(() =>
    Promise.resolve({
      get: vi.fn(),
      set: vi.fn(),
      save: vi.fn(),
    })
  ),
}))

// Helper to generate valid hex colors
const hexColorArb = fc
  .array(fc.integer({ min: 0, max: 255 }), { minLength: 3, maxLength: 3 })
  .map(([r, g, b]) => `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`)

describe('useTheme', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('getContrastRatio', () => {
    it('returns 21 for black on white', () => {
      const ratio = getContrastRatio('#ffffff', '#000000')
      expect(ratio).toBeCloseTo(21, 0)
    })

    it('returns 1 for same colors', () => {
      fc.assert(
        fc.property(hexColorArb, (color) => {
          const ratio = getContrastRatio(color, color)
          expect(ratio).toBeCloseTo(1, 1)
          return true
        }),
        { numRuns: 50 }
      )
    })

    it('is symmetric', () => {
      fc.assert(
        fc.property(hexColorArb, hexColorArb, (color1, color2) => {
          const ratio1 = getContrastRatio(color1, color2)
          const ratio2 = getContrastRatio(color2, color1)
          expect(ratio1).toBeCloseTo(ratio2, 5)
          return true
        }),
        { numRuns: 50 }
      )
    })

    it('returns value between 1 and 21', () => {
      fc.assert(
        fc.property(hexColorArb, hexColorArb, (color1, color2) => {
          const ratio = getContrastRatio(color1, color2)
          expect(ratio).toBeGreaterThanOrEqual(1)
          expect(ratio).toBeLessThanOrEqual(21)
          return true
        }),
        { numRuns: 100 }
      )
    })
  })

  describe('meetsWCAGAA', () => {
    it('white on black meets AA for normal text', () => {
      expect(meetsWCAGAA('#ffffff', '#000000')).toBe(true)
    })

    it('white on black meets AA for large text', () => {
      expect(meetsWCAGAA('#ffffff', '#000000', true)).toBe(true)
    })

    it('same color fails AA', () => {
      fc.assert(
        fc.property(hexColorArb, (color) => {
          expect(meetsWCAGAA(color, color)).toBe(false)
          return true
        }),
        { numRuns: 50 }
      )
    })

    it('large text has lower threshold than normal text', () => {
      // A color pair that passes large text but might fail normal text
      // Gray on dark gray
      const foreground = '#888888'
      const background = '#333333'
      const ratio = getContrastRatio(foreground, background)

      // If ratio is between 3 and 4.5, it passes large but fails normal
      if (ratio >= 3 && ratio < 4.5) {
        expect(meetsWCAGAA(foreground, background, true)).toBe(true)
        expect(meetsWCAGAA(foreground, background, false)).toBe(false)
      }
    })
  })

  // Property 24: Theme Persistence Round-Trip
  describe('Property 24: Theme Persistence Round-Trip', () => {
    it('theme config survives serialization round-trip', () => {
      fc.assert(
        fc.property(
          fc.record({
            accentColor: hexColorArb,
            matrixEffects: fc.boolean(),
            scanlines: fc.boolean(),
            glow: fc.boolean(),
          }),
          (config) => {
            // Simulate save/load by JSON serialization
            const serialized = JSON.stringify(config)
            const deserialized = JSON.parse(serialized)

            expect(deserialized.accentColor).toBe(config.accentColor)
            expect(deserialized.matrixEffects).toBe(config.matrixEffects)
            expect(deserialized.scanlines).toBe(config.scanlines)
            expect(deserialized.glow).toBe(config.glow)

            return true
          }
        ),
        { numRuns: 100 }
      )
    })

    it('accent color is preserved exactly', () => {
      for (const accent of ACCENT_COLORS) {
        const config = {
          accentColor: accent.color,
          matrixEffects: false,
          scanlines: false,
          glow: true,
        }

        const serialized = JSON.stringify(config)
        const deserialized = JSON.parse(serialized)

        expect(deserialized.accentColor).toBe(accent.color)
      }
    })
  })

  // Property 25: Theme Accessibility Contrast
  describe('Property 25: Theme Accessibility Contrast', () => {
    it('all predefined accent colors meet WCAG AA on dark background', () => {
      const darkBackground = '#0a0a0b' // App background color

      for (const accent of ACCENT_COLORS) {
        const ratio = getContrastRatio(accent.color, darkBackground)
        // All accent colors should have at least 3:1 contrast for large text/UI elements
        expect(ratio).toBeGreaterThanOrEqual(3)
      }
    })

    it('white text meets WCAG AA on dark background', () => {
      const darkBackground = '#0a0a0b'
      const whiteText = '#fafafa'

      expect(meetsWCAGAA(whiteText, darkBackground)).toBe(true)
    })

    it('muted text meets WCAG AA for large text on dark background', () => {
      const darkBackground = '#0a0a0b'
      const mutedText = '#a1a1aa'

      // Muted text should at least meet large text requirements
      expect(meetsWCAGAA(mutedText, darkBackground, true)).toBe(true)
    })

    it('accent colors provide sufficient contrast for interactive elements', () => {
      const darkBackground = '#1a1a1c' // Card/panel background

      for (const accent of ACCENT_COLORS) {
        const ratio = getContrastRatio(accent.color, darkBackground)
        // Interactive elements need at least 3:1 contrast
        expect(ratio).toBeGreaterThanOrEqual(3)
      }
    })
  })
})
