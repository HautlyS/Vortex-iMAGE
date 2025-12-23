<script setup lang="ts">
import { ref } from 'vue'

export interface ContextMenuItem {
  id: string
  label: string
  icon?: string
  color?: string
  submenu?: ContextMenuItem[]
  action?: () => void
  disabled?: boolean
  divider?: boolean
}

defineProps<{
  x: number
  y: number
  items: ContextMenuItem[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', item: ContextMenuItem): void
}>()

const hoveredItem = ref<string | null>(null)

function handleItemClick(item: ContextMenuItem) {
  if (item.disabled || item.divider || item.submenu) return
  if (item.action) item.action()
  emit('select', item)
  emit('close')
}

function handleSubItemClick(item: ContextMenuItem) {
  if (item.disabled) return
  if (item.action) item.action()
  emit('select', item)
  emit('close')
}

function handleClickOutside() {
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div class="context-overlay" @click="handleClickOutside">
      <div 
        class="context-menu" 
        :style="{ left: x + 'px', top: y + 'px' }"
        @click.stop
      >
        <template v-for="item in items" :key="item.id">
          <div v-if="item.divider" class="divider" />
          <div
            v-else
            class="menu-item"
            :class="{ disabled: item.disabled, 'has-submenu': item.submenu }"
            @click="handleItemClick(item)"
            @mouseenter="hoveredItem = item.id"
            @mouseleave="hoveredItem = null"
          >
            <span v-if="item.color" class="color-dot" :style="{ backgroundColor: item.color }" />
            <span v-else-if="item.icon" class="icon" v-html="item.icon" />
            <span class="label">{{ item.label }}</span>
            <span v-if="item.submenu" class="arrow">▶</span>
            
            <!-- Submenu -->
            <div v-if="item.submenu && hoveredItem === item.id" class="submenu">
              <div
                v-for="subItem in item.submenu"
                :key="subItem.id"
                class="menu-item"
                @click="handleSubItemClick(subItem)"
              >
                <span v-if="subItem.color" class="color-dot" :style="{ backgroundColor: subItem.color }" />
                <span class="label">{{ subItem.label }}</span>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.context-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
}

.context-menu {
  position: absolute;
  min-width: 180px;
  background: #1a1a1c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 4px 0;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.menu-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  color: #fafafa;
  font-size: 14px;
}

.menu-item:hover {
  background: rgba(99, 102, 241, 0.15);
}

.menu-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.menu-item.disabled:hover {
  background: transparent;
}

.color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.3);
}

.icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.label {
  flex: 1;
}

.arrow {
  font-size: 10px;
  color: #71717a;
}

.divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin: 4px 0;
}

.submenu {
  position: absolute;
  left: 100%;
  top: 0;
  min-width: 160px;
  background: #1a1a1c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 4px 0;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  margin-left: 4px;
}
</style>
