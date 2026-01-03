<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from 'vue';

const props = withDefaults(defineProps<{
  color?: string;
  intensity?: number;
  speed?: number;
}>(), {
  color: '#00ff87',
  intensity: 1,
  speed: 1
});

const canvasRef = useTemplateRef<HTMLCanvasElement>('canvasRef');
let animationId = 0;
let ctx: CanvasRenderingContext2D | null = null;

interface LightningBolt {
  points: { x: number; y: number }[];
  alpha: number;
  width: number;
}

let bolts: LightningBolt[] = [];
let lastBoltTime = 0;

const hexToRgb = (hex: string) => {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : { r: 0, g: 255, b: 135 };
};

const generateBolt = (startX: number, startY: number, endY: number): LightningBolt => {
  const points: { x: number; y: number }[] = [{ x: startX, y: startY }];
  let currentX = startX;
  let currentY = startY;
  const segments = 8 + Math.floor(Math.random() * 8);
  const segmentHeight = (endY - startY) / segments;

  for (let i = 0; i < segments; i++) {
    currentY += segmentHeight;
    // Pixel-snap the x offset
    currentX += Math.round((Math.random() - 0.5) * 60 / 4) * 4;
    points.push({ x: currentX, y: Math.round(currentY) });
  }

  return {
    points,
    alpha: 1,
    width: 2 + Math.random() * 2
  };
};

const drawBolt = (bolt: LightningBolt) => {
  if (!ctx || !canvasRef.value) return;
  
  const rgb = hexToRgb(props.color);
  
  // Main bolt
  ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${bolt.alpha})`;
  ctx.lineWidth = bolt.width;
  ctx.lineCap = 'square';
  ctx.lineJoin = 'miter';
  
  ctx.beginPath();
  ctx.moveTo(bolt.points[0].x, bolt.points[0].y);
  for (let i = 1; i < bolt.points.length; i++) {
    ctx.lineTo(bolt.points[i].x, bolt.points[i].y);
  }
  ctx.stroke();

  // Glow effect
  ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${bolt.alpha * 0.3})`;
  ctx.lineWidth = bolt.width * 4;
  ctx.stroke();

  // Core (brighter)
  ctx.strokeStyle = `rgba(255, 255, 255, ${bolt.alpha * 0.8})`;
  ctx.lineWidth = 1;
  ctx.stroke();
};

const resize = () => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width;
  canvas.height = rect.height;
};

const animate = (timestamp: number) => {
  if (!ctx || !canvasRef.value) return;
  
  const canvas = canvasRef.value;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Spawn new bolts
  const spawnInterval = 2000 / props.speed;
  if (timestamp - lastBoltTime > spawnInterval && Math.random() < 0.3 * props.intensity) {
    const startX = Math.random() * canvas.width;
    bolts.push(generateBolt(startX, 0, canvas.height));
    lastBoltTime = timestamp;
  }

  // Update and draw bolts
  bolts = bolts.filter(bolt => {
    bolt.alpha -= 0.05 * props.speed;
    if (bolt.alpha <= 0) return false;
    drawBolt(bolt);
    return true;
  });

  animationId = requestAnimationFrame(animate);
};

onMounted(() => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  ctx = canvas.getContext('2d');
  resize();
  window.addEventListener('resize', resize);
  animationId = requestAnimationFrame(animate);
});

onUnmounted(() => {
  cancelAnimationFrame(animationId);
  window.removeEventListener('resize', resize);
});
</script>

<template>
  <canvas 
    ref="canvasRef" 
    class="pixel-lightning"
    :style="{ imageRendering: 'pixelated' }"
  />
</template>

<style scoped>
.pixel-lightning {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  mix-blend-mode: screen;
}
</style>
