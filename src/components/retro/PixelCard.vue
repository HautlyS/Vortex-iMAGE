<script setup lang="ts">
defineProps<{
  title?: string
  noPadding?: boolean
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info'
  glow?: boolean
}>()
</script>

<template>
  <div class="pixel-card" :class="[variant, { glow }]">
    <!-- 8-Bit Corner Decorations -->
    <div class="corner tl">
      <div class="corner-inner"></div>
    </div>
    <div class="corner tr">
      <div class="corner-inner"></div>
    </div>
    <div class="corner bl">
      <div class="corner-inner"></div>
    </div>
    <div class="corner br">
      <div class="corner-inner"></div>
    </div>
    
    <!-- Header -->
    <div v-if="title" class="pixel-card-header">
      <div class="header-pattern"></div>
      <div class="header-content">
        <span class="pixel-card-title">{{ title }}</span>
        <div class="pixel-card-actions">
          <slot name="actions"></slot>
        </div>
      </div>
      <div class="header-decoration">
        <span class="deco-dot"></span>
        <span class="deco-dot"></span>
        <span class="deco-dot"></span>
      </div>
    </div>
    
    <!-- Body -->
    <div class="pixel-card-body" :class="{ 'no-padding': noPadding }">
      <slot></slot>
    </div>
    
    <!-- Footer slot -->
    <div v-if="$slots.footer" class="pixel-card-footer">
      <slot name="footer"></slot>
    </div>
  </div>
</template>

<style scoped>
.pixel-card {
  position: relative;
  background: #1a1a2e;
  border: 4px solid #000;
  box-shadow: 8px 8px 0 rgba(0, 0, 0, 0.8);
  image-rendering: pixelated;
}

/* 8-Bit Corner Decorations */
.corner {
  position: absolute;
  width: 12px;
  height: 12px;
  background: #f15bb5;
  z-index: 10;
}

.corner-inner {
  position: absolute;
  width: 4px;
  height: 4px;
  background: #fff;
}

.corner.tl { top: -6px; left: -6px; }
.corner.tl .corner-inner { top: 2px; left: 2px; }

.corner.tr { top: -6px; right: -6px; }
.corner.tr .corner-inner { top: 2px; right: 2px; }

.corner.bl { bottom: -6px; left: -6px; }
.corner.bl .corner-inner { bottom: 2px; left: 2px; }

.corner.br { bottom: -6px; right: -6px; }
.corner.br .corner-inner { bottom: 2px; right: 2px; }

/* Header */
.pixel-card-header {
  position: relative;
  background: linear-gradient(180deg, #0099db 0%, #006b99 100%);
  color: #fff;
  padding: 0;
  border-bottom: 4px solid #000;
  overflow: hidden;
}

.header-pattern {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    90deg,
    transparent 0px,
    transparent 8px,
    rgba(0,0,0,0.1) 8px,
    rgba(0,0,0,0.1) 16px
  );
}

.header-content {
  position: relative;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  padding-left: 24px;
}

.header-decoration {
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.deco-dot {
  width: 4px;
  height: 4px;
  background: rgba(255,255,255,0.6);
}

.pixel-card-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  text-shadow: 2px 2px 0 #000;
  letter-spacing: 2px;
  text-transform: uppercase;
}

.pixel-card-actions {
  display: flex;
  gap: 8px;
}

/* Body */
.pixel-card-body {
  padding: 20px;
  background: linear-gradient(180deg, #16213e 0%, #1a1a2e 100%);
}

.pixel-card-body.no-padding {
  padding: 0;
}

/* Footer */
.pixel-card-footer {
  padding: 12px 20px;
  background: #0f0f23;
  border-top: 4px solid #000;
}

/* Variants */
.pixel-card.success .pixel-card-header {
  background: linear-gradient(180deg, #63c74d 0%, #3e8948 100%);
}

.pixel-card.success .corner {
  background: #63c74d;
}

.pixel-card.warning .pixel-card-header {
  background: linear-gradient(180deg, #feae34 0%, #c68b28 100%);
}

.pixel-card.warning .corner {
  background: #feae34;
}

.pixel-card.danger .pixel-card-header {
  background: linear-gradient(180deg, #e43b44 0%, #a82835 100%);
}

.pixel-card.danger .corner {
  background: #e43b44;
}

.pixel-card.info .pixel-card-header {
  background: linear-gradient(180deg, #9b5de5 0%, #7b3dc5 100%);
}

.pixel-card.info .corner {
  background: #9b5de5;
}

/* Glow Effect */
.pixel-card.glow {
  animation: card-glow 2s steps(4) infinite;
}

@keyframes card-glow {
  0%, 100% { 
    box-shadow: 8px 8px 0 rgba(0, 0, 0, 0.8), 0 0 8px rgba(241, 91, 181, 0.2); 
  }
  50% { 
    box-shadow: 8px 8px 0 rgba(0, 0, 0, 0.8), 0 0 24px rgba(241, 91, 181, 0.4); 
  }
}

.pixel-card.success.glow {
  animation: card-glow-green 2s steps(4) infinite;
}

@keyframes card-glow-green {
  0%, 100% { box-shadow: 8px 8px 0 rgba(0, 0, 0, 0.8), 0 0 8px rgba(99, 199, 77, 0.2); }
  50% { box-shadow: 8px 8px 0 rgba(0, 0, 0, 0.8), 0 0 24px rgba(99, 199, 77, 0.4); }
}
</style>
