<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, useTemplateRef } from 'vue';

const props = withDefaults(defineProps<{
  padding?: number;
  disabled?: boolean;
  strength?: number;
  glowOnHover?: boolean;
  glowColor?: string;
}>(), {
  padding: 80,
  disabled: false,
  strength: 3,
  glowOnHover: true,
  glowColor: 'var(--retro-accent-green, #00ff87)'
});

const magnetRef = useTemplateRef<HTMLDivElement>('magnetRef');
const isActive = ref(false);
const position = ref({ x: 0, y: 0 });

const transformStyle = computed(() => ({
  transform: `translate3d(${Math.round(position.value.x)}px, ${Math.round(position.value.y)}px, 0)`,
  transition: isActive.value ? 'transform 0.15s steps(4)' : 'transform 0.3s steps(6)',
  boxShadow: isActive.value && props.glowOnHover 
    ? `0 0 20px ${props.glowColor}, 0 0 40px ${props.glowColor}` 
    : 'none'
}));

const handleMouseMove = (e: MouseEvent) => {
  if (!magnetRef.value || props.disabled) return;

  const { left, top, width, height } = magnetRef.value.getBoundingClientRect();
  const centerX = left + width / 2;
  const centerY = top + height / 2;

  const distX = Math.abs(centerX - e.clientX);
  const distY = Math.abs(centerY - e.clientY);

  if (distX < width / 2 + props.padding && distY < height / 2 + props.padding) {
    isActive.value = true;
    // Snap to pixel grid (round to nearest 2px)
    const offsetX = Math.round((e.clientX - centerX) / props.strength / 2) * 2;
    const offsetY = Math.round((e.clientY - centerY) / props.strength / 2) * 2;
    position.value = { x: offsetX, y: offsetY };
  } else {
    isActive.value = false;
    position.value = { x: 0, y: 0 };
  }
};

onMounted(() => {
  window.addEventListener('mousemove', handleMouseMove);
});

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMouseMove);
});

watch(() => props.disabled, (disabled) => {
  if (disabled) {
    position.value = { x: 0, y: 0 };
    isActive.value = false;
  }
});
</script>

<template>
  <div ref="magnetRef" class="pixel-magnet">
    <div class="pixel-magnet-inner" :style="transformStyle">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.pixel-magnet {
  display: inline-block;
  position: relative;
}

.pixel-magnet-inner {
  will-change: transform;
}
</style>
