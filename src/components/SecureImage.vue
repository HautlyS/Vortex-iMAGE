<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useGitHubAuth } from '../composables/useGitHubAuth'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  src: string
  alt?: string
  className?: string
}>()

const emit = defineEmits<{
  (e: 'load'): void
  (e: 'error', err: any): void
}>()

const { token, repo, keypairBytes } = useGitHubAuth()
const objectUrl = ref<string | null>(null)
const error = ref<string | null>(null)
const isLoading = ref(true)

async function loadImage() {
  if (!props.src) return

  // Check if it's a secure GitHub URL
  const isGitHub = props.src.includes('github.com') || props.src.includes('githubusercontent.com')
  
  // If dev mode mock or not GitHub, just use src
  if (import.meta.env.DEV) {
    objectUrl.value = props.src
    isLoading.value = false
    emit('load')
    return
  }

  if (!isGitHub) {
     objectUrl.value = props.src
     isLoading.value = false
     emit('load')
     return
  }
  
  // Parse path from URL
  // Expected format: .../contents/photos/filename OR .../blob/main/photos/filename
  // Simple check: if it contains 'photos/', extract from there
  let remotePath = ''
  const parts = props.src.split('photos/')
  if (parts.length > 1) {
    remotePath = 'photos/' + parts[1]
  } else {
    // Fallback or maybe it's just the filename?
    // If not parseable, try loading as normal image
    objectUrl.value = props.src
    isLoading.value = false
    emit('load')
    return
  }

  // Remove query params if any
  remotePath = remotePath.split('?')[0]

  try {
    isLoading.value = true
    
    // Check if we have keys
    if (!keypairBytes.value) {
        // If no keys, maybe it's a public unencrypted image? Try fetching normally?
        // But for "mandatory" encryption, likely we need keys.
        // We'll try to download secure. If it fails due to keys, error out.
        throw new Error("Missing decryption keys")
    }

    const imageBytes = await invoke<number[]>('download_secure_photo', {
      remotePath,
      repo: repo.value,
      token: token.value,
      keypairBytes: keypairBytes.value
    })

    const blob = new Blob([new Uint8Array(imageBytes)], { type: 'image/jpeg' }) // Detect mime type?
    const url = URL.createObjectURL(blob)
    objectUrl.value = url
    isLoading.value = false
    emit('load')
  } catch (e) {
    console.error('Failed to load secure image:', e)
    error.value = String(e)
    // Fallback: try loading original src just in case it wasn't encrypted
    objectUrl.value = props.src
    isLoading.value = false
    // Don't emit error yet, let the img tag fail if it can't load
  }
}

watch(() => props.src, () => {
  // Revoke old URL
  if (objectUrl.value && objectUrl.value.startsWith('blob:')) {
    URL.revokeObjectURL(objectUrl.value)
  }
  loadImage()
})

onMounted(loadImage)

onUnmounted(() => {
  if (objectUrl.value && objectUrl.value.startsWith('blob:')) {
    URL.revokeObjectURL(objectUrl.value)
  }
})
</script>

<template>
  <img 
    v-if="objectUrl"
    :src="objectUrl" 
    :alt="alt" 
    :class="className"
    @load="$emit('load')"
    @error="$emit('error', $event)"
  />
  <div v-else-if="isLoading" class="skeleton-placeholder" />
</template>

<style scoped>
.skeleton-placeholder {
  width: 100%;
  height: 100%;
  background: var(--genericJoeColor, #333);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: .5; }
}
</style>
