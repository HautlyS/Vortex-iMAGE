<script setup lang="ts">
import { computed } from 'vue'
import { usePhotoUpload } from '../composables/usePhotoUpload'
import { useAccentColor } from '../composables/useAccentColor'

const { 
  queue, isUploading, pendingCount, failedCount, successCount, currentUpload,
  selectFiles, retryFailed, removeFromQueue, clearCompleted, clearAll 
} = usePhotoUpload()
const { accentHex } = useAccentColor()

const statusIcon: Record<string, string> = { pending: '◯', uploading: '◌', success: '✓', failed: '✗' }

const statusColor = computed(() => ({
  pending: '#666',
  uploading: accentHex.value,
  success: '#39ff14',
  failed: '#ff2d6a'
}))
</script>

<template>
  <div class="space-y-3">
    <!-- Upload Button -->
    <button @click="selectFiles" class="group relative w-full px-6 py-5 bg-amoled-dark/80 backdrop-blur border-2 border-dashed rounded-lg font-mono text-sm uppercase tracking-wider transition-all hover:border-solid overflow-hidden" :style="{ borderColor: accentHex, color: accentHex }">
      <div class="absolute inset-0 opacity-0 group-hover:opacity-10 transition-opacity" :style="{ backgroundColor: accentHex }"></div>
      <div class="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-current to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
      <span class="relative flex items-center justify-center gap-3">
        <span class="text-2xl group-hover:animate-float">⬆</span>
        <span>Select Photos</span>
      </span>
    </button>

    <!-- Queue Panel -->
    <Transition name="slide">
      <div v-if="queue.length > 0" class="bg-amoled-dark/80 backdrop-blur border border-amoled-gray/50 rounded-lg overflow-hidden">
        <!-- Header -->
        <div class="flex items-center justify-between px-3 py-2 border-b border-amoled-gray/50 bg-amoled/50">
          <div class="flex items-center gap-3 text-xs font-mono">
            <span v-if="pendingCount" class="flex items-center gap-1 text-gray-400">
              <span class="w-1.5 h-1.5 rounded-full bg-gray-400"></span>{{ pendingCount }}
            </span>
            <span v-if="isUploading" class="flex items-center gap-1" :style="{ color: accentHex }">
              <span class="w-1.5 h-1.5 rounded-full animate-pulse" :style="{ backgroundColor: accentHex }"></span>1
            </span>
            <span v-if="successCount" class="flex items-center gap-1 text-cyber-green">
              <span class="w-1.5 h-1.5 rounded-full bg-cyber-green"></span>{{ successCount }}
            </span>
            <span v-if="failedCount" class="flex items-center gap-1 text-cyber-pink">
              <span class="w-1.5 h-1.5 rounded-full bg-cyber-pink"></span>{{ failedCount }}
            </span>
          </div>
          <div class="flex gap-1">
            <button v-if="failedCount" @click="retryFailed" class="text-xs font-mono px-2 py-1 rounded hover:bg-amoled-light transition-colors" :style="{ color: accentHex }">↻ Retry</button>
            <button v-if="successCount" @click="clearCompleted" class="text-xs font-mono text-gray-500 px-2 py-1 rounded hover:bg-amoled-light transition-colors">Clear ✓</button>
            <button @click="clearAll" class="text-xs font-mono text-gray-500 px-2 py-1 rounded hover:bg-amoled-light transition-colors">✕</button>
          </div>
        </div>

        <!-- Queue Items -->
        <TransitionGroup name="list" tag="div" class="max-h-48 overflow-y-auto">
          <div v-for="(item, i) in queue" :key="item.id" class="group relative" :style="{ animationDelay: `${i * 50}ms` }">
            <div class="flex items-center gap-3 px-3 py-2 border-b border-amoled-gray/30 last:border-0 hover:bg-amoled-light/30 transition-colors">
              <span class="text-sm transition-transform" :class="{ 'animate-spin': item.status === 'uploading' }" :style="{ color: statusColor[item.status] }">{{ statusIcon[item.status] }}</span>
              <div class="flex-1 min-w-0">
                <span class="text-xs font-mono truncate block" :class="item.status === 'failed' ? 'text-cyber-pink' : item.status === 'success' ? 'text-gray-500' : 'text-gray-300'">{{ item.name }}</span>
                <!-- Progress bar for uploading items -->
                <div v-if="item.status === 'uploading'" class="mt-1 h-1 bg-amoled-gray rounded-full overflow-hidden">
                  <div class="h-full rounded-full transition-all duration-300" :style="{ backgroundColor: accentHex, width: item.progress + '%' }"></div>
                </div>
              </div>
              <span v-if="item.status === 'uploading'" class="text-xs font-mono" :style="{ color: accentHex }">{{ item.progress }}%</span>
              <button v-if="item.status !== 'uploading'" @click="removeFromQueue(item.id)" class="text-gray-600 hover:text-gray-400 text-xs opacity-0 group-hover:opacity-100 transition-opacity">✕</button>
            </div>
          </div>
        </TransitionGroup>

        <!-- Current Upload Summary -->
        <div v-if="currentUpload" class="px-3 py-2 border-t border-amoled-gray/50 bg-amoled/50">
          <div class="flex justify-between text-xs font-mono text-gray-500">
            <span>↑ Uploading...</span>
            <span :style="{ color: accentHex }">{{ currentUpload.progress }}%</span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.slide-enter-active, .slide-leave-active { transition: all 0.3s ease; }
.slide-enter-from, .slide-leave-to { opacity: 0; transform: translateY(-10px); }
.list-enter-active, .list-leave-active { transition: all 0.3s ease; }
.list-enter-from { opacity: 0; transform: translateX(-20px); }
.list-leave-to { opacity: 0; transform: translateX(20px); }
.list-move { transition: transform 0.3s ease; }
</style>
