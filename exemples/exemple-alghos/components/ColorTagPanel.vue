<script setup lang="ts">
import { ref, computed } from 'vue'
import { useColorTags, type ColorTagDefinition } from '../composables/useColorTags'

defineProps<{
  selectedTagId: string | null
}>()

const emit = defineEmits<{
  (e: 'select', tagId: string | null): void
}>()

const { usedTags, tagCounts, renameTag } = useColorTags()

const editingTagId = ref<string | null>(null)
const editingName = ref('')

const displayTags = computed(() => {
  return usedTags.value.map((tag) => ({
    ...tag,
    count: tagCounts.value[tag.id] || 0,
  }))
})

function handleSelect(tagId: string | null) {
  emit('select', tagId)
}

function startEditing(tag: ColorTagDefinition) {
  editingTagId.value = tag.id
  editingName.value = tag.name
}

function saveEdit() {
  if (editingTagId.value && editingName.value.trim()) {
    renameTag(editingTagId.value, editingName.value.trim())
  }
  editingTagId.value = null
  editingName.value = ''
}

function cancelEdit() {
  editingTagId.value = null
  editingName.value = ''
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    saveEdit()
  } else if (e.key === 'Escape') {
    cancelEdit()
  }
}
</script>

<template>
  <div class="color-tag-panel">
    <div class="panel-header">
      <h3>Etiquetas</h3>
    </div>

    <div v-if="displayTags.length === 0" class="empty-state">
      <p>Nenhuma etiqueta em uso</p>
      <span>Clique com botão direito em fotos para adicionar etiquetas</span>
    </div>

    <div v-else class="tag-list">
      <!-- All Tags (clear filter) -->
      <button
        class="tag-item"
        :class="{ active: selectedTagId === null }"
        @click="handleSelect(null)"
      >
        <span class="tag-color all" />
        <span class="tag-name">Todas</span>
      </button>

      <!-- Individual Tags -->
      <div v-for="tag in displayTags" :key="tag.id" class="tag-item-wrapper">
        <button
          v-if="editingTagId !== tag.id"
          class="tag-item"
          :class="{ active: selectedTagId === tag.id }"
          @click="handleSelect(tag.id)"
          @dblclick="startEditing(tag)"
        >
          <span class="tag-color" :style="{ backgroundColor: tag.color }" />
          <span class="tag-name">{{ tag.name }}</span>
          <span class="tag-count">{{ tag.count }}</span>
          <button class="edit-btn" @click.stop="startEditing(tag)" title="Renomear">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
            </svg>
          </button>
        </button>

        <!-- Edit Mode -->
        <div v-else class="tag-edit">
          <span class="tag-color" :style="{ backgroundColor: tag.color }" />
          <input
            v-model="editingName"
            type="text"
            class="edit-input"
            @keydown="handleKeydown"
            @blur="saveEdit"
            autofocus
          />
          <button class="save-btn" @click="saveEdit">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.color-tag-panel {
  padding: 0.5rem;
}

.panel-header {
  padding: 0.5rem 0.75rem;
  margin-bottom: 0.25rem;
}

.panel-header h3 {
  font-size: 0.75rem;
  font-weight: 600;
  color: #52525b;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.empty-state {
  padding: 1rem;
  text-align: center;
}

.empty-state p {
  font-size: 0.875rem;
  color: #71717a;
  margin-bottom: 0.25rem;
}

.empty-state span {
  font-size: 0.75rem;
  color: #52525b;
}

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.tag-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.5rem 0.75rem;
  background: transparent;
  border: none;
  color: #a1a1aa;
  font-size: 0.875rem;
  border-radius: 0.375rem;
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
}

.tag-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fafafa;
}

.tag-item.active {
  background: rgba(99, 102, 241, 0.15);
  color: #818cf8;
}

.tag-color {
  width: 0.875rem;
  height: 0.875rem;
  border-radius: 50%;
  flex-shrink: 0;
  border: 2px solid rgba(255, 255, 255, 0.2);
}

.tag-color.all {
  background: linear-gradient(135deg, #ef4444, #f97316, #eab308, #22c55e, #3b82f6, #a855f7);
}

.tag-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tag-count {
  font-size: 0.75rem;
  color: #52525b;
  background: rgba(255, 255, 255, 0.05);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  flex-shrink: 0;
}

.edit-btn {
  width: 1.25rem;
  height: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #52525b;
  cursor: pointer;
  border-radius: 0.25rem;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.tag-item:hover .edit-btn {
  opacity: 1;
}

.edit-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fafafa;
}

.edit-btn svg {
  width: 0.75rem;
  height: 0.75rem;
}

/* Edit Mode */
.tag-edit {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.5rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 0.375rem;
}

.edit-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #fafafa;
  font-size: 0.875rem;
  outline: none;
  min-width: 0;
}

.save-btn {
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #22c55e;
  border: none;
  border-radius: 0.25rem;
  color: white;
  cursor: pointer;
  flex-shrink: 0;
}

.save-btn svg {
  width: 0.875rem;
  height: 0.875rem;
}
</style>
