/**
 * Vue Component - 1 components, 0 composables
 * Main functionality: UI component with reactive state management
 * Dependencies: GlassSurface
 */

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGitHubAuth } from '../composables/useGitHubAuth'
import GlassSurface from './GlassSurface.vue'

const { user, repo, setRepo, startLogin, logout } = useGitHubAuth()

const showDropdown = ref(false)

const emit = defineEmits<{
  'create-repo': []
}>()

const repos = computed(() => {
  if (!repo.value) return []
  return [repo.value]
})

function selectRepo(repoName: string) {
  setRepo(repoName)
  showDropdown.value = false
}

function handleCreateRepo() {
  showDropdown.value = false
  emit('create-repo')
}
</script>

<template>
  <div class="top-header">
    <GlassSurface :border-radius="20" :border-width="0.08" class="header-pill">
      <!-- Not logged in -->
      <button v-if="!user" @click="startLogin" class="login-btn">
        <svg viewBox="0 0 24 24" fill="currentColor" class="w-5 h-5">
          <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
        </svg>
        <span>Login</span>
      </button>

      <!-- Logged in -->
      <template v-else>
        <button class="user-btn" @click="showDropdown = !showDropdown">
          <img :src="user.avatar_url" :alt="user.login" class="avatar" />
          <span class="username">{{ user.login }}</span>
          <span class="repo-name" v-if="repo">/ {{ repo.split('/')[1] }}</span>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chevron" :class="{ open: showDropdown }">
            <path d="m6 9 6 6 6-6"/>
          </svg>
        </button>

        <!-- Dropdown -->
        <Transition name="dropdown">
          <div v-if="showDropdown" class="dropdown">
            <div class="dropdown-section">
              <span class="section-label">Repositórios</span>
              <button 
                v-for="r in repos" 
                :key="r" 
                class="dropdown-item"
                :class="{ active: r === repo }"
                @click="selectRepo(r)"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="w-4 h-4">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                <span>{{ r }}</span>
                <svg v-if="r === repo" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" class="check">
                  <path d="M5 13l4 4L19 7"/>
                </svg>
              </button>
              <button class="dropdown-item add-new" @click="handleCreateRepo">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="w-4 h-4">
                  <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
                <span>Criar novo repositório</span>
              </button>
            </div>
            <div class="dropdown-divider" />
            <button class="dropdown-item logout" @click="logout">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="w-4 h-4">
                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
                <polyline points="16,17 21,12 16,7"/>
                <line x1="21" y1="12" x2="9" y2="12"/>
              </svg>
              <span>Sair</span>
            </button>
          </div>
        </Transition>
      </template>
    </GlassSurface>
  </div>
</template>

<style scoped>
.top-header {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 100;
}

.header-pill {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  position: relative;
}

.login-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 12px;
  transition: background 0.15s;
}

.login-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.user-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: white;
  font-size: 13px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 12px;
  transition: background 0.15s;
}

.user-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
}

.username {
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}

.repo-name {
  color: rgba(255, 255, 255, 0.6);
}

.chevron {
  width: 16px;
  height: 16px;
  color: rgba(255, 255, 255, 0.5);
  transition: transform 0.2s;
}

.chevron.open {
  transform: rotate(180deg);
}

.dropdown {
  position: absolute;
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  min-width: 220px;
  background: rgba(30, 30, 30, 0.95);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 14px;
  padding: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.dropdown-section {
  padding: 4px 0;
}

.section-label {
  display: block;
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.4);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.85);
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  border-radius: 8px;
  transition: background 0.15s;
}

.dropdown-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.dropdown-item.active {
  color: var(--accent-color, #00f0ff);
}

.dropdown-item .check {
  width: 16px;
  height: 16px;
  margin-left: auto;
  color: var(--accent-color, #00f0ff);
}

.dropdown-item.add-new {
  color: var(--accent-color, #00f0ff);
}

.dropdown-item.logout {
  color: #ff6b6b;
}

.dropdown-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin: 4px 0;
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-8px);
}

@media (max-width: 768px) {
  .top-header {
    top: 12px;
  }
  
  .repo-name {
    display: none;
  }
}
</style>