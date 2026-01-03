<script setup lang="ts">
import { ref, computed } from 'vue'
import FolderCreator from './FolderCreator.vue'

export interface Album {
  name: string
  path: string
  photo_count: number
  children: Album[]
}

const props = defineProps<{
  albums: Album[]
  selectedPath: string | null
  loading?: boolean
}>()

const emit = defineEmits<{
  (e: 'select', path: string | null): void
  (e: 'refresh'): void
}>()

const expandedPaths = ref<Set<string>>(new Set())
const showFolderCreator = ref(false)
const creatingInPath = ref<string | null>(null)
const contextMenuPath = ref<string | null>(null)

const totalPhotos = computed(() => {
  function countPhotos(albums: Album[]): number {
    return albums.reduce((sum, a) => sum + a.photo_count + countPhotos(a.children), 0)
  }
  return countPhotos(props.albums)
})

function toggleExpand(path: string, e: Event) {
  e.stopPropagation()
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

function openFolderCreator(parentPath: string | null = null) {
  creatingInPath.value = parentPath
  showFolderCreator.value = true
  contextMenuPath.value = null
}

function handleFolderCreated(_path: string) {
  showFolderCreator.value = false
  creatingInPath.value = null
  emit('refresh')
}

function showContextMenu(path: string, e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  contextMenuPath.value = contextMenuPath.value === path ? null : path
}

function closeContextMenu() {
  contextMenuPath.value = null
}
</script>

<template>
  <div class="album-tree" @click="closeContextMenu">
    <!-- Header with create button -->
    <div class="tree-header">
      <span class="tree-title">Álbuns</span>
      <button class="add-folder-btn" @click.stop="openFolderCreator(null)" title="Nova pasta">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 5v14m-7-7h14" />
        </svg>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="tree-loading">
      <div class="loading-shimmer" />
      <div class="loading-shimmer short" />
      <div class="loading-shimmer" />
    </div>

    <template v-else>
      <!-- All Photos -->
      <button
        class="tree-item root"
        :class="{ active: selectedPath === null }"
        @click="handleSelect(null)"
      >
        <div class="item-icon all-photos">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="14" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
          </svg>
        </div>
        <span class="item-name">Todas as Fotos</span>
        <span class="item-count">{{ totalPhotos }}</span>
      </button>

      <!-- Empty state -->
      <div v-if="albums.length === 0" class="empty-albums">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
        </div>
        <p>Nenhum álbum</p>
        <button class="create-first-btn" @click="openFolderCreator(null)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14m-7-7h14" />
          </svg>
          Criar primeiro álbum
        </button>
      </div>

      <!-- Album Items -->
      <div v-else class="albums-list">
        <template v-for="album in albums" :key="album.path">
          <div class="tree-branch">
            <button
              class="tree-item"
              :class="{ active: selectedPath === album.path }"
              @click="handleSelect(album.path)"
              @contextmenu="showContextMenu(album.path, $event)"
            >
              <!-- Expand/Collapse Button -->
              <button
                v-if="album.children.length > 0"
                class="expand-btn"
                @click="toggleExpand(album.path, $event)"
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
              <div class="item-icon folder">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                </svg>
              </div>

              <span class="item-name">{{ album.name }}</span>
              <span class="item-count">{{ album.photo_count }}</span>

              <!-- Quick add subfolder -->
              <button 
                class="quick-add-btn" 
                @click.stop="openFolderCreator(album.path.replace('photos/', ''))"
                title="Criar subpasta"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 5v14m-7-7h14" />
                </svg>
              </button>
            </button>

            <!-- Context Menu -->
            <Transition name="menu">
              <div v-if="contextMenuPath === album.path" class="context-menu">
                <button @click="openFolderCreator(album.path.replace('photos/', ''))">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 5v14m-7-7h14" />
                  </svg>
                  Nova subpasta
                </button>
              </div>
            </Transition>

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
                    <div class="item-icon folder child-folder">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                      </svg>
                    </div>
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

    <!-- Folder Creator Modal -->
    <FolderCreator
      v-if="showFolderCreator"
      :parent-path="creatingInPath"
      @created="handleFolderCreated"
      @close="showFolderCreator = false"
    />
  </div>
</template>

<style scoped>
.album-tree {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.tree-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  margin-bottom: 0.25rem;
}

.tree-title {
  font-size: 0.6875rem;
  font-weight: 700;
  color: var(--systemTertiary);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.add-folder-btn {
  width: 1.5rem;
  height: 1.5rem;
  background: transparent;
  border: none;
  color: var(--systemTertiary);
  cursor: pointer;
  border-radius: var(--global-border-radius-xsmall);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.add-folder-btn:hover {
  background: var(--systemQuinary);
  color: var(--keyColor);
}

.add-folder-btn svg {
  width: 0.875rem;
  height: 0.875rem;
}

/* Loading */
.tree-loading {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0 0.75rem;
}

.loading-shimmer {
  height: 2.25rem;
  background: linear-gradient(90deg, var(--systemQuinary) 25%, var(--systemQuaternary) 50%, var(--systemQuinary) 75%);
  background-size: 200% 100%;
  border-radius: var(--global-border-radius-small);
  animation: shimmer 1.5s infinite;
}

.loading-shimmer.short {
  width: 70%;
  margin-left: 1.5rem;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* Tree Items */
.tree-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 0.75rem;
  background: transparent;
  border: none;
  color: var(--systemSecondary);
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: var(--global-border-radius-small);
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
  position: relative;
}

.tree-item:hover {
  background: var(--systemQuinary);
  color: var(--systemPrimary);
}

.tree-item:hover .quick-add-btn {
  opacity: 1;
}

.tree-item.active {
  background: rgba(var(--keyColor-rgb), 0.12);
  color: var(--keyColor);
}

.tree-item.active .item-icon {
  background: var(--keyColor);
  color: #fff;
}

.tree-item.child {
  padding-left: 2.25rem;
}

/* Item Icon */
.item-icon {
  width: 1.75rem;
  height: 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--systemQuinary);
  border-radius: var(--global-border-radius-xsmall);
  flex-shrink: 0;
  transition: all 0.15s;
}

.item-icon svg {
  width: 1rem;
  height: 1rem;
}

.item-icon.all-photos {
  background: linear-gradient(135deg, var(--systemIndigo), var(--systemPurple));
  color: #fff;
}

.item-icon.folder {
  background: var(--systemGray5);
  color: var(--systemGray);
}

.item-icon.child-folder {
  width: 1.5rem;
  height: 1.5rem;
}

.item-icon.child-folder svg {
  width: 0.875rem;
  height: 0.875rem;
}

/* Expand Button */
.expand-btn {
  width: 1.25rem;
  height: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--systemTertiary);
  cursor: pointer;
  border-radius: var(--global-border-radius-xsmall);
  flex-shrink: 0;
  transition: all 0.15s;
}

.expand-btn:hover {
  background: var(--systemQuaternary);
  color: var(--systemPrimary);
}

.expand-btn svg {
  width: 0.75rem;
  height: 0.75rem;
  transition: transform 0.2s var(--ease-out);
}

.expand-btn svg.expanded {
  transform: rotate(90deg);
}

.expand-spacer {
  width: 1.25rem;
  flex-shrink: 0;
}

/* Item Name & Count */
.item-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-count {
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--systemTertiary);
  background: var(--systemQuinary);
  padding: 0.125rem 0.5rem;
  border-radius: 1000px;
  flex-shrink: 0;
  min-width: 1.5rem;
  text-align: center;
}

.tree-item.active .item-count {
  background: rgba(var(--keyColor-rgb), 0.15);
  color: var(--keyColor);
}

/* Quick Add Button */
.quick-add-btn {
  width: 1.25rem;
  height: 1.25rem;
  background: transparent;
  border: none;
  color: var(--systemTertiary);
  cursor: pointer;
  border-radius: var(--global-border-radius-xsmall);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.quick-add-btn:hover {
  background: var(--systemQuaternary);
  color: var(--keyColor);
}

.quick-add-btn svg {
  width: 0.75rem;
  height: 0.75rem;
}

/* Tree Children */
.tree-children {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  margin-left: 1rem;
  padding-left: 0.75rem;
  border-left: 1px solid var(--labelDivider);
}

/* Context Menu */
.context-menu {
  position: absolute;
  top: 100%;
  right: 0.5rem;
  background: var(--systemStandardThickMaterialSover);
  backdrop-filter: blur(20px);
  border: 1px solid var(--labelDivider);
  border-radius: var(--global-border-radius-small);
  box-shadow: var(--shadow-medium);
  padding: 0.25rem;
  z-index: 100;
  min-width: 140px;
}

.context-menu button {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: transparent;
  border: none;
  color: var(--systemPrimary);
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: var(--global-border-radius-xsmall);
  cursor: pointer;
  transition: all 0.1s;
}

.context-menu button:hover {
  background: var(--systemQuinary);
}

.context-menu button svg {
  width: 0.875rem;
  height: 0.875rem;
  color: var(--systemSecondary);
}

/* Empty State */
.empty-albums {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2rem 1rem;
  text-align: center;
}

.empty-icon {
  width: 3rem;
  height: 3rem;
  background: var(--systemQuinary);
  border-radius: var(--global-border-radius-medium);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 0.75rem;
}

.empty-icon svg {
  width: 1.5rem;
  height: 1.5rem;
  color: var(--systemTertiary);
}

.empty-albums p {
  font-size: 0.8125rem;
  color: var(--systemTertiary);
  margin-bottom: 1rem;
}

.create-first-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  background: rgba(var(--keyColor-rgb), 0.1);
  border: none;
  border-radius: var(--global-border-radius-small);
  color: var(--keyColor);
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
}

.create-first-btn:hover {
  background: rgba(var(--keyColor-rgb), 0.15);
}

.create-first-btn svg {
  width: 0.875rem;
  height: 0.875rem;
}

/* Transitions */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.2s var(--ease-out);
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

.menu-enter-active,
.menu-leave-active {
  transition: all 0.15s var(--ease-out);
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.95);
}
</style>
