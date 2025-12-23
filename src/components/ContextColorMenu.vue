  <div 
    v-if="visible"
    ref="menuRef"
    class="context-menu card-glass"
    :style="stylePosition"
    @click.stop
  >
    <div class="menu-header caption-1">Adicionar Etiqueta</div>
    <div class="color-grid">
      <button 
        v-for="color in colors" 
        :key="color"
        class="color-btn"
        :style="{ backgroundColor: color }"
        @click="selectColor(color)"
      >
        <div v-if="activeColor === color" class="active-dot"></div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';

const props = defineProps<{
  visible: boolean
  x: number
  y: number
  activeColor?: string
}>()

const emit = defineEmits<{
  select: [color: string]
  close: []
}>()

const menuRef = ref<HTMLElement | null>(null)
const adjustedPos = ref({ x: 0, y: 0 })

const calculatePosition = async () => {
  if (!props.visible) return
  
  // Reset first to measure
  adjustedPos.value = { x: props.x, y: props.y }
  
  await nextTick()
  
  if (!menuRef.value) return
  
  const rect = menuRef.value.getBoundingClientRect()
  const { innerWidth, innerHeight } = window
  
  let x = props.x
  let y = props.y
  
  // Check right edge
  if (x + rect.width > innerWidth) {
    x = x - rect.width
  }
  
  // Check bottom edge
  if (y + rect.height > innerHeight) {
    y = y - rect.height
  }
  
  adjustedPos.value = { x, y }
}

watch(() => [props.visible, props.x, props.y], calculatePosition, { immediate: true })

const stylePosition = computed(() => ({
  top: `${adjustedPos.value.y}px`,
  left: `${adjustedPos.value.x}px`
}))

function selectColor(color: string) {
  emit('select', color)
  emit('close')
}

// Premium curated colors (HSL tailored or system colors)
const colors = [
  '#ff3b30', // Red
  '#ff9500', // Orange
  '#ffcc00', // Yellow
  '#28cd41', // Green
  '#00c7be', // Mint
  '#59adc4', // Teal
  '#007aff', // Blue
  '#5856d6', // Indigo
  '#af52de', // Purple
  '#ff2d55', // Pink
  '#a2845e', // Brown
  '#8e8e93', // Gray
]

// Close on click outside (handled by parent usually, but could add listener here)
</script>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 1000;
  padding: 12px;
  min-width: 180px;
  animation: scale-up 0.15s ease-out;
  transform-origin: top left;
}

@keyframes scale-up {
  from { opacity: 0; transform: scale(0.9); }
  to { opacity: 1; transform: scale(1); }
}

.menu-header {
  color: var(--systemSecondary);
  margin-bottom: 8px;
  padding-left: 4px;
}

.color-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.color-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: 2px solid rgba(255,255,255,0.1);
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.1s;
}

.color-btn:hover {
  transform: scale(1.1);
  border-color: rgba(255,255,255,0.5);
}

.active-dot {
  width: 8px;
  height: 8px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 1px 2px rgba(0,0,0,0.3);
}
</style>
