<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'

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

const props = defineProps<{
  x: number
  y: number
  items: ContextMenuItem[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', item: ContextMenuItem): void
}>()

const menuRef = ref<HTMLElement | null>(null)
const activeSubmenu = ref<string | null>(null)

const menuStyle = computed(() => {
  // Adjust position to keep menu in viewport
  const menuWidth = 200
  const menuHeight = props.items.length * 36 + 16
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight

  let left = props.x
  let top = props.y

  if (left + menuWidth > viewportWidth) {
    left = viewportWidth - menuWidth - 8
  }
  if (top + menuHeight > viewportHeight) {
    top = viewportHeight - menuHeight - 8
  }

  return {
    left: `${left}px`,
    top: `${top}px`,
  }
})

function handleItemClick(item: ContextMenuItem) {
  if (item.disabled || item.divider) return
  if (item.submenu) {
    activeSubmenu.value = activeSubmenu.value === item.id ? null : item.id
    return
  }
  if (item.action) {
    item.action()
  }
  emit('select', item)
  emit('close')
}

function handleSubmenuItemClick(item: ContextMenuItem) {
  if (item.disabled) return
  if (item.action) {
    item.action()
  }
  emit('select', item)
  emit('close')
}

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('close')
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <div ref="menuRef" class="context-menu" :style="menuStyle" @click.stop>
      <template v-for="item in items" :key="item.id">
        <!-- Divider -->
        <div v-if="item.divider" class="menu-divider" />

        <!-- Regular Item -->
        <div
          v-else
          class="menu-item"
          :class="{ disabled: item.disabled, 'has-submenu': item.submenu }"
          @click="handleItemClick(item)"
        >
          <!-- Color indicator -->
          <span v-if="item.color" class="item-color" :style="{ backgroundColor: item.color }" />

          <!-- Icon -->
          <span v-else-if="item.icon" class="item-icon" v-html="item.icon" />

          <!-- Label -->
          <span class="item-label">{{ item.label }}</span>

          <!-- Submenu arrow -->
          <svg v-if="item.submenu" class="submenu-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>

          <!-- Submenu -->
          <Transition name="submenu">
            <div v-if="item.submenu && activeSubmenu === item.id" class="submenu">
              <div
                v-for="subItem in item.submenu"
                :key="subItem.id"
                class="menu-item"
                :class="{ disabled: subItem.disabled }"
                @click.stop="handleSubmenuItemClick(subItem)"
              >
                <span v-if="subItem.color" class="item-color" :style="{ backgroundColor: subItem.color }" />
                <span class="item-label">{{ subItem.label }}</span>
              </div>
            </div>
          </Transition>
        </div>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  min-width: 180px;
  background: #1a1a1c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.5rem;
  padding: 0.5rem 0;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  z-index: 1000;
  animation: menuFadeIn 0.15s ease-out;
}

@keyframes menuFadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
  transition: background 0.15s;
  position: relative;
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

.menu-item.has-submenu:hover {
  background: rgba(99, 102, 241, 0.15);
}

.item-color {
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.3);
  flex-shrink: 0;
}

.item-icon {
  width: 1rem;
  height: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #a1a1aa;
  flex-shrink: 0;
}

.item-icon :deep(svg) {
  width: 100%;
  height: 100%;
}

.item-label {
  flex: 1;
  font-size: 0.875rem;
  color: #fafafa;
}

.submenu-arrow {
  width: 1rem;
  height: 1rem;
  color: #71717a;
  flex-shrink: 0;
}

.menu-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin: 0.5rem 0;
}

/* Submenu */
.submenu {
  position: absolute;
  left: 100%;
  top: 0;
  min-width: 160px;
  background: #1a1a1c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.5rem;
  padding: 0.5rem 0;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  margin-left: 0.25rem;
}

.submenu .menu-item {
  padding: 0.5rem 0.75rem;
}

/* Submenu transition */
.submenu-enter-active,
.submenu-leave-active {
  transition: all 0.15s ease;
}

.submenu-enter-from,
.submenu-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}
</style>
