/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 2 modules
 */

import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { clampSize } from '../usePhotoPreviewSize'

const MIN_SIZE = 80
const MAX_SIZE = 400

describe('usePhotoPreviewSize', () => {
  describe('clampSize', () => {
    
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
      
      expect(clampSize(NaN)).toBeGreaterThanOrEqual(MIN_SIZE)
      expect(clampSize(NaN)).toBeLessThanOrEqual(MAX_SIZE)
      
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
  
  it('Property 8: clampSize is idempotent for valid sizes (round-trip foundation)', () => {
    fc.assert(
      fc.property(fc.integer({ min: MIN_SIZE, max: MAX_SIZE }), (validSize) => {
        
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
        
        return firstClamp === secondClamp && secondClamp === thirdClamp
      }),
      { numRuns: 100 }
    )
  })
})