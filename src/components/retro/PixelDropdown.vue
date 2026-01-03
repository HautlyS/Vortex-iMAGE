<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

export interface DropdownOption {
  id: string
  label: string
  icon?: string
  disabled?: boolean
  divider?: boolean
}

const props = withDefaults(defineProps<{
  options: DropdownOption[]
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  align?: 'left' | 'right'
}>(), {
  placeholder: 'Selecionar...',
  disabled: false,
  align: 'left'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  select: [option: DropdownOption]
}>()

const isOpen = ref(false)
const dropdownRef = ref<HTMLDivElement | null>(null)
const selectedIndex = ref(-1)

const selectedOption = computed(() => 
  props.options.find(o => o.id === props.modelValue && !o.divider)
)

const selectableOptions = computed(() => 
  props.options.filter(o => !o.divider && !o.disabled)
)

const toggle = () => {
  if (!props.disabled) {
    isOpen.value = !isOpen.value
    if (isOpen.value) {
      selectedIndex.value = selectableOptions.value.findIndex(o => o.id === props.modelValue)
    }
  }
}

const select = (option: DropdownOption) => {
  if (option.disabled || option.divider) return
  emit('update:modelValue', option.id)
  emit('select', option)
  isOpen.value = false
}

const handleKeydown = (e: KeyboardEvent) => {
  if (!isOpen.value) {
    if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
      e.preventDefault()
      isOpen.value = true
    }
    return
  }

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, selectableOptions.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault()
    if (selectedIndex.value >= 0) {
      select(selectableOptions.value[selectedIndex.value])
    }
  } else if (e.key === 'Escape') {
    isOpen.value = false
  }
}

const handleClickOutside = (e: MouseEvent) => {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => document.addEventListener('click', handleClickOutside))
onUnmounted(() => document.removeEventListener('click', handleClickOutside))
</script>

<template>
  <div 
    ref="dropdownRef"
    class="pixel-dropdown"
    :class="{ open: isOpen, disabled }"
    @keydown="handleKeydown"
  >
    <button 
      class="dropdown-trigger"
      :disabled="disabled"
      @click="toggle"
      aria-haspopup="listbox"
      :aria-expanded="isOpen"
    >
      <span v-if="selectedOption" class="dropdown-value">
        <span v-if="selectedOption.icon" class="dropdown-icon" v-html="selectedOption.icon" />
        {{ selectedOption.label }}
      </span>
      <span v-else class="dropdown-placeholder">{{ placeholder }}</span>
      <svg class="dropdown-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m6 9 6 6 6-6"/>
      </svg>
    </button>

    <Transition name="dropdown">
      <div v-if="isOpen" class="dropdown-menu" :class="[align]" role="listbox">
        <template v-for="(option, index) in options" :key="option.id">
          <div v-if="option.divider" class="dropdown-divider" />
          <button
            v-else
            class="dropdown-item"
            :class="{ 
              selected: option.id === modelValue,
              highlighted: selectableOptions[selectedIndex]?.id === option.id,
              disabled: option.disabled 
            }"
            :disabled="option.disabled"
            role="option"
            :aria-selected="option.id === modelValue"
            @click="select(option)"
          >
            <span v-if="option.icon" class="dropdown-icon" v-html="option.icon" />
            <span class="dropdown-label">{{ option.label }}</span>
            <svg v-if="option.id === modelValue" class="dropdown-check" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20 6L9 17l-5-5"/>
            </svg>
          </button>
        </template>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.pixel-dropdown {
  position: relative;
  display: inline-block;
  min-width: 180px;
}

.dropdown-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  background: var(--retro-bg-card, #251842);
  border: 2px solid var(--retro-bg-lighter, #2d1f4d);
  box-shadow: 2px 2px 0 #000;
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: var(--retro-text-main, #fff);
  text-align: left;
  cursor: pointer;
  transition: all 0.1s;
}

.pixel-dropdown.open .dropdown-trigger,
.dropdown-trigger:hover:not(:disabled) {
  border-color: var(--retro-accent-cyan, #00d4ff);
}

.pixel-dropdown.open .dropdown-trigger {
  box-shadow: 0 0 0 1px var(--retro-accent-cyan), 2px 2px 0 #000;
}

.dropdown-trigger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.dropdown-value {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dropdown-placeholder {
  color: var(--retro-text-muted, #9d8ec2);
}

.dropdown-arrow {
  width: 16px;
  height: 16px;
  color: var(--retro-text-muted, #9d8ec2);
  transition: transform 0.15s ease;
  flex-shrink: 0;
}

.pixel-dropdown.open .dropdown-arrow {
  transform: rotate(180deg);
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 100%;
  background: var(--retro-bg-panel, #1a1030);
  border: 2px solid var(--retro-bg-lighter, #2d1f4d);
  box-shadow: 4px 4px 0 rgba(0,0,0,0.5);
  z-index: 100;
  max-height: 240px;
  overflow-y: auto;
}

.dropdown-menu.right {
  left: auto;
  right: 0;
}

.dropdown-divider {
  height: 2px;
  background: var(--retro-bg-lighter, #2d1f4d);
  margin: 4px 0;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  background: transparent;
  border: none;
  box-shadow: none;
  font-family: 'VT323', monospace;
  font-size: 16px;
  color: var(--retro-text-muted, #9d8ec2);
  text-align: left;
  cursor: pointer;
  transition: all 0.1s;
}

.dropdown-item:hover:not(:disabled),
.dropdown-item.highlighted {
  background: var(--retro-bg-card, #251842);
  color: var(--retro-text-main, #fff);
}

.dropdown-item.selected {
  color: var(--retro-accent-green, #00ff87);
}

.dropdown-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.dropdown-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.dropdown-icon :deep(svg) { width: 100%; height: 100%; }

.dropdown-label { flex: 1; }

.dropdown-check {
  width: 16px;
  height: 16px;
  color: var(--retro-accent-green, #00ff87);
  flex-shrink: 0;
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
