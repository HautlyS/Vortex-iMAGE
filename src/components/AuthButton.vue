<script setup lang="ts">
import { ref } from 'vue'
import { useGitHubAuth } from '../composables/useGitHubAuth'

const { 
  user, loading, userCode, error, validating,
  canUseDeviceFlow, isWebMode,
  startLogin, loginWithToken, logout 
} = useGitHubAuth()

const showManualInput = ref(false)
const manualToken = ref('')

async function copyCode() {
  if (!userCode.value) return
  try {
    await navigator.clipboard.writeText(userCode.value)
    if ('vibrate' in navigator) navigator.vibrate(10)
  } catch {}
}

async function handleManualLogin() {
  const success = await loginWithToken(manualToken.value)
  if (success) {
    manualToken.value = ''
    showManualInput.value = false
  }
}

function toggleManualInput() {
  showManualInput.value = !showManualInput.value
  if (!showManualInput.value) manualToken.value = ''
}
</script>

<template>
  <div class="auth">
    <!-- Logged In -->
    <div v-if="user" class="user-card">
      <img :src="user.avatar_url" :alt="user.login" class="avatar" />
      <div class="user-info">
        <span class="user-name">{{ user.login }}</span>
        <span class="user-status">Conectado</span>
      </div>
      <button @click="logout" class="btn-logout" title="Sair">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
          <polyline points="16,17 21,12 16,7"/>
          <line x1="21" y1="12" x2="9" y2="12"/>
        </svg>
      </button>
    </div>

    <!-- Device Code (Tauri only) -->
    <div v-else-if="userCode" class="code-card">
      <div class="code-header">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/></svg>
        <span>Código de Acesso</span>
      </div>
      <button class="code-value" @click="copyCode" title="Toque para copiar">
        {{ userCode }}
      </button>
      <p class="code-hint">Toque no código para copiar • Abra github.com/login/device</p>
      <div class="code-loading">
        <div class="code-spinner" />
        <span>Aguardando autorização...</span>
      </div>
    </div>

    <!-- Login Options -->
    <div v-else class="login-options">
      <!-- Device Flow Button (Tauri only) -->
      <button 
        v-if="canUseDeviceFlow" 
        @click="startLogin" 
        :disabled="loading" 
        class="btn-login"
      >
        <svg v-if="loading" class="spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" opacity="0.25"/>
          <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
        </svg>
        <svg v-else viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
        </svg>
        <span>{{ loading ? 'Conectando...' : 'Entrar com GitHub' }}</span>
      </button>

      <!-- Manual Token Toggle -->
      <button 
        @click="toggleManualInput" 
        class="btn-manual-toggle"
        :class="{ active: showManualInput }"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
        <span>{{ isWebMode ? 'Entrar com Token' : 'Usar Token Manual' }}</span>
        <svg class="chevron" :class="{ rotated: showManualInput }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
      </button>

      <!-- Manual Token Input -->
      <div v-if="showManualInput" class="manual-input-section">
        <div class="token-help">
          <p>Gere um token em <a href="https://github.com/settings/tokens/new" target="_blank" rel="noopener">github.com/settings/tokens</a></p>
          <p class="scope-hint">Selecione o escopo <code>repo</code></p>
        </div>
        <div class="input-group">
          <input
            v-model="manualToken"
            type="password"
            placeholder="ghp_xxxxxxxxxxxx"
            class="token-input"
            @keyup.enter="handleManualLogin"
            :disabled="validating"
          />
          <button 
            @click="handleManualLogin" 
            :disabled="validating || !manualToken.trim()"
            class="btn-validate"
          >
            <svg v-if="validating" class="spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" opacity="0.25"/>
              <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.auth { 
  width: 100%;
  padding-bottom: env(safe-area-inset-bottom, 0);
}

.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--pageBG);
  border-radius: var(--global-border-radius-medium);
  box-shadow: var(--shadow-small);
}

.avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  box-shadow: 0 0 0 2px var(--pageBG), 0 0 0 3px var(--systemGray4);
}

.user-info { flex: 1; min-width: 0; }

.user-name {
  display: block;
  font-size: 15px;
  font-weight: 600;
  color: var(--systemPrimary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-status {
  font-size: 13px;
  color: var(--systemGreen);
}

.btn-logout {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--systemGray6);
  border: none;
  color: var(--systemSecondary);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-logout:active {
  background: rgba(255, 59, 48, 0.12);
  color: var(--systemRed);
  transform: scale(0.95);
}

.btn-logout svg { width: 18px; height: 18px; }

.code-card {
  padding: 24px 20px;
  background: var(--pageBG);
  border-radius: var(--global-border-radius-large);
  box-shadow: var(--shadow-medium);
  text-align: center;
}

.code-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 16px;
  color: var(--systemSecondary);
  font-size: 13px;
  font-weight: 500;
}

.code-header svg { width: 16px; height: 16px; }

.code-value {
  display: block;
  width: 100%;
  font-family: 'SF Mono', ui-monospace, monospace;
  font-size: 32px;
  font-weight: 700;
  letter-spacing: 0.2em;
  color: var(--systemPrimary);
  margin-bottom: 8px;
  padding: 16px;
  background: rgba(var(--accent-rgb, 0, 122, 255), 0.08);
  border: 1px solid rgba(var(--accent-rgb, 0, 122, 255), 0.2);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.code-value:active {
  transform: scale(0.98);
  background: rgba(var(--accent-rgb, 0, 122, 255), 0.15);
}

.code-hint {
  font-size: 13px;
  color: var(--systemTertiary);
  margin-bottom: 16px;
}

.code-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--systemSecondary);
  font-size: 13px;
}

.code-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(var(--accent-rgb, 0, 122, 255), 0.2);
  border-top-color: var(--accent-color, #007aff);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.login-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.btn-login {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 16px 20px;
  min-height: 50px;
  background: var(--keyColor);
  border: none;
  color: #ffffff;
  font-size: 17px;
  font-weight: 600;
  border-radius: 1000px;
  cursor: pointer;
  transition: all 0.14s ease-out;
}

.btn-login:hover { background: color-mix(in srgb, var(--keyColor), #000 8%); }
.btn-login:active { background: color-mix(in srgb, var(--keyColor), #000 15%); transform: scale(0.98); }
.btn-login:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-login svg { width: 20px; height: 20px; }

.btn-manual-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 14px 16px;
  background: var(--systemGray6);
  border: none;
  color: var(--systemSecondary);
  font-size: 15px;
  font-weight: 500;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-manual-toggle:hover { background: var(--systemGray5); }
.btn-manual-toggle.active { background: var(--systemGray5); color: var(--systemPrimary); }
.btn-manual-toggle svg { width: 18px; height: 18px; }
.btn-manual-toggle .chevron { width: 16px; height: 16px; margin-left: auto; transition: transform 0.2s; }
.btn-manual-toggle .chevron.rotated { transform: rotate(180deg); }

.manual-input-section {
  padding: 16px;
  background: var(--systemGray6);
  border-radius: 12px;
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}

.token-help {
  margin-bottom: 12px;
  font-size: 13px;
  color: var(--systemSecondary);
  text-align: center;
}

.token-help a {
  color: var(--keyColor);
  text-decoration: none;
}

.token-help a:hover { text-decoration: underline; }

.scope-hint {
  margin-top: 4px;
  color: var(--systemTertiary);
}

.scope-hint code {
  background: var(--systemGray5);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'SF Mono', ui-monospace, monospace;
  font-size: 12px;
}

.input-group {
  display: flex;
  gap: 8px;
}

.token-input {
  flex: 1;
  padding: 12px 14px;
  background: var(--pageBG);
  border: 1px solid var(--systemGray4);
  border-radius: 10px;
  font-size: 15px;
  font-family: 'SF Mono', ui-monospace, monospace;
  color: var(--systemPrimary);
  outline: none;
  transition: border-color 0.2s;
}

.token-input:focus { border-color: var(--keyColor); }
.token-input::placeholder { color: var(--systemTertiary); }

.btn-validate {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--keyColor);
  border: none;
  color: #fff;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-validate:hover { background: color-mix(in srgb, var(--keyColor), #000 8%); }
.btn-validate:active { transform: scale(0.95); }
.btn-validate:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-validate svg { width: 20px; height: 20px; }

.spinner { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.error {
  margin-top: 12px;
  padding: 12px;
  background: rgba(255, 59, 48, 0.12);
  border-radius: var(--global-border-radius-small);
  font-size: 13px;
  color: var(--systemRed);
}
</style>
