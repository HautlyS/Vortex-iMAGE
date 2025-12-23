/**
 * Vue Component - 0 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: 
 */

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTheme, ACCENT_COLORS } from '../composables/useTheme'

const emit = defineEmits<{ close: [] }>()

const {
  theme,
  setAccentPreset,
  setAccentColor,
  setBorderRadius,
  setGlassOpacity,
  setGlowIntensity,
  resetTheme,
} = useTheme()

const customColor = ref(theme.value.accentColor)
const customSecondary = ref(theme.value.accentSecondary)

const currentPreset = computed(() => 
  ACCENT_COLORS.find(c => c.color === theme.value.accentColor)?.id || 'custom'
)

function handlePrimaryChange(e: Event) {
  const color = (e.target as HTMLInputElement).value
  customColor.value = color
  setAccentColor(color, customSecondary.value)
}

function handleSecondaryChange(e: Event) {
  const color = (e.target as HTMLInputElement).value
  customSecondary.value = color
  setAccentColor(customColor.value, color)
}

function selectPreset(id: string) {
  const preset = ACCENT_COLORS.find(c => c.id === id)
  if (preset) {
    customColor.value = preset.color
    customSecondary.value = preset.secondary
    setAccentPreset(id)
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="panel">
      <header class="header">
        <h2>Aparência</h2>
        <button class="close" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>
      </header>

      <div class="content">
        <!-- Accent Colors -->
        <section>
          <label>Cor de Destaque</label>
          <div class="presets">
            <button
              v-for="c in ACCENT_COLORS"
              :key="c.id"
              :class="['preset', { active: currentPreset === c.id }]"
              :style="{ '--c1': c.color, '--c2': c.secondary }"
              @click="selectPreset(c.id)"
              :title="c.name"
            />
          </div>
        </section>

        <!-- Custom Colors -->
        <section>
          <label>Cores Personalizadas</label>
          <div class="color-row">
            <div class="color-field">
              <span>Primária</span>
              <div class="color-input-group">
                <input type="color" :value="customColor" @input="handlePrimaryChange" />
                <input type="text" :value="customColor" @change="handlePrimaryChange" />
              </div>
            </div>
            <div class="color-field">
              <span>Secundária</span>
              <div class="color-input-group">
                <input type="color" :value="customSecondary" @input="handleSecondaryChange" />
                <input type="text" :value="customSecondary" @change="handleSecondaryChange" />
              </div>
            </div>
          </div>
        </section>

        <!-- Transparency -->
        <section>
          <label>Transparência: {{ 100 - theme.glassOpacity }}%</label>
          <input 
            type="range" 
            min="40" 
            max="95" 
            :value="theme.glassOpacity"
            @input="(e) => setGlassOpacity(Number((e.target as HTMLInputElement).value))"
            class="slider"
          />
        </section>

        <!-- Glow -->
        <section>
          <label>Intensidade do Brilho: {{ theme.glowIntensity }}%</label>
          <input 
            type="range" 
            min="0" 
            max="100" 
            :value="theme.glowIntensity"
            @input="(e) => setGlowIntensity(Number((e.target as HTMLInputElement).value))"
            class="slider"
          />
        </section>

        <!-- Border Radius -->
        <section>
          <label>Bordas</label>
          <div class="radius-options">
            <button 
              v-for="r in (['sharp', 'soft', 'round'] as const)" 
              :key="r"
              :class="['radius-btn', { active: theme.borderRadius === r }]"
              @click="setBorderRadius(r)"
            >
              <span :class="['radius-preview', r]" />
              {{ r === 'sharp' ? 'Retas' : r === 'soft' ? 'Suaves' : 'Redondas' }}
            </button>
          </div>
        </section>
      </div>

      <footer class="footer">
        <button class="reset" @click="resetTheme">Restaurar Padrão</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(12px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.panel {
  width: 100%;
  max-width: 400px;
  background: rgba(var(--surface-1-rgb, 14, 14, 20), 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: var(--radius-xl);
  overflow: hidden;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6);
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.header h2 {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.close {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all 0.15s;
}
.close:hover { background: rgba(255,255,255,0.08); color: var(--text-primary); }
.close svg { width: 1.25rem; height: 1.25rem; }

.content {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.75rem;
}

section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.presets {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.preset {
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--radius-md);
  border: 2px solid transparent;
  background: linear-gradient(135deg, var(--c1), var(--c2));
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
}
.preset:hover { transform: scale(1.08); }
.preset.active { 
  border-color: #fff; 
  box-shadow: 0 0 0 2px var(--c1), 0 4px 16px rgba(0,0,0,0.4);
}

.color-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.color-field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.color-field > span {
  font-size: 0.6875rem;
  color: var(--text-tertiary);
}

.color-input-group {
  display: flex;
  gap: 0.5rem;
}
.color-input-group input[type="color"] {
  width: 2.5rem;
  height: 2.5rem;
  padding: 0;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  background: transparent;
}
.color-input-group input[type="color"]::-webkit-color-swatch-wrapper { padding: 2px; }
.color-input-group input[type="color"]::-webkit-color-swatch { 
  border: 1px solid rgba(255,255,255,0.15); 
  border-radius: var(--radius-sm); 
}
.color-input-group input[type="text"] {
  flex: 1;
  min-width: 0;
  padding: 0.5rem 0.625rem;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--text-primary);
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: var(--radius-sm);
}
.color-input-group input[type="text"]:focus {
  outline: none;
  border-color: var(--accent-color);
}

.slider {
  width: 100%;
  height: 6px;
  background: rgba(255,255,255,0.1);
  border-radius: var(--radius-full);
  appearance: none;
  cursor: pointer;
}
.slider::-webkit-slider-thumb {
  appearance: none;
  width: 18px;
  height: 18px;
  background: var(--accent-color);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 0 12px var(--accent-glow);
  transition: transform 0.15s;
}
.slider::-webkit-slider-thumb:hover { transform: scale(1.15); }

.radius-options {
  display: flex;
  gap: 0.5rem;
}

.radius-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 0.875rem 0.5rem;
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--text-tertiary);
  background: rgba(255,255,255,0.04);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s;
}
.radius-btn:hover { background: rgba(255,255,255,0.08); color: var(--text-secondary); }
.radius-btn.active { 
  border-color: var(--accent-color); 
  color: var(--accent-color);
  background: rgba(var(--accent-rgb), 0.1);
}

.radius-preview {
  width: 1.75rem;
  height: 1.75rem;
  background: var(--accent-color);
}
.radius-preview.sharp { border-radius: 3px; }
.radius-preview.soft { border-radius: 8px; }
.radius-preview.round { border-radius: 50%; }

.footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid rgba(255,255,255,0.06);
}

.reset {
  width: 100%;
  padding: 0.75rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-tertiary);
  background: transparent;
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s;
}
.reset:hover { 
  background: rgba(var(--error-rgb), 0.1); 
  border-color: rgba(var(--error-rgb), 0.3);
  color: var(--error);
}
</style>