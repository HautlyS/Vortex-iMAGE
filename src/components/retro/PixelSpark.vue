<script setup lang="ts">
import { ref, onMounted, onUnmounted, useTemplateRef } from 'vue';

interface Spark {
  x: number;
  y: number;
  angle: number;
  startTime: number;
}

const props = withDefaults(defineProps<{
  sparkColor?: string;
  sparkSize?: number;
  sparkRadius?: number;
  sparkCount?: number;
  duration?: number;
}>(), {
  sparkColor: 'var(--retro-accent-green, #00ff87)',
  sparkSize: 8,
  sparkRadius: 20,
  sparkCount: 8,
  duration: 300
});

const containerRef = useTemplateRef<HTMLDivElement>('containerRef');
const canvasRef = useTemplateRef<HTMLCanvasElement>('canvasRef');
const sparks = ref<Spark[]>([]);
let animationId: number | null = null;

const handleClick = (e: MouseEvent) => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;

  const now = performance.now();
  const newSparks: Spark[] = Array.from({ length: props.sparkCount }, (_, i) => ({
    x,
    y,
    angle: (2 * Math.PI * i) / props.sparkCount,
    startTime: now
  }));

  sparks.value.push(...newSparks);
};

const draw = (timestamp: number) => {
  const canvas = canvasRef.value;
  const ctx = canvas?.getContext('2d');
  if (!ctx || !canvas) return;

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.imageSmoothingEnabled = false;

  sparks.value = sparks.value.filter((spark: Spark) => {
    const elapsed = timestamp - spark.startTime;
    if (elapsed >= props.duration) return false;

    const progress = elapsed / props.duration;
    const eased = progress * (2 - progress);
    const distance = eased * props.sparkRadius;
    const size = Math.floor(props.sparkSize * (1 - eased));

    const x = Math.floor(spark.x + distance * Math.cos(spark.angle));
    const y = Math.floor(spark.y + distance * Math.sin(spark.angle));

    // Draw pixel-style spark (small square)
    ctx.fillStyle = props.sparkColor;
    ctx.fillRect(x - size / 2, y - size / 2, size, size);
    
    // Add glow effect
    ctx.shadowColor = props.sparkColor;
    ctx.shadowBlur = 4;
    ctx.fillRect(x - size / 2, y - size / 2, size, size);
    ctx.shadowBlur = 0;

    return true;
  });

  animationId = requestAnimationFrame(draw);
};

const resizeCanvas = () => {
  const canvas = canvasRef.value;
  const parent = canvas?.parentElement;
  if (!canvas || !parent) return;

  const { width, height } = parent.getBoundingClientRect();
  canvas.width = width;
  canvas.height = height;
};

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  const canvas = canvasRef.value;
  const parent = canvas?.parentElement;
  if (!canvas || !parent) return;

  resizeObserver = new ResizeObserver(resizeCanvas);
  resizeObserver.observe(parent);
  resizeCanvas();
  animationId = requestAnimationFrame(draw);
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  if (animationId) cancelAnimationFrame(animationId);
});
</script>

<template>
  <div ref="containerRef" class="pixel-spark-container" @click="handleClick">
    <canvas ref="canvasRef" class="pixel-spark-canvas" />
    <slot />
  </div>
</template>

<style scoped>
.pixel-spark-container {
  position: relative;
  width: 100%;
  height: 100%;
}

.pixel-spark-canvas {
  position: absolute;
  inset: 0;
  pointer-events: none;
  image-rendering: pixelated;
  z-index: 100;
}
</style>
