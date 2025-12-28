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

/* User Card */
.user-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 0.5rem;
}
.avatar {
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 50%;
  border: 2px solid rgba(255,255,255,0.1);
}
.user-info { flex: 1; min-width: 0; }
.user-name {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: #fafafa;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.user-status { font-size: 0.75rem; color: #22c55e; }
.btn-logout {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #71717a;
  border-radius: 0.375rem;
  cursor: pointer;
}
.btn-logout:hover { background: rgba(239, 68, 68, 0.1); color: #ef4444; }
.btn-logout svg { width: 1.125rem; height: 1.125rem; }

/* Code Card */
.code-card {
  padding: 1rem;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 0.5rem;
  text-align: center;
}
.code-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
  color: #818cf8;
  font-size: 0.75rem;
  font-weight: 500;
}
.code-header svg { width: 1rem; height: 1rem; }
.code-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 1.5rem;
  font-weight: 700;
  letter-spacing: 0.15em;
  color: #fafafa;
  user-select: all;
  margin-bottom: 0.5rem;
}
.code-hint { font-size: 0.75rem; color: #71717a; }

/* Login Button */
.btn-login {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.625rem;
  padding: 0.75rem;
  background: #fafafa;
  border: none;
  color: #18181b;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-login:hover { background: #e4e4e7; }
.btn-login:disabled { opacity: 0.7; cursor: not-allowed; }
.btn-login svg { width: 1.25rem; height: 1.25rem; }
.spinner { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.error {
  margin-top: 0.75rem;
  padding: 0.625rem;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 0.375rem;
  font-size: 0.75rem;
  color: #fca5a5;
}
</style>
