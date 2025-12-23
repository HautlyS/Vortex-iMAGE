/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { useAccentColor, type AccentColor } from '../composables/useAccentColor'

const { accent, setAccent, colors } = useAccentColor()
</script>

<template>
  <div class="accent-picker">
    <button 
      v-for="(hex, name) in colors" 
      :key="name"
      @click="setAccent(name as AccentColor)"
      class="accent-btn"
      :class="{ active: accent === name }"
      :style="{ backgroundColor: hex, boxShadow: accent === name ? `0 0 12px ${hex}` : 'none' }"
      :title="name"
    />
  </div>
</template>

<style scoped>
.accent-picker {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.accent-btn {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  opacity: 0.6;
  transition: all var(--duration) var(--ease);
}

.accent-btn:hover { opacity: 1; transform: scale(1.1); }
.accent-btn.active { opacity: 1; transform: scale(1.15); }
</style>