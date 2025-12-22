<script setup lang="ts">
import { ref } from 'vue'

export interface Album {
  name: string
  path: string
  photo_count: number
  children: Album[]
}

defineProps<{
  albums: Album[]
  selectedPath: string | null
}>()

const emit = defineEmits<{
  (e: 'select', path: string | null): void
}>()

const expandedPaths = ref<Set<string>>(new Set())

function toggleExpand(path: string) {
  if (expandedPaths.value.has(path)) {
    expandedPaths.value.delete(path)
  } else {
    expandedPaths.value.add(path)
  }
}

function isExpanded(path: string): boolean {
  return expandedPaths.value.has(path)
}

function handleSelect(path: string | null) {
  emit('select', path)
}
</script>

<template>
  <div class="album-tree">
    <!-- All Photos -->
    <button
      class="tree-item root"
      :class="{ active: selectedPath === null }"
      @click="handleSelect(null)"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="7" height="7" />
        <rect x="14" y="3" width="7" height="7" />
        <rect x="14" y="14" width="7" height="7" />
        <rect x="3" y="14" width="7" height="7" />
      </svg>
      <span class="item-name">Todas as Fotos</span>
    </button>

    <!-- Album Items -->
    <template v-for="album in albums" :key="album.path">
      <div class="tree-branch">
        <button
          class="tree-item"
          :class="{ active: selectedPath === album.path }"
          @click="handleSelect(album.path)"
        >
          <!-- Expand/Collapse Button -->
          <button
            v-if="album.children.length > 0"
            class="expand-btn"
            @click.stop="toggleExpand(album.path)"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              :class="{ expanded: isExpanded(album.path) }"
            >
              <path d="M9 18l6-6-6-6" />
            </svg>
          </button>
          <span v-else class="expand-spacer" />

          <!-- Folder Icon -->
          <svg class="folder-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>

          <span class="item-name">{{ album.name }}</span>
          <span class="item-count">{{ album.photo_count }}</span>
        </button>

        <!-- Children -->
        <Transition name="expand">
          <div v-if="album.children.length > 0 && isExpanded(album.path)" class="tree-children">
            <template v-for="child in album.children" :key="child.path">
              <button
                class="tree-item child"
                :class="{ active: selectedPath === child.path }"
                @click="handleSelect(child.path)"
              >
                <span class="expand-spacer" />
                <svg class="folder-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                </svg>
                <span class="item-name">{{ child.name }}</span>
                <span class="item-count">{{ child.photo_count }}</span>
              </button>
            </template>
          </div>
        </Transition>
      </div>
    </template>
  </div>
</template>

<style scoped>
.album-tree {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.tree-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.5rem;
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

.tree-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fafafa;
}

.tree-item.active {
  background: rgba(99, 102, 241, 0.15);
  color: #818cf8;
}

.tree-item.root > svg {
  width: 1.125rem;
  height: 1.125rem;
  flex-shrink: 0;
}

.tree-item.child {
  padding-left: 2rem;
}

.expand-btn {
  width: 1.25rem;
  height: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #71717a;
  cursor: pointer;
  border-radius: 0.25rem;
  flex-shrink: 0;
  transition: all 0.15s;
}

.expand-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fafafa;
}

.expand-btn svg {
  width: 0.875rem;
  height: 0.875rem;
  transition: transform 0.2s;
}

.expand-btn svg.expanded {
  transform: rotate(90deg);
}

.expand-spacer {
  width: 1.25rem;
  flex-shrink: 0;
}

.folder-icon {
  width: 1rem;
  height: 1rem;
  flex-shrink: 0;
}

.item-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-count {
  font-size: 0.75rem;
  color: #52525b;
  background: rgba(255, 255, 255, 0.05);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  flex-shrink: 0;
}

.tree-children {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  margin-left: 0.75rem;
  padding-left: 0.75rem;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
}

/* Expand transition */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 500px;
}
</style>
