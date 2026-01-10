<script setup lang="ts">
import { ref, computed } from 'vue';

const props = withDefaults(defineProps<{
  modelValue: string;
  placeholder?: string;
  disabled?: boolean;
  loading?: boolean;
  variant?: 'default' | 'compact' | 'expanded';
}>(), {
  modelValue: '',
  placeholder: 'SEARCH...',
  disabled: false,
  loading: false,
  variant: 'default'
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
  search: [query: string];
  clear: [];
}>();

const isFocused = ref(false);

const hasValue = computed(() => props.modelValue.length > 0);

const handleInput = (e: Event) => {
  emit('update:modelValue', (e.target as HTMLInputElement).value);
};

const handleSubmit = () => {
  emit('search', props.modelValue);
};

const handleClear = () => {
  emit('update:modelValue', '');
  emit('clear');
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    handleSubmit();
  }
  if (e.key === 'Escape') {
    handleClear();
  }
};
</script>

<template>
  <div 
    class="pixel-search" 
    :class="[variant, { focused: isFocused, disabled, loading, 'has-value': hasValue }]"
  >
    <!-- Search icon -->
    <div class="search-icon">
      <svg v-if="!loading" viewBox="0 0 16 16">
        <rect x="4" y="2" width="6" height="2" fill="currentColor"/>
        <rect x="2" y="4" width="2" height="6" fill="currentColor"/>
        <rect x="10" y="4" width="2" height="6" fill="currentColor"/>
        <rect x="4" y="10" width="6" height="2" fill="currentColor"/>
        <rect x="10" y="10" width="2" height="2" fill="currentColor"/>
        <rect x="12" y="12" width="2" height="2" fill="currentColor"/>
        <rect x="14" y="14" width="2" height="2" fill="currentColor"/>
      </svg>
      <div v-else class="search-spinner">
        <span></span>
        <span></span>
        <span></span>
      </div>
    </div>
    
    <!-- Input -->
    <input
      type="text"
      class="search-input"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      @input="handleInput"
      @focus="isFocused = true"
      @blur="isFocused = false"
      @keydown="handleKeydown"
    />
    
    <!-- Clear button -->
    <button 
      v-if="hasValue && !loading" 
      class="clear-btn"
      @click="handleClear"
      type="button"
    >
      <svg viewBox="0 0 16 16">
        <rect x="2" y="6" width="4" height="4" fill="currentColor"/>
        <rect x="6" y="2" width="4" height="4" fill="currentColor"/>
        <rect x="6" y="10" width="4" height="4" fill="currentColor"/>
        <rect x="10" y="6" width="4" height="4" fill="currentColor"/>
      </svg>
    </button>
    
    <!-- Search button -->
    <button 
      v-if="variant === 'expanded'"
      class="search-btn"
      @click="handleSubmit"
      :disabled="disabled || !hasValue"
      type="button"
    >
      GO
    </button>
    
    <!-- Corner decorations -->
    <div class="corner tl"></div>
    <div class="corner tr"></div>
    <div class="corner bl"></div>
    <div class="corner br"></div>
  </div>
</template>

<style scoped>
.pixel-search {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  background: #000;
  border: 4px solid #3a3a5c;
  padding: 8px 16px;
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    4px 4px 0 rgba(0, 0, 0, 0.5);
  image-rendering: pixelated;
}

/* Corners */
.corner {
  position: absolute;
  width: 4px;
  height: 4px;
  background: #3a3a5c;
}

.corner.tl { top: -4px; left: -4px; }
.corner.tr { top: -4px; right: -4px; }
.corner.bl { bottom: -4px; left: -4px; }
.corner.br { bottom: -4px; right: -4px; }

.pixel-search.focused {
  border-color: #39ff14;
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    0 0 0 2px #39ff14,
    0 0 16px rgba(57, 255, 20, 0.3);
}

.pixel-search.focused .corner {
  background: #39ff14;
}

.pixel-search.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Search icon */
.search-icon {
  width: 20px;
  height: 20px;
  color: #808080;
  flex-shrink: 0;
}

.pixel-search.focused .search-icon {
  color: #39ff14;
}

.search-icon svg {
  width: 100%;
  height: 100%;
}

/* Spinner */
.search-spinner {
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.search-spinner span {
  width: 4px;
  height: 4px;
  background: #39ff14;
  animation: spinner-dot 0.6s steps(3) infinite;
}

.search-spinner span:nth-child(2) { animation-delay: 0.2s; }
.search-spinner span:nth-child(3) { animation-delay: 0.4s; }

@keyframes spinner-dot {
  0%, 100% { transform: translateY(0); opacity: 0.4; }
  50% { transform: translateY(-4px); opacity: 1; }
}

/* Input */
.search-input {
  flex: 1;
  font-family: 'VT323', monospace;
  font-size: 20px;
  background: transparent;
  border: none;
  color: #39ff14;
  outline: none;
  min-width: 0;
  text-shadow: 0 0 4px rgba(57, 255, 20, 0.3);
}

.search-input::placeholder {
  color: #4a4a6c;
  text-shadow: none;
}

.search-input:disabled {
  cursor: not-allowed;
}

/* Clear button */
.clear-btn {
  width: 24px;
  height: 24px;
  padding: 4px;
  background: transparent;
  border: none;
  color: #808080;
  cursor: pointer;
  flex-shrink: 0;
  box-shadow: none;
}

.clear-btn:hover {
  color: #e43b44;
  transform: none;
}

.clear-btn svg {
  width: 100%;
  height: 100%;
}

/* Search button (expanded variant) */
.search-btn {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  padding: 8px 16px;
  background: linear-gradient(180deg, #39ff14 0%, #2d8a1a 100%);
  color: #000;
  border: 2px solid #000;
  cursor: pointer;
  box-shadow: 2px 2px 0 #000;
}

.search-btn:hover:not(:disabled) {
  background: linear-gradient(180deg, #8cff7a 0%, #39ff14 100%);
  transform: translate(-1px, -1px);
  box-shadow: 3px 3px 0 #000;
}

.search-btn:active:not(:disabled) {
  transform: translate(2px, 2px);
  box-shadow: none;
}

.search-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Compact variant */
.pixel-search.compact {
  padding: 4px 12px;
}

.pixel-search.compact .search-input {
  font-size: 16px;
}

.pixel-search.compact .search-icon {
  width: 16px;
  height: 16px;
}

/* Expanded variant */
.pixel-search.expanded {
  padding: 12px 16px;
}

.pixel-search.expanded .search-input {
  font-size: 24px;
}
</style>
