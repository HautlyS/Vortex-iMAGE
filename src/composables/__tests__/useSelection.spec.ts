/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 2 modules
 */

import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { selectAdditive, selectRange, useSelection } from '../useSelection'

describe('useSelection', () => {
  describe('selectAdditive', () => {
    
    it('Property 19: additive selection adds new item to existing selection', () => {
      fc.assert(
        fc.property(
          fc.array(fc.string({ minLength: 1, maxLength: 10 }), { minLength: 0, maxLength: 20 }),
          fc.string({ minLength: 1, maxLength: 10 }),
          (existingIds, newId) => {
            const currentSelection = new Set(existingIds)
            const wasSelected = currentSelection.has(newId)
            const result = selectAdditive(newId, currentSelection)

            if (wasSelected) {
              
              return !result.has(newId)
            } else {
              
              return result.has(newId)
            }
          }
        ),
        { numRuns: 100 }
      )
    })

    it('Property 19: additive selection preserves other selected items', () => {
      fc.assert(
        fc.property(
          fc.array(fc.string({ minLength: 1, maxLength: 10 }), { minLength: 1, maxLength: 20 }),
          fc.string({ minLength: 1, maxLength: 10 }),
          (existingIds, newId) => {
            const currentSelection = new Set(existingIds)
            const result = selectAdditive(newId, currentSelection)

            for (const id of existingIds) {
              if (id !== newId) {
                if (!result.has(id)) return false
              }
            }
            return true
          }
        ),
        { numRuns: 100 }
      )
    })

    it('additive selection on empty set adds the item', () => {
      const result = selectAdditive('item1', new Set())
      expect(result.has('item1')).toBe(true)
      expect(result.size).toBe(1)
    })

    it('additive selection toggles existing item', () => {
      const initial = new Set(['item1', 'item2'])
      const result = selectAdditive('item1', initial)
      expect(result.has('item1')).toBe(false)
      expect(result.has('item2')).toBe(true)
    })
  })

  describe('selectRange', () => {
    
    it('Property 20: range selection includes all items between anchor and target', () => {
      fc.assert(
        fc.property(
          fc.array(fc.string({ minLength: 1, maxLength: 10 }), { minLength: 2, maxLength: 20 }),
          (allIds) => {
            
            const uniqueIds = [...new Set(allIds)]
            if (uniqueIds.length < 2) return true 

            const anchorIndex = Math.floor(Math.random() * uniqueIds.length)
            let targetIndex = Math.floor(Math.random() * uniqueIds.length)
            if (targetIndex === anchorIndex) {
              targetIndex = (targetIndex + 1) % uniqueIds.length
            }

            const anchorId = uniqueIds[anchorIndex]
            const targetId = uniqueIds[targetIndex]

            const result = selectRange(anchorId, targetId, uniqueIds, new Set())

            const start = Math.min(anchorIndex, targetIndex)
            const end = Math.max(anchorIndex, targetIndex)

            for (let i = start; i <= end; i++) {
              if (!result.has(uniqueIds[i])) return false
            }
            return true
          }
        ),
        { numRuns: 100 }
      )
    })

    it('Property 20: range selection works regardless of direction', () => {
      const allIds = ['a', 'b', 'c', 'd', 'e']

      const forward = selectRange('b', 'd', allIds, new Set())
      expect(forward.has('b')).toBe(true)
      expect(forward.has('c')).toBe(true)
      expect(forward.has('d')).toBe(true)

      const backward = selectRange('d', 'b', allIds, new Set())
      expect(backward.has('b')).toBe(true)
      expect(backward.has('c')).toBe(true)
      expect(backward.has('d')).toBe(true)
    })

    it('range selection with no anchor selects only target', () => {
      const result = selectRange(null, 'item2', ['item1', 'item2', 'item3'], new Set())
      expect(result.size).toBe(1)
      expect(result.has('item2')).toBe(true)
    })

    it('range selection preserves existing selection and adds range', () => {
      const existing = new Set(['x', 'y'])
      const result = selectRange('b', 'd', ['a', 'b', 'c', 'd', 'e'], existing)
      expect(result.has('x')).toBe(true)
      expect(result.has('y')).toBe(true)
      expect(result.has('b')).toBe(true)
      expect(result.has('c')).toBe(true)
      expect(result.has('d')).toBe(true)
    })
  })

  describe('useSelection composable', () => {
    it('select without options replaces selection', () => {
      const { select, isSelected, clearSelection } = useSelection()
      clearSelection()

      select('item1')
      expect(isSelected('item1')).toBe(true)

      select('item2')
      expect(isSelected('item1')).toBe(false)
      expect(isSelected('item2')).toBe(true)
    })

    it('clearSelection removes all selections', () => {
      const { select, clearSelection, selectedCount } = useSelection()
      select('item1')
      select('item2', { additive: true })
      expect(selectedCount.value).toBe(2)

      clearSelection()
      expect(selectedCount.value).toBe(0)
    })

    it('getSelected returns array of selected ids', () => {
      const { select, getSelected, clearSelection } = useSelection()
      clearSelection()

      select('item1')
      select('item2', { additive: true })
      const selected = getSelected()
      expect(selected).toContain('item1')
      expect(selected).toContain('item2')
      expect(selected.length).toBe(2)
    })
  })
})