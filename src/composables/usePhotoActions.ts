/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 3 modules
 */

import { useSelection } from './useSelection'
import { useFavorites } from './useFavorites'
import { useColorTagStore } from './useColorTags'
import { useSyncStatus } from './useSyncStatus'
import type { Photo } from '../types/photo'

export function usePhotoActions() {
  const { select, isSelected } = useSelection()
  const { isFavorite, toggleFavorite } = useFavorites()
  const { getPhotoTag } = useColorTagStore()
  const { getStatus, uploadPhoto, downloadPhoto, removeLocalCopy, deleteFromRemote } = useSyncStatus()

  function getPhotoColorTag(photo: Photo): string | undefined {
    const tag = getPhotoTag(photo.sha)
    return tag?.color
  }

  function handleSelect(photoId: string, options: { additive: boolean; range: boolean }, allIds: string[]) {
    select(photoId, options, allIds)
  }

  function handleFavorite(photo: Photo) {
    toggleFavorite({ type: 'photo', id: photo.sha, path: photo.name })
  }

  function handleSyncAction(action: string, photoId: string) {
    switch (action) {
      case 'upload': uploadPhoto(photoId); break
      case 'download': downloadPhoto(photoId); break
      case 'remove-local': removeLocalCopy(photoId); break
      case 'delete-remote': deleteFromRemote(photoId); break
    }
  }

  function copyUrl(url: string) {
    navigator.clipboard.writeText(url)
  }

  return {
    isSelected,
    isFavorite,
    getStatus,
    getPhotoColorTag,
    handleSelect,
    handleFavorite,
    handleSyncAction,
    copyUrl
  }
}