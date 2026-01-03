<script setup lang="ts">
import { onMounted, onBeforeUnmount, useTemplateRef } from 'vue';

const props = withDefaults(defineProps<{
  opacity?: number;
  speed?: number;
  pixelSize?: number;
}>(), {
  opacity: 0.08,
  speed: 3,
  pixelSize: 2
});

const canvasRef = useTemplateRef<HTMLCanvasElement>('canvasRef');
let animationId = 0;
let frame = 0;
const canvasSize = 256;

let noiseData: ImageData;
let noise32: Uint32Array;

const resize = () => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  canvas.width = canvasSize;
  canvas.height = canvasSize;
};

const initImageData = (ctx: CanvasRenderingContext2D) => {
  noiseData = ctx.createImageData(canvasSize, canvasSize);
  noise32 = new Uint32Array(noiseData.data.buffer);
};

const drawNoise = () => {
  const alpha = Math.floor(props.opacity * 255) << 24;
  for (let i = 0; i < noise32.length; i++) {
    const v = (Math.random() * 255) | 0;
    noise32[i] = alpha | (v << 16) | (v << 8) | v;
  }
};

const loop = (ctx: CanvasRenderingContext2D) => {
  if (frame % Math.max(1, Math.round(props.speed)) === 0) {
    drawNoise();
    ctx.putImageData(noiseData, 0, 0);
  }
  frame++;
  animationId = requestAnimationFrame(() => loop(ctx));
};

onMounted(() => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d', { alpha: true });
  if (!ctx) return;

  resize();
  initImageData(ctx);
  drawNoise();
  ctx.putImageData(noiseData, 0, 0);
  loop(ctx);
});

onBeforeUnmount(() => {
  cancelAnimationFrame(animationId);
});
</script>

<template>
  <canvas
    ref="canvasRef"
    class="pixel-noise"
    :style="{ 
      imageRendering: 'pixelated',
      '--pixel-size': `${pixelSize}px`
    }"
  />
</template>

<style scoped>
.pixel-noise {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  mix-blend-mode: overlay;
  opacity: 1;
}
</style>
