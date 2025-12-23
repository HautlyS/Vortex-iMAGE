<template>
  <div class="lazy-image-container" :style="{ width: `${width}px`, height: `${height}px` }">
    <div
      v-if="!loaded && !error"
      class="image-placeholder"
      :style="{ width: `${width}px`, height: `${height}px` }"
    >
      <div class="loading-spinner"></div>
    </div>
    
    <img
      v-show="loaded && !error"
      ref="imageRef"
      :src="shouldLoad ? src : undefined"
      :alt="alt"
      :width="width"
      :height="height"
      class="lazy-image"
      @load="onLoad"
      @error="onError"
    />
    
    <div
      v-if="error"
      class="error-placeholder"
      :style="{ width: `${width}px`, height: `${height}px` }"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
        <circle cx="8.5" cy="8.5" r="1.5"/>
        <polyline points="21,15 16,10 5,21"/>
      </svg>
      <span>Failed to load</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

interface Props {
  src: string
  alt?: string
  width: number
  height: number
  rootMargin?: string
  threshold?: number
}

const props = withDefaults(defineProps<Props>(), {
  alt: '',
  rootMargin: '50px',
  threshold: 0.1
})

const emit = defineEmits<{
  load: []
  error: []
}>()

const imageRef = ref<HTMLImageElement>()
const loaded = ref(false)
const error = ref(false)
const shouldLoad = ref(false)
const observer = ref<IntersectionObserver>()

const onLoad = () => {
  loaded.value = true
  error.value = false
  emit('load')
}

const onError = () => {
  loaded.value = false
  error.value = true
  emit('error')
}

const startObserving = () => {
  if (!imageRef.value || !('IntersectionObserver' in window)) {
    shouldLoad.value = true
    return
  }

  observer.value = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          shouldLoad.value = true
          observer.value?.disconnect()
        }
      })
    },
    {
      rootMargin: props.rootMargin,
      threshold: props.threshold
    }
  )

  observer.value.observe(imageRef.value.parentElement!)
}

onMounted(() => {
  startObserving()
})

onUnmounted(() => {
  observer.value?.disconnect()
})

// Reset state when src changes
watch(() => props.src, () => {
  loaded.value = false
  error.value = false
  shouldLoad.value = false
  observer.value?.disconnect()
  startObserving()
})
</script>

<style scoped>
.lazy-image-container {
  position: relative;
  overflow: hidden;
}

.lazy-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: opacity 0.3s ease;
}

.image-placeholder,
.error-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.6);
  color: var(--text-muted, #666);
  flex-direction: column;
  gap: 8px;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-default, #333);
  border-top-color: var(--accent-color, #00f0ff);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.error-placeholder svg {
  width: 32px;
  height: 32px;
  opacity: 0.5;
}

.error-placeholder span {
  font-size: 0.8rem;
  opacity: 0.7;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
