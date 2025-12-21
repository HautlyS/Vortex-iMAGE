<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'

const emit = defineEmits<{ complete: [] }>()
const frame = ref(0)
const progress = ref(0)

let interval: ReturnType<typeof setInterval> | null = null

// Pre-compute star positions once to avoid flickering
const stars = Array.from({ length: 20 }, () => ({
  left: `${Math.random() * 100}%`,
  top: `${Math.random() * 100}%`,
  delay: `${Math.random() * 2}s`,
  opacity: Math.random() * 0.5 + 0.2
}))

const frames = [
  `                    ·  ✦                    ·
         ·                      ✦                   ·
    ✦         ·            ·           ✦
                   ·    ✦        ·
         ✦                              ·
    ·              ✦         ·                 ✦
              ·                    ✦
                        ·`,
  `                    ·  ✦                    ·
         ·              ████                ·
    ✦         ·       ████████      ✦
                   ·  ████████  ·
         ✦            ████████          ·
    ·              ✦    ████   ·               ✦
              ·                    ✦
                        ·`,
  `                    ·  ✦                    ·
         ·              ████                ·
    ✦         ·       ████████      ✦
              △    ·  ████████  ·
         ✦   ╱█╲         ████████          ·
    ·       ╱███╲   ✦    ████   ·               ✦
           ▼▼▼▼▼               ✦
                        ·`,
  `                    ·  ✦         △          ·
         ·              ████    ╱█╲         ·
    ✦         ·       ████████ ╱███╲  ✦
                   ·  ████████ ▼▼▼▼▼
         ✦            ████████          ·
    ·              ✦    ████   ·               ✦
              ·                    ✦
                        ·`,
  `              ◇         ✦         △          ·
         ·        ◆     ████    ╱█╲         ·
    ✦       ◇   ·     ████████ ╱███╲  ✦
                   ·  ████████ ▼▼▼▼▼
         ✦      ◆     ████████          ·
    ·        ◇     ✦    ████   ·               ✦
              ·    ◆               ✦
                        ·`,
  `              ◇         ✦                    ·
         ·        ◆     ████                ·
    ✦       ◇   ·     ████████        ✦
                   ·  ████████    △
         ✦      ◆     ████████   ╱█╲     ·
    ·        ◇     ✦    ████    ╱███╲          ✦
              ·    ◆            ▼▼▼▼▼
                        ·`,
  `                    ·  ✦                    ·
         ·              ████                ·
    ✦         ·       ████████      ✦    ▓▓▓
                   ·  ████████  ·       ▓▓▓▓▓
         ✦            ████████    △     ▓▓▓▓▓
    ·              ✦    ████    ╱█╲      ▓▓▓    ✦
              ·                ╱███╲
                        ·      ▼▼▼▼▼`,
  `                    ·  ✦                    ·
         ·              ████                ·
    ✦         ·       ████████      ✦    ▓▓▓
                   ·  ████████  ·       ▓▓▓▓▓
         ✦            ████████          ▓█▓▓▓
    ·              ✦    ████           ▓╱█╲▓    ✦
              ·                       ▓╱███╲▓
                        ·              ▓▓▓▓▓`,
]

const messages = [
  'INITIALIZING QUANTUM DRIVE...',
  'SCANNING NEBULA CLUSTERS...',
  'IGNITION SEQUENCE ACTIVE...',
  'ENTERING HYPERSPACE...',
  'ASTEROID FIELD DETECTED!',
  'EVASIVE MANEUVERS...',
  'NEW SYSTEM DISCOVERED!',
  'LANDING PROTOCOL ENGAGED...',
]

const currentFrame = computed(() => frames[Math.min(Math.floor(frame.value / 2), frames.length - 1)])
const currentMessage = computed(() => messages[Math.min(Math.floor(frame.value / 2), messages.length - 1)])

onMounted(() => {
  interval = setInterval(() => {
    frame.value++
    progress.value = Math.min((frame.value / 16) * 100, 100)
    if (frame.value >= 16) {
      if (interval) clearInterval(interval)
      interval = null
      setTimeout(() => emit('complete'), 500)
    }
  }, 500)
})

onUnmounted(() => { if (interval) clearInterval(interval) })
</script>

<template>
  <div class="fixed inset-0 bg-amoled flex flex-col items-center justify-center font-mono z-50 overflow-hidden">
    <!-- Ambient stars - positions pre-computed -->
    <div class="absolute inset-0 overflow-hidden">
      <div v-for="(star, i) in stars" :key="i" class="absolute w-1 h-1 bg-white rounded-full animate-pulse" :style="{ left: star.left, top: star.top, animationDelay: star.delay, opacity: star.opacity }"></div>
    </div>

    <!-- Glow orbs -->
    <div class="absolute top-1/4 left-1/4 w-64 h-64 bg-cyber-cyan/10 rounded-full blur-[100px] animate-pulse"></div>
    <div class="absolute bottom-1/4 right-1/4 w-64 h-64 bg-cyber-pink/10 rounded-full blur-[100px] animate-pulse" style="animation-delay: 1s"></div>

    <!-- ASCII Art -->
    <div class="relative">
      <pre class="text-cyber-cyan text-xs sm:text-sm leading-tight select-none transition-all duration-300" :class="{ 'animate-glitch': frame % 4 === 0 }">{{ currentFrame }}</pre>
      <div class="absolute inset-0 bg-gradient-to-t from-amoled via-transparent to-amoled pointer-events-none"></div>
    </div>

    <!-- Info Panel -->
    <div class="mt-10 text-center relative">
      <div class="inline-block px-6 py-4 bg-amoled-dark/50 backdrop-blur border border-amoled-gray/50 rounded-lg">
        <p class="text-cyber-pink text-sm tracking-[0.2em] mb-4 h-5">
          <span class="inline-block">{{ currentMessage }}</span>
        </p>
        
        <!-- Progress bar -->
        <div class="w-64 h-1.5 bg-amoled-gray rounded-full overflow-hidden relative">
          <div class="absolute inset-0 bg-gradient-to-r from-cyber-pink via-cyber-purple to-cyber-cyan opacity-30"></div>
          <div class="h-full bg-gradient-to-r from-cyber-pink via-cyber-purple to-cyber-cyan rounded-full transition-all duration-300 relative" :style="{ width: progress + '%' }">
            <div class="absolute right-0 top-1/2 -translate-y-1/2 w-2 h-2 bg-white rounded-full shadow-lg shadow-cyber-cyan/50"></div>
          </div>
        </div>
        
        <div class="flex justify-between mt-2 text-xs text-gray-600">
          <span>◇ LOADING</span>
          <span class="text-cyber-cyan">{{ Math.round(progress) }}%</span>
        </div>
      </div>
    </div>

    <!-- Bottom decoration -->
    <div class="absolute bottom-8 flex gap-8 text-xs text-gray-700 font-mono">
      <span class="animate-pulse">◈ QUANTUM</span>
      <span class="animate-pulse" style="animation-delay: 0.3s">◈ SECURE</span>
      <span class="animate-pulse" style="animation-delay: 0.6s">◈ SYNCED</span>
    </div>
  </div>
</template>

<style scoped>
@keyframes glitch {
  0%, 90%, 100% { transform: translate(0); filter: none; }
  92% { transform: translate(-2px, 1px); filter: hue-rotate(90deg); }
  94% { transform: translate(2px, -1px); filter: hue-rotate(-90deg); }
  96% { transform: translate(-1px, 2px); }
  98% { transform: translate(1px, -2px); }
}
.animate-glitch { animation: glitch 0.3s ease; }
</style>
