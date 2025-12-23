/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 0 modules
 */

export interface Photo {
  name: string
  url: string
  sha: string
  path?: string
  size?: number
}

export interface PhotoGalleryProps {
  photos: Photo[]
  loading: boolean
  previewSize?: number
}

export interface PhotoGalleryEmits {
  refresh: []
  resize: [size: number]
}

export interface PhotoActionHandlers {
  isSelected: (id: string) => boolean
  isFavorite: (id: string) => boolean
  getPhotoColorTag: (photo: Photo) => string | undefined
  getStatus: (id: string) => any
}

export interface PhotoSelectEvent {
  photo: Photo
  additive: boolean
  range: boolean
}

export interface PhotoSyncEvent {
  action: string
  photoId: string
}

export interface PhotoContextMenuEvent {
  event: MouseEvent
  photo: Photo
}