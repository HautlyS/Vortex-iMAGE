<script setup lang="ts">
import { ref, computed } from 'vue';

const props = withDefaults(defineProps<{
  modelValue?: string;
  placeholder?: string;
  type?: 'text' | 'password' | 'email' | 'number';
  disabled?: boolean;
  error?: string;
  label?: string;
  icon?: string;
  variant?: 'default' | 'terminal' | 'search';
}>(), {
  modelValue: '',
  placeholder: '',
  type: 'text',
  disabled: false,
  variant: 'default'
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
  focus: [];
  blur: [];
  enter: [];
}>();

const isFocused = ref(false);

const inputStyle = computed(() => ({
  '--focus-color': props.error 
    ? '#e43b44' 
    : '#39ff14'
}));

const handleInput = (e: Event) => {
  emit('update:modelValue', (e.target as HTMLInputElement).value);
};

const handleFocus = () => {
  isFocused.value = true;
  emit('focus');
};

const handleBlur = () => {
  isFocused.value = false;
  emit('blur');
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    emit('enter');
  }
};
</script>

<template>
  <div class="pixel-input-wrapper" :class="[variant]" :style="inputStyle">
    <label v-if="label" class="pixel-input-label">{{ label }}</label>
    
    <div 
      class="pixel-input-container"
      :class="{ focused: isFocused, error: !!error, disabled }"
    >
      <!-- Corner decorations -->
      <div class="input-corner tl"></div>
      <div class="input-corner tr"></div>
      <div class="input-corner bl"></div>
      <div class="input-corner br"></div>
      
      <!-- Terminal prefix -->
      <span v-if="variant === 'terminal'" class="terminal-prefix">&gt;</span>
      
      <!-- Search icon -->
      <span v-if="variant === 'search'" class="search-icon">
        <svg viewBox="0 0 16 16" width="16" height="16">
          <rect x="2" y="2" width="8" height="8" fill="none" stroke="currentColor" stroke-width="2"/>
          <rect x="10" y="10" width="4" height="2" fill="currentColor" transform="rotate(45 12 11)"/>
        </svg>
      </span>
      
      <!-- Custom icon -->
      <span v-else-if="icon" class="pixel-input-icon">{{ icon }}</span>
      
      <input
        class="pixel-input"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        @input="handleInput"
        @focus="handleFocus"
        @blur="handleBlur"
        @keydown="handleKeydown"
      />
      
      <!-- Blinking cursor -->
      <div v-if="isFocused && variant === 'terminal'" class="pixel-cursor"></div>
    </div>
    
    <span v-if="error" class="pixel-input-error">
      <svg viewBox="0 0 16 16" width="12" height="12">
        <rect x="7" y="3" width="2" height="6" fill="currentColor"/>
        <rect x="7" y="11" width="2" height="2" fill="currentColor"/>
      </svg>
      {{ error }}
    </span>
  </div>
</template>

<style scoped>
.pixel-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
  image-rendering: pixelated;
}

.pixel-input-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #808080;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.pixel-input-container {
  position: relative;
  display: flex;
  align-items: center;
  background: #000;
  border: 4px solid #3a3a5c;
  padding: 0 16px;
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    4px 4px 0 rgba(0, 0, 0, 0.5);
  transition: none;
}

/* Corner decorations */
.input-corner {
  position: absolute;
  width: 4px;
  height: 4px;
  background: #3a3a5c;
  z-index: 1;
}

.input-corner.tl { top: -4px; left: -4px; }
.input-corner.tr { top: -4px; right: -4px; }
.input-corner.bl { bottom: -4px; left: -4px; }
.input-corner.br { bottom: -4px; right: -4px; }

.pixel-input-container.focused {
  border-color: var(--focus-color);
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    0 0 0 2px var(--focus-color),
    0 0 16px var(--focus-color);
}

.pixel-input-container.focused .input-corner {
  background: var(--focus-color);
}

.pixel-input-container.error {
  border-color: #e43b44;
}

.pixel-input-container.error .input-corner {
  background: #e43b44;
}

.pixel-input-container.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Terminal prefix */
.terminal-prefix {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  color: #39ff14;
  margin-right: 8px;
  text-shadow: 0 0 8px #39ff14;
}

/* Search icon */
.search-icon {
  color: #808080;
  margin-right: 12px;
}

.pixel-input-container.focused .search-icon {
  color: var(--focus-color);
}

.pixel-input-icon {
  font-size: 16px;
  margin-right: 12px;
  color: #808080;
}

.pixel-input {
  flex: 1;
  font-family: 'VT323', monospace;
  font-size: 24px;
  background: transparent;
  border: none;
  color: #39ff14;
  padding: 12px 0;
  outline: none;
  caret-color: transparent;
  text-shadow: 0 0 4px rgba(57, 255, 20, 0.3);
}

.pixel-input::placeholder {
  color: #4a4a6c;
  text-shadow: none;
}

.pixel-input:disabled {
  cursor: not-allowed;
  color: #4a4a6c;
}

/* Blinking cursor */
.pixel-cursor {
  width: 12px;
  height: 20px;
  background: #39ff14;
  animation: cursor-blink 0.6s steps(1) infinite;
  box-shadow: 0 0 8px #39ff14;
}

@keyframes cursor-blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

/* Error message */
.pixel-input-error {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #e43b44;
  text-shadow: 0 0 8px rgba(228, 59, 68, 0.5);
}

/* Search variant */
.pixel-input-wrapper.search .pixel-input-container {
  border-radius: 0;
  background: rgba(0, 0, 0, 0.8);
}

.pixel-input-wrapper.search .pixel-input {
  font-size: 18px;
}

/* Terminal variant */
.pixel-input-wrapper.terminal .pixel-input-container {
  background: #0a0a14;
  border-color: #1a3a1a;
}

.pixel-input-wrapper.terminal .pixel-input-container.focused {
  border-color: #39ff14;
  box-shadow: 
    inset 4px 4px 0 rgba(0, 0, 0, 0.5),
    0 0 20px rgba(57, 255, 20, 0.3);
}
</style>
