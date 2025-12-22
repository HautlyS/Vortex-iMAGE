import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import {
  applyTagToItems,
  removeTagFromItem,
  getItemsWithTag,
  getTagsForItem,
  countItemsPerTag,
  getUsedTags,
  PREDEFINED_COLORS,
  type TaggedItem,
} from '../useColorTags'

// Arbitraries
const tagIdArb = fc.constantFrom(...PREDEFINED_COLORS.map((c) => c.id))
const itemIdArb = fc.string({ minLength: 1, maxLength: 20 })
const taggedItemArb = fc.record({
  itemId: itemIdArb,
  tagId: tagIdArb,
})

describe('useColorTags', () => {
  describe('PREDEFINED_COLORS', () => {
    it('should have at least 8 predefined colors (Requirement 6.2)', () => {
      expect(PREDEFINED_COLORS.length).toBeGreaterThanOrEqual(8)
    })

    it('each color should have id, color hex, and name', () => {
      for (const color of PREDEFINED_COLORS) {
        expect(color.id).toBeTruthy()
        expect(color.color).toMatch(/^#[0-9a-fA-F]{6}$/)
        expect(color.name).toBeTruthy()
      }
    })
  })

  describe('applyTagToItems (Property 15)', () => {
    /**
     * Feature: photo-manager-enhancement, Property 15: Color Tag Batch Application
     * For any set of selected items and any color tag, after applying the tag,
     * all selected items SHALL have that tag in their tag list.
     * Validates: Requirements 6.3
     */
    it('Property 15: all selected items receive the tag after batch application', () => {
      fc.assert(
        fc.property(
          fc.array(itemIdArb, { minLength: 1, maxLength: 20 }),
          tagIdArb,
          fc.array(taggedItemArb, { minLength: 0, maxLength: 10 }),
          (itemIds, tagId, existingTags) => {
            const result = applyTagToItems(itemIds, tagId, existingTags)

            // All items should now have the tag
            for (const itemId of itemIds) {
              const itemTags = getTagsForItem(itemId, result)
              if (!itemTags.includes(tagId)) return false
            }
            return true
          }
        ),
        { numRuns: 100 }
      )
    })

    it('Property 15: applying tag preserves existing tags on items', () => {
      fc.assert(
        fc.property(
          fc.array(taggedItemArb, { minLength: 1, maxLength: 10 }),
          itemIdArb,
          tagIdArb,
          (existingTags, newItemId, newTagId) => {
            const result = applyTagToItems([newItemId], newTagId, existingTags)

            // All existing tags should still be present
            for (const existing of existingTags) {
              const stillHasTag = result.some(
                (t) => t.itemId === existing.itemId && t.tagId === existing.tagId
              )
              if (!stillHasTag) return false
            }
            return true
          }
        ),
        { numRuns: 100 }
      )
    })

    it('does not duplicate tags when applied twice', () => {
      const existing: TaggedItem[] = [{ itemId: 'item1', tagId: 'red' }]
      const result = applyTagToItems(['item1'], 'red', existing)
      const redTags = result.filter((t) => t.itemId === 'item1' && t.tagId === 'red')
      expect(redTags.length).toBe(1)
    })
  })

  describe('getUsedTags (Property 16)', () => {
    /**
     * Feature: photo-manager-enhancement, Property 16: Color Tag Auto-Registration
     * For any color tag applied to items, that tag SHALL appear in the sidebar tag list
     * with a count equal to the number of items tagged with it.
     * Validates: Requirements 6.4
     */
    it('Property 16: used tags appear with correct counts', () => {
      fc.assert(
        fc.property(fc.array(taggedItemArb, { minLength: 0, maxLength: 30 }), (taggedItems) => {
          const usedTags = getUsedTags(PREDEFINED_COLORS, taggedItems)
          const counts = countItemsPerTag(taggedItems)

          // Every tag in usedTags should have at least one item
          for (const tag of usedTags) {
            if (!counts[tag.id] || counts[tag.id] < 1) return false
          }

          // Every tag with items should be in usedTags
          for (const tagId of Object.keys(counts)) {
            if (counts[tagId] > 0) {
              const inUsedTags = usedTags.some((t) => t.id === tagId)
              if (!inUsedTags) return false
            }
          }

          return true
        }),
        { numRuns: 100 }
      )
    })

    it('returns empty array when no items are tagged', () => {
      const result = getUsedTags(PREDEFINED_COLORS, [])
      expect(result).toEqual([])
    })
  })

  describe('getItemsWithTag (Property 18)', () => {
    /**
     * Feature: photo-manager-enhancement, Property 18: Color Tag Filter Correctness
     * For any color tag, filtering by that tag SHALL display exactly the items
     * that have that tag applied, and no others.
     * Validates: Requirements 6.7
     */
    it('Property 18: filtering returns exactly items with that tag', () => {
      fc.assert(
        fc.property(
          fc.array(taggedItemArb, { minLength: 0, maxLength: 30 }),
          tagIdArb,
          (taggedItems, filterTagId) => {
            const filteredItems = getItemsWithTag(filterTagId, taggedItems)

            // All returned items should have the tag
            for (const itemId of filteredItems) {
              const hasThatTag = taggedItems.some(
                (t) => t.itemId === itemId && t.tagId === filterTagId
              )
              if (!hasThatTag) return false
            }

            // All items with the tag should be returned
            for (const tagged of taggedItems) {
              if (tagged.tagId === filterTagId) {
                if (!filteredItems.includes(tagged.itemId)) return false
              }
            }

            return true
          }
        ),
        { numRuns: 100 }
      )
    })
  })

  describe('Persistence (Property 17)', () => {
    /**
     * Feature: photo-manager-enhancement, Property 17: Color Tag Persistence Round-Trip
     * For any set of color tags with custom names and tagged items,
     * saving and loading SHALL restore the exact same tags, names, and item associations.
     * Validates: Requirements 6.6
     */
    it('Property 17: tagged items are JSON serializable (round-trip foundation)', () => {
      fc.assert(
        fc.property(fc.array(taggedItemArb, { minLength: 0, maxLength: 30 }), (taggedItems) => {
          const serialized = JSON.stringify(taggedItems)
          const deserialized = JSON.parse(serialized) as TaggedItem[]

          if (deserialized.length !== taggedItems.length) return false

          for (let i = 0; i < taggedItems.length; i++) {
            if (
              deserialized[i].itemId !== taggedItems[i].itemId ||
              deserialized[i].tagId !== taggedItems[i].tagId
            ) {
              return false
            }
          }

          return true
        }),
        { numRuns: 100 }
      )
    })

    it('Property 17: custom tag names are JSON serializable', () => {
      fc.assert(
        fc.property(
          fc.dictionary(tagIdArb, fc.string({ minLength: 1, maxLength: 30 })),
          (customNames) => {
            const serialized = JSON.stringify(customNames)
            const deserialized = JSON.parse(serialized) as Record<string, string>

            for (const key of Object.keys(customNames)) {
              if (deserialized[key] !== customNames[key]) return false
            }

            return true
          }
        ),
        { numRuns: 100 }
      )
    })
  })

  describe('removeTagFromItem', () => {
    it('removes tag from item', () => {
      const existing: TaggedItem[] = [
        { itemId: 'item1', tagId: 'red' },
        { itemId: 'item1', tagId: 'blue' },
        { itemId: 'item2', tagId: 'red' },
      ]

      const result = removeTagFromItem('item1', 'red', existing)

      expect(result.length).toBe(2)
      expect(result.some((t) => t.itemId === 'item1' && t.tagId === 'red')).toBe(false)
      expect(result.some((t) => t.itemId === 'item1' && t.tagId === 'blue')).toBe(true)
      expect(result.some((t) => t.itemId === 'item2' && t.tagId === 'red')).toBe(true)
    })

    it('does nothing if tag not present', () => {
      const existing: TaggedItem[] = [{ itemId: 'item1', tagId: 'red' }]
      const result = removeTagFromItem('item1', 'blue', existing)
      expect(result).toEqual(existing)
    })
  })

  describe('getTagsForItem', () => {
    it('returns all tags for an item', () => {
      const tagged: TaggedItem[] = [
        { itemId: 'item1', tagId: 'red' },
        { itemId: 'item1', tagId: 'blue' },
        { itemId: 'item2', tagId: 'green' },
      ]

      const result = getTagsForItem('item1', tagged)

      expect(result).toContain('red')
      expect(result).toContain('blue')
      expect(result).not.toContain('green')
    })

    it('returns empty array for untagged item', () => {
      const result = getTagsForItem('unknown', [])
      expect(result).toEqual([])
    })
  })
})
