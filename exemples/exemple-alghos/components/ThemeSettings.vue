<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTheme, ACCENT_COLORS } from '../composables/useTheme'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  theme,
  setAccentPreset,
  setAccentColor,
  setMatrixEffects,
  setScanlines,
  setGlow,
  setGlowIntensity,
  setBorderRadius,
  setGlassMorphism,
  setGlassOpacity,
  setAnimationSpeed,
  setReduceMotion,
  setFontFamily,
  setFontSize,
  setAnarchyMode,
  setAnarchyHue,
  setAnarchyNoise,
  setAnarchyGlitch,
  resetTheme,
  exportTheme,
  importTheme,
} = useTheme()

const activeTab = ref<'colors' | 'effects' | 'ui' | 'anarchy'>('colors')
const customColor = ref(theme.value.accentColor)
const importJson = ref('')
const showImport = ref(false)

const currentPreset = computed(() => 
  ACCENT_COLORS.find(c => c.color === theme.value.accentColor)?.id || 'custom'
)

function handleColorChange(e: Event) {
  const color = (e.target as HTMLInputElement).value
  customColor.value = color
  setAccentColor(color)
}

function handleExport() {
  const json = exportTheme()
  navigator.clipboard.writeText(json)
}

function handleImport() {
  if (importTheme(importJson.value)) {
    showImport.value = false
    importJson.value = ''
  }
}

function handleReset() {
  if (confirm('Reset all theme settings to default?')) {
    resetTheme()
  }
}
</script>

<template>
  <div class="theme-overlay" @click.self="emit('close')">
    <div class="theme-panel">
      <!-- Header -->
      <div class="panel-header">
        <h2>Theme Customization</h2>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tabs -->
      <div class="tabs">
        <button 
          v-for="tab in ['colors', 'effects', 'ui', 'anarchy']" 
          :key="tab"
          :class="['tab', { active: activeTab === tab }]"
          @click="activeTab = tab as typeof activeTab"
        >
          {{ tab === 'colors' ? '🎨 Colors' : tab === 'effects' ? '✨ Effects' : tab === 'ui' ? '🎛️ UI' : '🔥 Anarchy' }}
        </button>
      </div>

      <!-- Content -->
      <div class="panel-content">
        <!-- Colors Tab -->
        <div v-if="activeTab === 'colors'" class="tab-content">
          <div class="section">
            <label class="section-label">Accent Color Presets</label>
            <div class="color-grid">
              <button
                v-for="color in ACCENT_COLORS"
                :key="color.id"
                :class="['color-btn', { active: currentPreset === color.id }]"
                :style="{ '--color': color.color, '--secondary': color.secondary }"
                @click="setAccentPreset(color.id)"
                :title="color.name"
              >
                <span class="color-preview" />
                <span class="color-name">{{ color.name }}</span>
              </button>
            </div>
          </div>

          <div class="section">
            <label class="section-label">Custom Color</label>
            <div class="color-picker-row">
              <input 
                type="color" 
                :value="customColor"
                @input="handleColorChange"
                class="color-picker"
              />
              <input 
                type="text" 
                :value="customColor"
                @change="(e) => setAccentColor((e.target as HTMLInputElement).value)"
                class="color-input"
                placeholder="#00ff41"
              />
            </div>
          </div>
        </div>

        <!-- Effects Tab -->
        <div v-if="activeTab === 'effects'" class="tab-content">
          <div class="section">
            <label class="section-label">Visual Effects</label>
            
            <div class="toggle-row">
              <span>Matrix Ambient Glow</span>
              <button 
                :class="['toggle', { active: theme.matrixEffects }]"
                @click="setMatrixEffects(!theme.matrixEffects)"
              >
                <span class="toggle-thumb" />
              </button>
            </div>

            <div class="toggle-row">
              <span>CRT Scanlines</span>
              <button 
                :class="['toggle', { active: theme.scanlines }]"
                @click="setScanlines(!theme.scanlines)"
              >
                <span class="toggle-thumb" />
              </button>
            </div>

            <div class="toggle-row">
              <span>Neon Glow Effects</span>
              <button 
                :class="['toggle', { active: theme.glow }]"
                @click="setGlow(!theme.glow)"
              >
                <span class="toggle-thumb" />
              </button>
            </div>
          </div>

          <div class="section">
            <label class="section-label">Glow Intensity: {{ theme.glowIntensity }}%</label>
            <input 
              type="range" 
              min="0" 
              max="100" 
              :value="theme.glowIntensity"
              @input="(e) => setGlowIntensity(Number((e.target as HTMLInputElement).value))"
              class="slider"
            />
          </div>

          <div class="section">
            <label class="section-label">Glass Morphism</label>
            
            <div class="toggle-row">
              <span>Enable Glass Effect</span>
              <button 
                :class="['toggle', { active: theme.glassMorphism }]"
                @click="setGlassMorphism(!theme.glassMorphism)"
              >
                <span class="toggle-thumb" />
              </button>
            </div>

            <label class="section-label" style="margin-top: 1rem;">Glass Opacity: {{ theme.glassOpacity }}%</label>
            <input 
              type="range" 
              min="50" 
              max="100" 
              :value="theme.glassOpacity"
              @input="(e) => setGlassOpacity(Number((e.target as HTMLInputElement).value))"
              class="slider"
              :disabled="!theme.glassMorphism"
            />
          </div>
        </div>

        <!-- UI Tab -->
        <div v-if="activeTab === 'ui'" class="tab-content">
          <div class="section">
            <label class="section-label">Border Radius</label>
            <div class="radio-group">
              <button 
                v-for="radius in ['sharp', 'soft', 'round']" 
                :key="radius"
                :class="['radio-btn', { active: theme.borderRadius === radius }]"
                @click="setBorderRadius(radius as 'sharp' | 'soft' | 'round')"
              >
                <span class="radius-preview" :class="radius" />
                {{ radius }}
              </button>
            </div>
          </div>

          <div class="section">
            <label class="section-label">Animation Speed</label>
            <div class="radio-group">
              <button 
                v-for="speed in ['instant', 'fast', 'normal', 'slow']" 
                :key="speed"
                :class="['radio-btn', { active: theme.animationSpeed === speed }]"
                @click="setAnimationSpeed(speed as 'instant' | 'fast' | 'normal' | 'slow')"
              >
                {{ speed }}
              </button>
            </div>
          </div>

          <div class="section">
            <label class="section-label">Typography</label>
            
            <div class="select-row">
              <span>Font Family</span>
              <select 
                :value="theme.fontFamily"
                @change="(e) => setFontFamily((e.target as HTMLSelectElement).value as 'inter' | 'system' | 'mono')"
                class="select"
              >
                <option value="inter">Inter</option>
                <option value="system">System</option>
                <option value="mono">Monospace</option>
              </select>
            </div>

            <div class="select-row">
              <span>Font Size</span>
              <select 
                :value="theme.fontSize"
                @change="(e) => setFontSize((e.target as HTMLSelectElement).value as 'compact' | 'normal' | 'large')"
                class="select"
              >
                <option value="compact">Compact</option>
                <option value="normal">Normal</option>
                <option value="large">Large</option>
              </select>
            </div>
          </div>

          <div class="section">
            <label class="section-label">Accessibility</label>
            <div class="toggle-row">
              <span>Reduce Motion</span>
              <button 
                :class="['toggle', { active: theme.reduceMotion }]"
                @click="setReduceMotion(!theme.reduceMotion)"
              >
                <span class="toggle-thumb" />
              </button>
            </div>
          </div>
        </div>

        <!-- Anarchy Tab -->
        <div v-if="activeTab === 'anarchy'" class="tab-content">
          <div class="anarchy-warning">
            <span class="warning-icon">⚠️</span>
            <p>Anarchy Mode enables chaotic visual effects. Use at your own risk!</p>
          </div>

          <div class="section">
            <div class="toggle-row large">
              <div>
                <span class="toggle-label">🔥 Enable Anarchy Mode</span>
                <span class="toggle-desc">Unleash visual chaos</span>
              </div>
              <button 
                :class="['toggle', { active: theme.anarchyMode }]"
                @click="setAnarchyMode(!theme.anarchyMode)"
              >
                <span class="toggle-thumb" />
              </button>
            </div>
          </div>

          <div class="section" :class="{ disabled: !theme.anarchyMode }">
            <label class="section-label">Hue Rotation: {{ theme.anarchyHue }}°</label>
            <input 
              type="range" 
              min="0" 
              max="360" 
              :value="theme.anarchyHue"
              @input="(e) => setAnarchyHue(Number((e.target as HTMLInputElement).value))"
              class="slider hue-slider"
              :disabled="!theme.anarchyMode"
            />
          </div>

          <div class="section" :class="{ disabled: !theme.anarchyMode }">
            <div class="toggle-row">
              <span>Noise Overlay</span>
              <button 
                :class="['toggle', { active: theme.anarchyNoise }]"
                @click="setAnarchyNoise(!theme.anarchyNoise)"
                :disabled="!theme.anarchyMode"
              >
                <span class="toggle-thumb" />
              </button>
            </div>

            <div class="toggle-row">
              <span>Glitch Effect</span>
              <button 
                :class="['toggle', { active: theme.anarchyGlitch }]"
                @click="setAnarchyGlitch(!theme.anarchyGlitch)"
                :disabled="!theme.anarchyMode"
              >
                <span class="toggle-thumb" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="panel-footer">
        <button class="footer-btn" @click="handleExport" title="Copy theme to clipboard">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
            <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
          </svg>
          Export
        </button>
        <button class="footer-btn" @click="showImport = !showImport">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7,10 12,15 17,10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          Import
        </button>
        <button class="footer-btn danger" @click="handleReset">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
            <path d="M3 3v5h5" />
          </svg>
          Reset
        </button>
      </div>

      <!-- Import Modal -->
      <Transition name="fade">
        <div v-if="showImport" class="import-modal">
          <textarea 
            v-model="importJson" 
            placeholder="Paste theme JSON here..."
            class="import-textarea"
          />
          <div class="import-actions">
            <button class="btn-secondary" @click="showImport = false">Cancel</button>
            <button class="btn-primary" @click="handleImport">Import</button>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.theme-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.theme-panel {
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  background: var(--surface-1);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-xl);
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--border-subtle);
}

.panel-header h2 {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all var(--duration-fast) var(--ease-out);
}

.close-btn:hover {
  background: var(--surface-2);
  color: var(--text-primary);
}

.close-btn svg {
  width: 1.25rem;
  height: 1.25rem;
}

/* Tabs */
.tabs {
  display: flex;
  padding: 0 1rem;
  border-bottom: 1px solid var(--border-subtle);
  gap: 0.25rem;
}

.tab {
  padding: 0.75rem 1rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-tertiary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.tab:hover {
  color: var(--text-secondary);
}

.tab.active {
  color: var(--accent-color);
  border-bottom-color: var(--accent-color);
}

/* Content */
.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem 1.5rem;
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.section.disabled {
  opacity: 0.5;
  pointer-events: none;
}

.section-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Color Grid */
.color-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 0.5rem;
}

.color-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem;
  background: var(--surface-2);
  border: 2px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.color-btn:hover {
  background: var(--surface-3);
}

.color-btn.active {
  border-color: var(--color);
  box-shadow: 0 0 12px var(--color);
}

.color-preview {
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--color), var(--secondary));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.color-name {
  font-size: 0.625rem;
  color: var(--text-tertiary);
}

/* Color Picker */
.color-picker-row {
  display: flex;
  gap: 0.75rem;
}

.color-picker {
  width: 3rem;
  height: 2.5rem;
  padding: 0;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  background: transparent;
}

.color-picker::-webkit-color-swatch-wrapper {
  padding: 0;
}

.color-picker::-webkit-color-swatch {
  border: 2px solid var(--border-default);
  border-radius: var(--radius-md);
}

.color-input {
  flex: 1;
  padding: 0.625rem 0.875rem;
  font-family: var(--font-mono);
  font-size: 0.875rem;
  color: var(--text-primary);
  background: var(--surface-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
}

.color-input:focus {
  outline: none;
  border-color: var(--accent-color);
}

/* Toggle */
.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0;
}

.toggle-row span {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.toggle-row.large {
  padding: 0.75rem 0;
}

.toggle-row.large > div {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.toggle-label {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--text-primary);
}

.toggle-desc {
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

.toggle {
  width: 2.75rem;
  height: 1.5rem;
  padding: 2px;
  background: var(--surface-3);
  border: none;
  border-radius: var(--radius-full);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.toggle.active {
  background: var(--accent-color);
}

.toggle:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toggle-thumb {
  display: block;
  width: 1.25rem;
  height: 1.25rem;
  background: white;
  border-radius: 50%;
  transition: transform var(--duration-fast) var(--ease-spring);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.toggle.active .toggle-thumb {
  transform: translateX(1.25rem);
}

/* Slider */
.slider {
  width: 100%;
  height: 4px;
  background: var(--surface-3);
  border-radius: var(--radius-full);
  appearance: none;
  cursor: pointer;
}

.slider::-webkit-slider-thumb {
  appearance: none;
  width: 1rem;
  height: 1rem;
  background: var(--accent-color);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 0 8px var(--accent-glow);
}

.slider:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hue-slider {
  background: linear-gradient(to right, 
    hsl(0, 100%, 50%), 
    hsl(60, 100%, 50%), 
    hsl(120, 100%, 50%), 
    hsl(180, 100%, 50%), 
    hsl(240, 100%, 50%), 
    hsl(300, 100%, 50%), 
    hsl(360, 100%, 50%)
  );
}

/* Radio Group */
.radio-group {
  display: flex;
  gap: 0.5rem;
}

.radio-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.375rem;
  padding: 0.75rem 0.5rem;
  font-size: 0.75rem;
  color: var(--text-tertiary);
  background: var(--surface-2);
  border: 2px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
  text-transform: capitalize;
}

.radio-btn:hover {
  background: var(--surface-3);
  color: var(--text-secondary);
}

.radio-btn.active {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.radius-preview {
  width: 1.5rem;
  height: 1.5rem;
  background: var(--accent-color);
}

.radius-preview.sharp { border-radius: 2px; }
.radius-preview.soft { border-radius: 6px; }
.radius-preview.round { border-radius: 50%; }

/* Select */
.select-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0;
}

.select-row span {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.select {
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  color: var(--text-primary);
  background: var(--surface-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  cursor: pointer;
}

.select:focus {
  outline: none;
  border-color: var(--accent-color);
}

/* Anarchy Warning */
.anarchy-warning {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 0.875rem 1rem;
  background: rgba(var(--warning-rgb), 0.1);
  border: 1px solid rgba(var(--warning-rgb), 0.3);
  border-radius: var(--radius-md);
}

.warning-icon {
  font-size: 1.25rem;
}

.anarchy-warning p {
  font-size: 0.8125rem;
  color: var(--warning);
  line-height: 1.4;
}

/* Footer */
.panel-footer {
  display: flex;
  gap: 0.5rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-subtle);
}

.footer-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.625rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--surface-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.footer-btn:hover {
  background: var(--surface-3);
  color: var(--text-primary);
}

.footer-btn.danger:hover {
  background: rgba(var(--error-rgb), 0.15);
  border-color: rgba(var(--error-rgb), 0.3);
  color: var(--error);
}

.footer-btn svg {
  width: 1rem;
  height: 1rem;
}

/* Import Modal */
.import-modal {
  position: absolute;
  inset: 0;
  background: var(--surface-1);
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.import-textarea {
  flex: 1;
  padding: 1rem;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--text-primary);
  background: var(--surface-0);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  resize: none;
}

.import-textarea:focus {
  outline: none;
  border-color: var(--accent-color);
}

.import-actions {
  display: flex;
  gap: 0.75rem;
  justify-content: flex-end;
}

.btn-secondary,
.btn-primary {
  padding: 0.625rem 1.25rem;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.btn-secondary {
  background: var(--surface-2);
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
}

.btn-primary {
  background: var(--accent-color);
  border: none;
  color: var(--void);
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--duration-normal) var(--ease-out);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
