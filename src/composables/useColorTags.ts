import { ref, computed } from 'vue'
import { load } from '@tauri-apps/plugin-store'

export interface ColorTagDefinition {
  id: string
  color: string
  name: string
}

export interface TaggedItem {
  itemId: string
  tagId: string
}

// Predefined colors (8+ as per requirements)
export const PREDEFINED_COLORS: ColorTagDefinition[] = [
  { id: 'red', color: '#ef4444', name: 'Vermelho' },
  { id: 'orange', color: '#f97316', name: 'Laranja' },
  { id: 'yellow', color: '#eab308', name: 'Amarelo' },
  { id: 'green', color: '#22c55e', name: 'Verde' },
  { id: 'blue', color: '#3b82f6', name: 'Azul' },
  { id: 'purple', color: '#a855f7', name: 'Roxo' },
  { id: 'pink', color: '#ec4899', name: 'Rosa' },
  { id: 'gray', color: '#6b7280', name: 'Cinza' },
  { id: 'cyan', color: '#06b6d4', name: 'Ciano' },
  { id: 'lime', color: '#84cc16', name: 'Lima' },
]

const tags = ref<ColorTagDefinition[]>([...PREDEFINED_COLORS])
const taggedItems = ref<TaggedItem[]>([])
const customTagNames = ref<Record<string, string>>({})
let initialized = false

/**
 * Applies a tag to multiple items
 */
export function applyTagToItems(
  itemIds: string[],
  tagId: string,
  currentTaggedItems: TaggedItem[]
): TaggedItem[] {
  const result = [...currentTaggedItems]

  for (const itemId of itemIds) {
    // Check if item already has this tag
    const existingIndex = result.findIndex((t) => t.itemId === itemId && t.tagId === tagId)
    if (existingIndex === -1) {
      result.push({ itemId, tagId })
    }
  }

  return result
}

/**
 * Removes a tag from an item
 */
export function removeTagFromItem(
  itemId: string,
  tagId: string,
  currentTaggedItems: TaggedItem[]
): TaggedItem[] {
  return currentTaggedItems.filter((t) => !(t.itemId === itemId && t.tagId === tagId))
}

/**
 * Gets all items with a specific tag
 */
export function getItemsWithTag(tagId: string, taggedItemsList: TaggedItem[]): string[] {
  return taggedItemsList.filter((t) => t.tagId === tagId).map((t) => t.itemId)
}

/**
 * Gets all tags for a specific item
 */
export function getTagsForItem(itemId: string, taggedItemsList: TaggedItem[]): string[] {
  return taggedItemsList.filter((t) => t.itemId === itemId).map((t) => t.tagId)
}

/**
 * Counts items per tag
 */
export function countItemsPerTag(taggedItemsList: TaggedItem[]): Record<string, number> {
  const counts: Record<string, number> = {}
  for (const item of taggedItemsList) {
    counts[item.tagId] = (counts[item.tagId] || 0) + 1
  }
  return counts
}

/**
 * Gets tags that are actually in use (have items)
 */
export function getUsedTags(
  allTags: ColorTagDefinition[],
  taggedItemsList: TaggedItem[]
): ColorTagDefinition[] {
  const usedTagIds = new Set(taggedItemsList.map((t) => t.tagId))
  return allTags.filter((tag) => usedTagIds.has(tag.id))
}

export function useColorTags() {
  const usedTags = computed(() => getUsedTags(tags.value, taggedItems.value))
  const tagCounts = computed(() => countItemsPerTag(taggedItems.value))

  async function loadTags(): Promise<void> {
    if (initialized) return
    try {
      const store = await load('settings.json')
      const savedTaggedItems = await store.get<TaggedItem[]>('taggedItems')
      const savedCustomNames = await store.get<Record<string, string>>('customTagNames')

      if (savedTaggedItems && Array.isArray(savedTaggedItems)) {
        taggedItems.value = savedTaggedItems
      }
      if (savedCustomNames && typeof savedCustomNames === 'object') {
        customTagNames.value = savedCustomNames
        // Apply custom names to tags
        for (const tag of tags.value) {
          if (savedCustomNames[tag.id]) {
            tag.name = savedCustomNames[tag.id]
          }
        }
      }
      initialized = true
    } catch {
      // Use defaults on error
    }
  }

  async function saveTags(): Promise<void> {
    try {
      const store = await load('settings.json')
      await store.set('taggedItems', taggedItems.value)
      await store.set('customTagNames', customTagNames.value)
      await store.save()
    } catch {
      // Silent fail
    }
  }

  function tagItems(itemIds: string[], tagId: string): void {
    taggedItems.value = applyTagToItems(itemIds, tagId, taggedItems.value)
    saveTags()
  }

  function removeTag(itemId: string, tagId: string): void {
    taggedItems.value = removeTagFromItem(itemId, tagId, taggedItems.value)
    saveTags()
  }

  function getItemTags(itemId: string): ColorTagDefinition[] {
    const tagIds = getTagsForItem(itemId, taggedItems.value)
    return tags.value.filter((t) => tagIds.includes(t.id))
  }

  function getItemsByTag(tagId: string): string[] {
    return getItemsWithTag(tagId, taggedItems.value)
  }

  function renameTag(tagId: string, newName: string): void {
    const tag = tags.value.find((t) => t.id === tagId)
    if (tag) {
      tag.name = newName
      customTagNames.value[tagId] = newName
      saveTags()
    }
  }

  function getTagById(tagId: string): ColorTagDefinition | undefined {
    return tags.value.find((t) => t.id === tagId)
  }

  function clearAllTags(): void {
    taggedItems.value = []
    saveTags()
  }

  return {
    tags,
    taggedItems,
    usedTags,
    tagCounts,
    tagItems,
    removeTag,
    getItemTags,
    getItemsByTag,
    renameTag,
    getTagById,
    loadTags,
    saveTags,
    clearAllTags,
    PREDEFINED_COLORS,
  }
}
