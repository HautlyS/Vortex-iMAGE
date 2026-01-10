<script setup lang="ts">
defineProps<{
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info' | 'legendary' | 'epic' | 'rare';
  size?: 'sm' | 'md' | 'lg';
  glow?: boolean;
  animated?: boolean;
  icon?: 'star' | 'heart' | 'coin' | 'shield' | 'crown';
}>();
</script>

<template>
  <span class="pixel-badge" :class="[variant, size, { glow, animated }]">
    <!-- Icon -->
    <svg v-if="icon === 'star'" class="badge-icon" viewBox="0 0 16 16">
      <rect x="6" y="0" width="4" height="4" fill="currentColor"/>
      <rect x="0" y="6" width="16" height="4" fill="currentColor"/>
      <rect x="2" y="10" width="4" height="4" fill="currentColor"/>
      <rect x="10" y="10" width="4" height="4" fill="currentColor"/>
    </svg>
    
    <svg v-else-if="icon === 'heart'" class="badge-icon" viewBox="0 0 16 16">
      <rect x="2" y="2" width="4" height="4" fill="currentColor"/>
      <rect x="10" y="2" width="4" height="4" fill="currentColor"/>
      <rect x="0" y="4" width="16" height="4" fill="currentColor"/>
      <rect x="2" y="8" width="12" height="4" fill="currentColor"/>
      <rect x="6" y="12" width="4" height="4" fill="currentColor"/>
    </svg>
    
    <svg v-else-if="icon === 'coin'" class="badge-icon" viewBox="0 0 16 16">
      <rect x="4" y="0" width="8" height="2" fill="currentColor"/>
      <rect x="2" y="2" width="12" height="12" fill="currentColor"/>
      <rect x="4" y="14" width="8" height="2" fill="currentColor"/>
    </svg>
    
    <svg v-else-if="icon === 'shield'" class="badge-icon" viewBox="0 0 16 16">
      <rect x="2" y="0" width="12" height="4" fill="currentColor"/>
      <rect x="0" y="2" width="16" height="8" fill="currentColor"/>
      <rect x="2" y="10" width="12" height="2" fill="currentColor"/>
      <rect x="4" y="12" width="8" height="2" fill="currentColor"/>
      <rect x="6" y="14" width="4" height="2" fill="currentColor"/>
    </svg>
    
    <svg v-else-if="icon === 'crown'" class="badge-icon" viewBox="0 0 16 16">
      <rect x="0" y="4" width="4" height="4" fill="currentColor"/>
      <rect x="6" y="0" width="4" height="4" fill="currentColor"/>
      <rect x="12" y="4" width="4" height="4" fill="currentColor"/>
      <rect x="0" y="8" width="16" height="6" fill="currentColor"/>
      <rect x="2" y="14" width="12" height="2" fill="currentColor"/>
    </svg>
    
    <span class="badge-text"><slot /></span>
    
    <!-- Shine effect -->
    <span v-if="animated" class="badge-shine"></span>
  </span>
</template>

<style scoped>
.pixel-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-family: 'Press Start 2P', monospace;
  text-transform: uppercase;
  letter-spacing: 1px;
  border: 2px solid #000;
  position: relative;
  overflow: hidden;
  image-rendering: pixelated;
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5);
}

/* Sizes */
.pixel-badge.sm {
  font-size: 6px;
  padding: 4px 8px;
}

.pixel-badge.md {
  font-size: 8px;
  padding: 6px 12px;
}

.pixel-badge.lg {
  font-size: 10px;
  padding: 8px 16px;
}

.badge-icon {
  width: 1.2em;
  height: 1.2em;
}

/* Default variant */
.pixel-badge {
  background: linear-gradient(180deg, #4a4a6c 0%, #3a3a5c 100%);
  color: #fff;
}

/* Success */
.pixel-badge.success {
  background: linear-gradient(180deg, #63c74d 0%, #3e8948 100%);
  color: #000;
}

/* Warning */
.pixel-badge.warning {
  background: linear-gradient(180deg, #feae34 0%, #c68b28 100%);
  color: #000;
}

/* Danger */
.pixel-badge.danger {
  background: linear-gradient(180deg, #e43b44 0%, #a82835 100%);
  color: #fff;
}

/* Info */
.pixel-badge.info {
  background: linear-gradient(180deg, #0099db 0%, #006b99 100%);
  color: #fff;
}

/* Legendary - Gold with rainbow effect */
.pixel-badge.legendary {
  background: linear-gradient(180deg, #ffd700 0%, #ff8c00 50%, #ffd700 100%);
  color: #000;
  border-color: #8b4513;
  animation: legendary-shine 2s steps(8) infinite;
}

@keyframes legendary-shine {
  0%, 100% { filter: brightness(1); }
  50% { filter: brightness(1.3); }
}

/* Epic - Purple */
.pixel-badge.epic {
  background: linear-gradient(180deg, #9b5de5 0%, #7b3dc5 100%);
  color: #fff;
  border-color: #4a1d85;
}

/* Rare - Blue */
.pixel-badge.rare {
  background: linear-gradient(180deg, #00d4ff 0%, #0099db 100%);
  color: #000;
  border-color: #006b99;
}

/* Glow effect */
.pixel-badge.glow {
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 12px currentColor;
}

.pixel-badge.success.glow {
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 12px rgba(99, 199, 77, 0.5);
}

.pixel-badge.danger.glow {
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 12px rgba(228, 59, 68, 0.5);
}

.pixel-badge.legendary.glow {
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 16px rgba(255, 215, 0, 0.6);
}

.pixel-badge.epic.glow {
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5), 0 0 12px rgba(155, 93, 229, 0.5);
}

/* Animated shine */
.badge-shine {
  position: absolute;
  top: 0;
  left: -100%;
  width: 50%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.4), transparent);
  transform: skewX(-20deg);
}

.pixel-badge.animated .badge-shine {
  animation: badge-shine 2s steps(8) infinite;
}

@keyframes badge-shine {
  0% { left: -100%; }
  50%, 100% { left: 150%; }
}

/* Pulse animation for legendary */
.pixel-badge.legendary.animated {
  animation: legendary-pulse 1s steps(4) infinite;
}

@keyframes legendary-pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.05); }
}
</style>
