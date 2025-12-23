/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import {
  toggleFavoriteItem,
  groupFavoritesByType,
  isItemFavorite,
  type FavoriteItem,
} from '../useFavorites'

const favoriteItemArb = fc.record({
  type: fc.constantFrom('photo' as const, 'album' as const),
  id: fc.string({ minLength: 1, maxLength: 20 }),
  path: fc.string({ minLength: 1, maxLength: 50 }),
  addedAt: fc.integer({ min: 0, max: Date.now() + 1000000 }),
})

describe('useFavorites', () => {
  describe('toggleFavoriteItem', () => {
    
    it('Property 10: toggling twice returns to original state', () => {
      fc.assert(
        fc.property(
          fc.array(favoriteItemArb, { minLength: 0, maxLength: 10 }),
          fc.record({
            type: fc.constantFrom('photo' as const, 'album' as const),
            id: fc.string({ minLength: 1, maxLength: 20 }),
            path: fc.string({ minLength: 1, maxLength: 50 }),
          }),
          (initialFavorites, itemToToggle) => {
            const wasInitiallyFavorite = isItemFavorite(itemToToggle.id, initialFavorites)

            const afterFirstToggle = toggleFavoriteItem(itemToToggle, initialFavorites)
            const isFavoriteAfterFirst = isItemFavorite(itemToToggle.id, afterFirstToggle)

            const afterSecondToggle = toggleFavoriteItem(itemToToggle, afterFirstToggle)
            const isFavoriteAfterSecond = isItemFavorite(itemToToggle.id, afterSecondToggle)

            return isFavoriteAfterFirst !== wasInitiallyFavorite && isFavoriteAfterSecond === wasInitiallyFavorite
          }
        ),
        { numRuns: 100 }
      )
    })

    it('toggling adds item when not in favorites', () => {
      const favorites: FavoriteItem[] = []
      const item = { type: 'photo' as const, id: 'photo1', path: '/photos/1.jpg' }

      const result = toggleFavoriteItem(item, favorites)

      expect(isItemFavorite('photo1', result)).toBe(true)
      expect(result.length).toBe(1)
    })

    it('toggling removes item when already in favorites', () => {
      const favorites: FavoriteItem[] = [
        { type: 'photo', id: 'photo1', path: '/photos/1.jpg', addedAt: Date.now() },
      ]
      const item = { type: 'photo' as const, id: 'photo1', path: '/photos/1.jpg' }

      const result = toggleFavoriteItem(item, favorites)

      expect(isItemFavorite('photo1', result)).toBe(false)
      expect(result.length).toBe(0)
    })
  })

  describe('groupFavoritesByType', () => {
    
    it('Property 14: grouping partitions items correctly with no missing or duplicated items', () => {
      fc.assert(
        fc.property(fc.array(favoriteItemArb, { minLength: 0, maxLength: 20 }), (favorites) => {
          const { photos, albums } = groupFavoritesByType(favorites)

          if (photos.length + albums.length !== favorites.length) return false

          if (!photos.every((p) => p.type === 'photo')) return false

          if (!albums.every((a) => a.type === 'album')) return false

          for (const item of favorites) {
            const inPhotos = photos.some((p) => p.id === item.id && p.type === item.type)
            const inAlbums = albums.some((a) => a.id === item.id && a.type === item.type)

            if (item.type === 'photo' && !inPhotos) return false
            if (item.type === 'album' && !inAlbums) return false
            if (inPhotos && inAlbums) return false 
          }

          return true
        }),
        { numRuns: 100 }
      )
    })

    it('groups empty array correctly', () => {
      const { photos, albums } = groupFavoritesByType([])
      expect(photos).toEqual([])
      expect(albums).toEqual([])
    })

    it('groups mixed items correctly', () => {
      const favorites: FavoriteItem[] = [
        { type: 'photo', id: 'p1', path: '/p1', addedAt: 1 },
        { type: 'album', id: 'a1', path: '/a1', addedAt: 2 },
        { type: 'photo', id: 'p2', path: '/p2', addedAt: 3 },
      ]

      const { photos, albums } = groupFavoritesByType(favorites)

      expect(photos.length).toBe(2)
      expect(albums.length).toBe(1)
      expect(photos.map((p) => p.id)).toContain('p1')
      expect(photos.map((p) => p.id)).toContain('p2')
      expect(albums.map((a) => a.id)).toContain('a1')
    })
  })

  describe('isItemFavorite', () => {
    
    it('Property 12: isItemFavorite returns true only for items in favorites list', () => {
      fc.assert(
        fc.property(
          fc.array(favoriteItemArb, { minLength: 0, maxLength: 10 }),
          fc.string({ minLength: 1, maxLength: 20 }),
          (favorites, testId) => {
            const result = isItemFavorite(testId, favorites)
            const actuallyInList = favorites.some((f) => f.id === testId)
            return result === actuallyInList
          }
        ),
        { numRuns: 100 }
      )
    })
  })

  describe('Favorites persistence (Property 11)', () => {
    
    it('Property 11: favorites data structure is JSON serializable (round-trip foundation)', () => {
      fc.assert(
        fc.property(fc.array(favoriteItemArb, { minLength: 0, maxLength: 20 }), (favorites) => {
          
          const serialized = JSON.stringify(favorites)
          const deserialized = JSON.parse(serialized) as FavoriteItem[]

          if (deserialized.length !== favorites.length) return false

          for (let i = 0; i < favorites.length; i++) {
            if (
              deserialized[i].id !== favorites[i].id ||
              deserialized[i].type !== favorites[i].type ||
              deserialized[i].path !== favorites[i].path ||
              deserialized[i].addedAt !== favorites[i].addedAt
            ) {
              return false
            }
          }

          return true
        }),
        { numRuns: 100 }
      )
    })
  })

  describe('Favorites type support (Property 13)', () => {
    
    it('Property 13: both photo and album types can be favorited', () => {
      fc.assert(
        fc.property(
          fc.constantFrom('photo' as const, 'album' as const),
          fc.string({ minLength: 1, maxLength: 20 }),
          fc.string({ minLength: 1, maxLength: 50 }),
          (type, id, path) => {
            const item = { type, id, path }
            const result = toggleFavoriteItem(item, [])

            return result.length === 1 && result[0].type === type && result[0].id === id
          }
        ),
        { numRuns: 100 }
      )
    })
  })
})