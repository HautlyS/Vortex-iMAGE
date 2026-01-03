<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  src?: string
  name?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
  status?: 'online' | 'offline' | 'busy' | 'away'
  bordered?: boolean
}>(), {
  size: 'md',
  bordered: false
})

const sizeMap = { sm: 32, md: 40, lg: 56, xl: 80 }
const pixelSize = computed(() => sizeMap[props.size])

const initials = computed(() => {
  if (!props.name) return '?'
  return props.name.split(' ').map(n => n[0]).join('').slice(0, 2).toUpperCase()
})

const bgColor = computed(() => {
  if (!props.name) return 'var(--retro-accent-purple)'
  const colors = [
    'var(--retro-accent-pink)',
    'var(--retro-accent-green)',
    'var(--retro-accent-blue)',
    'var(--retro-accent-yellow)',
    'var(--retro-accent-purple)',
    'var(--retro-accent-cyan)',
    'var(--retro-accent-orange)'
  ]
  const hash = props.name.split('').reduce((a, c) => a + c.charCodeAt(0), 0)
  return colors[hash % colors.length]
})

const statusColor = computed(() => ({
  online: 'var(--retro-accent-green)',
  offline: 'var(--retro-text-muted)',
  busy: 'var(--retro-accent-red)',
  away: 'var(--retro-accent-yellow)'
}[props.status || 'offline']))
</script>

<template>
  <div 
    class="pixel-avatar"
    :class="[size, { bordered }]"
    :style="{ 
      '--avatar-size': `${pixelSize}px`,
      '--avatar-bg': bgColor
    }"
  >
    <img v-if="src" :src="src" :alt="name || 'Avatar'" class="avatar-image" />
    <span v-else class="avatar-initials">{{ initials }}</span>
    <span v-if="status" class="avatar-status" :style="{ '--status-color': statusColor }" />
  </div>
</template>

<style scoped>
.pixel-avatar {
  position: relative;
  width: var(--avatar-size);
  height: var(--avatar-size);
  background: var(--avatar-bg);
  border: 2px solid #000;
  box-shadow: 2px 2px 0 #000;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
}

.pixel-avatar.bordered {
  border-width: 3px;
  box-shadow: 3px 3px 0 #000, 0 0 0 2px var(--avatar-bg);
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: auto;
  pointer-events: auto;
}

.avatar-initials {
  font-family: 'Press Start 2P', monospace;
  font-size: calc(var(--avatar-size) * 0.3);
  color: #000;
  text-shadow: none;
  font-weight: normal;
}

.avatar-status {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: calc(var(--avatar-size) * 0.3);
  height: calc(var(--avatar-size) * 0.3);
  min-width: 8px;
  min-height: 8px;
  background: var(--status-color);
  border: 2px solid #000;
}

.pixel-avatar.sm .avatar-initials { font-size: 8px; }
.pixel-avatar.lg .avatar-initials { font-size: 14px; }
.pixel-avatar.xl .avatar-initials { font-size: 18px; }
</style>
