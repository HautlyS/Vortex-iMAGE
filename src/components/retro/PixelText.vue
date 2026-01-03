<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick, useTemplateRef } from 'vue';

const props = withDefaults(defineProps<{
  text: string;
  speed?: number;
  maxIterations?: number;
  sequential?: boolean;
  characters?: string;
  animateOn?: 'view' | 'hover';
  glowColor?: string;
}>(), {
  speed: 40,
  maxIterations: 12,
  sequential: true,
  characters: '!@#$%^&*()_+-=[]{}|;:,.<>?/~`01',
  animateOn: 'hover',
  glowColor: 'var(--retro-accent-green, #00ff87)'
});

const emit = defineEmits<{
  animationComplete: [];
}>();

const containerRef = useTemplateRef<HTMLSpanElement>('containerRef');
const displayText = ref(props.text);
const isHovering = ref(false);
const isScrambling = ref(false);
const revealedIndices = ref(new Set<number>());
const hasAnimated = ref(false);

let interval: ReturnType<typeof setInterval> | null = null;
let intersectionObserver: IntersectionObserver | null = null;

watch(
  [() => isHovering.value, () => props.text],
  () => {
    let currentIteration = 0;
    const chars = props.characters.split('');

    const shuffleText = (original: string, revealed: Set<number>): string => {
      return original
        .split('')
        .map((char, i) => {
          if (char === ' ') return ' ';
          if (revealed.has(i)) return original[i];
          return chars[Math.floor(Math.random() * chars.length)];
        })
        .join('');
    };

    if (interval) {
      clearInterval(interval);
      interval = null;
    }

    if (isHovering.value) {
      isScrambling.value = true;
      interval = setInterval(() => {
        if (props.sequential) {
          if (revealedIndices.value.size < props.text.length) {
            const newRevealed = new Set(revealedIndices.value);
            newRevealed.add(revealedIndices.value.size);
            revealedIndices.value = newRevealed;
            displayText.value = shuffleText(props.text, newRevealed);
          } else {
            clearInterval(interval!);
            interval = null;
            isScrambling.value = false;
            emit('animationComplete');
          }
        } else {
          displayText.value = shuffleText(props.text, revealedIndices.value);
          currentIteration++;
          if (currentIteration >= props.maxIterations) {
            clearInterval(interval!);
            interval = null;
            isScrambling.value = false;
            displayText.value = props.text;
            emit('animationComplete');
          }
        }
      }, props.speed);
    } else {
      displayText.value = props.text;
      revealedIndices.value = new Set();
      isScrambling.value = false;
    }
  }
);

const handleMouseEnter = () => {
  if (props.animateOn === 'hover') isHovering.value = true;
};

const handleMouseLeave = () => {
  if (props.animateOn === 'hover') isHovering.value = false;
};

onMounted(async () => {
  if (props.animateOn === 'view') {
    await nextTick();
    intersectionObserver = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.value) {
          isHovering.value = true;
          hasAnimated.value = true;
        }
      },
      { threshold: 0.1 }
    );
    if (containerRef.value) intersectionObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  if (interval) clearInterval(interval);
  if (intersectionObserver && containerRef.value) {
    intersectionObserver.unobserve(containerRef.value);
  }
});
</script>

<template>
  <span
    ref="containerRef"
    class="pixel-text"
    :class="{ scrambling: isScrambling }"
    :style="{ '--glow-color': glowColor }"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <span class="sr-only">{{ displayText }}</span>
    <span aria-hidden="true">
      <span
        v-for="(char, index) in displayText.split('')"
        :key="index"
        class="pixel-char"
        :class="{ revealed: revealedIndices.has(index) || !isScrambling }"
      >{{ char }}</span>
    </span>
  </span>
</template>

<style scoped>
.pixel-text {
  font-family: 'Press Start 2P', monospace;
  display: inline-block;
  white-space: pre-wrap;
}

.pixel-char {
  display: inline-block;
  transition: color 0.1s steps(2), text-shadow 0.1s steps(2);
}

.pixel-char:not(.revealed) {
  color: var(--retro-accent-pink, #ff2d95);
  text-shadow: 0 0 4px var(--retro-accent-pink);
}

.pixel-char.revealed {
  color: var(--glow-color);
  text-shadow: 0 0 8px var(--glow-color);
}

.scrambling .pixel-char:not(.revealed) {
  animation: glitch 0.1s steps(2) infinite;
}

@keyframes glitch {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-1px); }
  75% { transform: translateX(1px); }
}
</style>
