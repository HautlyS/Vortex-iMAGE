/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSyncStatus, type SyncStatus } from '../composables/useSyncStatus'

const props = defineProps<{
  photoId: string
  status: SyncStatus
}>()

const emit = defineEmits<{
  (e: 'action', action: string, photoId: string): void
}>()

const { getStatusTooltip, getAvailableActions, getStatusColor } = useSyncStatus()

const showMenu = ref(false)
const menuRef = ref<HTMLElement | null>(null)

const tooltip = computed(() => getStatusTooltip(props.status))
const actions = computed(() => getAvailableActions(props.status, props.photoId))
const iconColor = computed(() => getStatusColor(props.status))

const menuStyle = ref({ top: '0px', left: '0px' })
const buttonRef = ref<HTMLElement | null>(null)

function updatePosition() {
  if (!buttonRef.value) return
  const rect = buttonRef.value.getBoundingClientRect()
  
  let left = rect.left + (rect.width / 2)

  menuStyle.value = {
    top: `${rect.top}px`, 
    left: `${left}px`
  }
}

function toggleMenu(e: MouseEvent) {
  e.stopPropagation()
  if (!showMenu.value) {
    updatePosition()
    showMenu.value = true
  } else {
    showMenu.value = false
  }
}

function handleAction(actionId: string) {
  emit('action', actionId, props.photoId)
  showMenu.value = false
}

function closeMenu() {
  showMenu.value = false
}

import { onUnmounted, watch, nextTick } from 'vue'

watch(showMenu, (isOpen) => {
  if (isOpen) {
    nextTick(updatePosition)
    window.addEventListener('scroll', updatePosition, true)
    window.addEventListener('resize', updatePosition)
    setTimeout(() => document.addEventListener('click', handleClickOutside), 0)
  } else {
    window.removeEventListener('scroll', updatePosition, true)
    window.removeEventListener('resize', updatePosition)
    document.removeEventListener('click', handleClickOutside)
  }
})

function handleClickOutside(e: MouseEvent) {
  
  const menuEl = document.querySelector('.sync-menu')
  if (menuEl && menuEl.contains(e.target as Node)) return
  
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    closeMenu()
  }
}

onUnmounted(() => {
  window.removeEventListener('scroll', updatePosition, true)
  window.removeEventListener('resize', updatePosition)
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div class="sync-indicator" ref="menuRef">
    <!-- Status Icon Button -->
    <div ref="buttonRef">
    <button 
      class="indicator-btn" 
      :title="tooltip"
      @click="toggleMenu"
    >
      <!-- Local Only Icon (Device) -->
      <svg v-if="status === 'local-only'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :style="{ color: iconColor }">
        <rect x="5" y="2" width="14" height="20" rx="2" ry="2" />
        <line x1="12" y1="18" x2="12.01" y2="18" />
      </svg>
      
      <!-- Remote Only Icon (Cloud) -->
      <svg v-else-if="status === 'remote-only'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :style="{ color: iconColor }">
        <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z" />
      </svg>
      
      <!-- Synced Icon (Cloud with Check) -->
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :style="{ color: iconColor }">
        <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z" />
        <path d="M9 12l2 2 4-4" />
      </svg>
    </button>
    </div>

    <!-- Action Menu -->
    <Teleport to="body">
      <Transition name="menu">
        <div 
          v-if="showMenu" 
          class="sync-menu"
          :style="menuStyle"
        >
          <div class="menu-header">
            <span class="menu-title">Sync Status</span>
            <span class="status-badge" :style="{ backgroundColor: iconColor }">
              {{ status === 'local-only' ? 'Local' : status === 'remote-only' ? 'Remote' : 'Synced' }}
            </span>
          </div>
          <p class="menu-description">{{ tooltip }}</p>
          <div class="menu-divider" />
          <div class="menu-actions">
            <button 
              v-for="action in actions" 
              :key="action.id"
              class="menu-action"
              @click="handleAction(action.id)"
            >
              <!-- Upload Icon -->
              <svg v-if="action.icon === 'upload'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17,8 12,3 7,8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              <!-- Download Icon -->
              <svg v-else-if="action.icon === 'download'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7,10 12,15 17,10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              <!-- Trash Icon -->
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
              <span>{{ action.label }}</span>
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.sync-indicator {
  position: relative;
}

.indicator-btn {
  width: 1.75rem;
  height: 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  opacity: 0;
  transition: all 0.2s ease;
  z-index: 2;
}

.decay-card:hover .indicator-btn,
.photo-preview:hover .indicator-btn {
  opacity: 1;
}

.indicator-btn:hover {
  background: rgba(0, 0, 0, 0.8);
  transform: scale(1.1);
}

.indicator-btn svg {
  width: 1rem;
  height: 1rem;
}

.sync-menu {
  position: fixed;
  top: 0;
  left: 0;
  margin-top: -10px; 
  transform: translate(-50%, -100%);
  width: 220px;
  background: rgba(17, 17, 19, 0.98);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.625rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  z-index: 100;
  overflow: hidden;
}

.menu-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 0.875rem 0.5rem;
}

.menu-title {
  font-size: 0.6875rem;
  font-weight: 600;
  color: #71717a;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.status-badge {
  font-size: 0.625rem;
  font-weight: 600;
  color: #fff;
  padding: 0.125rem 0.5rem;
  border-radius: 1rem;
}

.menu-description {
  padding: 0 0.875rem 0.75rem;
  font-size: 0.75rem;
  color: #a1a1aa;
  line-height: 1.4;
  margin: 0;
}

.menu-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
}

.menu-actions {
  padding: 0.375rem;
}

.menu-action {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.5rem 0.625rem;
  background: transparent;
  border: none;
  border-radius: 0.375rem;
  color: #e4e4e7;
  font-size: 0.8125rem;
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
}

.menu-action:hover {
  background: rgba(255, 255, 255, 0.08);
}

.menu-action svg {
  width: 1rem;
  height: 1rem;
  color: #71717a;
  flex-shrink: 0;
}

.menu-action:hover svg {
  color: var(--accent-color, #818cf8);
}

.menu-enter-active {
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.menu-leave-active {
  transition: all 0.15s ease;
}

.menu-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(8px) scale(0.95);
}

.menu-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(4px) scale(0.98);
}
</style>