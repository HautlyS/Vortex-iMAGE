<script setup lang="ts">
import { ref, computed, watch } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  suggestions?: string[]
  loading?: boolean
  autofocus?: boolean
}>(), {
  modelValue: '',
  placeholder: 'Pesquisar...',
  suggestions: () => [],
  loading: false,
  autofocus: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  search: [query: string]
  clear: []
}>()

const inputRef = ref<HTMLInputElement | null>(null)
const isFocused = ref(false)
const showSuggestions = ref(false)
const selectedIndex = ref(-1)

const filteredSuggestions = computed(() => {
  if (!props.modelValue || props.modelValue.length < 2) return []
  const query = props.modelValue.toLowerCase()
  return props.suggestions.filter(s => s.toLowerCase().includes(query)).slice(0, 5)
})

const handleInput = (e: Event) => {
  const value = (e.target as HTMLInputElement).value
  emit('update:modelValue', value)
  showSuggestions.value = true
  selectedIndex.value = -1
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, filteredSuggestions.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, -1)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    if (selectedIndex.value >= 0) {
      selectSuggestion(filteredSuggestions.value[selectedIndex.value])
    } else {
      emit('search', props.modelValue)
    }
    showSuggestions.value = false
  } else if (e.key === 'Escape') {
    showSuggestions.value = false
    inputRef.value?.blur()
  }
}

const selectSuggestion = (suggestion: string) => {
  emit('update:modelValue', suggestion)
  emit('search', suggestion)
  showSuggestions.value = false
}

const clear = () => {
  emit('update:modelValue', '')
  emit('clear')
  inputRef.value?.focus()
}

watch(() => props.modelValue, (val) => {
  if (!val) showSuggestions.value = false
})
</script>

<template>
  <div class="pixel-search" :class="{ focused: isFocused, loading }">
    <div class="search-icon">
      <svg v-if="!loading" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <div v-else class="search-spinner" />
    </div>
    
    <input
      ref="inputRef"
      type="text"
      :value="modelValue"
      :placeholder="placeholder"
      :autofocus="autofocus"
      class="search-input"
      @input="handleInput"
      @focus="isFocused = true; showSuggestions = true"
      @blur="isFocused = false; setTimeout(() => showSuggestions = false, 150)"
      @keydown="handleKeydown"
    />
    
    <button v-if="modelValue" class="search-clear" @click="clear" aria-label="Limpar">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>

    <Transition name="dropdown">
      <div v-if="showSuggestions && filteredSuggestions.length > 0" class="search-suggestions">
        <button
          v-for="(suggestion, index) in filteredSuggestions"
          :key="suggestion"
          class="suggestion-item"
          :class="{ selected: index === selectedIndex }"
          @mousedown.prevent="selectSuggestion(suggestion)"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
          </svg>
          <span>{{ suggestion }}</span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.pixel-search {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--retro-bg-dark, #0a0612);
  border: 2px solid var(--retro-bg-lighter, #2d1f4d);
  padding: 8px 14px;
  transition: all 0.15s ease;
}

.pixel-search.focused {
  border-color: var(--retro-accent-cyan, #00d4ff);
  box-shadow: 0 0 0 1px var(--retro-accent-cyan), 0 0 20px rgba(0, 212, 255, 0.2);
}

.search-icon {
  width: 18px;
  height: 18px;
  color: var(--retro-text-muted, #9d8ec2);
  flex-shrink: 0;
}

.search-icon svg { width: 100%; height: 100%; }

.search-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--retro-bg-lighter);
  border-top-color: var(--retro-accent-cyan);
  animation: spin 0.6s steps(8) infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--retro-accent-green, #00ff87);
  font-family: 'VT323', monospace;
  font-size: 18px;
  padding: 0;
  outline: none;
}

.search-input::placeholder {
  color: var(--retro-text-muted, #9d8ec2);
  opacity: 0.6;
}

.search-clear {
  width: 20px;
  height: 20px;
  padding: 2px;
  background: transparent;
  border: none;
  box-shadow: none;
  color: var(--retro-text-muted, #9d8ec2);
  flex-shrink: 0;
}

.search-clear:hover { color: var(--retro-accent-red, #ff4757); }

.search-suggestions {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--retro-bg-panel, #1a1030);
  border: 2px solid var(--retro-bg-lighter, #2d1f4d);
  box-shadow: 4px 4px 0 rgba(0,0,0,0.5);
  z-index: 100;
  max-height: 200px;
  overflow-y: auto;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--retro-bg-lighter, #2d1f4d);
  box-shadow: none;
  color: var(--retro-text-muted, #9d8ec2);
  font-family: 'VT323', monospace;
  font-size: 16px;
  text-align: left;
  cursor: pointer;
}

.suggestion-item:last-child { border-bottom: none; }

.suggestion-item:hover,
.suggestion-item.selected {
  background: var(--retro-bg-card, #251842);
  color: var(--retro-accent-green, #00ff87);
}

.suggestion-item svg {
  width: 14px;
  height: 14px;
  opacity: 0.5;
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
