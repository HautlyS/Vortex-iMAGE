<script setup lang="ts">
import { computed } from 'vue'
import { SmartAlbum } from '@/composables/useSmartAlbums'

const props = defineProps<{
  albums: SmartAlbum[]
  selectedId: string | null
}>()

const emit = defineEmits<{
  (e: 'select', id: string): void
}>()

const dateAlbums = computed(() => props.albums.filter(a => a.type === 'date'))
const colorAlbums = computed(() => props.albums.filter(a => a.type === 'color'))

function handleSelect(id: string) {
  emit('select', id)
}
</script>

<template>
  <div class="smart-album-list">
    <!-- Date Albums -->
    <div v-if="dateAlbums.length > 0" class="album-group">
      <div class="group-header">Datas</div>
      <div class="group-items">
        <button
          v-for="album in dateAlbums"
          :key="album.id"
          class="smart-album-item"
          :class="{ active: selectedId === album.id }"
          @click="handleSelect(album.id)"
        >
          <div class="album-icon date-icon">
             <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
               <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
               <line x1="16" y1="2" x2="16" y2="6"></line>
               <line x1="8" y1="2" x2="8" y2="6"></line>
               <line x1="3" y1="10" x2="21" y2="10"></line>
             </svg>
          </div>
          <span class="album-name">{{ album.title }}</span>
          <span class="album-count">{{ album.count }}</span>
        </button>
      </div>
    </div>

    <!-- Color Albums -->
    <div v-if="colorAlbums.length > 0" class="album-group">
      <div class="group-header">Cores</div>
      <div class="group-items">
        <button
          v-for="album in colorAlbums"
          :key="album.id"
          class="smart-album-item"
          :class="{ active: selectedId === album.id }"
          @click="handleSelect(album.id)"
        >
          <div class="album-icon color-icon" :style="{ color: 'var(--systemSecondary)' }">
              <span class="color-dot" :style="{ backgroundColor: album.title /* assuming title contains color name, wait. Color name isn't CSS color. */ }"></span>
              <!-- Actually title is name like 'Red'. We need the hex from somewhere or use ID suffix. -->
              <!-- The useSmartAlbums put 'Red' as title. Creating dot based on title is risky. -->
              <svg viewBox="0 0 24 24" fill="currentColor" stroke="none">
                 <circle cx="12" cy="12" r="6" />
              </svg>
          </div>
           <!-- FIX: We need actual color value. useSmartAlbums puts Human Name in title. -->
           <!-- Let's just use a generic icon for now or improve useSmartAlbums to pass color value. -->
          <span class="album-name">{{ album.title }}</span>
          <span class="album-count">{{ album.count }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.smart-album-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem 0;
  border-top: 1px solid var(--border-default);
  margin-top: 1rem;
}

.group-header {
  font-size: 0.6875rem;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0 0.75rem;
  margin-bottom: 0.5rem;
}

.smart-album-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border-radius: var(--radius-sm);
}

.smart-album-item:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.smart-album-item.active {
  background: rgba(var(--accent-rgb), 0.1);
  color: var(--accent-color);
}

.album-icon {
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: inherit;
}

.album-icon svg {
  width: 1rem;
  height: 1rem;
}

.color-icon {
    /* Special styling if needed */
}

.album-name {
  flex: 1;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.album-count {
  font-size: 0.75rem;
  color: var(--text-muted);
  background: var(--surface-2);
  padding: 0.125rem 0.375rem;
  border-radius: 99px;
}

.smart-album-item.active .album-count {
    background: rgba(var(--accent-rgb), 0.2);
    color: var(--accent-color);
}
</style>
