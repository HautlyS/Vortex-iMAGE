import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useSmartAlbums } from '../useSmartAlbums'
import { useColorTagStore } from '../useColorTags'

// Mock useColorTagStore
const mockPhotoTags = { value: {} as Record<string, string> }
vi.mock('../useColorTags', () => ({
    useColorTagStore: () => ({
        photoTags: mockPhotoTags,
        COLORS: [
            { id: 'red', name: 'Red', color: '#ff0000' },
            { id: 'blue', name: 'Blue', color: '#0000ff' }
        ]
    })
}))

describe('useSmartAlbums', () => {
    beforeEach(() => {
        mockPhotoTags.value = {}
    })

    it('should generate color albums correctly', () => {
        const photos = [
            { name: 'photo1.jpg', url: 'url1', sha: 'sha1' },
            { name: 'photo2.jpg', url: 'url2', sha: 'sha2' }
        ]

        // Assign tags
        mockPhotoTags.value = {
            'photo1.jpg': 'red',
            'photo2.jpg': 'blue'
        }

        const { albums, generateAlbums } = useSmartAlbums()
        generateAlbums(photos)

        expect(albums.value).toHaveLength(2)

        const redAlbum = albums.value.find(a => a.id === 'color-red')
        expect(redAlbum).toBeDefined()
        expect(redAlbum?.count).toBe(1)
        expect(redAlbum?.photoIds).toContain('photo1.jpg')
    })

    it('should generate date albums from filenames', () => {
        const photos = [
            { name: 'IMG_20230115_123456.jpg', url: 'url1', sha: 'sha1' }, // Jan 2023
            { name: 'IMG_20230120_654321.jpg', url: 'url2', sha: 'sha2' }, // Jan 2023
            { name: 'IMG_20230210_111111.jpg', url: 'url3', sha: 'sha3' }  // Feb 2023
        ]

        const { albums, generateAlbums } = useSmartAlbums()
        generateAlbums(photos)

        const janAlbum = albums.value.find(a => a.title === 'January 2023')
        const febAlbum = albums.value.find(a => a.title === 'February 2023')

        expect(janAlbum).toBeDefined()
        expect(janAlbum?.count).toBe(2)
        expect(febAlbum).toBeDefined()
        expect(febAlbum?.count).toBe(1)
    })

    it('should handle photos without dates or tags', () => {
        const photos = [
            { name: 'random_name.jpg', url: 'url1', sha: 'sha1' }
        ]
        const { albums, generateAlbums } = useSmartAlbums()
        generateAlbums(photos)

        // Should create no albums if it doesn't match date pattern and has no tag
        expect(albums.value).toHaveLength(0)
    })
})
