import { ref, computed } from 'vue'

export interface Tag {
    id: string
    name: string
    color: string
}

export interface TaggedItem {
    itemId: string // sha or path
    tagId: string
}

const tags = ref<Tag[]>([])
const taggedItems = ref<TaggedItem[]>([])

export function useTags() {

    const addTag = (color: string) => {
        // Check if a tag with this color already exists (optional: allow multiple same-color tags?)
        // For now, let's allow multiple, but typically user might want unique colors. 
        // The prompt says "painel deve aparecer ... com nomes default"
        const existing = tags.value.find(t => t.color === color)
        if (existing) return existing

        const newTag: Tag = {
            id: crypto.randomUUID(),
            name: `Color ${tags.value.length + 1}`, // Default name
            color
        }
        tags.value.push(newTag)
        return newTag
    }

    const removeTag = (tagId: string) => {
        tags.value = tags.value.filter(t => t.id !== tagId)
        // Also remove assignments
        taggedItems.value = taggedItems.value.filter(t => t.tagId !== tagId)
    }

    const updateTagName = (tagId: string, name: string) => {
        const tag = tags.value.find(t => t.id === tagId)
        if (tag) {
            tag.name = name
        }
    }

    const assignTag = (itemId: string, tagId: string) => {
        // Remove existing tag for this item if we only want one tag per item?
        // Prompt implies "modifica a cor da borda", usually implies single primary color.
        // Let's assume one tag per item for the border color effect.
        const existingIndex = taggedItems.value.findIndex(t => t.itemId === itemId)
        if (existingIndex !== -1) {
            // Update existing
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
