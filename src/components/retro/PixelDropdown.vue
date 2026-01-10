<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';

interface Option {
  value: string | number;
  label: string;
  icon?: string;
  disabled?: boolean;
}

const props = withDefaults(defineProps<{
  modelValue: string | number | null;
  options: Option[];
  placeholder?: string;
  disabled?: boolean;
  label?: string;
  variant?: 'default' | 'compact';
}>(), {
  placeholder: 'SELECT...',
  disabled: false,
  variant: 'default'
});

const emit = defineEmits<{
  'update:modelValue': [value: string | number];
}>();

const isOpen = ref(false);
const dropdownRef = ref<HTMLElement | null>(null);
const highlightedIndex = ref(-1);

const selectedOption = computed(() => {
  return props.options.find(opt => opt.value === props.modelValue);
});

const toggle = () => {
  if (!props.disabled) {
    isOpen.value = !isOpen.value;
    if (isOpen.value) {
      highlightedIndex.value = props.options.findIndex(opt => opt.value === props.modelValue);
    }
  }
};

const select = (option: Option) => {
  if (!option.disabled) {
    emit('update:modelValue', option.value);
    isOpen.value = false;
  }
};

const handleKeydown = (e: KeyboardEvent) => {
  if (!isOpen.value) {
    if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
      e.preventDefault();
      isOpen.value = true;
    }
    return;
  }

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      highlightedIndex.value = Math.min(highlightedIndex.value + 1, props.options.length - 1);
      break;
    case 'ArrowUp':
      e.preventDefault();
      highlightedIndex.value = Math.max(highlightedIndex.value - 1, 0);
      break;
    case 'Enter':
    case ' ':
      e.preventDefault();
      if (highlightedIndex.value >= 0) {
        select(props.options[highlightedIndex.value]);
      }
      break;
    case 'Escape':
      isOpen.value = false;
      break;
  }
};

const handleClickOutside = (e: MouseEvent) => {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    isOpen.value = false;
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div class="pixel-dropdown-wrapper" :class="[variant]">
    <label v-if="label" class="dropdown-label">{{ label }}</label>
    
    <div 
      ref="dropdownRef"
      class="pixel-dropdown"
      :class="{ open: isOpen, disabled }"
      @keydown="handleKeydown"
      tabindex="0"
    >
      <!-- Trigger -->
      <button 
        class="dropdown-trigger"
        :disabled="disabled"
        @click="toggle"
        type="button"
      >
        <span class="trigger-text">
          {{ selectedOption?.label || placeholder }}
        </span>
        <span class="trigger-arrow" :class="{ rotated: isOpen }">
          <svg viewBox="0 0 16 16">
            <rect x="2" y="4" width="4" height="4" fill="currentColor"/>
            <rect x="6" y="8" width="4" height="4" fill="currentColor"/>
            <rect x="10" y="4" width="4" height="4" fill="currentColor"/>
          </svg>
        </span>
        
        <!-- Corner decorations -->
        <div class="corner tl"></div>
        <div class="corner tr"></div>
        <div class="corner bl"></div>
        <div class="corner br"></div>
      </button>
      
      <!-- Options -->
      <Transition name="dropdown">
        <div v-if="isOpen" class="dropdown-options">
          <div class="options-scroll">
            <button
              v-for="(option, index) in options"
              :key="option.value"
              class="dropdown-option"
              :class="{ 
                selected: option.value === modelValue,
                highlighted: index === highlightedIndex,
                disabled: option.disabled
              }"
              @click="select(option)"
              @mouseenter="highlightedIndex = index"
              type="button"
            >
              <span v-if="option.icon" class="option-icon">{{ option.icon }}</span>
              <span class="option-label">{{ option.label }}</span>
              <span v-if="option.value === modelValue" class="option-check">
                <svg viewBox="0 0 16 16">
                  <rect x="2" y="8" width="4" height="4" fill="currentColor"/>
                  <rect x="6" y="10" width="4" height="4" fill="currentColor"/>
                  <rect x="10" y="4" width="4" height="4" fill="currentColor"/>
                  <rect x="14" y="0" width="2" height="4" fill="currentColor"/>
                </svg>
              </span>
            </button>
          </div>
          
          <!-- Corner decorations -->
          <div class="corner tl"></div>
          <div class="corner tr"></div>
          <div class="corner bl"></div>
          <div class="corner br"></div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.pixel-dropdown-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
  image-rendering: pixelated;
}

.dropdown-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #808080;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.pixel-dropdown {
  position: relative;
  outline: none;
}

.pixel-dropdown.disabled {
  opacity: 0.5;
  pointer-events: none;
}

/* Trigger */
.dropdown-trigger {
  position: relative;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  background: linear-gradient(180deg, #3a3a5c 0%, #2a2a4c 100%);
  border: 4px solid #000;
  font-family: 'VT323', monospace;
  font-size: 20px;
  color: #fff;
  cursor: pointer;
  text-align: left;
  box-shadow: 
    inset -4px -4px 0 #1a1a3c,
    inset 4px 4px 0 #4a4a6c,
    4px 4px 0 #000;
}

.dropdown-trigger:hover:not(:disabled) {
  background: linear-gradient(180deg, #4a4a6c 0%, #3a3a5c 100%);
}

.pixel-dropdown.open .dropdown-trigger {
  border-color: #39ff14;
  box-shadow: 
    inset -4px -4px 0 #1a1a3c,
    inset 4px 4px 0 #4a4a6c,
    0 0 0 2px #39ff14,
    0 0 16px rgba(57, 255, 20, 0.3);
}

/* Corners */
.corner {
  position: absolute;
  width: 4px;
  height: 4px;
  background: #4a4a6c;
  z-index: 1;
}

.corner.tl { top: -4px; left: -4px; }
.corner.tr { top: -4px; right: -4px; }
.corner.bl { bottom: -4px; left: -4px; }
.corner.br { bottom: -4px; right: -4px; }

.pixel-dropdown.open .dropdown-trigger .corner {
  background: #39ff14;
}

.trigger-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trigger-arrow {
  width: 16px;
  height: 16px;
  color: #808080;
  transition: transform 0.1s steps(2);
}

.trigger-arrow.rotated {
  transform: rotate(180deg);
}

.pixel-dropdown.open .trigger-arrow {
  color: #39ff14;
}

/* Options */
.dropdown-options {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  background: #1a1a2e;
  border: 4px solid #000;
  box-shadow: 
    8px 8px 0 rgba(0, 0, 0, 0.8),
    inset 0 0 20px rgba(57, 255, 20, 0.05);
  z-index: 100;
  max-height: 240px;
  overflow: hidden;
}

.dropdown-options .corner {
  background: #39ff14;
}

.options-scroll {
  max-height: 232px;
  overflow-y: auto;
}

.dropdown-option {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: transparent;
  border: none;
  border-bottom: 2px solid #2a2a4c;
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: #c0c0c0;
  cursor: pointer;
  text-align: left;
  box-shadow: none;
}

.dropdown-option:last-child {
  border-bottom: none;
}

.dropdown-option:hover:not(.disabled),
.dropdown-option.highlighted:not(.disabled) {
  background: #2a2a4c;
  color: #fff;
}

.dropdown-option.selected {
  background: rgba(57, 255, 20, 0.1);
  color: #39ff14;
}

.dropdown-option.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.option-icon {
  font-size: 16px;
}

.option-label {
  flex: 1;
}

.option-check {
  width: 16px;
  height: 16px;
  color: #39ff14;
}

/* Compact variant */
.pixel-dropdown-wrapper.compact .dropdown-trigger {
  padding: 8px 12px;
  font-size: 16px;
}

.pixel-dropdown-wrapper.compact .dropdown-option {
  padding: 8px 12px;
  font-size: 14px;
}

/* Animation */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s steps(4);
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* Focus state */
.pixel-dropdown:focus-visible .dropdown-trigger {
  outline: 4px solid #39ff14;
  outline-offset: 2px;
}
</style>
