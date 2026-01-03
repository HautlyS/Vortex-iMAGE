import { ref } from 'vue'
import { Photo } from '@/types/photo'
import { useColorTagStore } from '@/composables/useColorTags'

export interface SmartAlbum {
    id: string
    title: string
    type: 'date' | 'color' | 'tag'
    coverPhoto?: Photo
    photoIds: string[]
    count: number
}

export function useSmartAlbums() {
    const { photoTags, COLORS } = useColorTagStore()

    const albums = ref<SmartAlbum[]>([])

    const generateAlbums = (photos: Photo[]) => {
        const newAlbums: SmartAlbum[] = []

        // 1. Color Albums
        const colorGroups: Record<string, string[]> = {}

        // Initialize groups for all colors
        COLORS.forEach(c => {
            colorGroups[c.id] = []
        })

        // Group photos by color
        photos.forEach(photo => {
            // Assuming photo.url or photo.name is the ID used in useColorTags. 
            // Looking at useColorTags, it maps ID -> ColorID.
            // Photo interface doesn't have explicit ID, but usage implies name or url acts as one.
            // Let's assume unique identifier is name for now (or whatever ID useColorTags uses).
            // If photo.sha exists, that might be ID. Let's use name as default key.
            const colorId = photoTags.value[photo.name]
            if (colorId && colorGroups[colorId]) {
                colorGroups[colorId].push(photo.name)
            }
        })

        // Create albums from color groups
        Object.entries(colorGroups).forEach(([colorId, photoIds]) => {
            if (photoIds.length > 0) {
                const color = COLORS.find(c => c.id === colorId)
                if (color) {
                    newAlbums.push({
                        id: `color-${colorId}`,
                        title: `${color.name}`,
                        type: 'color',
                        photoIds,
                        count: photoIds.length,
                        coverPhoto: photos.find(p => p.name === photoIds[0])
                    })
                }
            }
        })

        // 2. Date Albums (Month/Year)
        const dateGroups: Record<string, string[]> = {}

        photos.forEach(photo => {
            const date = parseDateFromFilename(photo.name)
            const key = date ? `${date.toLocaleString('en-US', { month: 'long' })} ${date.getFullYear()}` : 'Undated'

            if (!dateGroups[key]) {
                dateGroups[key] = []
            }
            dateGroups[key].push(photo.name)
        })

        Object.entries(dateGroups).forEach(([dateKey, photoIds]) => {
            if (dateKey !== 'Undated' && photoIds.length > 0) {
                newAlbums.push({
                    id: `date-${dateKey.replace(/\s+/g, '-').toLowerCase()}`,
                    title: dateKey,
                    type: 'date',
                    photoIds,
                    count: photoIds.length,
                    coverPhoto: photos.find(p => p.name === photoIds[0])
                })
            }
        })

        albums.value = newAlbums
    }

    // Helper to parse date from common filename patterns (IMG_20230101_...)
    const parseDateFromFilename = (filename: string): Date | null => {
        // Matches YYYYMMDD or YYYY-MM-DD
        const matches = filename.match(/(20\d{2})[-_]?(\d{2})[-_]?(\d{2})/)
        if (matches) {
            const year = parseInt(matches[1])
            const month = parseInt(matches[2]) - 1
            const day = parseInt(matches[3])
            // Basic validation
            if (month >= 0 && month <= 11 && day >= 1 && day <= 31) {
                return new Date(year, month, day)
            }
        }
        return null
    }

    return {
        albums,
        generateAlbums
    }
}
