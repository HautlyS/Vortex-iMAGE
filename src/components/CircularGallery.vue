<template>
  <div class="relative w-full h-full"
    @touchstart="handleTouchStart"
    @touchmove="handleTouchMove" 
    @touchend="handleTouchEnd"
  >
    <div ref="containerRef" class="w-full h-full overflow-hidden cursor-grab active:cursor-grabbing" />
    
    <!-- Center image info overlay -->
    <div v-if="centerItem" class="absolute bottom-[12%] left-1/2 -translate-x-1/2 pointer-events-none text-center">
      <span class="text-white text-xl font-semibold drop-shadow-[0_2px_12px_rgba(0,0,0,0.9)]">{{ centerItem.text }}</span>
      
      <!-- Status icons -->
      <div class="flex items-center justify-center gap-3 mt-2">
        <span v-if="centerItem.syncedGithub" class="text-green-400 text-xs flex items-center gap-1" title="Synced to GitHub">
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
        </span>
        <span v-if="centerItem.syncedLocal" class="text-blue-400 text-xs flex items-center gap-1" title="Saved locally">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4"/></svg>
        </span>
        <span v-if="centerItem.favorite" class="text-red-500 text-xs">
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"/></svg>
        </span>
      </div>
      
      <!-- Metadata -->
      <div class="flex flex-wrap items-center justify-center gap-x-3 gap-y-1 mt-2 text-white/70 text-xs drop-shadow-[0_1px_4px_rgba(0,0,0,0.9)]">
        <span v-if="centerItem.size">{{ formatSize(centerItem.size) }}</span>
        <span v-if="centerItem.dimensions">{{ centerItem.dimensions }}</span>
        <span v-if="centerItem.date">{{ formatDate(centerItem.date) }}</span>
        <span v-if="centerItem.camera">📷 {{ centerItem.camera }}</span>
        <span v-if="centerItem.location">📍 {{ centerItem.location }}</span>
        <span v-if="centerItem.iso">ISO {{ centerItem.iso }}</span>
        <span v-if="centerItem.aperture">f/{{ centerItem.aperture }}</span>
        <span v-if="centerItem.shutter">{{ centerItem.shutter }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, useTemplateRef } from 'vue';
import { Camera as OGLCamera, Mesh, Plane, Program, Renderer, Texture, Transform } from 'ogl';
import { useGitHubAuth } from '../composables/useGitHubAuth';
import { invoke } from '@tauri-apps/api/core';

interface GalleryItem {
  image: string;
  text: string;
  syncedGithub?: boolean;
  syncedLocal?: boolean;
  favorite?: boolean;
  // Metadata
  size?: number;
  dimensions?: string;
  date?: string | Date;
  camera?: string;
  location?: string;
  iso?: number;
  aperture?: number;
  shutter?: string;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

function formatDate(date: string | Date): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toLocaleDateString('pt-BR', { day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' });
}

interface CircularGalleryProps {
  items?: GalleryItem[];
  bend?: number;
  textColor?: string;
  borderRadius?: number;
  font?: string;
  scrollSpeed?: number;
  scrollEase?: number;
  initialIndex?: number;
  showTitles?: boolean;
}

const props = withDefaults(defineProps<CircularGalleryProps>(), {
  bend: 3,
  textColor: '#ffffff',
  borderRadius: 0.05,
  font: 'bold 30px Figtree',
  scrollSpeed: 2,
  scrollEase: 0.05,
  showTitles: false
});

const containerRef = useTemplateRef<HTMLDivElement>('containerRef');
const centerItem = ref<GalleryItem | null>(null);
const centerIndex = ref(0);
const auth = useGitHubAuth();
let app: App | null = null;

const emit = defineEmits<{
  'index-change': [index: number]
}>();
let rafId: number | null = null;

// Pinch-to-zoom for circular gallery
let initialPinchDistance = 0
let isPinching = false
let currentZoom = 1

const getDistance = (touch1: Touch, touch2: Touch) => {
  const dx = touch1.clientX - touch2.clientX
  const dy = touch1.clientY - touch2.clientY
  return Math.sqrt(dx * dx + dy * dy)
}

const handleTouchStart = (event: TouchEvent) => {
  if (event.touches.length === 2) {
    isPinching = true
    initialPinchDistance = getDistance(event.touches[0], event.touches[1])
    event.preventDefault()
  }
}

const handleTouchMove = (event: TouchEvent) => {
  if (isPinching && event.touches.length === 2) {
    const currentDistance = getDistance(event.touches[0], event.touches[1])
    const scaleChange = currentDistance / initialPinchDistance
    currentZoom = Math.max(0.5, Math.min(3, currentZoom * scaleChange))
    
    if (app) {
      app.setZoom(currentZoom)
    }
    
    initialPinchDistance = currentDistance
    event.preventDefault()
  }
}

const handleTouchEnd = () => {
  if (isPinching) {
    isPinching = false
  }
}

function updateCenterItem() {
  if (app && props.items?.length) {
    const idx = app.getCenterIndex();
    if (idx !== centerIndex.value) {
      centerIndex.value = idx;
      emit('index-change', idx);
    }
    centerItem.value = props.items[idx] || null;
  }
  rafId = requestAnimationFrame(updateCenterItem);
}

type GL = Renderer['gl'];

function debounce<T extends (...args: unknown[]) => void>(func: T, wait: number) {
  let timeout: number;
  return function (this: unknown, ...args: Parameters<T>) {
    window.clearTimeout(timeout);
    timeout = window.setTimeout(() => func.apply(this, args), wait);
  };
}

function lerp(p1: number, p2: number, t: number): number {
  return p1 + (p2 - p1) * t;
}

function autoBind<T extends object>(instance: T): void {
  const proto = Object.getPrototypeOf(instance) as Record<string, unknown> | null;
  if (!proto) return;
  Object.getOwnPropertyNames(proto).forEach(key => {
    if (key !== 'constructor') {
      const desc = Object.getOwnPropertyDescriptor(proto, key);
      if (desc && typeof desc.value === 'function') {
        const fn = desc.value as (...args: unknown[]) => unknown;
        (instance as Record<string, unknown>)[key] = fn.bind(instance);
      }
    }
  });
}

function getFontSize(font: string): number {
  const match = font.match(/(\d+)px/);
  return match ? parseInt(match[1], 10) : 30;
}

function createTextTexture(
  gl: GL,
  text: string,
  font: string = 'bold 30px monospace',
  color: string = 'black'
): { texture: Texture; width: number; height: number } {
  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d');
  if (!context) throw new Error('Could not get 2d context');

  context.font = font;
  const metrics = context.measureText(text);
  const textWidth = Math.ceil(metrics.width);
  const fontSize = getFontSize(font);
  const textHeight = Math.ceil(fontSize * 1.2);

  canvas.width = textWidth + 20;
  canvas.height = textHeight + 20;

  context.font = font;
  context.fillStyle = color;
  context.textBaseline = 'middle';
  context.textAlign = 'center';
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.fillText(text, canvas.width / 2, canvas.height / 2);

  const texture = new Texture(gl, { generateMipmaps: false });
  texture.image = canvas;
  return { texture, width: canvas.width, height: canvas.height };
}

interface TitleProps {
  gl: GL;
  plane: Mesh;
  renderer: Renderer;
  text: string;
  textColor?: string;
  font?: string;
}

class Title {
  gl: GL;
  plane: Mesh;
  renderer: Renderer;
  text: string;
  textColor: string;
  font: string;
  mesh!: Mesh;

  constructor({ gl, plane, renderer, text, textColor = '#545050', font = '30px sans-serif' }: TitleProps) {
    autoBind(this);
    this.gl = gl;
    this.plane = plane;
    this.renderer = renderer;
    this.text = text;
    this.textColor = textColor;
    this.font = font;
    this.createMesh();
  }

  createMesh() {
    const { texture, width, height } = createTextTexture(this.gl, this.text, this.font, this.textColor);
    const geometry = new Plane(this.gl);
    const program = new Program(this.gl, {
      vertex: `
        attribute vec3 position;
        attribute vec2 uv;
        uniform mat4 modelViewMatrix;
        uniform mat4 projectionMatrix;
        varying vec2 vUv;
        void main() {
          vUv = uv;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragment: `
        precision highp float;
        uniform sampler2D tMap;
        varying vec2 vUv;
        void main() {
          vec4 color = texture2D(tMap, vUv);
          if (color.a < 0.1) discard;
          gl_FragColor = color;
        }
      `,
      uniforms: { tMap: { value: texture } },
      transparent: true
    });
    this.mesh = new Mesh(this.gl, { geometry, program });
    const aspect = width / height;
    const textHeightScaled = this.plane.scale.y * 0.15;
    const textWidthScaled = textHeightScaled * aspect;
    this.mesh.scale.set(textWidthScaled, textHeightScaled, 1);
    this.mesh.position.y = -this.plane.scale.y * 0.5 - textHeightScaled * 0.5 - 0.05;
    this.mesh.setParent(this.plane);
  }
}

interface ScreenSize {
  width: number;
  height: number;
}

interface Viewport {
  width: number;
  height: number;
}

interface MediaProps {
  geometry: Plane;
  gl: GL;
  image: string;
  index: number;
  length: number;
  renderer: Renderer;
  scene: Transform;
  screen: ScreenSize;
  text: string;
  viewport: Viewport;
  bend: number;
  textColor: string;
  borderRadius?: number;
  font?: string;
  showTitle?: boolean;
  auth?: any;
}


class Media {
  extra: number = 0;
  geometry: Plane;
  gl: GL;
  image: string;
  index: number;
  length: number;
  renderer: Renderer;
  scene: Transform;
  screen: ScreenSize;
  text: string;
  viewport: Viewport;
  bend: number;
  textColor: string;
  borderRadius: number;
  font?: string;
  showTitle: boolean;
  auth?: any;
  program!: Program;
  plane!: Mesh;
  title?: Title;
  scale!: number;
  padding!: number;
  width!: number;
  widthTotal!: number;
  x!: number;
  speed: number = 0;
  isBefore: boolean = false;
  isAfter: boolean = false;
  
  // Store base scales to apply dynamic scaling on top
  baseScaleX: number = 1;
  baseScaleY: number = 1;

  constructor({
    geometry,
    gl,
    image,
    index,
    length,
    renderer,
    scene,
    screen,
    text,
    viewport,
    bend,
    textColor,
    borderRadius = 0,
    font,
    showTitle = false,
    auth
  }: MediaProps) {
    this.geometry = geometry;
    this.gl = gl;
    this.image = image;
    this.index = index;
    this.length = length;
    this.renderer = renderer;
    this.scene = scene;
    this.screen = screen;
    this.text = text;
    this.viewport = viewport;
    this.bend = bend;
    this.textColor = textColor;
    this.borderRadius = borderRadius;
    this.font = font;
    this.showTitle = showTitle;
    this.auth = auth;
    this.createShader();
    this.createMesh();
    if (this.showTitle) this.createTitle();
    this.onResize();
  }

  createShader() {
    const texture = new Texture(this.gl, { generateMipmaps: false });
    this.program = new Program(this.gl, {
      depthTest: true,
      depthWrite: true,
      vertex: `
        precision highp float;
        attribute vec3 position;
        attribute vec2 uv;
        uniform mat4 modelViewMatrix;
        uniform mat4 projectionMatrix;
        uniform float uTime;
        uniform float uSpeed;
        varying vec2 vUv;
        void main() {
          vUv = uv;
          vec3 p = position;
          p.z = (sin(p.x * 4.0 + uTime) * 1.5 + cos(p.y * 2.0 + uTime) * 1.5) * (0.1 + uSpeed * 0.5);
          gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);
        }
      `,
      fragment: `
        precision highp float;
        uniform vec2 uImageSizes;
        uniform vec2 uPlaneSizes;
        uniform sampler2D tMap;
        uniform float uBorderRadius;
        uniform float uCenterFactor;
        varying vec2 vUv;
        
        float roundedBoxSDF(vec2 p, vec2 b, float r) {
          vec2 d = abs(p) - b;
          return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - r;
        }
        
        void main() {
          vec2 ratio = vec2(
            min((uPlaneSizes.x / uPlaneSizes.y) / (uImageSizes.x / uImageSizes.y), 1.0),
            min((uPlaneSizes.y / uPlaneSizes.x) / (uImageSizes.y / uImageSizes.x), 1.0)
          );
          vec2 uv = vec2(
            vUv.x * ratio.x + (1.0 - ratio.x) * 0.5,
            vUv.y * ratio.y + (1.0 - ratio.y) * 0.5
          );
          vec4 color = texture2D(tMap, uv);
          
          float d = roundedBoxSDF(vUv - 0.5, vec2(0.5 - uBorderRadius), uBorderRadius);
          if(d > 0.0) {
            discard;
          }
          
          // Bottom shadow gradient (stronger when centered)
          float shadowStrength = smoothstep(0.35, 0.0, vUv.y) * 0.7 * uCenterFactor;
          color.rgb = mix(color.rgb, vec3(0.0), shadowStrength);
          
          gl_FragColor = vec4(color.rgb, 1.0);
        }
      `,
      uniforms: {
        tMap: { value: texture },
        uPlaneSizes: { value: [0, 0] },
        uImageSizes: { value: [0, 0] },
        uSpeed: { value: 0 },
        uTime: { value: 100 * Math.random() },
        uBorderRadius: { value: this.borderRadius },
        uCenterFactor: { value: 0 }
      },
      transparent: true
    });

    
    this.loadTexture(texture);
  }

  async loadTexture(texture: Texture) {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    
    // Check if secure load needed
    let src = this.image;
    const isGitHub = src.includes('github.com') || src.includes('githubusercontent.com');
    // Only try secure load if auth is present and it is a GitHub URL
    // And not in dev mode mock (picsum)
    if (this.auth && isGitHub && !src.includes('picsum')) {
       // Parse path
       const parts = src.split('photos/');
       if (parts.length > 1) {
         const remotePath = 'photos/' + parts[1].split('?')[0];
         const token = this.auth.token.value;
         const repo = this.auth.repo.value;
         const keypairBytes = this.auth.keypairBytes.value;
         
         if (token && repo && keypairBytes) {
           try {
             // In class context, we need to handle async loading
             // We can fetch data, create blob, set src
             const imageBytes = await invoke<number[]>('download_secure_photo', {
                remotePath,
                repo,
                token,
                keypairBytes
             });
             const blob = new Blob([new Uint8Array(imageBytes)]);
             src = URL.createObjectURL(blob);
           } catch (e) {
             console.error("Failed to load secure texture, falling back", e);
           }
         }
       }
    }

    img.src = src;
    img.onload = () => {
      texture.image = img;
      this.program.uniforms.uImageSizes.value = [img.naturalWidth, img.naturalHeight];
      // Clean up blob if we created one
      if (src.startsWith('blob:')) {
        // Keep it for now as texture needs it? No, once uploaded to GPU (texture.image = img) 
        // ogl handles it? OGL uploads on first usage.
        // We shouldn't revoke immediately if OGL hasn't uploaded yet.
        // OGL Texture uploads on update.
        // We can revoke later or let GC handle it if we trust it, but blobs need explicit revoke.
        // However, keeping it in memory for the duration of the gallery is acceptable for now.
        // Or cleaner: revoke after a timeout or when Media is destroyed.
      }
    };
  }

  createMesh() {
    this.plane = new Mesh(this.gl, {
      geometry: this.geometry,
      program: this.program
    });
    this.plane.setParent(this.scene);
  }

  createTitle() {
    this.title = new Title({
      gl: this.gl,
      plane: this.plane,
      renderer: this.renderer,
      text: this.text,
      textColor: this.textColor,
      font: this.font
    });
  }

  setZoom(zoom: number) {
    // Apply zoom to the current scale
    this.plane.scale.x = this.baseScaleX * zoom;
    this.plane.scale.y = this.baseScaleY * zoom;
    this.plane.program.uniforms.uPlaneSizes.value = [this.plane.scale.x, this.plane.scale.y];
  }

  update(scroll: { current: number; last: number }, direction: 'right' | 'left') {
    this.plane.position.x = this.x - scroll.current - this.extra;

    const x = this.plane.position.x;
    const H = this.viewport.width / 2;

    // Calculate distance from center
    const dist = Math.abs(x);
    // Center factor (1 at center, 0 at edges)
    const centerFactor = Math.max(0, 1 - (dist / H) * 1.2);
    this.program.uniforms.uCenterFactor.value = centerFactor;
    
    // Z position: center image in front, others pushed back
    this.plane.position.z = centerFactor * 5;
    
    // Scale: center image shows at full size, others smaller
    const minScale = 0.6;
    const scaleFactor = minScale + (1 - minScale) * centerFactor;
    
    // Use the base scales calculated in onResize
    this.plane.scale.x = this.baseScaleX * scaleFactor;
    this.plane.scale.y = this.baseScaleY * scaleFactor;

    // Update uPlaneSizes for correct aspect ratio in fragment shader
    this.program.uniforms.uPlaneSizes.value = [this.plane.scale.x, this.plane.scale.y];


    if (this.bend === 0) {
      this.plane.position.y = 0;
      this.plane.rotation.z = 0;
    } else {
      const B_abs = Math.abs(this.bend);
      const R = (H * H + B_abs * B_abs) / (2 * B_abs);
      const effectiveX = Math.min(Math.abs(x), H);

      const arc = R - Math.sqrt(R * R - effectiveX * effectiveX);
      if (this.bend > 0) {
        this.plane.position.y = -arc;
        this.plane.rotation.z = -Math.sign(x) * Math.asin(effectiveX / R);
      } else {
        this.plane.position.y = arc;
        this.plane.rotation.z = Math.sign(x) * Math.asin(effectiveX / R);
      }
    }

    this.speed = scroll.current - scroll.last;
    this.program.uniforms.uTime.value += 0.04;
    this.program.uniforms.uSpeed.value = this.speed;

    const planeOffset = this.plane.scale.x / 2;
    const viewportOffset = this.viewport.width / 2;
    this.isBefore = this.plane.position.x + planeOffset < -viewportOffset;
    this.isAfter = this.plane.position.x - planeOffset > viewportOffset;
    if (direction === 'right' && this.isBefore) {
      this.extra -= this.widthTotal;
      this.isBefore = this.isAfter = false;
    }
    if (direction === 'left' && this.isAfter) {
      this.extra += this.widthTotal;
      this.isBefore = this.isAfter = false;
    }
  }

  onResize({ screen, viewport }: { screen?: ScreenSize; viewport?: Viewport } = {}) {
    if (screen) this.screen = screen;
    if (viewport) {
      this.viewport = viewport;
      if (this.plane.program.uniforms.uViewportSizes) {
        this.plane.program.uniforms.uViewportSizes.value = [this.viewport.width, this.viewport.height];
      }
    }
    this.scale = this.screen.height / 1500;
    
    // Responsive scaling - adapt to screen size and orientation
    const isPortrait = this.screen.height > this.screen.width;
    const isMobile = this.screen.width < 768;
    
    if (isMobile) {
      if (isPortrait) {
        // Mobile portrait: use most of screen width, reasonable height
        this.baseScaleX = this.viewport.width * 0.9;
        this.baseScaleY = this.viewport.height * 0.7;
      } else {
        // Mobile landscape: use most of screen height, reasonable width
        this.baseScaleY = this.viewport.height * 0.8;
        this.baseScaleX = this.viewport.width * 0.6;
      }
    } else {
      // Desktop: original responsive logic
      if (isPortrait) {
        this.baseScaleX = this.viewport.width * 0.7;
        this.baseScaleY = this.viewport.height * 0.8;
      } else {
        this.baseScaleY = this.viewport.height * 0.6;
        this.baseScaleX = this.viewport.width * 0.5;
      }
    }
    
    this.plane.scale.x = this.baseScaleX;
    this.plane.scale.y = this.baseScaleY;
    
    this.plane.program.uniforms.uPlaneSizes.value = [this.plane.scale.x, this.plane.scale.y];
    this.padding = isMobile ? 0.5 : (isPortrait ? 1 : 2);
    
    this.width = this.baseScaleX + this.padding;
    this.widthTotal = this.width * this.length;
    this.x = this.width * this.index;
  }
}

interface AppConfig {
  items?: GalleryItem[];
  bend?: number;
  textColor?: string;
  borderRadius?: number;
  font?: string;
  scrollSpeed?: number;
  scrollEase?: number;
  initialIndex?: number;
  showTitles?: boolean;
  auth?: any;
}

class App {
  // ... (previous properties)
  container: HTMLElement;
  scrollSpeed: number;
  showTitles: boolean;
  scroll: {
    ease: number;
    current: number;
    target: number;
    last: number;
    position?: number;
  };
  auth?: any;
  onCheckDebounce: (...args: unknown[]) => void;
  renderer!: Renderer;
  gl!: GL;
  camera!: OGLCamera;
  scene!: Transform;
  planeGeometry!: Plane;
  medias: Media[] = [];
  mediasImages: GalleryItem[] = [];
  screen!: { width: number; height: number };
  viewport!: { width: number; height: number };
  raf: number = 0;
  zoomFactor: number = 1;

  boundOnResize!: () => void;
  boundOnWheel!: (e: Event) => void;
  boundOnTouchDown!: (e: MouseEvent | TouchEvent) => void;
  boundOnTouchMove!: (e: MouseEvent | TouchEvent) => void;
  boundOnTouchUp!: () => void;

  isDown: boolean = false;
  start: number = 0;

  constructor(
    container: HTMLElement,
    {
      items,
      bend = 1,
      textColor = '#ffffff',
      borderRadius = 0,
      font = 'bold 30px Figtree',
      scrollSpeed = 2,
      scrollEase = 0.05,
      initialIndex = 0,
      showTitles = false,
      auth
    }: AppConfig
  ) {
    document.documentElement.classList.remove('no-js');
    this.container = container;
    this.scrollSpeed = scrollSpeed;
    this.showTitles = showTitles;
    this.auth = auth;
    this.scroll = { ease: scrollEase, current: 0, target: 0, last: 0 };
    this.onCheckDebounce = debounce(this.onCheck.bind(this), 200);
    this.createRenderer();
    this.createCamera();
    this.createScene();
    this.onResize();
    this.createGeometry();
    this.createMedias(items, bend, textColor, borderRadius, font);
    
    // Set initial scroll relative to index
    if (this.medias.length > 0 && initialIndex !== undefined) {
        const width = this.medias[0].width;
        this.scroll.target = width * initialIndex;
        this.scroll.current = this.scroll.target;
        this.scroll.last = this.scroll.target;
    }

    this.update();
    this.addEventListeners();
  }

  // ... (rest of class)
  
  createRenderer() {
    this.renderer = new Renderer({ alpha: true });
    this.gl = this.renderer.gl;
    this.gl.clearColor(0, 0, 0, 0);
    this.container.appendChild(this.renderer.gl.canvas as HTMLCanvasElement);
  }

  createCamera() {
    this.camera = new OGLCamera(this.gl);
    this.camera.fov = 45;
    this.camera.position.z = 20;
  }

  createScene() {
    this.scene = new Transform();
  }

  createGeometry() {
    this.planeGeometry = new Plane(this.gl, {
      heightSegments: 50,
      widthSegments: 100
    });
  }

  createMedias(
    items: GalleryItem[] | undefined,
    bend: number = 1,
    textColor: string,
    borderRadius: number,
    font: string
  ) {
    const itemsToUse = items && items.length ? items : [];
    if (!itemsToUse.length) return;
    this.mediasImages = itemsToUse.concat(itemsToUse);
    
    this.medias = this.mediasImages.map((data, index) => {
      return new Media({
        geometry: this.planeGeometry,
        gl: this.gl,
        image: data.image,
        index,
        length: this.mediasImages.length,
        renderer: this.renderer,
        scene: this.scene,
        screen: this.screen,
        text: data.text,
        viewport: this.viewport,
        bend,
        textColor,
        borderRadius,
        font,
        showTitle: this.showTitles,
        auth: this.auth
      });
    });
  }

  setZoom(zoom: number) {
    this.zoomFactor = zoom;
    if (this.medias) {
      this.medias.forEach(media => media.setZoom(zoom));
    }
  }

  getCenterIndex(): number {
    if (!this.medias.length) return -1;
    const width = this.medias[0].width;
    const idx = Math.round(this.scroll.current / width);
    return Math.abs(idx) % (this.mediasImages.length / 2);
  }

  onTouchDown(e: MouseEvent | TouchEvent) {
    this.isDown = true;
    this.scroll.position = this.scroll.current;
    this.start = 'touches' in e ? e.touches[0].clientX : e.clientX;
  }

  onTouchMove(e: MouseEvent | TouchEvent) {
    if (!this.isDown) return;
    const x = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const distance = (this.start - x) * (this.scrollSpeed * 0.025);
    this.scroll.target = (this.scroll.position ?? 0) + distance;
  }

  onTouchUp() {
    this.isDown = false;
    this.onCheck();
  }

  onWheel(e: Event) {
    const wheelEvent = e as WheelEvent;
    // Support legacy wheel events if present
    const legacy = wheelEvent as unknown as { wheelDelta?: number; detail?: number };
    const delta = wheelEvent.deltaY ?? legacy.wheelDelta ?? legacy.detail ?? 0;
    this.scroll.target += delta > 0 ? this.scrollSpeed : -this.scrollSpeed;
    this.onCheckDebounce();
  }

  onCheck() {
    if (!this.medias || !this.medias[0]) return;
    const width = this.medias[0].width;
    const itemIndex = Math.round(Math.abs(this.scroll.target) / width);
    const item = width * itemIndex;
    this.scroll.target = this.scroll.target < 0 ? -item : item;
  }

  onResize() {
    this.screen = {
      width: this.container.clientWidth,
      height: this.container.clientHeight
    };
    this.renderer.setSize(this.screen.width, this.screen.height);
    this.camera.perspective({
      aspect: this.screen.width / this.screen.height
    });
    const fov = (this.camera.fov * Math.PI) / 180;
    const height = 2 * Math.tan(fov / 2) * this.camera.position.z;
    const width = height * this.camera.aspect;
    this.viewport = { width, height };
    if (this.medias) {
      this.medias.forEach(media => media.onResize({ screen: this.screen, viewport: this.viewport }));
    }
  }

  update() {
    this.scroll.current = lerp(this.scroll.current, this.scroll.target, this.scroll.ease);
    const direction = this.scroll.current > this.scroll.last ? 'right' : 'left';
    if (this.medias) {
      this.medias.forEach(media => media.update(this.scroll, direction));
    }
    this.renderer.render({ scene: this.scene, camera: this.camera });
    this.scroll.last = this.scroll.current;
    this.raf = window.requestAnimationFrame(this.update.bind(this));
  }

  addEventListeners() {
    this.boundOnResize = this.onResize.bind(this);
    this.boundOnWheel = this.onWheel.bind(this);
    this.boundOnTouchDown = this.onTouchDown.bind(this);
    this.boundOnTouchMove = this.onTouchMove.bind(this);
    this.boundOnTouchUp = this.onTouchUp.bind(this);

    window.addEventListener('resize', this.boundOnResize);

    this.container.addEventListener('wheel', this.boundOnWheel);
    this.container.addEventListener('mousedown', this.boundOnTouchDown);
    this.container.addEventListener('touchstart', this.boundOnTouchDown);

    window.addEventListener('mousemove', this.boundOnTouchMove);
    window.addEventListener('mouseup', this.boundOnTouchUp);
    window.addEventListener('touchmove', this.boundOnTouchMove);
    window.addEventListener('touchend', this.boundOnTouchUp);
  }

  destroy() {
    window.cancelAnimationFrame(this.raf);

    window.removeEventListener('resize', this.boundOnResize);
    window.removeEventListener('mousemove', this.boundOnTouchMove);
    window.removeEventListener('mouseup', this.boundOnTouchUp);
    window.removeEventListener('touchmove', this.boundOnTouchMove);
    window.removeEventListener('touchend', this.boundOnTouchUp);

    this.container.removeEventListener('wheel', this.boundOnWheel);
    this.container.removeEventListener('mousedown', this.boundOnTouchDown);
    this.container.removeEventListener('touchstart', this.boundOnTouchDown);

    if (this.renderer && this.renderer.gl && this.renderer.gl.canvas.parentNode) {
      this.renderer.gl.canvas.parentNode.removeChild(this.renderer.gl.canvas as HTMLCanvasElement);
    }
  }
}

onMounted(() => {
  if (!containerRef.value) return;

  app = new App(containerRef.value, {
    items: props.items,
    bend: props.bend,
    textColor: props.textColor,
    borderRadius: props.borderRadius,
    font: props.font,
    scrollSpeed: props.scrollSpeed,
    scrollEase: props.scrollEase,
    initialIndex: props.initialIndex,
    showTitles: props.showTitles,
    auth
  });
  updateCenterItem();
});


onUnmounted(() => {
  if (rafId) cancelAnimationFrame(rafId);
  if (app) {
    app.destroy();
    app = null;
  }
});

watch(
  () => ({
    items: props.items,
    bend: props.bend,
    textColor: props.textColor,
    borderRadius: props.borderRadius,
    font: props.font,
    scrollSpeed: props.scrollSpeed,
    scrollEase: props.scrollEase,
    initialIndex: props.initialIndex,
    showTitles: props.showTitles
  }),
  newProps => {
    if (app) {
      app.destroy();
    }
    if (containerRef.value) {
      app = new App(containerRef.value, { ...newProps, auth });
    }
  },
  { deep: true }
);
</script>
