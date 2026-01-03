<script setup lang="ts">
defineProps<{
  title?: string
  noPadding?: boolean
  variant?: 'default' | 'success' | 'warning' | 'danger'
}>()
</script>

<template>
  <div class="pixel-card" :class="[variant]">
    <!-- Corner Decorations -->
    <div class="corner tl"></div>
    <div class="corner tr"></div>
    <div class="corner bl"></div>
    <div class="corner br"></div>
    
    <!-- Header -->
    <div v-if="title" class="pixel-card-header">
      <div class="header-decoration"></div>
      <span class="pixel-card-title">{{ title }}</span>
      <div class="pixel-card-actions">
        <slot name="actions"></slot>
      </div>
    </div>
    
    <!-- Body -->
    <div class="pixel-card-body" :class="{ 'no-padding': noPadding }">
      <slot></slot>
    </div>
  </div>
</template>

<style scoped>
.pixel-card {
  position: relative;
  background: var(--retro-bg-panel, #1a1030);
  border: 3px solid #000;
  box-shadow: 6px 6px 0 rgba(0, 0, 0, 0.6);
}

/* Corner Decorations */
.corner {
  position: absolute;
  width: 8px;
  height: 8px;
  background: var(--retro-accent-pink, #ff2d95);
  z-index: 1;
}

.corner.tl { top: -4px; left: -4px; }
.corner.tr { top: -4px; right: -4px; }
.corner.bl { bottom: -4px; left: -4px; }
.corner.br { bottom: -4px; right: -4px; }

/* Header */
.pixel-card-header {
  position: relative;
  background: linear-gradient(90deg, var(--retro-accent-blue, #00d4ff), var(--retro-accent-purple, #b24dff));
  color: #fff;
  padding: 10px 16px;
  border-bottom: 3px solid #000;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-decoration {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 6px;
  background: repeating-linear-gradient(
    0deg,
    #000 0px,
    #000 4px,
    transparent 4px,
    transparent 8px
  );
}

.pixel-card-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  text-shadow: 2px 2px 0 #000;
  letter-spacing: 1px;
}

.pixel-card-actions {
  display: flex;
  gap: 8px;
}

/* Body */
.pixel-card-body {
  padding: 20px;
  background: var(--retro-bg-card, #251842);
}

.pixel-card-body.no-padding {
  padding: 0;
}

/* Variants */
.pixel-card.success .pixel-card-header {
  background: linear-gradient(90deg, var(--retro-accent-green, #00ff87), #00cc6a);
}

.pixel-card.success .corner {
  background: var(--retro-accent-green, #00ff87);
}

.pixel-card.warning .pixel-card-header {
  background: linear-gradient(90deg, var(--retro-accent-yellow, #ffd000), #ffaa00);
}

.pixel-card.warning .corner {
  background: var(--retro-accent-yellow, #ffd000);
}

.pixel-card.danger .pixel-card-header {
  background: linear-gradient(90deg, var(--retro-accent-red, #ff3b30), #cc2020);
}

.pixel-card.danger .corner {
  background: var(--retro-accent-red, #ff3b30);
}
</style>
