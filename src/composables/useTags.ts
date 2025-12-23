/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed } from 'vue'

export interface Tag {
    id: string
    name: string
    color: string
}

export interface TaggedItem {
    itemId: string 
    tagId: string
}

const tags = ref<Tag[]>([])
const taggedItems = ref<TaggedItem[]>([])

export function useTags() {

    const addTag = (color: string) => {

        const existing = tags.value.find(t => t.color === color)
        if (existing) return existing

        const newTag: Tag = {
            id: crypto.randomUUID(),
            name: `Color ${tags.value.length + 1}`, 
            color
        }
        tags.value.push(newTag)
        return newTag
    }

    const removeTag = (tagId: string) => {
        tags.value = tags.value.filter(t => t.id !== tagId)
        
        taggedItems.value = taggedItems.value.filter(t => t.tagId !== tagId)
    }

    const updateTagName = (tagId: string, name: string) => {
        const tag = tags.value.find(t => t.id === tagId)
        if (tag) {
            tag.name = name
        }
    }

    const assignTag = (itemId: string, tagId: string) => {

        const existingIndex = taggedItems.value.findIndex(t => t.itemId === itemId)
        if (existingIndex !== -1) {
            
            taggedItems.value[existingIndex].tagId = tagId
        } else {
            taggedItems.value.push({ itemId, tagId })
        }
    }

    const unassignTag = (itemId: string) => {
        taggedItems.value = taggedItems.value.filter(t => t.itemId !== itemId)
    }

    const getTagForItem = (itemId: string) => {
        const assignment = taggedItems.value.find(t => t.itemId === itemId)
        if (!assignment) return null
        return tags.value.find(t => t.id === assignment.tagId)
    }

    const activeTags = computed(() => tags.value)

    return {
        tags: activeTags,
        taggedItems,
        addTag,
        removeTag,
        updateTagName,
        assignTag,
        unassignTag,
        getTagForItem
    }
}