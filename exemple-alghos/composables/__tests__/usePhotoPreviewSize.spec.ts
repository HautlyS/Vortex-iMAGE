import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { clampSize } from '../usePhotoPreviewSize'

const MIN_SIZE = 80
const MAX_SIZE = 400

describe('usePhotoPreviewSize', () => {
  describe('clampSize', () => {
    /**
     * Feature: photo-manager-enhancement, Property 7: Preview Size Bounds Enforcement
     * For any attempted preview size value, the resulting size SHALL be clamped to [80, 400] pixels.
     * Validates: Requirements 4.4
     */
    it('Property 7: should clamp any input size to [80, 400] range', () => {
      fc.assert(
        fc.property(fc.integer({ min: -10000, max: 10000 }), (attemptedSize) => {
          const result = clampSize(attemptedSize)
          return result >= MIN_SIZE && result <= MAX_SIZE
        }),
        { numRuns: 100 }
      )
    })

    it('Property 7: should handle extreme values', () => {
      fc.assert(
        fc.property(fc.double({ min: -1e10, max: 1e10, noNaN: true }), (attemptedSize) => {
          const result = clampSize(attemptedSize)
          return result >= MIN_SIZE && result <= MAX_SIZE
        }),
        { numRuns: 100 }
      )
    })

    it('Property 7: should handle NaN and Infinity', () => {
      // NaN returns default (which is within bounds)
      expect(clampSize(NaN)).toBeGreaterThanOrEqual(MIN_SIZE)
      expect(clampSize(NaN)).toBeLessThanOrEqual(MAX_SIZE)
      // Infinity is not finite, so returns default
      expect(clampSize(Infinity)).toBeGreaterThanOrEqual(MIN_SIZE)
      expect(clampSize(Infinity)).toBeLessThanOrEqual(MAX_SIZE)
      expect(clampSize(-Infinity)).toBeGreaterThanOrEqual(MIN_SIZE)
      expect(clampSize(-Infinity)).toBeLessThanOrEqual(MAX_SIZE)
    })

    it('should preserve values within valid range', () => {
      fc.assert(
        fc.property(fc.integer({ min: MIN_SIZE, max: MAX_SIZE }), (validSize) => {
          return clampSize(validSize) === validSize
        }),
        { numRuns: 100 }
      )
    })

    it('should clamp values below minimum to MIN_SIZE', () => {
      fc.assert(
        fc.property(fc.integer({ min: -10000, max: MIN_SIZE - 1 }), (smallSize) => {
          return clampSize(smallSize) === MIN_SIZE
        }),
        { numRuns: 100 }
      )
    })

    it('should clamp values above maximum to MAX_SIZE', () => {
      fc.assert(
        fc.property(fc.integer({ min: MAX_SIZE + 1, max: 10000 }), (largeSize) => {
          return clampSize(largeSize) === MAX_SIZE
        }),
        { numRuns: 100 }
      )
    })
  })
})


describe('usePhotoPreviewSize persistence', () => {
  /**
   * Feature: photo-manager-enhancement, Property 8: Preview Size Persistence Round-Trip
   * For any valid preview size set by the user, saving and then loading settings 
   * SHALL restore the exact same size value.
   * Validates: Requirements 4.3
   * 
   * Note: This test validates the round-trip property at the logic level.
   * Full integration testing with Tauri store requires the Tauri runtime.
   */
  it('Property 8: clampSize is idempotent for valid sizes (round-trip foundation)', () => {
    fc.assert(
      fc.property(fc.integer({ min: MIN_SIZE, max: MAX_SIZE }), (validSize) => {
        // Simulating save (clamp) then load (clamp again) should return same value
        const saved = clampSize(validSize)
        const loaded = clampSize(saved)
        return loaded === saved && loaded === validSize
      }),
      { numRuns: 100 }
    )
  })

  it('Property 8: any clamped value remains stable through multiple clamp operations', () => {
    fc.assert(
      fc.property(fc.integer({ min: -10000, max: 10000 }), (anySize) => {
        const firstClamp = clampSize(anySize)
        const secondClamp = clampSize(firstClamp)
        const thirdClamp = clampSize(secondClamp)
        // Once clamped, value should be stable
        return firstClamp === secondClamp && secondClamp === thirdClamp
      }),
      { numRuns: 100 }
    )
  })
})
