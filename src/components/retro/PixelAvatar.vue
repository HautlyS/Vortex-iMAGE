<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  src?: string;
  name?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  variant?: 'default' | 'rounded' | 'status';
  status?: 'online' | 'offline' | 'away' | 'busy';
  level?: number;
}>(), {
  size: 'md',
  variant: 'default'
});

const initials = computed(() => {
  if (!props.name) return '?';
  return props.name
    .split(' ')
    .map(word => word[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);
});

const bgColor = computed(() => {
  if (!props.name) return '#3a3a5c';
  const colors = ['#e43b44', '#f77622', '#feae34', '#63c74d', '#0099db', '#9b5de5', '#f15bb5'];
  const hash = props.name.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
  return colors[hash % colors.length];
});

const statusColors = {
  online: '#39ff14',
  offline: '#808080',
  away: '#feae34',
  busy: '#e43b44'
};
</script>

<template>
  <div class="pixel-avatar" :class="[size, variant]">
    <!-- Avatar container -->
    <div class="avatar-container" :style="{ '--bg-color': bgColor }">
      <!-- Image -->
      <img v-if="src" :src="src" :alt="name || 'Avatar'" class="avatar-image" />
      
      <!-- Initials fallback -->
      <span v-else class="avatar-initials">{{ initials }}</span>
      
      <!-- Pixel frame -->
      <div class="avatar-frame">
        <div class="frame-corner tl"></div>
        <div class="frame-corner tr"></div>
        <div class="frame-corner bl"></div>
        <div class="frame-corner br"></div>
      </div>
      
      <!-- Shine effect -->
      <div class="avatar-shine"></div>
    </div>
    
    <!-- Status indicator -->
    <div v-if="variant === 'status' && status" class="status-indicator" :class="status">
      <span class="status-dot" :style="{ background: statusColors[status] }"></span>
    </div>
    
    <!-- Level badge -->
    <div v-if="level" class="level-badge">
      <span>{{ level }}</span>
    </div>
  </div>
</template>

<style scoped>
.pixel-avatar {
  position: relative;
  display: inline-flex;
  image-rendering: pixelated;
}

/* Sizes */
.pixel-avatar.sm .avatar-container { width: 32px; height: 32px; }
.pixel-avatar.md .avatar-container { width: 48px; height: 48px; }
.pixel-avatar.lg .avatar-container { width: 64px; height: 64px; }
.pixel-avatar.xl .avatar-container { width: 96px; height: 96px; }

.avatar-container {
  position: relative;
  background: var(--bg-color);
  border: 4px solid #000;
  box-shadow: 4px 4px 0 rgba(0, 0, 0, 0.5);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Rounded variant */
.pixel-avatar.rounded .avatar-container {
  border-radius: 50%;
}

.pixel-avatar.rounded .frame-corner {
  display: none;
}

/* Image */
.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
}

/* Initials */
.avatar-initials {
  font-family: 'Press Start 2P', monospace;
  color: #fff;
  text-shadow: 1px 1px 0 rgba(0, 0, 0, 0.5);
}

.pixel-avatar.sm .avatar-initials { font-size: 8px; }
.pixel-avatar.md .avatar-initials { font-size: 12px; }
.pixel-avatar.lg .avatar-initials { font-size: 16px; }
.pixel-avatar.xl .avatar-initials { font-size: 24px; }

/* Frame corners */
.avatar-frame {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.frame-corner {
  position: absolute;
  width: 4px;
  height: 4px;
  background: #f15bb5;
}

.frame-corner.tl { top: 0; left: 0; }
.frame-corner.tr { top: 0; right: 0; }
.frame-corner.bl { bottom: 0; left: 0; }
.frame-corner.br { bottom: 0; right: 0; }

/* Shine effect */
.avatar-shine {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 30%;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.3) 0%, transparent 100%);
  pointer-events: none;
}

/* Status indicator */
.status-indicator {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 16px;
  height: 16px;
  background: #1a1a2e;
  border: 2px solid #000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-dot {
  width: 8px;
  height: 8px;
}

.status-indicator.online .status-dot {
  animation: status-pulse 1s steps(2) infinite;
}

@keyframes status-pulse {
  0%, 100% { box-shadow: 0 0 4px #39ff14; }
  50% { box-shadow: 0 0 8px #39ff14; }
}

/* Level badge */
.level-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 20px;
  height: 20px;
  padding: 0 4px;
  background: linear-gradient(180deg, #feae34 0%, #c68b28 100%);
  border: 2px solid #000;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5);
}

.level-badge span {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #000;
}

/* Hover effect */
.pixel-avatar:hover .avatar-container {
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 rgba(0, 0, 0, 0.5);
}
</style>
