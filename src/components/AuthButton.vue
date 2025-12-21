<script setup lang="ts">
import { useGitHubAuth } from '../composables/useGitHubAuth'
import { useAccentColor } from '../composables/useAccentColor'

const { user, loading, userCode, error, startLogin, logout } = useGitHubAuth()
const { accentHex } = useAccentColor()
</script>

<template>
  <div class="flex items-center gap-3">
    <!-- Logged in -->
    <Transition name="fade" mode="out-in">
      <div v-if="user" class="flex items-center gap-3 animate-fade-in">
        <div class="relative group">
          <img :src="user.avatar_url" :alt="user.login" class="w-10 h-10 rounded-full border-2 transition-all group-hover:scale-105" :style="{ borderColor: accentHex }" />
          <div class="absolute inset-0 rounded-full opacity-0 group-hover:opacity-30 transition-opacity blur-md" :style="{ backgroundColor: accentHex }"></div>
        </div>
        <span class="text-white font-mono text-sm hidden sm:block">{{ user.login }}</span>
        <button @click="logout" class="px-3 py-2 bg-amoled-gray/50 border border-red-500/30 text-red-400 rounded font-mono text-xs uppercase tracking-wider hover:bg-red-500/10 hover:border-red-500/50 transition-all">
          ⏻
        </button>
      </div>

      <!-- Code display -->
      <div v-else-if="userCode" class="relative px-5 py-4 bg-amoled-dark/80 backdrop-blur border rounded-lg font-mono overflow-hidden animate-fade-in" :style="{ borderColor: accentHex }">
        <div class="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-current to-transparent" :style="{ color: accentHex }"></div>
        <p class="text-xs text-gray-400 uppercase tracking-wider mb-1">◇ Enter code at GitHub</p>
        <p class="text-3xl tracking-[0.3em] glow-text" :style="{ color: accentHex, '--accent': accentHex }">{{ userCode }}</p>
        <div class="absolute bottom-0 left-0 w-full h-1 bg-amoled-gray overflow-hidden">
          <div class="h-full animate-pulse" :style="{ backgroundColor: accentHex, width: '100%' }"></div>
        </div>
      </div>

      <!-- Login button -->
      <div v-else class="flex flex-col items-end gap-2">
        <button @click="startLogin" :disabled="loading" class="group relative px-6 py-3 bg-amoled-dark/80 backdrop-blur border rounded-lg font-mono text-sm uppercase tracking-wider transition-all hover:scale-105 disabled:opacity-50 disabled:hover:scale-100 overflow-hidden" :style="{ borderColor: accentHex, color: accentHex }">
          <div class="absolute inset-0 opacity-0 group-hover:opacity-10 transition-opacity" :style="{ backgroundColor: accentHex }"></div>
          <div class="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-current to-transparent opacity-50"></div>
          <span class="relative flex items-center gap-2">
            <span :class="{ 'animate-spin': loading }">{{ loading ? '◌' : '◈' }}</span>
            {{ loading ? 'Connecting...' : 'Login' }}
          </span>
        </button>
        <Transition name="slide-up">
          <p v-if="error" class="text-red-400 text-xs font-mono px-2">{{ error }}</p>
        </Transition>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: all 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; transform: translateY(-5px); }
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.2s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateY(5px); }
</style>
