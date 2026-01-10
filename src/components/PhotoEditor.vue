<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue'
import { registerOverlay } from '../composables/useKeyboardShortcuts'

const props = defineProps<{
  imageUrl: string
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'save', blob: Blob): void
}>()

// Register ESC key handler when visible
let unregisterOverlay: (() => void) | null = null;

watch(() => props.visible, (isVisible) => {
  if (isVisible) {
    unregisterOverlay = registerOverlay('photo-editor', () => emit('close'));
  } else {
    if (unregisterOverlay) {
      unregisterOverlay();
      unregisterOverlay = null;
    }
  }
}, { immediate: true });

onUnmounted(() => {
  if (unregisterOverlay) {
    unregisterOverlay();
  }
});

const canvas = ref<HTMLCanvasElement | null>(null)
const rotation = ref(0)
const scale = ref(1)
const grayscale = ref(0)
const sepia = ref(0)
const contrast = ref(100)
const brightness = ref(100)

// Reset filters when image changes or opens
watch(() => props.visible, (val) => {
    if (val) reset()
})

function reset() {
    rotation.value = 0
    scale.value = 1
    grayscale.value = 0
    sepia.value = 0
    contrast.value = 100
    brightness.value = 100
    drawImage()
}

function rotate() {
    rotation.value = (rotation.value + 90) % 360
    drawImage()
}

function drawImage() {
    if (!canvas.value) return
    const ctx = canvas.value.getContext('2d')
    if (!ctx) return

    const img = new Image()
    img.crossOrigin = 'anonymous'
    img.src = props.imageUrl
    
    img.onload = () => {
        if (!canvas.value) return
        
        // Handle Rotation Dimensions
        if (rotation.value % 180 !== 0) {
            canvas.value.width = img.height
            canvas.value.height = img.width
        } else {
            canvas.value.width = img.width
            canvas.value.height = img.height
        }

        ctx.filter = `grayscale(${grayscale.value}%) sepia(${sepia.value}%) contrast(${contrast.value}%) brightness(${brightness.value}%)`
        
        ctx.save()
        ctx.translate(canvas.value.width / 2, canvas.value.height / 2)
        ctx.rotate((rotation.value * Math.PI) / 180)
        ctx.drawImage(img, -img.width / 2, -img.height / 2)
        ctx.restore()
    }
}

// Watch filters to redraw
watch([grayscale, sepia, contrast, brightness], () => {
    drawImage()
})

async function save() {
    if (!canvas.value) return
    canvas.value.toBlob((blob) => {
        if (blob) emit('save', blob)
    }, 'image/jpeg', 0.9)
}
</script>

<template>
  <div v-if="visible" class="editor-overlay">
    <div class="editor-container glass-morphism">
        <div class="editor-header">
            <h3>Editar Foto</h3>
            <button @click="emit('close')" class="close-btn">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
            </button>
        </div>
        
        <div class="editor-workspace">
            <canvas ref="canvas" class="editor-canvas"></canvas>
        </div>

        <div class="editor-controls">
            <div class="control-group">
                <button @click="rotate" class="tool-btn" title="Rotacionar">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6"></path><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path></svg>
                    Rotar
                </button>
            </div>
            
            <div class="sliders">
                <div class="slider-item">
                    <label>P/B</label>
                    <input type="range" v-model.number="grayscale" min="0" max="100">
                </div>
                <div class="slider-item">
                    <label>Sépia</label>
                    <input type="range" v-model.number="sepia" min="0" max="100">
                </div>
                 <div class="slider-item">
                    <label>Contraste</label>
                    <input type="range" v-model.number="contrast" min="50" max="200">
                </div>
            </div>

            <div class="actions">
                <button @click="reset" class="btn-text">Reset</button>
                <button @click="save" class="btn-primary">Salvar</button>
            </div>
        </div>
    </div>
  </div>
</template>

<style scoped>
.editor-overlay {
    position: fixed;
    inset: 0;
    z-index: 10000;
    background: rgba(0,0,0,0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(10px);
}

.editor-container {
    width: 90vw;
    height: 90vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-1);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    overflow: hidden;
}

.editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-default);
}

.editor-workspace {
    flex: 1;
    background: #111;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    padding: 1rem;
}

.editor-canvas {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
}

.editor-controls {
    padding: 1rem;
    display: flex;
    gap: 2rem;
    align-items: center;
    background: var(--surface-2);
    border-top: 1px solid var(--border-default);
}

.sliders {
    display: flex;
    gap: 1rem;
    flex: 1;
}

.slider-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    flex: 1;
}

.actions {
    display: flex;
    gap: 1rem;
}

.tool-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
}

.tool-btn svg {
    width: 24px;
    height: 24px;
}
</style>
