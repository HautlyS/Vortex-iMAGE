/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<template>
  <div 
    v-if="tags.length > 0" 

    class="floating-tag-panel card-glass"
    :style="{ left: `${pos.x}px`, top: `${pos.y}px` }"
  >
    <div 
      class="panel-header"
      @mousedown="startDrag"
    >
      <span class="headline">Etiquetas</span>
      <div class="drag-handle">
        <svg xmlns="http:
      </div>
    </div>
    
    <div class="tags-list">
      <div v-for="tag in tags" :key="tag.id" class="tag-item">
        <div class="tag-color" :style="{ backgroundColor: tag.color, boxShadow: `0 0 8px ${tag.color}` }"></div>
        
        <div v-if="editingId === tag.id" class="tag-edit">
          <input 
            ref="editInput"
            v-model="editName"
            @blur="saveEdit(tag)"
            @keydown.enter="saveEdit(tag)"
            class="edit-input"
          />
        </div>
        <div v-else class="tag-name" @dblclick="startEdit(tag)">
          {{ tag.name }}
        </div>
        
        <button class="edit-btn" @click="startEdit(tag)">
          <svg xmlns="http:
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import type { Tag } from '../composables/useTags'

defineProps<{
  tags: Tag[]
}>()

const emit = defineEmits<{
  updateTag: [id: string, name: string]
}>()

const editingId = ref<string | null>(null)
const editName = ref('')
const editInput = ref<HTMLInputElement[] | null>(null)

const pos = ref({ x: window.innerWidth / 2 - 160, y: window.innerHeight / 2 - 200 })
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })
const initialPos = ref({ x: 0, y: 0 })

function startDrag(e: MouseEvent) {
  
  isDragging.value = true
  dragStart.value = { x: e.clientX, y: e.clientY }
  initialPos.value = { x: pos.value.x, y: pos.value.y }
  
  window.addEventListener('mousemove', onDrag)
  window.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value) return
  const dx = e.clientX - dragStart.value.x
  const dy = e.clientY - dragStart.value.y
  pos.value = {
    x: initialPos.value.x + dx,
    y: initialPos.value.y + dy
  }
}

function stopDrag() {
  isDragging.value = false
  window.removeEventListener('mousemove', onDrag)
  window.removeEventListener('mouseup', stopDrag)
}

function startEdit(tag: Tag) {
  editingId.value = tag.id
  editName.value = tag.name
  nextTick(() => {
    if (editInput.value && editInput.value[0]) {
      editInput.value[0].focus()
    }
  })
}

function saveEdit(tag: Tag) {
  if (editingId.value === tag.id && editName.value.trim()) {
    emit('updateTag', tag.id, editName.value)
  }
  editingId.value = null
}

</script>

<style scoped>
.floating-tag-panel {
  position: fixed;
  width: 320px;
  max-height: 400px;
  display: flex;
  flex-direction: column;
  z-index: 100;
  padding: 16px;
  box-shadow: 0 10px 40px rgba(0,0,0,0.2), 0 0 0 1px rgba(255,255,255,0.1);
  transition: opacity 0.2s;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255,255,255,0.1);
  cursor: grab;
}

.panel-header:active {
  cursor: grabbing;
}

.drag-handle {
  opacity: 0.5;
}

.drag-handle {
  cursor: grab;
  opacity: 0.5;
}

.tags-list {
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tag-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border-radius: 8px;
  transition: background-color 0.2s;
}

.tag-item:hover {
  background-color: rgba(255,255,255,0.05);
}

.tag-color {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid rgba(255,255,255,0.2);
}

.tag-name {
  flex: 1;
  font-size: 15px;
  font-weight: 500;
  cursor: text;
}

.tag-edit {
  flex: 1;
}

.edit-input {
  width: 100%;
  background: rgba(0,0,0,0.2);
  border: 1px solid rgba(255,255,255,0.2);
  border-radius: 4px;
  padding: 2px 6px;
  color: white;
  font-size: 15px;
}

.edit-btn {
  padding: 4px;
  opacity: 0;
  transition: opacity 0.2s;
  background: transparent;
  border: none;
  color: rgba(255,255,255,0.6);
  cursor: pointer;
}

.tag-item:hover .edit-btn {
  opacity: 1;
}

.edit-btn:hover {
  color: white;
}
</style>