/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useColorTags, type ColorTagDefinition } from '../composables/useColorTags'

defineProps<{
  selectedTagId: string | null
}>()

const emit = defineEmits<{
  (e: 'select', tagId: string | null): void
}>()

const { usedTags, tagCounts } = useColorTags()

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
        <div
          v-if="editingTagId !== tag.id"
          class="tag-item"
          :class="{ active: selectedTagId === tag.id }"
          :style="selectedTagId === tag.id ? { 
            '--tag-accent': tag.color,
            backgroundColor: `${tag.color}15`,
            borderColor: tag.color
          } : {}"
          @click="handleSelect(tag.id)"
          @dblclick="startEditing(tag)"
          role="button"
          tabindex="0"
        >
          <span class="tag-color" :style="{ backgroundColor: tag.color, boxShadow: `0 0 8px ${tag.color}40` }" />
          <span class="tag-name">{{ tag.name }}</span>
          <span class="tag-count">{{ tag.count }}</span>
          <button class="edit-btn" @click.stop="startEditing(tag)" title="Renomear">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
            </svg>
          </button>
        </div>

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
  font-size: 0.6875rem;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.empty-state {
  padding: 1rem;
  text-align: center;
}

.empty-state p {
  font-size: 0.875rem;
  color: var(--text-tertiary);
  margin-bottom: 0.25rem;
}

.empty-state span {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.tag-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 0.875rem;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 0.875rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-liquid);
  text-align: left;
}

.tag-item:hover {
  background: var(--surface-2);
  color: var(--text-primary);
}

.tag-item.active {
  background: var(--tag-accent, var(--accent-light));
  color: var(--accent-color);
  border: 1px solid var(--tag-accent, var(--accent-color));
  box-shadow: 0 0 12px rgba(var(--tag-accent-rgb, var(--accent-rgb)), 0.3);
}

.tag-color {
  width: 1rem;
  height: 1rem;
  border-radius: var(--radius-full);
  flex-shrink: 0;
  border: 2px solid rgba(255, 255, 255, 0.15);
  box-shadow: 0 0 8px rgba(var(--accent-rgb), 0.3);
}

.tag-color.all {
  background: conic-gradient(
    var(--cyber-pink),
    var(--cyber-orange),
    var(--cyber-yellow),
    var(--cyber-green),
    var(--cyber-cyan),
    var(--cyber-purple),
    var(--cyber-pink)
  );
}

.tag-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tag-count {
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--text-muted);
  background: var(--surface-2);
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  flex-shrink: 0;
}

.edit-btn {
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  opacity: 0;
  transition: all var(--duration-fast);
  flex-shrink: 0;
}

.tag-item:hover .edit-btn {
  opacity: 1;
}

.edit-btn:hover {
  background: var(--surface-3);
  color: var(--text-primary);
}

.edit-btn svg {
  width: 0.875rem;
  height: 0.875rem;
}

.tag-edit {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--surface-2);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
}

.edit-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-size: 0.875rem;
  outline: none;
  min-width: 0;
}

.save-btn {
  width: 1.75rem;
  height: 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--success);
  border: none;
  border-radius: var(--radius-sm);
  color: #000;
  cursor: pointer;
  flex-shrink: 0;
  transition: all var(--duration-fast);
}

.save-btn:hover {
  transform: scale(1.05);
  box-shadow: 0 0 12px rgba(var(--success-rgb), 0.5);
}

.save-btn svg {
  width: 1rem;
  height: 1rem;
}
</style>