<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useAccentColor } from '../composables/useAccentColor'
import { useTheme } from '../composables/useTheme'
import { registerOverlay } from '../composables/useKeyboardShortcuts'

const emit = defineEmits<{
  close: []
}>()

// Register ESC key handler
let unregisterOverlay: (() => void) | null = null;

onMounted(() => {
  unregisterOverlay = registerOverlay('settings-panel', () => emit('close'));
});

onUnmounted(() => {
  if (unregisterOverlay) {
    unregisterOverlay();
  }
});

const { accentHex, setAccent, colors } = useAccentColor()
const { theme, setAccentPreset, ACCENT_COLORS } = useTheme()

// Settings tabs
type SettingsTab = 'appearance' | 'storage' | 'shortcuts' | 'about'
const activeTab = ref<SettingsTab>('appearance')

// Appearance settings - use accent colors from composable
const accentColors = [
  { id: 'green', color: colors.green, name: 'Matrix' },
  { id: 'pink', color: colors.pink, name: 'Neon' },
  { id: 'cyan', color: colors.cyan, name: 'Cyber' },
  { id: 'yellow', color: colors.yellow, name: 'Gold' },
  { id: 'purple', color: colors.purple, name: 'Grape' },
  { id: 'orange', color: colors.orange, name: 'Fire' },
]

// Theme presets from useTheme
const themePresets = ACCENT_COLORS.slice(0, 3).map(c => ({
  id: c.id,
  name: c.name,
  icon: c.id === 'cyber' ? '🌙' : c.id === 'neon' ? '👾' : '⬛'
}))

// Grid size
const gridSize = ref(180)

onMounted(async () => {
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    const savedSize = await store.get<number>('gridSize')
    if (savedSize) gridSize.value = savedSize
  } catch {}
})

async function saveGridSize() {
  try {
    const { load } = await import('@tauri-apps/plugin-store')
    const store = await load('settings.json')
    await store.set('gridSize', gridSize.value)
    await store.save()
  } catch {}
}

// Keyboard shortcuts
const shortcuts = [
  { key: 'Ctrl + A', action: 'Selecionar tudo' },
  { key: 'Ctrl + D', action: 'Desmarcar tudo' },
  { key: 'Delete', action: 'Excluir selecionados' },
  { key: 'F', action: 'Favoritar' },
  { key: 'Esc', action: 'Fechar / Cancelar' },
  { key: 'Enter', action: 'Abrir foto' },
  { key: '←/→', action: 'Navegar fotos' },
  { key: 'Shift + Click', action: 'Seleção em range' },
  { key: 'Ctrl + Click', action: 'Adicionar à seleção' },
]

// App info
const appVersion = '2.0.0'
const buildDate = '2026-01-03'
</script>

<template>
  <Teleport to="body">
    <div class="settings-overlay" @click.self="emit('close')">
      <div class="settings-modal">
        <!-- Header -->
        <div class="settings-header">
          <h2>CONFIGURAÇÕES</h2>
          <button class="close-btn" @click="emit('close')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <!-- Tabs -->
        <div class="settings-tabs">
          <button 
            v-for="tab in ['appearance', 'storage', 'shortcuts', 'about']" 
            :key="tab"
            class="tab-btn"
            :class="{ active: activeTab === tab }"
            @click="activeTab = tab as SettingsTab"
          >
            <svg v-if="tab === 'appearance'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
            </svg>
            <svg v-else-if="tab === 'storage'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <svg v-else-if="tab === 'shortcuts'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h8M6 16h.01M10 16h.01M14 16h.01M18 16h.01"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>
            </svg>
            <span>{{ tab === 'appearance' ? 'Visual' : tab === 'storage' ? 'Armazenamento' : tab === 'shortcuts' ? 'Atalhos' : 'Sobre' }}</span>
          </button>
        </div>

        <!-- Content -->
        <div class="settings-content">
          <!-- Appearance Tab -->
          <div v-if="activeTab === 'appearance'" class="tab-content">
            <div class="setting-group">
              <h3>COR DE DESTAQUE</h3>
              <div class="color-grid">
                <button 
                  v-for="c in accentColors" 
                  :key="c.id"
                  class="color-btn"
                  :class="{ active: accentHex === c.color }"
                  :style="{ '--color': c.color }"
                  @click="setAccent(c.id as any)"
                >
                  <span class="color-preview"></span>
                  <span class="color-name">{{ c.name }}</span>
                </button>
              </div>
            </div>

            <div class="setting-group">
              <h3>TEMA</h3>
              <div class="theme-grid">
                <button 
                  v-for="t in themePresets" 
                  :key="t.id"
                  class="theme-btn"
                  :class="{ active: theme.accentColor === ACCENT_COLORS.find(c => c.id === t.id)?.color }"
                  @click="setAccentPreset(t.id)"
                >
                  <span class="theme-icon">{{ t.icon }}</span>
                  <span class="theme-name">{{ t.name }}</span>
                </button>
              </div>
            </div>

            <div class="setting-group">
              <h3>TAMANHO DA GRADE</h3>
              <div class="slider-container">
                <input 
                  type="range" 
                  v-model.number="gridSize" 
                  min="100" 
                  max="400" 
                  step="10"
                  @change="saveGridSize"
                />
                <span class="slider-value">{{ gridSize }}px</span>
              </div>
              <p class="setting-hint">Arraste o controle no canto inferior direito da galeria para ajustar em tempo real.</p>
            </div>
          </div>

          <!-- Storage Tab -->
          <div v-if="activeTab === 'storage'" class="tab-content">
            <div class="setting-group">
              <h3>REPOSITÓRIO GITHUB</h3>
              <div class="input-group">
                <input type="text" placeholder="usuario/repositorio" />
                <button class="btn-primary">Salvar</button>
              </div>
            </div>

            <div class="setting-group">
              <h3>CACHE LOCAL</h3>
              <div class="stat-row">
                <span>Fotos em cache</span>
                <span class="stat-value">0 MB</span>
              </div>
              <div class="stat-row">
                <span>Miniaturas</span>
                <span class="stat-value">0 MB</span>
              </div>
              <button class="btn-danger">Limpar Cache</button>
            </div>

            <div class="setting-group">
              <h3>BACKUP</h3>
              <div class="toggle-row">
                <span>Backup automático</span>
                <label class="toggle">
                  <input type="checkbox" />
                  <span class="toggle-slider"></span>
                </label>
              </div>
            </div>
          </div>

          <!-- Shortcuts Tab -->
          <div v-if="activeTab === 'shortcuts'" class="tab-content">
            <div class="setting-group">
              <h3>ATALHOS DE TECLADO</h3>
              <div class="shortcuts-list">
                <div v-for="s in shortcuts" :key="s.key" class="shortcut-row">
                  <kbd>{{ s.key }}</kbd>
                  <span>{{ s.action }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- About Tab -->
          <div v-if="activeTab === 'about'" class="tab-content">
            <div class="about-header">
              <div class="app-logo">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="3" width="18" height="18" rx="2"/>
                  <circle cx="9" cy="9" r="2"/>
                  <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
                </svg>
              </div>
              <h2>iMAGE</h2>
              <p class="version">v{{ appVersion }}</p>
            </div>

            <div class="setting-group">
              <div class="stat-row">
                <span>Versão</span>
                <span class="stat-value">{{ appVersion }}</span>
              </div>
              <div class="stat-row">
                <span>Build</span>
                <span class="stat-value">{{ buildDate }}</span>
              </div>
              <div class="stat-row">
                <span>Framework</span>
                <span class="stat-value">Tauri + Vue 3</span>
              </div>
            </div>

            <div class="about-footer">
              <p>Feito com 💜 em 8-bit style</p>
              <p class="copyright">© 2026 iMAGE</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.92);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  image-rendering: pixelated;
}

/* Scanlines on overlay */
.settings-overlay::before {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent 0px,
    transparent 2px,
    rgba(0, 0, 0, 0.15) 2px,
    rgba(0, 0, 0, 0.15) 4px
  );
  pointer-events: none;
  z-index: 1;
}

.settings-modal {
  position: relative;
  z-index: 2;
  width: 100%;
  max-width: 600px;
  max-height: 90vh;
  background: #1a1a2e;
  border: 4px solid #000;
  box-shadow: 8px 8px 0 rgba(0,0,0,0.8);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Corner decorations */
.settings-modal::before,
.settings-modal::after {
  content: '';
  position: absolute;
  width: 12px;
  height: 12px;
  background: #f15bb5;
  z-index: 10;
}

.settings-modal::before { top: -6px; left: -6px; }
.settings-modal::after { top: -6px; right: -6px; }

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: linear-gradient(180deg, #9b5de5 0%, #7b3dc5 100%);
  border-bottom: 4px solid #000;
  position: relative;
}

/* Header pattern */
.settings-header::before {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    90deg,
    transparent 0px,
    transparent 8px,
    rgba(0,0,0,0.1) 8px,
    rgba(0,0,0,0.1) 16px
  );
  pointer-events: none;
}

.settings-header h2 {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  color: #fff;
  text-shadow: 2px 2px 0 #000;
  margin: 0;
  letter-spacing: 2px;
  position: relative;
}

.close-btn {
  width: 36px;
  height: 36px;
  padding: 8px;
  background: rgba(0,0,0,0.3);
  border: 2px solid #000;
  color: #fff;
  box-shadow: 2px 2px 0 #000;
  position: relative;
}

.close-btn:hover {
  background: #e43b44;
  transform: translate(-2px, -2px);
  box-shadow: 4px 4px 0 #000;
}

.close-btn:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

/* Tabs */
.settings-tabs {
  display: flex;
  background: #16213e;
  border-bottom: 4px solid #000;
  overflow-x: auto;
}

.tab-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 14px 8px;
  background: transparent;
  border: none;
  border-right: 2px solid #0f0f23;
  color: #808080;
  font-family: 'VT323', monospace;
  font-size: 16px;
  box-shadow: none;
  transition: none;
}

.tab-btn:last-child {
  border-right: none;
}

.tab-btn svg {
  width: 24px;
  height: 24px;
}

.tab-btn:hover {
  background: #1a1a2e;
  color: #fff;
  transform: none;
}

.tab-btn.active {
  background: #1a1a2e;
  color: #39ff14;
  border-bottom: 4px solid #39ff14;
  margin-bottom: -4px;
  box-shadow: inset 0 0 20px rgba(57, 255, 20, 0.1);
}

/* Content */
.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  background: linear-gradient(180deg, #16213e 0%, #1a1a2e 100%);
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-group h3 {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #feae34;
  text-shadow: 2px 2px 0 #000;
  margin: 0;
  letter-spacing: 2px;
}

/* Color Grid */
.color-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.color-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px;
  background: #0f0f23;
  border: 4px solid #3a3a5c;
  box-shadow: 4px 4px 0 #000;
}

.color-btn:hover {
  border-color: var(--color);
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000;
}

.color-btn.active {
  border-color: var(--color);
  box-shadow: 0 0 16px var(--color), 4px 4px 0 #000;
}

.color-preview {
  width: 28px;
  height: 28px;
  background: var(--color);
  border: 2px solid #000;
}

.color-name {
  font-family: 'VT323', monospace;
  font-size: 16px;
  color: #808080;
}

.color-btn.active .color-name {
  color: var(--color);
  text-shadow: 0 0 8px var(--color);
}

/* Theme Grid */
.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.theme-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 20px 16px;
  background: #0f0f23;
  border: 4px solid #3a3a5c;
  box-shadow: 4px 4px 0 #000;
}

.theme-btn:hover {
  border-color: #39ff14;
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000;
}

.theme-btn.active {
  border-color: #39ff14;
  background: #1a1a2e;
  box-shadow: 0 0 16px rgba(57, 255, 20, 0.3), 4px 4px 0 #000;
}

.theme-icon {
  font-size: 28px;
}

.theme-name {
  font-family: 'VT323', monospace;
  font-size: 16px;
  color: #808080;
}

.theme-btn.active .theme-name {
  color: #39ff14;
  text-shadow: 0 0 8px rgba(57, 255, 20, 0.5);
}

/* Slider */
.slider-container {
  display: flex;
  align-items: center;
  gap: 20px;
}

.slider-container input[type="range"] {
  flex: 1;
  height: 16px;
  background: #000;
  border: 4px solid #3a3a5c;
  appearance: none;
  cursor: pointer;
  box-shadow: inset 4px 4px 0 rgba(0,0,0,0.5);
}

.slider-container input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 24px;
  height: 28px;
  background: linear-gradient(180deg, #808080 0%, #4a4a4a 100%);
  border: 2px solid #000;
  cursor: pointer;
  box-shadow: 2px 2px 0 #000;
}

.slider-container input[type="range"]::-webkit-slider-thumb:hover {
  background: linear-gradient(180deg, #39ff14 0%, #2d8a1a 100%);
  box-shadow: 2px 2px 0 #000, 0 0 12px rgba(57, 255, 20, 0.5);
}

.slider-value {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: #39ff14;
  min-width: 70px;
  text-align: right;
  text-shadow: 0 0 8px rgba(57, 255, 20, 0.5);
}

.setting-hint {
  font-family: 'VT323', monospace;
  font-size: 16px;
  color: #808080;
  margin: 0;
}

/* Input Group */
.input-group {
  display: flex;
  gap: 8px;
}

.input-group input {
  flex: 1;
}

/* Stat Row */
.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 2px solid #3a3a5c;
  font-family: 'VT323', monospace;
  font-size: 20px;
  color: #808080;
}

.stat-value {
  color: #fff;
  text-shadow: 0 0 4px rgba(255, 255, 255, 0.3);
}

/* Toggle */
.toggle-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  font-family: 'VT323', monospace;
  font-size: 20px;
  color: #808080;
}

.toggle {
  position: relative;
  width: 56px;
  height: 28px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background: #0f0f23;
  border: 4px solid #3a3a5c;
  cursor: pointer;
  transition: none;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  top: 2px;
  background: linear-gradient(180deg, #808080 0%, #4a4a4a 100%);
  border: 2px solid #000;
  transition: transform 0.1s steps(4);
}

.toggle input:checked + .toggle-slider {
  background: #1a3a1a;
  border-color: #39ff14;
  box-shadow: 0 0 12px rgba(57, 255, 20, 0.3);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(24px);
  background: linear-gradient(180deg, #8cff7a 0%, #39ff14 100%);
}

/* Shortcuts */
.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shortcut-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: #0f0f23;
  border: 2px solid #3a3a5c;
}

.shortcut-row kbd {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  padding: 6px 10px;
  background: #1a1a2e;
  border: 2px solid #000;
  color: #39ff14;
  box-shadow: 2px 2px 0 #000;
  text-shadow: 0 0 8px rgba(57, 255, 20, 0.5);
}

.shortcut-row span {
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: #808080;
}

/* About */
.about-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 24px;
  text-align: center;
}

.app-logo {
  width: 80px;
  height: 80px;
  padding: 16px;
  background: linear-gradient(180deg, #9b5de5 0%, #7b3dc5 100%);
  border: 4px solid #000;
  box-shadow: 6px 6px 0 #000;
  color: #fff;
}

.app-logo svg {
  width: 100%;
  height: 100%;
}

.about-header h2 {
  font-family: 'Press Start 2P', monospace;
  font-size: 20px;
  color: #feae34;
  text-shadow: 4px 4px 0 #000, 0 0 20px rgba(254, 174, 52, 0.5);
  margin: 0;
  letter-spacing: 4px;
}

.version {
  font-family: 'VT323', monospace;
  font-size: 20px;
  color: #808080;
  margin: 0;
}

.about-footer {
  text-align: center;
  padding-top: 24px;
  border-top: 2px solid #3a3a5c;
}

.about-footer p {
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: #808080;
  margin: 6px 0;
}

.copyright {
  font-size: 16px !important;
  opacity: 0.6;
}

/* Buttons */
.btn-primary {
  background: linear-gradient(180deg, #39ff14 0%, #2d8a1a 100%);
  color: #000;
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  padding: 12px 24px;
  border: 4px solid #000;
  box-shadow: 4px 4px 0 #000;
  text-shadow: 1px 1px 0 rgba(255,255,255,0.3);
}

.btn-primary:hover {
  background: linear-gradient(180deg, #8cff7a 0%, #39ff14 100%);
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000, 0 0 16px rgba(57, 255, 20, 0.5);
}

.btn-primary:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

.btn-danger {
  background: linear-gradient(180deg, #e43b44 0%, #a82835 100%);
  color: #fff;
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  padding: 12px 24px;
  border: 4px solid #000;
  box-shadow: 4px 4px 0 #000;
  margin-top: 16px;
}

.btn-danger:hover {
  background: linear-gradient(180deg, #ff6b6b 0%, #e43b44 100%);
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0 #000, 0 0 16px rgba(228, 59, 68, 0.5);
}

.btn-danger:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

/* Mobile */
@media (max-width: 768px) {
  .settings-modal {
    max-height: 100vh;
    height: 100%;
    border: none;
    box-shadow: none;
  }
  
  .color-grid,
  .theme-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .tab-btn span {
    display: none;
  }
  
  .tab-btn {
    padding: 12px;
  }
}
</style>
