/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

defineProps<{
  text: string
  tag?: 'h1' | 'h2' | 'h3' | 'span' | 'p'
  intensity?: 'low' | 'medium' | 'high'
}>()

const glitching = ref(false)
let interval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  interval = setInterval(() => {
    glitching.value = true
    setTimeout(() => glitching.value = false, 200)
  }, 3000 + Math.random() * 2000)
})

onUnmounted(() => { if (interval) clearInterval(interval) })
</script>

<template>
  <component 
    :is="tag || 'span'" 
    class="glitch" 
    :class="[intensity || 'medium', { active: glitching }]"
    :data-text="text"
  >
    {{ text }}
  </component>
</template>

<style scoped>
.glitch {
  position: relative;
  display: inline-block;
  color: var(--text-primary, #fff);
}

.glitch::before,
.glitch::after {
  content: attr(data-text);
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
}

.glitch::before {
  color: var(--cyber-pink, #ff2d6a);
  z-index: -1;
}

.glitch::after {
  color: var(--cyber-cyan, #00f0ff);
  z-index: -2;
}

.glitch.low.active::before {
  animation: glitch-1 0.2s linear;
  opacity: 0.8;
}
.glitch.low.active::after {
  animation: glitch-2 0.2s linear;
  opacity: 0.8;
}

.glitch.medium.active::before {
  animation: glitch-1 0.3s linear;
  opacity: 0.9;
}
.glitch.medium.active::after {
  animation: glitch-2 0.3s linear;
  opacity: 0.9;
}

.glitch.high::before,
.glitch.high::after {
  opacity: 0.7;
  animation: glitch-loop 4s infinite linear alternate-reverse;
}
.glitch.high::after {
  animation-delay: -2s;
}
.glitch.high.active::before {
  animation: glitch-1 0.4s linear, glitch-loop 4s infinite linear alternate-reverse;
}
.glitch.high.active::after {
  animation: glitch-2 0.4s linear, glitch-loop 4s infinite linear alternate-reverse;
  animation-delay: 0s, -2s;
}

@keyframes glitch-1 {
  0%, 100% { clip-path: inset(0 0 0 0); transform: translate(0); }
  20% { clip-path: inset(20% 0 60% 0); transform: translate(-3px, 2px); }
  40% { clip-path: inset(40% 0 40% 0); transform: translate(3px, -1px); }
  60% { clip-path: inset(60% 0 20% 0); transform: translate(-2px, 1px); }
  80% { clip-path: inset(80% 0 5% 0); transform: translate(2px, -2px); }
}

@keyframes glitch-2 {
  0%, 100% { clip-path: inset(0 0 0 0); transform: translate(0); }
  20% { clip-path: inset(60% 0 20% 0); transform: translate(2px, -1px); }
  40% { clip-path: inset(20% 0 60% 0); transform: translate(-2px, 2px); }
  60% { clip-path: inset(80% 0 5% 0); transform: translate(1px, -1px); }
  80% { clip-path: inset(5% 0 80% 0); transform: translate(-1px, 1px); }
}

@keyframes glitch-loop {
  0% { clip-path: inset(40% 0 61% 0); transform: translate(-1px, 0); }
  20% { clip-path: inset(92% 0 1% 0); transform: translate(1px, 0); }
  40% { clip-path: inset(43% 0 1% 0); transform: translate(-1px, 0); }
  60% { clip-path: inset(25% 0 58% 0); transform: translate(1px, 0); }
  80% { clip-path: inset(54% 0 7% 0); transform: translate(-1px, 0); }
  100% { clip-path: inset(58% 0 43% 0); transform: translate(0); }
}
</style>