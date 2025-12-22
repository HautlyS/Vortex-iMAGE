import { ref } from 'vue'

const getMockState = () => (window as any).__MOCK_PHOTOS__ || {}

const photos = ref<string[]>(getMockState().photos || [])
const queue = ref<any[]>(getMockState().queue || [])
const isUploading = ref(false)

export function usePhotoUpload() {
    // Refresh from window
    const state = getMockState()
    photos.value = state.photos || []
    if (state.queue) queue.value = state.queue

    return {
        photos,
        loadingPhotos: ref(false),
        queue,
        isUploading,
        pendingCount: ref(0),
        failedCount: ref(0),
        successCount: ref(0),
        currentUpload: ref(null),
        loadPhotos: async () => { },
        addToQueue: (files: string[]) => {
            queue.value.push(...files.map(f => ({ id: f, name: f, status: 'pending' })))
        },
        selectFiles: async () => { },
        retryFailed: () => { },
        removeFromQueue: (id: string) => {
            queue.value = queue.value.filter(i => i.id !== id)
        },
        clearCompleted: () => { },
        clearAll: () => { queue.value = [] }
    }
}
