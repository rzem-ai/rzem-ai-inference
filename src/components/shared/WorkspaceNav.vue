<template>
  <div ref="containerRef" class="relative flex flex-col w-20 h-full overflow-hidden min-w-20">
    <!-- Content (z-index to stay above canvas) -->
    <div class="relative z-10 flex flex-col h-full">
      <div class="flex items-center justify-center h-16 p-2 min-h-16 shrink">
        <div class="flex items-center justify-center w-12 h-12 text-lg font-medium rounded bg-linear-to-br from-blue-400 to-blue-600 text-surface-800">
          rz.ai
        </div>
      </div>
      <div class="flex flex-col items-center justify-start py-4 grow">
        <WorkspaceNavItem v-for="(workspace, index) in workspaces" :key="workspace.path" :id="'nav_' + index" :workspace="workspace" />
      </div>
      <div class="flex items-center justify-center h-16 min-h-16 shrink"> </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, useTemplateRef } from 'vue';
import WorkspaceNavItem, { type Workspace } from './WorkspaceNavItem.vue';

const workspaces: Workspace[] = [
  { label: 'Generate', path: '/generate', to: 'generate', icon: 'sparkles', enabled: true },
  { label: 'Gallery', path: '/gallery', to: 'gallery', icon: 'images', enabled: true },
  { label: 'Styles', path: '/styles', to: 'styles', icon: 'palette', enabled: true },
  { label: 'Models', path: '/models', to: 'models-bundles', icon: 'database', enabled: true },
];

// Pixel effect implementation
class Pixel {
  width: number;
  height: number;
  ctx: CanvasRenderingContext2D;
  x: number;
  y: number;
  color: string;
  speed: number;
  size: number;
  sizeStep: number;
  minSize: number;
  maxSizeInteger: number;
  maxSize: number;
  delay: number;
  counter: number;
  counterStep: number;
  isIdle: boolean;
  isReverse: boolean;
  isShimmer: boolean;

  constructor(canvas: HTMLCanvasElement, context: CanvasRenderingContext2D, x: number, y: number, color: string, speed: number, delay: number) {
    this.width = canvas.width;
    this.height = canvas.height;
    this.ctx = context;
    this.x = x;
    this.y = y;
    this.color = color;
    this.speed = this.getRandomValue(0.1, 0.9) * speed;
    this.size = 0;
    this.sizeStep = Math.random() * 0.4;
    this.minSize = 0.5;
    this.maxSizeInteger = 2;
    this.maxSize = this.getRandomValue(this.minSize, this.maxSizeInteger);
    this.delay = delay;
    this.counter = 0;
    this.counterStep = Math.random() * 4 + (this.width + this.height) * 0.01;
    this.isIdle = false;
    this.isReverse = false;
    this.isShimmer = false;
  }

  getRandomValue(min: number, max: number) {
    return Math.random() * (max - min) + min;
  }

  draw() {
    const centerOffset = this.maxSizeInteger * 0.5 - this.size * 0.5;
    this.ctx.fillStyle = this.color;
    this.ctx.fillRect(this.x + centerOffset, this.y + centerOffset, this.size, this.size);
  }

  appear() {
    this.isIdle = false;
    if (this.counter <= this.delay) {
      this.counter += this.counterStep;
      return;
    }
    if (this.size >= this.maxSize) {
      this.isShimmer = true;
    }
    if (this.isShimmer) {
      this.shimmer();
    } else {
      this.size += this.sizeStep;
    }
    this.draw();
  }

  shimmer() {
    if (this.size >= this.maxSize) {
      this.isReverse = true;
    } else if (this.size <= this.minSize) {
      this.isReverse = false;
    }
    if (this.isReverse) {
      this.size -= this.speed;
    } else {
      this.size += this.speed;
    }
  }
}

const containerRef = useTemplateRef<HTMLDivElement>('containerRef');
const canvasRef = useTemplateRef<HTMLCanvasElement>('canvasRef');
const pixelsRef = ref<Pixel[]>([]);
const animationRef = ref<number | null>(null);
const timePreviousRef = ref(performance.now());

// Config for subtle dark theme effect
const gap = 8;
const speed = 0.02; // Subtle shimmer
const colors = ['#3b82f6', '#60a5fa', '#93c5fd']; // Blue shades

const initPixels = () => {
  if (!containerRef.value || !canvasRef.value) return;

  const rect = containerRef.value.getBoundingClientRect();
  const width = Math.floor(rect.width);
  const height = Math.floor(rect.height);
  const ctx = canvasRef.value.getContext('2d');

  if (!ctx) return;

  canvasRef.value.width = width;
  canvasRef.value.height = height;
  canvasRef.value.style.width = `${width}px`;
  canvasRef.value.style.height = `${height}px`;

  const pxs = [];
  for (let x = 0; x < width; x += gap) {
    for (let y = 0; y < height; y += gap) {
      const color = colors[Math.floor(Math.random() * colors.length)];
      const dx = x - width / 2;
      const dy = y - height / 2;
      const distance = Math.sqrt(dx * dx + dy * dy);
      const delay = distance * 0.5; // Gradual appearance
      pxs.push(new Pixel(canvasRef.value, ctx, x, y, color, speed, delay));
    }
  }
  pixelsRef.value = pxs;
};

const animate = () => {
  animationRef.value = requestAnimationFrame(animate);
  const timeNow = performance.now();
  const timePassed = timeNow - timePreviousRef.value;
  const timeInterval = 1000 / 60;

  if (timePassed < timeInterval) return;
  timePreviousRef.value = timeNow - (timePassed % timeInterval);

  const ctx = canvasRef.value?.getContext('2d');
  if (!ctx || !canvasRef.value) return;

  ctx.clearRect(0, 0, canvasRef.value.width, canvasRef.value.height);

  for (let i = 0; i < pixelsRef.value.length; i++) {
    const pixel = pixelsRef.value[i];
    pixel.appear(); // Always appearing/shimmering
  }
};

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  initPixels();

  // Start animation
  animationRef.value = requestAnimationFrame(animate);

  // Handle resize
  resizeObserver = new ResizeObserver(() => {
    initPixels();
  });
  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  if (animationRef.value !== null) {
    cancelAnimationFrame(animationRef.value);
  }
});
</script>
