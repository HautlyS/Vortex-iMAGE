/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
defineProps<{
  hasChin?: boolean
  disabled?: boolean
}>()
</script>

<template>
  <article class="hover-card" :class="{ 'has-chin': hasChin, disabled }">
    <slot />
  </article>
</template>

<style scoped>
.hover-card {
  position: relative;
  display: flex;
  overflow: hidden;
  align-items: center;
  cursor: pointer;
  border-radius: var(--radius-lg, 16px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.hover-card::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.15) 0%,
    rgba(255, 255, 255, 0.05) 50%,
    transparent 100%
  );
  opacity: 0;
  mix-blend-mode: soft-light;
  pointer-events: none;
  transition: opacity 0.21s ease-out;
}

.hover-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

.hover-card:hover::after {
  opacity: 1;
}

.hover-card.has-chin {
  border-radius: var(--radius-lg, 16px) var(--radius-lg, 16px) 0 0;
}

.hover-card.has-chin::after {
  border-radius: var(--radius-lg, 16px) var(--radius-lg, 16px) 0 0;
}

.hover-card.disabled {
  cursor: default;
  pointer-events: none;
  opacity: 0.6;
}

.hover-card.disabled::after {
  display: none;
}
</style>