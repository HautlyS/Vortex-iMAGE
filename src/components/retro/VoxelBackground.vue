<script setup lang="ts">
import { onMounted, ref } from 'vue'

const props = defineProps<{
  count?: number
}>()

interface Voxel {
  id: number
  type: number
  x: number
  delay: number
  speed: number
  size: number
}

const voxels = ref<Voxel[]>([])

onMounted(() => {
  const voxelCount = props.count || 25
  for (let i = 0; i < voxelCount; i++) {
    voxels.value.push({
      id: i,
      type: Math.floor(Math.random() * 4),
      x: Math.random() * 100,
      delay: Math.random() * 8,
      speed: 8 + Math.random() * 12,
      size: 20 + Math.random() * 30
    })
  }
})
</script>

<template>
  <div class="voxel-bg">
    <!-- Gradient Sky -->
    <div class="sky"></div>
    
    <!-- Grid Floor -->
    <div class="grid-floor"></div>
    
    <!-- Floating Voxels -->
    <div 
      v-for="v in voxels" 
      :key="v.id"
      class="voxel"
      :class="`type-${v.type}`"
      :style="{ 
        left: `${v.x}%`,
        width: `${v.size}px`,
        height: `${v.size}px`,
        animationDelay: `-${v.delay}s`,
        animationDuration: `${v.speed}s`
      }"
    >
      <div class="face front"></div>
      <div class="face back"></div>
      <div class="face right"></div>
      <div class="face left"></div>
      <div class="face top"></div>
      <div class="face bottom"></div>
    </div>

    <!-- Stars -->
    <div class="stars">
      <div v-for="i in 50" :key="`star-${i}`" class="star" :style="{
        left: `${Math.random() * 100}%`,
        top: `${Math.random() * 60}%`,
        animationDelay: `${Math.random() * 3}s`
      }"></div>
    </div>
  </div>
</template>

<style scoped>
.voxel-bg {
  position: fixed;
  inset: 0;
  overflow: hidden;
  z-index: -1;
  background: var(--retro-bg-dark, #0f0a1e);
  perspective: 800px;
}

.sky {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    180deg,
    #0a0515 0%,
    #1a1030 30%,
    #2d1f4d 70%,
    #1a1030 100%
  );
}

.grid-floor {
  position: absolute;
  bottom: -30%;
  left: -50%;
  width: 200%;
  height: 80%;
  background-image: 
    linear-gradient(rgba(0, 255, 135, 0.15) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 255, 135, 0.15) 1px, transparent 1px);
  background-size: 60px 60px;
  transform: rotateX(70deg);
  animation: grid-move 20s linear infinite;
  mask-image: linear-gradient(to top, rgba(0,0,0,1) 0%, transparent 80%);
  -webkit-mask-image: linear-gradient(to top, rgba(0,0,0,1) 0%, transparent 80%);
}

@keyframes grid-move {
  0% { transform: rotateX(70deg) translateY(0); }
  100% { transform: rotateX(70deg) translateY(60px); }
}

.stars {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.star {
  position: absolute;
  width: 2px;
  height: 2px;
  background: #fff;
  animation: twinkle 2s ease-in-out infinite;
}

@keyframes twinkle {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 1; }
}

.voxel {
  position: absolute;
  bottom: -80px;
  transform-style: preserve-3d;
  animation: float-up linear infinite;
  opacity: 0.7;
}

@keyframes float-up {
  0% {
    transform: translateY(110vh) rotateX(0deg) rotateY(0deg);
    opacity: 0;
  }
  5% { opacity: 0.7; }
  95% { opacity: 0.7; }
  100% {
    transform: translateY(-20vh) rotateX(360deg) rotateY(720deg);
    opacity: 0;
  }
}

.face {
  position: absolute;
  width: 100%;
  height: 100%;
  border: 2px solid rgba(0, 0, 0, 0.5);
  backface-visibility: visible;
}

.front  { transform: rotateY(0deg) translateZ(calc(var(--size, 25px) / 2)); }
.back   { transform: rotateY(180deg) translateZ(calc(var(--size, 25px) / 2)); }
.right  { transform: rotateY(90deg) translateZ(calc(var(--size, 25px) / 2)); }
.left   { transform: rotateY(-90deg) translateZ(calc(var(--size, 25px) / 2)); }
.top    { transform: rotateX(90deg) translateZ(calc(var(--size, 25px) / 2)); }
.bottom { transform: rotateX(-90deg) translateZ(calc(var(--size, 25px) / 2)); }

/* Voxel Colors */
.type-0 .face {
  background: linear-gradient(135deg, #ff2d95, #ff6ac2);
  box-shadow: inset 0 0 20px rgba(255, 45, 149, 0.5);
}

.type-1 .face {
  background: linear-gradient(135deg, #00ff87, #00cc6a);
  box-shadow: inset 0 0 20px rgba(0, 255, 135, 0.5);
}

.type-2 .face {
  background: linear-gradient(135deg, #00d4ff, #00a8cc);
  box-shadow: inset 0 0 20px rgba(0, 212, 255, 0.5);
}

.type-3 .face {
  background: linear-gradient(135deg, #ffd000, #ffaa00);
  box-shadow: inset 0 0 20px rgba(255, 208, 0, 0.5);
}

/* Responsive */
@media (max-width: 768px) {
  .voxel {
    opacity: 0.4;
  }
  
  .grid-floor {
    background-size: 40px 40px;
  }
}
</style>
