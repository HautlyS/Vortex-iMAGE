<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ complete: [] }>()
const pixels = ref<{ x: number; y: number; color: string; delay: number }[]>([])
const GRID = 6
const COLORS = ['#00f0ff', '#ff2d6a', '#b026ff', '#39ff14', '#ffd700']

onMounted(() => {
  // Create pixel grid
  for (let y = 0; y < GRID; y++) {
    for (let x = 0; x < GRID; x++) {
      pixels.value.push({
        x, y,
        color: COLORS[Math.floor(Math.random() * COLORS.length)],
        delay: Math.random() * 0.5
      })
    }
  }
  // Auto-complete after max 2s
  setTimeout(() => emit('complete'), 2000)
})

onUnmounted(() => {})
</script>

<template>
  <div class="pixel-loader">
    <div class="pixel-grid">
      <div
        v-for="(p, i) in pixels"
        :key="i"
        class="pixel"
        :style="{
          '--x': p.x,
          '--y': p.y,
          '--color': p.color,
          '--delay': p.delay + 's'
        }"
      />
    </div>
  </div>
</template>

<style scoped>
.pixel-loader {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.pixel-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 4px;
  width: 120px;
  height: 120px;
}

.pixel {
  width: 100%;
  aspect-ratio: 1;
  background: var(--color);
  border-radius: 2px;
  animation: pixel-mix 0.6s ease-in-out infinite alternate;
  animation-delay: var(--delay);
  box-shadow: 0 0 8px var(--color);
}

@keyframes pixel-mix {
  0% {
    transform: scale(0.3) rotate(0deg);
    opacity: 0.3;
  }
  50% {
    transform: scale(1.1) rotate(180deg);
    opacity: 1;
  }
  100% {
    transform: scale(0.8) rotate(360deg);
    opacity: 0.6;
  }
}
</style>
