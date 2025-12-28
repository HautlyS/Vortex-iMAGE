<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useBackupSettings } from '../composables/useBackupSettings'
import { useDataDriver } from '../composables/useDataDriver'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  config,
  stats: _stats,
  isConfigured,
  canAutoBackup: _canAutoBackup,
  loadConfig,
  setWatchFolder,
  setTargetDriver,
  setEnabled,
  setAutoUpload,
  setDeleteAfterUpload,
  setSyncDeletions,
  setFileTypes: _setFileTypes,
  addExcludePattern,
  removeExcludePattern,
  formatFileSize: _formatFileSize,
  formatLastBackup
} = useBackupSettings()
void _stats
void _canAutoBackup
void _setFileTypes
void _formatFileSize

const { drivers, githubDrivers, loadDrivers } = useDataDriver()

const activeSection = ref<'folder' | 'sync' | 'advanced'>('folder')
const newExcludePattern = ref('')

onMounted(async () => {
  await Promise.all([loadConfig(), loadDrivers()])
})

async function handleSelectFolder() {
  const folder = await open({ multiple: false, directory: true })
  if (folder && typeof folder === 'string') {
    await setWatchFolder(folder)
  }
}

async function handleClearFolder() {
  await setWatchFolder(null)
}

async function handleAddExclude() {
  if (newExcludePattern.value.trim()) {
    await addExcludePattern(newExcludePattern.value.trim())
    newExcludePattern.value = ''
  }
}

const targetDriverName = computed(() => {
  if (!config.value.targetDriverId) return null
  const _driver = drivers.value.find(d => d.id === config.value.targetDriverId)
  return _driver?.name || null
})
void targetDriverName

const folderName = computed(() => {
  if (!config.value.watchFolder) return null
  return config.value.watchFolder.split('/').pop() || config.value.watchFolder
})
</script>

<template>
  <div class="backup-overlay" @click.self="emit('close')">
    <div class="backup-panel">
      <!-- Header -->
      <div class="panel-header">
        <div class="header-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          <h2>Backup Automático</h2>
        </div>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Status Bar -->
      <div class="status-bar" :class="{ enabled: config.enabled && isConfigured }">
        <div class="status-info">
          <span class="status-dot" :class="{ active: config.enabled && isConfigured }"></span>
          <span class="status-text">
            {{ config.enabled && isConfigured ? 'Backup ativo' : 'Backup desativado' }}
          </span>
        </div>
        <div class="status-stats">
          <span>{{ config.totalBackedUp }} arquivos sincronizados</span>
          <span class="separator">•</span>
          <span>Último: {{ formatLastBackup() }}</span>
        </div>
      </div>

      <!-- Navigation -->
      <div class="nav-tabs">
        <button 
          :class="['nav-tab', { active: activeSection === 'folder' }]"
          @click="activeSection = 'folder'"
        >
          📁 Pasta de Origem
        </button>
        <button 
          :class="['nav-tab', { active: activeSection === 'sync' }]"
          @click="activeSection = 'sync'"
        >
          🔄 Sincronização
        </button>
        <button 
          :class="['nav-tab', { active: activeSection === 'advanced' }]"
          @click="activeSection = 'advanced'"
        >
          ⚙️ Avançado
        </button>
      </div>

      <!-- Content -->
      <div class="panel-content">
        <!-- Folder Section -->
        <div v-if="activeSection === 'folder'" class="section">
          <div class="section-block">
            <h3>Pasta para Monitorar</h3>
            <p class="section-desc">
              Selecione a pasta do seu computador que contém as fotos para backup
            </p>
            
            <div v-if="config.watchFolder" class="folder-display">
              <div class="folder-info">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                <div class="folder-details">
                  <span class="folder-name">{{ folderName }}</span>
                  <span class="folder-path">{{ config.watchFolder }}</span>
                </div>
              </div>
              <div class="folder-actions">
                <button class="btn-change" @click="handleSelectFolder">Alterar</button>
                <button class="btn-remove" @click="handleClearFolder">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M18 6L6 18M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>
            
            <button v-else class="btn-select-folder" @click="handleSelectFolder">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                <line x1="12" y1="11" x2="12" y2="17"/>
                <line x1="9" y1="14" x2="15" y2="14"/>
              </svg>
              Selecionar Pasta
            </button>
          </div>

          <div class="section-block">
            <h3>Destino do Backup</h3>
            <p class="section-desc">
              Escolha para qual repositório as fotos serão enviadas
            </p>
            
            <div class="driver-select">
              <select 
                :value="config.targetDriverId || ''"
                @change="setTargetDriver(($event.target as HTMLSelectElement).value || null)"
              >
                <option value="">Selecione um destino...</option>
                <option 
                  v-for="driver in githubDrivers" 
                  :key="driver.id" 
                  :value="driver.id"
                >
                  {{ driver.name }} ({{ driver.path }})
                </option>
              </select>
            </div>
            
            <p v-if="githubDrivers.length === 0" class="warning">
              Adicione um repositório GitHub primeiro nas Fontes de Dados
            </p>
          </div>
        </div>

        <!-- Sync Section -->
        <div v-if="activeSection === 'sync'" class="section">
          <div class="section-block">
            <h3>Comportamento de Sincronização</h3>
            
            <div class="toggle-group">
              <label class="toggle-item">
                <div class="toggle-info">
                  <span class="toggle-label">Ativar Backup</span>
                  <span class="toggle-desc">Habilita o sistema de backup automático</span>
                </div>
                <button 
                  :class="['toggle-switch', { active: config.enabled }]"
                  @click="setEnabled(!config.enabled)"
                >
                  <span class="toggle-knob"></span>
                </button>
              </label>

              <label class="toggle-item">
                <div class="toggle-info">
                  <span class="toggle-label">Upload Automático</span>
                  <span class="toggle-desc">Envia novas fotos automaticamente quando detectadas</span>
                </div>
                <button 
                  :class="['toggle-switch', { active: config.autoUpload }]"
                  :disabled="!config.enabled"
                  @click="setAutoUpload(!config.autoUpload)"
                >
                  <span class="toggle-knob"></span>
                </button>
              </label>

              <label class="toggle-item warning-toggle">
                <div class="toggle-info">
                  <span class="toggle-label">🗑️ Excluir Após Upload</span>
                  <span class="toggle-desc">Remove o arquivo local após enviar para o GitHub</span>
                </div>
                <button 
                  :class="['toggle-switch', { active: config.deleteAfterUpload }]"
                  :disabled="!config.enabled"
                  @click="setDeleteAfterUpload(!config.deleteAfterUpload)"
                >
                  <span class="toggle-knob"></span>
                </button>
              </label>

              <label class="toggle-item warning-toggle">
                <div class="toggle-info">
                  <span class="toggle-label">🔗 Sincronizar Exclusões</span>
                  <span class="toggle-desc">Quando excluir do GitHub, também exclui da pasta local</span>
                </div>
                <button 
                  :class="['toggle-switch', { active: config.syncDeletions }]"
                  :disabled="!config.enabled"
                  @click="setSyncDeletions(!config.syncDeletions)"
                >
                  <span class="toggle-knob"></span>
                </button>
              </label>
            </div>
          </div>

          <div v-if="config.deleteAfterUpload || config.syncDeletions" class="warning-box">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
              <line x1="12" y1="9" x2="12" y2="13"/>
              <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
            <div>
              <strong>Atenção!</strong>
              <p>Arquivos serão excluídos permanentemente. Certifique-se de que o backup está funcionando corretamente antes de ativar estas opções.</p>
            </div>
          </div>
        </div>

        <!-- Advanced Section -->
        <div v-if="activeSection === 'advanced'" class="section">
          <div class="section-block">
            <h3>Tipos de Arquivo</h3>
            <p class="section-desc">Extensões de arquivo que serão incluídas no backup</p>
            
            <div class="file-types">
              <span 
                v-for="type in config.fileTypes" 
                :key="type" 
                class="file-type-tag"
              >
                .{{ type }}
              </span>
            </div>
          </div>

          <div class="section-block">
            <h3>Padrões de Exclusão</h3>
            <p class="section-desc">Arquivos que correspondem a estes padrões serão ignorados</p>
            
            <div class="exclude-list">
              <div 
                v-for="pattern in config.excludePatterns" 
                :key="pattern" 
                class="exclude-item"
              >
                <code>{{ pattern }}</code>
                <button @click="removeExcludePattern(pattern)">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M18 6L6 18M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>
            
            <div class="add-exclude">
              <input 
                v-model="newExcludePattern" 
                type="text" 
                placeholder="Ex: *.tmp, backup_*"
                @keyup.enter="handleAddExclude"
              />
              <button @click="handleAddExclude" :disabled="!newExcludePattern.trim()">
                Adicionar
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backup-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.backup-panel {
  width: 90%;
  max-width: 600px;
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

.header-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.header-title svg {
  width: 1.5rem;
  height: 1.5rem;
  color: var(--accent-color, #00ff41);
}

.header-title h2 {
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

.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.875rem 1.5rem;
  background: var(--surface-0, #0a0a0b);
  border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
}

.status-bar.enabled {
  background: rgba(var(--accent-rgb, 0, 255, 65), 0.05);
}

.status-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.status-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  background: var(--text-muted, #52525b);
}

.status-dot.active {
  background: var(--accent-color, #00ff41);
  box-shadow: 0 0 8px var(--accent-glow, rgba(0, 255, 65, 0.6));
}

.status-text {
  font-size: 0.875rem;
  font-weight: 500;
}

.status-stats {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--text-tertiary, #71717a);
}

.separator {
  color: var(--text-muted, #52525b);
}

.nav-tabs {
  display: flex;
  padding: 0 1rem;
  border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
}

.nav-tab {
  padding: 0.875rem 1rem;
  background: transparent;
  border: none;
  color: var(--text-secondary, #a1a1aa);
  font-size: 0.875rem;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.nav-tab:hover {
  color: var(--text-primary, #fafafa);
}

.nav-tab.active {
  color: var(--accent-color, #00ff41);
  border-bottom-color: var(--accent-color, #00ff41);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.section-block {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.section-block h3 {
  font-size: 0.9375rem;
  font-weight: 600;
}

.section-desc {
  font-size: 0.8125rem;
  color: var(--text-secondary, #a1a1aa);
  margin-top: -0.25rem;
}

.folder-display {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  border-radius: var(--radius-md, 10px);
}

.folder-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.folder-info svg {
  width: 1.5rem;
  height: 1.5rem;
  color: var(--accent-color, #00ff41);
}

.folder-details {
  display: flex;
  flex-direction: column;
}

.folder-name {
  font-weight: 500;
}

.folder-path {
  font-size: 0.75rem;
  color: var(--text-tertiary, #71717a);
  font-family: var(--font-mono, monospace);
}

.folder-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-change {
  padding: 0.375rem 0.75rem;
  background: var(--surface-3, #27272a);
  border: none;
  color: var(--text-primary, #fafafa);
  font-size: 0.75rem;
  border-radius: var(--radius-sm, 6px);
  cursor: pointer;
}

.btn-remove {
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
}

.btn-remove:hover {
  background: rgba(var(--error-rgb, 239, 68, 68), 0.15);
  color: var(--error, #ef4444);
}

.btn-remove svg {
  width: 1rem;
  height: 1rem;
}

.btn-select-folder {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 1rem;
  background: var(--surface-2, #18181b);
  border: 2px dashed var(--border-default, rgba(255,255,255,0.08));
  color: var(--text-secondary, #a1a1aa);
  font-size: 0.875rem;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-select-folder:hover {
  border-color: var(--accent-color, #00ff41);
  color: var(--accent-color, #00ff41);
  background: rgba(var(--accent-rgb, 0, 255, 65), 0.05);
}

.btn-select-folder svg {
  width: 1.25rem;
  height: 1.25rem;
}

.driver-select select {
  width: 100%;
  padding: 0.75rem 1rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  border-radius: var(--radius-md, 10px);
  color: var(--text-primary, #fafafa);
  font-size: 0.875rem;
  cursor: pointer;
}

.driver-select select:focus {
  outline: none;
  border-color: var(--accent-color, #00ff41);
}

.warning {
  font-size: 0.75rem;
  color: var(--warning, #eab308);
}

.toggle-group {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.toggle-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
}

.toggle-item.warning-toggle {
  border-color: rgba(var(--warning-rgb, 234, 179, 8), 0.2);
}

.toggle-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.toggle-label {
  font-weight: 500;
  font-size: 0.875rem;
}

.toggle-desc {
  font-size: 0.75rem;
  color: var(--text-tertiary, #71717a);
}

.toggle-switch {
  width: 2.75rem;
  height: 1.5rem;
  background: var(--surface-4, #3f3f46);
  border: none;
  border-radius: var(--radius-full, 9999px);
  cursor: pointer;
  position: relative;
  transition: all 0.2s;
}

.toggle-switch.active {
  background: var(--accent-color, #00ff41);
}

.toggle-switch:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 1.25rem;
  height: 1.25rem;
  background: white;
  border-radius: 50%;
  transition: all 0.2s;
}

.toggle-switch.active .toggle-knob {
  left: calc(100% - 1.25rem - 2px);
}

.warning-box {
  display: flex;
  gap: 0.75rem;
  padding: 1rem;
  background: rgba(var(--warning-rgb, 234, 179, 8), 0.1);
  border: 1px solid rgba(var(--warning-rgb, 234, 179, 8), 0.2);
  border-radius: var(--radius-md, 10px);
}

.warning-box svg {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--warning, #eab308);
  flex-shrink: 0;
}

.warning-box strong {
  color: var(--warning, #eab308);
  font-size: 0.875rem;
}

.warning-box p {
  font-size: 0.75rem;
  color: var(--text-secondary, #a1a1aa);
  margin-top: 0.25rem;
}

.file-types {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.file-type-tag {
  padding: 0.25rem 0.625rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-subtle, rgba(255,255,255,0.04));
  border-radius: var(--radius-sm, 6px);
  font-size: 0.75rem;
  font-family: var(--font-mono, monospace);
  color: var(--text-secondary, #a1a1aa);
}

.exclude-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.exclude-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--surface-2, #18181b);
  border-radius: var(--radius-sm, 6px);
}

.exclude-item code {
  font-size: 0.8125rem;
  color: var(--text-secondary, #a1a1aa);
}

.exclude-item button {
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted, #52525b);
  cursor: pointer;
  border-radius: var(--radius-sm, 6px);
}

.exclude-item button:hover {
  color: var(--error, #ef4444);
}

.exclude-item button svg {
  width: 0.875rem;
  height: 0.875rem;
}

.add-exclude {
  display: flex;
  gap: 0.5rem;
}

.add-exclude input {
  flex: 1;
  padding: 0.5rem 0.75rem;
  background: var(--surface-2, #18181b);
  border: 1px solid var(--border-default, rgba(255,255,255,0.08));
  border-radius: var(--radius-sm, 6px);
  color: var(--text-primary, #fafafa);
  font-size: 0.8125rem;
}

.add-exclude input:focus {
  outline: none;
  border-color: var(--accent-color, #00ff41);
}

.add-exclude button {
  padding: 0.5rem 1rem;
  background: var(--surface-3, #27272a);
  border: none;
  color: var(--text-primary, #fafafa);
  font-size: 0.8125rem;
  border-radius: var(--radius-sm, 6px);
  cursor: pointer;
}

.add-exclude button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
