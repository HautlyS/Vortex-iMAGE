<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAccentColor } from '../composables/useAccentColor'
import { useTheme } from '../composables/useTheme'

const emit = defineEmits<{
  close: []
}>()

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
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.settings-modal {
  width: 100%;
  max-width: 600px;
  max-height: 90vh;
  background: var(--retro-bg-panel, #1a1030);
  border: 3px solid #000;
  box-shadow: 8px 8px 0 rgba(0,0,0,0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: linear-gradient(135deg, var(--retro-accent-pink, #ff2d95), var(--retro-accent-purple, #b24dff));
  border-bottom: 3px solid #000;
}

.settings-header h2 {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  color: #fff;
  text-shadow: 2px 2px 0 #000;
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  padding: 6px;
  background: rgba(0,0,0,0.3);
  border: 2px solid #000;
  color: #fff;
  box-shadow: 2px 2px 0 #000;
}

.close-btn:hover {
  background: var(--retro-accent-red, #ff3b30);
}

/* Tabs */
.settings-tabs {
  display: flex;
  background: var(--retro-bg-card, #251842);
  border-bottom: 2px solid #000;
  overflow-x: auto;
}

.tab-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 8px;
  background: transparent;
  border: none;
  border-right: 1px solid var(--retro-bg-lighter, #2d1f4d);
  color: var(--retro-text-muted, #9d8ec2);
  font-family: 'VT323', monospace;
  font-size: 14px;
  box-shadow: none;
  transition: all 0.1s;
}

.tab-btn:last-child {
  border-right: none;
}

.tab-btn svg {
  width: 20px;
  height: 20px;
}

.tab-btn:hover {
  background: var(--retro-bg-lighter, #2d1f4d);
  color: #fff;
}

.tab-btn.active {
  background: var(--retro-bg-panel, #1a1030);
  color: var(--retro-accent-green, #00ff87);
  border-bottom: 3px solid var(--retro-accent-green, #00ff87);
  margin-bottom: -2px;
}

/* Content */
.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-group h3 {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--retro-accent-yellow, #ffd000);
  text-shadow: 1px 1px 0 #000;
  margin: 0;
}

/* Color Grid */
.color-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.color-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px;
  background: var(--retro-bg-card, #251842);
  border: 2px solid var(--retro-bg-lighter, #2d1f4d);
  box-shadow: 2px 2px 0 #000;
}

.color-btn:hover {
  border-color: var(--color);
}

.color-btn.active {
  border-color: var(--color);
  box-shadow: 0 0 15px var(--color), 2px 2px 0 #000;
}

.color-preview {
  width: 24px;
  height: 24px;
  background: var(--color);
  border: 2px solid #000;
}

.color-name {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: var(--retro-text-muted, #9d8ec2);
}

.color-btn.active .color-name {
  color: var(--color);
}

/* Theme Grid */
.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.theme-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px 12px;
  background: var(--retro-bg-card, #251842);
  border: 2px solid var(--retro-bg-lighter, #2d1f4d);
  box-shadow: 2px 2px 0 #000;
}

.theme-btn:hover {
  border-color: var(--retro-accent-green, #00ff87);
}

.theme-btn.active {
  border-color: var(--retro-accent-green, #00ff87);
  background: var(--retro-bg-lighter, #2d1f4d);
}

.theme-icon {
  font-size: 24px;
}

.theme-name {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: var(--retro-text-muted, #9d8ec2);
}

.theme-btn.active .theme-name {
  color: var(--retro-accent-green, #00ff87);
}

/* Slider */
.slider-container {
  display: flex;
  align-items: center;
  gap: 16px;
}

.slider-container input[type="range"] {
  flex: 1;
  height: 8px;
  background: var(--retro-bg-card, #251842);
  border: 2px solid #000;
  appearance: none;
  cursor: pointer;
}

.slider-container input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 20px;
  height: 20px;
  background: var(--retro-accent-green, #00ff87);
  border: 2px solid #000;
  cursor: pointer;
}

.slider-value {
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: var(--retro-accent-green, #00ff87);
  min-width: 60px;
  text-align: right;
}

.setting-hint {
  font-family: 'VT323', monospace;
  font-size: 14px;
  color: var(--retro-text-muted, #9d8ec2);
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
  padding: 10px 0;
  border-bottom: 1px solid var(--retro-bg-lighter, #2d1f4d);
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: var(--retro-text-muted, #9d8ec2);
}

.stat-value {
  color: var(--retro-text-main, #fff);
}

/* Toggle */
.toggle-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: var(--retro-text-muted, #9d8ec2);
}

.toggle {
  position: relative;
  width: 48px;
  height: 24px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--retro-bg-card, #251842);
  border: 2px solid #000;
  cursor: pointer;
  transition: all 0.2s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  top: 2px;
  background: var(--retro-text-muted, #9d8ec2);
  border: 2px solid #000;
  transition: all 0.2s;
}

.toggle input:checked + .toggle-slider {
  background: var(--retro-accent-green, #00ff87);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(24px);
  background: #fff;
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
  padding: 8px 12px;
  background: var(--retro-bg-card, #251842);
  border: 1px solid var(--retro-bg-lighter, #2d1f4d);
}

.shortcut-row kbd {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  padding: 4px 8px;
  background: var(--retro-bg-lighter, #2d1f4d);
  border: 2px solid #000;
  color: var(--retro-accent-green, #00ff87);
}

.shortcut-row span {
  font-family: 'VT323', monospace;
  font-size: 16px;
  color: var(--retro-text-muted, #9d8ec2);
}

/* About */
.about-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 20px;
  text-align: center;
}

.app-logo {
  width: 64px;
  height: 64px;
  padding: 12px;
  background: linear-gradient(135deg, var(--retro-accent-pink, #ff2d95), var(--retro-accent-purple, #b24dff));
  border: 3px solid #000;
  box-shadow: 4px 4px 0 #000;
  color: #fff;
}

.app-logo svg {
  width: 100%;
  height: 100%;
}

.about-header h2 {
  font-family: 'Press Start 2P', monospace;
  font-size: 16px;
  color: var(--retro-accent-yellow, #ffd000);
  text-shadow: 2px 2px 0 #000;
  margin: 0;
}

.version {
  font-family: 'VT323', monospace;
  font-size: 18px;
  color: var(--retro-text-muted, #9d8ec2);
  margin: 0;
}

.about-footer {
  text-align: center;
  padding-top: 20px;
  border-top: 1px solid var(--retro-bg-lighter, #2d1f4d);
}

.about-footer p {
  font-family: 'VT323', monospace;
  font-size: 16px;
  color: var(--retro-text-muted, #9d8ec2);
  margin: 4px 0;
}

.copyright {
  font-size: 14px !important;
  opacity: 0.6;
}

/* Buttons */
.btn-primary {
  background: linear-gradient(180deg, var(--retro-accent-green, #00ff87), #00cc6a);
  color: #000;
  font-family: 'VT323', monospace;
  font-size: 18px;
  padding: 10px 20px;
  border: 2px solid #000;
  box-shadow: 2px 2px 0 #000;
}

.btn-danger {
  background: linear-gradient(180deg, var(--retro-accent-red, #ff3b30), #cc2020);
  color: #fff;
  font-family: 'VT323', monospace;
  font-size: 18px;
  padding: 10px 20px;
  border: 2px solid #000;
  box-shadow: 2px 2px 0 #000;
  margin-top: 12px;
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
