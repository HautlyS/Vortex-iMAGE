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
}>(), {
  modelValue: '',
  placeholder: '',
  type: 'text',
  disabled: false
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
  focus: [];
  blur: [];
}>();

const isFocused = ref(false);

const inputStyle = computed(() => ({
  '--focus-color': props.error 
    ? 'var(--retro-accent-red, #ff3b30)' 
    : 'var(--retro-accent-green, #00ff87)'
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
</script>

<template>
  <div class="pixel-input-wrapper" :style="inputStyle">
    <label v-if="label" class="pixel-input-label">{{ label }}</label>
    <div 
      class="pixel-input-container"
      :class="{ focused: isFocused, error: !!error, disabled }"
    >
      <span v-if="icon" class="pixel-input-icon">{{ icon }}</span>
      <input
        class="pixel-input"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        @input="handleInput"
        @focus="handleFocus"
        @blur="handleBlur"
      />
      <div class="pixel-input-cursor" v-if="isFocused" />
    </div>
    <span v-if="error" class="pixel-input-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.pixel-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.pixel-input-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--retro-text-muted, #9d8ec2);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.pixel-input-container {
  position: relative;
  display: flex;
  align-items: center;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 3px solid var(--retro-bg-lighter, #2d1f4d);
  padding: 0 12px;
  transition: all 0.1s steps(2);
}

.pixel-input-container.focused {
  border-color: var(--focus-color);
  box-shadow: 0 0 0 1px var(--focus-color), 0 0 15px var(--focus-color);
}

.pixel-input-container.error {
  border-color: var(--retro-accent-red, #ff3b30);
}

.pixel-input-container.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pixel-input-icon {
  font-size: 14px;
  margin-right: 8px;
  color: var(--retro-text-muted, #9d8ec2);
}

.pixel-input {
  flex: 1;
  font-family: 'VT323', monospace;
  font-size: 20px;
  background: transparent;
  border: none;
  color: var(--retro-accent-green, #00ff87);
  padding: 12px 0;
  outline: none;
  caret-color: transparent;
}

.pixel-input::placeholder {
  color: var(--retro-text-muted, #9d8ec2);
  opacity: 0.5;
}

.pixel-input:disabled {
  cursor: not-allowed;
}

.pixel-input-cursor {
  position: absolute;
  right: 12px;
  width: 8px;
  height: 18px;
  background: var(--retro-accent-green, #00ff87);
  animation: blink 0.8s steps(1) infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.pixel-input-error {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--retro-accent-red, #ff3b30);
  text-shadow: 0 0 8px var(--retro-accent-red);
}
</style>
