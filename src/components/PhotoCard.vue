/**
 * Vue Component - 1 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: SecureImage
 */

<script setup lang="ts">
import { ref } from 'vue'
import SecureImage from './SecureImage.vue'

defineProps<{
  imageUrl: string
  title?: string
  subtitle?: string
  badge?: string
  favorited?: boolean
  selected?: boolean
}>()

const emit = defineEmits<{
  click: []
  favorite: []
}>()

const isHovered = ref(false)
const imageLoaded = ref(false)
</script>

<template>
  <div 
    class="photo-card" 
    :class="{ selected, favorited }"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    @click="emit('click')"
  >
    <!-- Artwork container (iOS style) -->
    <div class="artwork">
      <SecureImage 
        :src="imageUrl" 
        :alt="title || 'Photo'"
        :class-name="imageLoaded ? 'loaded' : ''"
        @load="imageLoaded = true"
      />
      <!-- Skeleton loading is handled inside SecureImage too, but we keep outer structure -->
      <div v-if="!imageLoaded" class="skeleton" />
      
      <!-- Inner border (iOS artwork style) -->
      <div class="artwork-border" />
    </div>
    
    <!-- Badge -->
    <span v-if="badge" class="card-badge">{{ badge }}</span>
    
    <!-- Favorite button -->
    <button 
      class="card-fav" 
      :class="{ active: favorited }"
      @click.stop="emit('favorite')"
      aria-label="Toggle favorite"
    >
      <svg viewBox="0 0 24 24" :fill="favorited ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
        <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
      </svg>
    </button>
    
    <!-- Metadata (iOS lockup style) -->
    <div v-if="title || subtitle" class="card-meta">
      <h3 v-if="title" class="card-title">{{ title }}</h3>
      <p v-if="subtitle" class="card-subtitle">{{ subtitle }}</p>
    </div>
    
    <!-- Selection indicator -->
    <div v-if="selected" class="card-selected">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
        <path d="M5 13l4 4L19 7"/>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.photo-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 8px;
  cursor: pointer;
}

.artwork {
  position: relative;
  aspect-ratio: 1;
  background: var(--genericJoeColor);
  border-radius: var(--global-border-radius-medium);
  overflow: hidden;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.photo-card:hover .artwork {
  transform: scale(1.02);
  box-shadow: var(--shadow-medium);
}

.photo-card:active .artwork {
  transform: scale(0.98);
}

.artwork img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.artwork img.loaded {
  opacity: 1;
}

.artwork-border {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  box-shadow: inset 0 0 0 0.5px rgba(128, 128, 128, 0.2);
  pointer-events: none;
}

.skeleton {
  position: absolute;
  inset: 0;
  background: var(--genericJoeColor);
}

.skeleton::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.08), transparent);
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

.card-badge {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 4px 10px;
  background: var(--systemStandardThickMaterialSover);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-radius: 1000px;
  font-size: 11px;
  font-weight: 600;
  color: var(--systemPrimary);
  z-index: 2;
}

.card-fav {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--systemStandardThickMaterialSover);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: none;
  border-radius: 50%;
  color: var(--systemSecondary);
  cursor: pointer;
  opacity: 0;
  transform: scale(0.9);
  transition: all 0.2s ease;
  z-index: 2;
}

.photo-card:hover .card-fav,
.card-fav.active {
  opacity: 1;
  transform: scale(1);
}

.card-fav:hover {
  background: var(--systemStandardUltrathickMaterialSover);
}

.card-fav.active {
  color: var(--systemPink);
}

.card-fav svg {
  width: 16px;
  height: 16px;
}

.card-meta {
  padding: 0 2px;
}

.card-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--systemPrimary);
  margin: 0;
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-subtitle {
  font-size: 13px;
  color: var(--systemSecondary);
  margin: 2px 0 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-selected {
  position: absolute;
  top: 8px;
  left: 8px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--keyColor);
  border-radius: 50%;
  color: #fff;
  z-index: 3;
  box-shadow: 0 2px 8px rgba(0, 122, 255, 0.4);
}

.card-selected svg {
  width: 14px;
  height: 14px;
}

.photo-card.selected .artwork {
  box-shadow: 0 0 0 3px var(--keyColor);
}
</style>