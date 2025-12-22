<script setup lang="ts">
import { ref, computed } from 'vue'
import HoverCard from './HoverCard.vue'
import AmbientBackground from './AmbientBackground.vue'

const props = defineProps<{
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

const cardStyle = computed(() => ({
  '--card-image': `url(${props.imageUrl})`
}))
</script>

<template>
  <HoverCard 
    class="photo-card" 
    :class="{ selected, favorited }"
    :style="cardStyle"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    @click="emit('click')"
  >
    <!-- Ambient background on hover -->
    <AmbientBackground 
      :image-url="imageUrl" 
      :active="isHovered"
      class="card-ambient"
    />
    
    <!-- Image -->
    <div class="card-image">
      <img 
        :src="imageUrl" 
        :alt="title || 'Photo'"
        loading="lazy"
        @load="imageLoaded = true"
        :class="{ loaded: imageLoaded }"
      />
      <div v-if="!imageLoaded" class="skeleton" />
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
    
    <!-- Gradient overlay -->
    <div class="card-gradient" />
    
    <!-- Metadata chin -->
    <div v-if="title || subtitle" class="card-chin">
      <h3 v-if="title" class="card-title">{{ title }}</h3>
      <p v-if="subtitle" class="card-subtitle">{{ subtitle }}</p>
    </div>
    
    <!-- Selection indicator -->
    <div v-if="selected" class="card-selected">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
        <path d="M5 13l4 4L19 7"/>
      </svg>
    </div>
  </HoverCard>
</template>

<style scoped>
.photo-card {
  position: relative;
  aspect-ratio: 1;
  background: var(--surface-1);
  flex-direction: column;
}

.card-ambient {
  position: absolute;
  inset: -20%;
  z-index: -1;
}

.card-image {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.card-image img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 0.3s ease, transform 0.4s ease;
}

.card-image img.loaded {
  opacity: 1;
}

.photo-card:hover .card-image img {
  transform: scale(1.05);
}

.skeleton {
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, var(--surface-2) 25%, var(--surface-3) 50%, var(--surface-2) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

.card-badge {
  position: absolute;
  top: 0.75rem;
  left: 0.75rem;
  padding: 0.25rem 0.625rem;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  border-radius: var(--radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  z-index: 2;
}

.card-fav {
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(8px);
  border: none;
  border-radius: 50%;
  color: var(--text-secondary);
  cursor: pointer;
  opacity: 0;
  transform: scale(0.8);
  transition: all 0.2s ease;
  z-index: 2;
}

.photo-card:hover .card-fav,
.card-fav.active {
  opacity: 1;
  transform: scale(1);
}

.card-fav:hover {
  background: rgba(0, 0, 0, 0.7);
  color: var(--text-primary);
}

.card-fav.active {
  color: var(--cyber-pink);
}

.card-fav svg {
  width: 1rem;
  height: 1rem;
}

.card-gradient {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 50%;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.8));
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.photo-card:hover .card-gradient {
  opacity: 1;
}

.card-chin {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 1rem;
  transform: translateY(100%);
  transition: transform 0.3s ease;
  z-index: 1;
}

.photo-card:hover .card-chin {
  transform: translateY(0);
}

.card-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-subtitle {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin: 0.25rem 0 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-selected {
  position: absolute;
  top: 0.75rem;
  left: 0.75rem;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-color);
  border-radius: 50%;
  z-index: 3;
}

.card-selected svg {
  width: 0.875rem;
  height: 0.875rem;
  color: #000;
}

.photo-card.selected {
  outline: 2px solid var(--accent-color);
  outline-offset: 2px;
}
</style>
