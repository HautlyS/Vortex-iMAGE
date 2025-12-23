<script setup lang="ts">
import { useGitHubAuth } from '../composables/useGitHubAuth'

const { user, loading, userCode, error, startLogin, logout } = useGitHubAuth()
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

    <!-- Device Code -->
    <div v-else-if="userCode" class="code-card">
      <div class="code-header">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/></svg>
        <span>Código de Acesso</span>
      </div>
      <div class="code-value">{{ userCode }}</div>
      <p class="code-hint">Digite este código no GitHub</p>
    </div>

    <!-- Login Button -->
    <button v-else @click="startLogin" :disabled="loading" class="btn-login">
      <svg v-if="loading" class="spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" opacity="0.25"/>
        <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
      </svg>
      <span>{{ loading ? 'Conectando...' : 'Entrar com GitHub' }}</span>
    </button>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.auth { width: 100%; }

/* User Card - iOS Style */
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
  width: 36px;
  height: 36px;
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

.btn-logout:hover {
  background: rgba(255, 59, 48, 0.12);
  color: var(--systemRed);
}

.btn-logout svg { width: 18px; height: 18px; }

/* Code Card - iOS Style */
.code-card {
  padding: 20px;
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
  margin-bottom: 12px;
  color: var(--systemSecondary);
  font-size: 13px;
  font-weight: 500;
}

.code-header svg { width: 16px; height: 16px; }

.code-value {
  font-family: 'SF Mono', ui-monospace, monospace;
  font-size: 28px;
  font-weight: 700;
  letter-spacing: 0.15em;
  color: var(--systemPrimary);
  user-select: all;
  margin-bottom: 8px;
}

.code-hint {
  font-size: 13px;
  color: var(--systemTertiary);
}

/* Login Button - iOS Get Button Style */
.btn-login {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 14px 20px;
  background: var(--keyColor);
  border: none;
  color: #ffffff;
  font-size: 17px;
  font-weight: 600;
  border-radius: 1000px;
  cursor: pointer;
  transition: background-color 0.14s ease-out;
}

.btn-login:hover {
  background: color-mix(in srgb, var(--keyColor), #000 8%);
}

.btn-login:active {
  background: color-mix(in srgb, var(--keyColor), #000 15%);
}

.btn-login:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-login svg { width: 20px; height: 20px; }

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
