<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useDataDriver, type DataDriver } from '../composables/useDataDriver'
import { useGitHubAuth } from '../composables/useGitHubAuth'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'driver-changed', driver: DataDriver): void
}>()

const {
  drivers,
  activeDriver,
  githubDrivers,
  localDrivers,
  loading,
  loadDrivers,
  addGitHubDriver,
  addLocalDriver,
  removeDriver,
  setActiveDriver,
  getDriverIcon: _getDriverIcon
} = useDataDriver()
void _getDriverIcon

const { token } = useGitHubAuth()

const activeTab = ref<'drivers' | 'add'>('drivers')
const newRepoInput = ref('')
const newDriverName = ref('')
const addingDriver = ref(false)
const error = ref<string | null>(null)

onMounted(async () => {
  await loadDrivers()
})

async function handleAddGitHub() {
  if (!newRepoInput.value.trim()) return
  
  const repo = newRepoInput.value.trim()
  if (!/^[\w-]+\/[\w.-]+$/.test(repo)) {
    error.value = 'Formato inválido. Use: usuario/repositorio'
    return
  }
  
  addingDriver.value = true
  error.value = null
  
  try {
    const driver = await addGitHubDriver(repo, newDriverName.value || undefined)
    await setActiveDriver(driver.id)
    emit('driver-changed', driver)
    newRepoInput.value = ''
    newDriverName.value = ''
    activeTab.value = 'drivers'
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Erro ao adicionar repositório'
  } finally {
    addingDriver.value = false
  }
}

async function handleAddLocal() {
  addingDriver.value = true
  error.value = null
  
  try {
    const folder = await open({ multiple: false, directory: true })
    if (folder && typeof folder === 'string') {
      const _driver = await addLocalDriver(folder, newDriverName.value || undefined)
      void _driver
      newDriverName.value = ''
      activeTab.value = 'drivers'
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Erro ao adicionar pasta'
  } finally {
    addingDriver.value = false
  }
}

async function handleSelectDriver(driver: DataDriver) {
  await setActiveDriver(driver.id)
  emit('driver-changed', driver)
}

async function handleRemoveDriver(driver: DataDriver) {
  if (confirm(`Remover "${driver.name}" da lista? Os arquivos não serão excluídos.`)) {
    await removeDriver(driver.id)
  }
}

function formatDate(timestamp?: number): string {
  if (!timestamp) return 'Nunca'
  return new Date(timestamp).toLocaleDateString('pt-BR', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<template>
  <div class="driver-overlay" @click.self="emit('close')">
    <div class="driver-panel">
      <!-- Header -->
      <div class="panel-header">
        <h2>Gerenciar Fontes de Dados</h2>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tabs -->
      <div class="tabs">
        <button 
          :class="['tab', { active: activeTab === 'drivers' }]"
          @click="activeTab = 'drivers'"
        >
          📁 Minhas Fontes
        </button>
        <button 
          :class="['tab', { active: activeTab === 'add' }]"
          @click="activeTab = 'add'"
        >
          ➕ Adicionar
        </button>
      </div>

      <!-- Content -->
      <div class="panel-content">
        <!-- Drivers List -->
        <div v-if="activeTab === 'drivers'" class="drivers-list">
          <div v-if="loading" class="loading">Carregando...</div>
          
          <template v-else>
            <!-- GitHub Drivers -->
            <div v-if="githubDrivers.length > 0" class="driver-section">
              <h3 class="section-title">
                <svg viewBox="0 0 24 24" fill="currentColor" class="github-icon">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                Repositórios GitHub
              </h3>
              <div 
                v-for="driver in githubDrivers" 
                :key="driver.id"
                :class="['driver-item', { active: activeDriver?.id === driver.id }]"
                @click="handleSelectDriver(driver)"
              >
                <div class="driver-info">
                  <span class="driver-name">{{ driver.name }}</span>
                  <span class="driver-path">{{ driver.path }}</span>
                  <span v-if="driver.photoCount" class="driver-stats">
                    {{ driver.photoCount }} fotos • Sync: {{ formatDate(driver.lastSyncAt) }}
                  </span>
                </div>
                <div class="driver-actions">
                  <span v-if="activeDriver?.id === driver.id" class="active-badge">Ativo</span>
                  <button class="remove-btn" @click.stop="handleRemoveDriver(driver)" title="Remover">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <!-- Local Drivers -->
            <div v-if="localDrivers.length > 0" class="driver-section">
              <h3 class="section-title">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                Pastas Locais
              </h3>
              <div 
                v-for="driver in localDrivers" 
                :key="driver.id"
                :class="['driver-item', { active: activeDriver?.id === driver.id }]"
                @click="handleSelectDriver(driver)"
              >
                <div class="driver-info">
                  <span class="driver-name">{{ driver.name }}</span>
                  <span class="driver-path">{{ driver.path }}</span>
                  <span v-if="driver.photoCount" class="driver-stats">
                    {{ driver.photoCount }} fotos
                  </span>
                </div>
                <div class="driver-actions">
                  <span v-if="activeDriver?.id === driver.id" class="active-badge">Ativo</span>
                  <button class="remove-btn" @click.stop="handleRemoveDriver(driver)" title="Remover">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <!-- Empty State -->
            <div v-if="drivers.length === 0" class="empty-state">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              <p>Nenhuma fonte de dados configurada</p>
              <button class="btn-add" @click="activeTab = 'add'">Adicionar Fonte</button>
            </div>
          </template>
        </div>

        <!-- Add Driver -->
        <div v-if="activeTab === 'add'" class="add-driver">
          <div class="add-section">
            <h3>Adicionar Repositório GitHub</h3>
            <p class="section-desc">Conecte um repositório GitHub para armazenar suas fotos na nuvem</p>
            
            <div class="input-group">
              <label>Nome (opcional)</label>
              <input 
                v-model="newDriverName" 
                type="text" 
                placeholder="Meu Álbum de Fotos"
              />
            </div>
            
            <div class="input-group">
              <label>Repositório</label>
              <input 
                v-model="newRepoInput" 
                type="text" 
                placeholder="usuario/repositorio"
                @keyup.enter="handleAddGitHub"
              />
            </div>
            
            <button 
              class="btn-primary" 
              :disabled="!newRepoInput.trim() || !token || addingDriver"
              @click="handleAddGitHub"
            >
              {{ addingDriver ? 'Adicionando...' : 'Adicionar GitHub' }}
            </button>
            
            <p v-if="!token" class="warning">Faça login no GitHub primeiro</p>
          </div>

          <div class="divider">
            <span>ou</span>
          </div>

          <div class="add-section">
            <h3>Adicionar Pasta Local</h3>
            <p class="section-desc">Selecione uma pasta do seu computador para visualizar e importar fotos</p>
            
            <button 
              class="btn-secondary" 
              :disabled="addingDriver"
              @click="handleAddLocal"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              Selecionar Pasta
            </button>
          </div>

          <div v-if="error" class="error-message">{{ error }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.driver-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.driver-panel {
  width: 90%;
  max-width: 560px;
  max-height: 85vh;
  background: var(--surface-1, #111113);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  border-radius: var(--radius-xl, 20px);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
}

.panel-header h2 {
  font-size: 1.125rem;
  font-weight: 600;
}

.close-btn {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  cursor: pointer;
  border-radius: var(--radius-sm, 6px);
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--surface-2, #18181b);
  color: var(--text-primary, #fafafa);
}

.close-btn svg {
  width: 1.25rem;
  height: 1.25rem;
}

.tabs {
  display: flex;
  padding: 0 1rem;
  border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
}

.tab {
  padding: 0.875rem 1rem;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  font-size: 0.875rem;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.tab:hover {
  color: var(--text-primary, #fafafa);
}

.tab.active {
  color: var(--accent-color, #00ff41);
  border-bottom-color: var(--accent-color, #00ff41);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.drivers-list {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.driver-section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-tertiary, #71717a);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.section-title svg {
  width: 1rem;
  height: 1rem;
}

.github-icon {
  width: 1rem;
  height: 1rem;
}

.driver-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
  transition: all 0.2s;
}

.driver-item:hover {
  border-color: var(--border-default, rgba(255,255,255,0.08));
  background: var(--surface-3, #27272a);
}

.driver-item.active {
  border-color: var(--accent-color, #00ff41);
  background: rgba(var(--accent-rgb, 0, 255, 65), 0.05);
}

.driver-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.driver-name {
  font-weight: 500;
  color: var(--text-primary, #fafafa);
}

.driver-path {
  font-size: 0.75rem;
  color: var(--text-tertiary, #71717a);
  font-family: var(--font-mono, monospace);
}

.driver-stats {
  font-size: 0.75rem;
  color: var(--text-muted, #52525b);
}

.driver-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.active-badge {
  padding: 0.25rem 0.5rem;
  background: rgba(var(--accent-rgb, 0, 255, 65), 0.15);
  color: var(--accent-color, #00ff41);
  font-size: 0.625rem;
  font-weight: 600;
  border-radius: var(--radius-full, 9999px);
  text-transform: uppercase;
}

.remove-btn {
  width: 1.75rem;
  height: 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted, #52525b);
  cursor: pointer;
  border-radius: var(--radius-sm, 6px);
  transition: all 0.2s;
}

.remove-btn:hover {
  background: rgba(var(--error-rgb, 239, 68, 68), 0.15);
  color: var(--error, #ef4444);
}

.remove-btn svg {
  width: 1rem;
  height: 1rem;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: 3rem 1rem;
  text-align: center;
}

.empty-state svg {
  width: 3rem;
  height: 3rem;
  color: var(--text-muted, #52525b);
}

.empty-state p {
  color: var(--text-secondary, #a1a1aa);
}

.btn-add {
  padding: 0.625rem 1.25rem;
  background: var(--accent-color, #00ff41);
  border: none;
  color: var(--void, #000);
  font-weight: 500;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
}

.add-driver {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.add-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.add-section h3 {
  font-size: 1rem;
  font-weight: 600;
}

.section-desc {
  font-size: 0.875rem;
  color: var(--text-secondary, #a1a1aa);
  margin-top: -0.5rem;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.input-group label {
  font-size: 0.75rem;
  color: var(--text-tertiary, #71717a);
}

.input-group input {
  padding: 0.625rem 0.875rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  border-radius: var(--radius-md, 10px);
  color: var(--text-primary, #fafafa);
  font-size: 0.875rem;
}

.input-group input:focus {
  outline: none;
  border-color: var(--accent-color, #00ff41);
}

.btn-primary {
  padding: 0.75rem 1.25rem;
  background: linear-gradient(135deg, var(--accent-color, #00ff41), var(--accent-secondary, #008f11));
  border: none;
  color: var(--void, #000);
  font-weight: 600;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  color: var(--text-primary, #fafafa);
  font-weight: 500;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover:not(:disabled) {
  background: var(--surface-3, #27272a);
  border-color: var(--border-strong, rgba(255,255,255,0.12));
}

.btn-secondary svg {
  width: 1.125rem;
  height: 1.125rem;
}

.divider {
  display: flex;
  align-items: center;
  gap: 1rem;
  color: var(--text-muted, #52525b);
  font-size: 0.75rem;
}

.divider::before,
.divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border-subtle, rgba(255,255,255,0.04));
}

.warning {
  font-size: 0.75rem;
  color: var(--warning, #eab308);
}

.error-message {
  padding: 0.75rem 1rem;
  background: rgba(var(--error-rgb, 239, 68, 68), 0.1);
  border: 1px solid rgba(var(--error-rgb, 239, 68, 68), 0.2);
  border-radius: var(--radius-md, 10px);
  color: var(--error, #ef4444);
  font-size: 0.875rem;
}

.loading {
  text-align: center;
  padding: 2rem;
  color: var(--text-secondary, #a1a1aa);
}
</style>
