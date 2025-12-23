/**
 * Vue Component - 1 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: GlassSurface
 */

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useTheme, ACCENT_COLORS } from '../composables/useTheme'
import { useGitHubAuth } from '../composables/useGitHubAuth'
import { useCrypto } from '../composables/useCrypto'
import { useCompression } from '../composables/useCompression'
import { useDataDriver } from '../composables/useDataDriver'
import GlassSurface from './GlassSurface.vue'

defineProps<{
  visible?: boolean 
}>()

const emit = defineEmits<{ close: [] }>()

type Tab = 'appearance' | 'accounts' | 'security' | 'performance'
const activeTab = ref<Tab>('appearance')
const activeTabLabel = computed(() => tabs.find(t => t.id === activeTab.value)?.label)
const gliderStyle = computed(() => {
  const options = ['sharp', 'soft', 'round'] as const
  const index = options.indexOf(theme.value.borderRadius)
  return {
    transform: `translateX(${index * 100}%)`
  }
})

const { theme, setAccentPreset, setAccentColor, setBorderRadius, setGlassOpacity, setReduceMotion, setMatrixEffects, setGlassMorphism, setGlow, resetTheme } = useTheme()
const customColor = ref(theme.value.accentColor)
const currentPreset = computed(() => ACCENT_COLORS.find(c => c.color === theme.value.accentColor)?.id || 'custom')

const { token, logout } = useGitHubAuth()

const { hasStoredKeypair, isUnlocked, cryptoInfo, initialize: initCrypto } = useCrypto()
const cryptoStatus = computed(() => hasStoredKeypair.value ? (isUnlocked.value ? 'Desbloqueado' : 'Bloqueado') : 'Não configurado')

const { availableAlgorithms, initialize: initCompression, getAlgorithmInfo } = useCompression()
const compressionInfo = computed(() => availableAlgorithms.value.map(a => ({ algo: a, info: getAlgorithmInfo(a) })))

const { githubDrivers, loadDrivers } = useDataDriver()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

onMounted(async () => {
  document.addEventListener('keydown', handleKeydown)
  await Promise.all([initCrypto(), initCompression(), loadDrivers()])
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

function selectPreset(id: string) {
  const preset = ACCENT_COLORS.find(c => c.id === id)
  if (preset) {
    customColor.value = preset.color
    setAccentPreset(id)
  }
}

function handleColorChange(e: Event) {
  const color = (e.target as HTMLInputElement).value
  customColor.value = color
  setAccentColor(color)
}

const tabs: { id: Tab; label: string; icon: string }[] = [
  { id: 'appearance', label: 'Aparência', icon: '🎨' },
  { id: 'accounts', label: 'Contas', icon: '👤' },
  { id: 'security', label: 'Segurança', icon: '🔐' },
  { id: 'performance', label: 'Performance', icon: '⚡' },
]
</script>

<template>
  <Teleport to="body">
    <Transition name="settings">
      <div v-if="visible" class="settings-overlay" @click.self="emit('close')">
        <GlassSurface
          class="settings-panel"
          :border-radius="24"
          :opacity="0.6"
          :blur="20"
          :saturation="1.2"
        >
          <!-- Sidebar -->
          <div class="settings-sidebar">
            <header class="panel-header">
              <h2>Configurações</h2>
            </header>
            
            <nav class="tabs">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                :class="['tab', { active: activeTab === tab.id }]"
                @click="activeTab = tab.id"
              >
                <span class="tab-icon">{{ tab.icon }}</span>
                <span class="tab-label">{{ tab.label }}</span>
                <div v-if="activeTab === tab.id" class="active-indicator" />
              </button>
            </nav>
          </div>

          <!-- Content -->
          <div class="panel-content">
            <div class="content-header">
              <h3>{{ activeTabLabel }}</h3>
              <button class="close-btn" @click="emit('close')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12"/>
                </svg>
              </button>
            </div>

            <div class="scroll-container">
              <!-- Appearance Tab -->
              <div v-if="activeTab === 'appearance'" class="tab-content">
                <section class="section">
                  <h4>Cor de Destaque</h4>
                  <div class="color-presets">
                    <button
                      v-for="c in ACCENT_COLORS"
                      :key="c.id"
                      :class="['color-preset', { active: currentPreset === c.id }]"
                      :style="{ '--c': c.color }"
                      :title="c.name"
                      @click="selectPreset(c.id)"
                    >
                      <div class="preset-ring" />
                    </button>
                    <label class="color-custom" :class="{ active: currentPreset === 'custom' }">
                      <input type="color" :value="customColor" @input="handleColorChange" />
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
                    </label>
                  </div>
                </section>

                <section class="section">
                  <h4>Transparência</h4>
                  <div class="control-row">
                    <input
                      type="range"
                      min="40"
                      max="95"
                      :value="theme.glassOpacity"
                      @input="(e) => setGlassOpacity(Number((e.target as HTMLInputElement).value))"
                      class="premium-slider"
                      :style="{ '--progress': `${((theme.glassOpacity - 40) / 55) * 100}%` }"
                    />
                    <span class="value-badge">{{ 100 - theme.glassOpacity }}%</span>
                  </div>
                </section>

                <section class="section">
                  <h4>Bordas</h4>
                  <div class="segment-control">
                    <button
                      v-for="r in (['sharp', 'soft', 'round'] as const)"
                      :key="r"
                      :class="['segment-btn', { active: theme.borderRadius === r }]"
                      @click="setBorderRadius(r)"
                    >
                      {{ r === 'sharp' ? 'Retas' : r === 'soft' ? 'Suaves' : 'Redondas' }}
                    </button>
                    <div class="segment-glider" :style="gliderStyle" />
                  </div>
                </section>

                 <div class="section-divider" />
                 
                 <section class="section">
                   <h4>Efeitos Visuais</h4>
                   <label class="toggle-item">
                     <div class="text-stack">
                       <span class="primary">Glassmorphism</span>
                       <span class="secondary">Efeito de vidro fosco</span>
                     </div>
                     <button :class="['toggle-switch', { active: theme.glassMorphism }]" @click="setGlassMorphism(!theme.glassMorphism)">
                       <div class="toggle-thumb" />
                     </button>
                   </label>
                   <label class="toggle-item">
                     <div class="text-stack">
                       <span class="primary">Brilho</span>
                       <span class="secondary">Efeito de glow nos elementos</span>
                     </div>
                     <button :class="['toggle-switch', { active: theme.glow }]" @click="setGlow(!theme.glow)">
                       <div class="toggle-thumb" />
                     </button>
                   </label>
                 </section>

                 <div class="section-divider" />
                 
                 <button class="btn-reset" @click="resetTheme">
                   Restaurar Padrão
                 </button>
              </div>

              <!-- Other tabs placeholder for brevity in this step, continuing in next replace chunks for full implementation -->
              <!-- Accounts Tab -->
              <div v-if="activeTab === 'accounts'" class="tab-content">
                 <!-- Re-implementing with new styles -->
                 <section class="section">
                    <h4>GitHub</h4>
                    <div class="premium-card">
                       <div class="card-content">
                          <div class="user-info" v-if="token">
                             <div class="avatar-placeholder">GH</div>
                             <div class="text-stack">
                                <span class="primary">Conectado</span>
                                <span class="secondary">Acesso autorizado</span>
                             </div>
                          </div>
                          <div class="text-stack" v-else>
                             <span class="primary">Desconectado</span>
                             <span class="secondary">Entre para sincronizar</span>
                          </div>
                       </div>
                       <button 
                          :class="token ? 'btn-danger' : 'btn-primary'"
                          @click="token ? logout() : null"
                       >
                          {{ token ? 'Desconectar' : 'Conectar' }}
                       </button>
                    </div>
                 </section>

                 <section class="section">
                   <h4>Repositórios ({{ githubDrivers.length }})</h4>
                   <div class="repo-grid">
                      <div v-for="driver in githubDrivers" :key="driver.id" class="premium-card repo-card">
                         <span class="repo-name">{{ driver.name }}</span>
                         <span class="repo-path">{{ driver.path }}</span>
                      </div>
                   </div>
                 </section>
              </div>

              <!-- Security Tab -->
              <div v-if="activeTab === 'security'" class="tab-content">
                  <section class="section">
                    <h4>Status</h4>
                    <div class="premium-card">
                      <div class="card-content">
                        <div class="text-stack">
                          <span class="primary">{{ cryptoStatus }}</span>
                          <span class="secondary">Chave de criptografia</span>
                        </div>
                      </div>
                    </div>
                  </section>
                  <section class="section">
                    <h4>Detalhes</h4>
                    <div class="premium-card column">
                       <div class="card-row" v-for="(value, key) in cryptoInfo" :key="key">
                          <span class="label">{{ key }}</span>
                          <span class="value">{{ value }}</span>
                       </div>
                    </div>
                  </section>
                  <section class="section">
                    <h4>Compressão</h4>
                    <div class="algo-grid">
                      <div v-for="algo in compressionInfo" :key="algo.algo" class="premium-card algo-card">
                        <span class="algo-name">{{ algo.algo }}</span>
                      </div>
                    </div>
                  </section>
              </div>
              
              <!-- Performance Tab -->
              <div v-if="activeTab === 'performance'" class="tab-content">
                  <section class="section">
                    <h4>Otimizações</h4>
                    <label class="toggle-item">
                       <div class="text-stack">
                          <span class="primary">Reduzir Movimento</span>
                          <span class="secondary">Desativa animações pesadas</span>
                       </div>
                       <button :class="['toggle-switch', { active: theme.reduceMotion }]" @click="setReduceMotion(!theme.reduceMotion)">
                          <div class="toggle-thumb" />
                       </button>
                    </label>
                    <label class="toggle-item">
                       <div class="text-stack">
                          <span class="primary">Efeitos Matrix</span>
                          <span class="secondary">Fundo animado na tela inicial</span>
                       </div>
                       <button :class="['toggle-switch', { active: theme.matrixEffects }]" @click="setMatrixEffects(!theme.matrixEffects)">
                          <div class="toggle-thumb" />
                       </button>
                    </label>
                  </section>
              </div>

            </div>
          </div>
        </GlassSurface>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4); 
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 500;
  padding: 2rem;
}

.settings-panel {
  width: 100%;
  max-width: 900px;
  height: 600px;
  display: grid;
  grid-template-columns: 240px 1fr;
  overflow: hidden;
  box-shadow: 
    0 24px 48px -12px rgba(0,0,0,0.5),
    0 0 0 1px rgba(255,255,255,0.1);
}

.settings-sidebar {
  background: rgba(0,0,0,0.2);
  border-right: 1px solid rgba(255,255,255,0.05);
  display: flex;
  flex-direction: column;
  padding: 1.5rem;
}

.panel-header h2 {
  font-size: 1.5rem;
  font-weight: 700;
  background: linear-gradient(to right, #fff, rgba(255,255,255,0.5));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin-bottom: 2rem;
}

.tabs {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.tab {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.875rem 1rem;
  background: transparent;
  border: none;
  border-radius: 12px;
  color: #a1a1aa;
  font-size: 0.9375rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.2, 0.8, 0.2, 1);
  position: relative;
  overflow: hidden;
  text-align: left;
}

.tab:hover {
  background: rgba(255,255,255,0.05);
  color: #fff;
  transform: translateX(4px);
}

.tab.active {
  background: rgba(var(--accent-rgb), 0.15);
  color: var(--accent-color);
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}

.active-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 60%;
  background: var(--accent-color);
  border-radius: 0 4px 4px 0;
}

.tab-icon {
  font-size: 1.25rem;
  width: 24px;
  display: flex;
  justify-content: center;
}

.panel-content {
  display: flex;
  flex-direction: column;
  padding: 2rem 2.5rem;
  background: rgba(255,255,255,0.01);
  min-width: 0; 
}

.content-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.content-header h3 {
  font-size: 1.5rem;
  font-weight: 600;
  color: #fff;
}

.scroll-container {
  overflow-y: auto;
  padding-right: 1rem;
  margin-right: -1rem;
  flex: 1;
}

.section {
  margin-bottom: 2.5rem;
}

.section h4 {
  font-size: 0.875rem;
  font-weight: 600;
  color: #71717a;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 1rem;
}

.color-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.color-preset {
  width: 3rem;
  height: 3rem;
  border-radius: 50%; 
  border: none;
  background: transparent;
  cursor: pointer;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preset-ring {
  width: 100%;
  height: 100%;
  background: var(--c);
  border-radius: 50%;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  border: 2px solid rgba(255,255,255,0.1);
}

.color-preset:hover .preset-ring {
  transform: scale(1.1);
}

.color-preset.active .preset-ring {
  transform: scale(0.9);
  box-shadow: 0 0 0 3px rgba(20,20,30,0.9), 0 0 0 5px var(--c);
  border-color: transparent;
}

.color-custom {
  width: 3rem;
  height: 3rem;
  border-radius: 50%;
  border: 1px dashed rgba(255,255,255,0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: relative;
  transition: all 0.2s;
  color: rgba(255,255,255,0.5);
}

.color-custom:hover, .color-custom.active {
  border-color: #fff;
  color: #fff;
}

.color-custom input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}

.control-row {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  background: rgba(0,0,0,0.2);
  padding: 1rem;
  border-radius: 12px;
}

.premium-slider {
  flex: 1;
  -webkit-appearance: none;
  height: 4px;
  background: rgba(255,255,255,0.1);
  border-radius: 2px;
  outline: none;
  position: relative;
}

.premium-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 20px;
  height: 20px;
  background: #fff;
  border-radius: 50%;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  cursor: grab;
  transform: scale(1);
  transition: transform 0.1s;
}

.premium-slider:active::-webkit-slider-thumb {
  transform: scale(1.2);
  cursor: grabbing;
}

.value-badge {
  font-family: monospace;
  background: rgba(255,255,255,0.1);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.875rem;
}

.segment-control {
  display: flex;
  background: rgba(0,0,0,0.2);
  padding: 0.25rem;
  border-radius: 12px;
  position: relative;
  isolation: isolate;
}

.segment-btn {
  flex: 1;
  padding: 0.625rem;
  background: transparent;
  border: none;
  color: #71717a;
  font-weight: 500;
  font-size: 0.875rem;
  cursor: pointer;
  z-index: 2;
  transition: color 0.2s;
}

.segment-btn.active {
  color: #fff;
}

.segment-glider {
  position: absolute;
  top: 4px;
  left: 4px;
  width: calc(33.33% - 2.66px);
  height: calc(100% - 8px);
  background: rgba(255,255,255,0.1);
  border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.05);
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  z-index: 1;
}

.premium-card {
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 16px;
  padding: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-content {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.avatar-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: linear-gradient(135deg, #333, #111);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  border: 1px solid rgba(255,255,255,0.1);
}

.text-stack {
  display: flex;
  flex-direction: column;
}

.text-stack .primary {
  color: #fff;
  font-weight: 500;
}

.text-stack .secondary {
  color: #71717a;
  font-size: 0.8125rem;
}

.btn-primary, .btn-danger {
  padding: 0.625rem 1.25rem;
  border-radius: 8px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-primary {
  background: #fff;
  color: #000;
}

.btn-primary:hover {
  background: #e4e4e7;
}

.btn-danger {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.btn-danger:hover {
  background: rgba(239, 68, 68, 0.2);
}

.premium-card.column {
  flex-direction: column;
  align-items: stretch;
  gap: 0.5rem;
}

.card-row {
  display: flex;
  justify-content: space-between;
  padding: 0.5rem 0;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.card-row:last-child {
  border-bottom: none;
}

.card-row .label {
  color: #71717a;
  font-size: 0.875rem;
}

.card-row .value {
  color: #fff;
  font-size: 0.875rem;
}

.algo-grid, .repo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 0.75rem;
}

.algo-card, .repo-card {
  flex-direction: column;
  align-items: flex-start;
  padding: 1rem;
}

.algo-name, .repo-name {
  color: #fff;
  font-weight: 500;
  font-size: 0.875rem;
}

.repo-path {
  color: #71717a;
  font-size: 0.75rem;
  margin-top: 0.25rem;
}

.toggle-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background: rgba(255,255,255,0.02);
  border-radius: 12px;
  cursor: pointer;
  transition: background 0.2s;
  margin-bottom: 0.5rem;
}

.toggle-item:hover {
  background: rgba(255,255,255,0.04);
}

.toggle-switch {
  width: 44px;
  height: 24px;
  background: rgba(255,255,255,0.1);
  border-radius: 99px;
  border: none;
  position: relative;
  transition: all 0.2s;
  cursor: pointer;
}

.toggle-switch.active {
  background: var(--accent-color);
}

.toggle-thumb {
  width: 20px;
  height: 20px;
  background: #fff;
  border-radius: 50%;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

.toggle-switch.active .toggle-thumb {
  transform: translateX(20px);
}

.section-divider {
  height: 1px;
  background: rgba(255,255,255,0.05);
  margin: 2rem 0;
}

.btn-reset {
  width: 100%;
  padding: 1rem;
  background: transparent;
  border: 1px dashed rgba(255,255,255,0.1);
  color: #71717a;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-reset:hover {
  border-color: rgba(255,255,255,0.3);
  color: #fff;
}

.close-btn {
  background: rgba(255,255,255,0.05);
  border: none;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255,255,255,0.1);
  transform: rotate(90deg);
}

.settings-enter-active, .settings-leave-active {
  transition: opacity 0.3s;
}

.settings-enter-from, .settings-leave-to {
  opacity: 0;
}

.settings-enter-active .settings-panel,
.settings-leave-active .settings-panel {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.settings-enter-from .settings-panel {
  transform: scale(0.95) translateY(20px);
  opacity: 0;
}

.settings-leave-to .settings-panel {
  transform: scale(0.95);
  opacity: 0;
}

@media (max-width: 768px) {
  .settings-overlay {
    padding: 0;
    align-items: flex-end;
  }
  
  .settings-panel {
    max-width: 100%;
    width: 100%;
    height: 90vh;
    max-height: 90vh;
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr;
    border-radius: 24px 24px 0 0;
  }
  
  .settings-sidebar {
    border-right: none;
    border-bottom: 1px solid rgba(255,255,255,0.05);
    padding: 1rem;
  }
  
  .panel-header h2 {
    font-size: 1.25rem;
    margin-bottom: 1rem;
  }
  
  .tabs {
    flex-direction: row;
    overflow-x: auto;
    gap: 0.25rem;
    padding-bottom: 0.5rem;
    -webkit-overflow-scrolling: touch;
  }
  
  .tab {
    flex-shrink: 0;
    padding: 0.75rem 1rem;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 70px;
    text-align: center;
  }
  
  .tab-label {
    font-size: 0.75rem;
  }
  
  .active-indicator {
    display: none;
  }
  
  .panel-content {
    padding: 1rem 1.25rem;
    padding-bottom: calc(1rem + env(safe-area-inset-bottom, 0));
  }
  
  .content-header h3 {
    font-size: 1.25rem;
  }
  
  .color-presets {
    gap: 0.5rem;
  }
  
  .color-preset {
    width: 2.5rem;
    height: 2.5rem;
  }
  
  .segment-control {
    flex-wrap: wrap;
  }
  
  .segment-btn {
    flex: 1 1 30%;
    min-width: 80px;
  }
  
  .algo-grid, .repo-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@supports (padding-bottom: env(safe-area-inset-bottom)) {
  .settings-panel {
    padding-bottom: env(safe-area-inset-bottom);
  }
}
</style>