<template>
  <component
    :is="is"
    class="rainbow-button"
    :class="props.class"
  >
    <slot />
  </component>
</template>

<script setup lang="ts">
import { computed } from "vue";

interface RainbowButtonProps {
  class?: string;
  is?: string;
  speed?: number;
}

const props = withDefaults(defineProps<RainbowButtonProps>(), {
  speed: 2,
  is: "button",
});

const speedInSeconds = computed(() => `${props.speed}s`);
</script>

<style scoped>
.rainbow-button {
  --color-1: hsl(0 100% 63%);
  --color-2: hsl(270 100% 63%);
  --color-3: hsl(210 100% 63%);
  --color-4: hsl(195 100% 63%);
  --color-5: hsl(90 100% 63%);
  --speed: v-bind(speedInSeconds);
  
  position: relative;
  display: inline-flex;
  height: 3.25rem;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  border-radius: 0.75rem;
  border: calc(0.08 * 1rem) solid transparent;
  padding: 0.75rem 2.5rem;
  font-size: 1rem;
  font-weight: 600;
  color: #fff;
  background: 
    linear-gradient(#121213, #121213),
    linear-gradient(#121213 50%, rgba(18, 18, 19, 0.6) 80%, rgba(18, 18, 19, 0)),
    linear-gradient(90deg, var(--color-1), var(--color-5), var(--color-3), var(--color-4), var(--color-2));
  background-clip: padding-box, border-box, border-box;
  background-origin: border-box;
  background-size: 200%;
  animation: rainbow var(--speed) infinite linear;
  transition: opacity 0.2s;
}

.rainbow-button::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  z-index: -1;
  height: 200%;
  width: 150%;
  transform: translate(-50%, -50%);
  background: linear-gradient(90deg, var(--color-1), var(--color-5), var(--color-3), var(--color-4), var(--color-2));
  background-size: 200%;
  filter: blur(2.5rem);
  opacity: 1;
  animation: rainbow var(--speed) infinite linear;
}

.rainbow-button:disabled {
  pointer-events: none;
  opacity: 0.5;
}

.rainbow-button:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px #121213, 0 0 0 4px var(--color-3);
}

@keyframes rainbow {
  0% { background-position: 0; }
  100% { background-position: 200%; }
}
</style>
