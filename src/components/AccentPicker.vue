<script setup lang="ts">
import { useAccentColor, type AccentColor } from '../composables/useAccentColor'

const { accent, setAccent, colors } = useAccentColor()
</script>

<template>
  <div class="flex gap-1.5 p-1 bg-amoled-dark/50 rounded-full border border-amoled-gray/30">
    <button
      v-for="(hex, name) in colors"
      :key="name"
      @click="setAccent(name as AccentColor)"
      class="relative w-5 h-5 rounded-full transition-all duration-300 hover:scale-125"
      :class="accent === name ? 'scale-110' : 'opacity-60 hover:opacity-100'"
      :style="{ backgroundColor: hex }"
    >
      <span v-if="accent === name" class="absolute inset-0 rounded-full animate-ping opacity-30" :style="{ backgroundColor: hex }"></span>
      <span v-if="accent === name" class="absolute inset-[-3px] rounded-full border border-white/30"></span>
    </button>
  </div>
</template>
