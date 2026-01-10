<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGitHubAuth } from '../composables/useGitHubAuth'
import PixelCard from './retro/PixelCard.vue'
import PixelButton from './retro/PixelButton.vue'
import PixelInput from './retro/PixelInput.vue'

const emit = defineEmits<{
  complete: []
  close: []
}>()

const { loginWithToken, setRepo, user, error, validating } = useGitHubAuth()

const tokenInput = ref('')
const repoInput = ref('')
const currentStep = ref(1)
const loginError = ref<string | null>(null)

const isTokenValid = computed(() => tokenInput.value.trim().length >= 10)
const isRepoValid = computed(() => /^[\w-]+\/[\w.-]+$/.test(repoInput.value.trim()))

async function handleTokenSubmit() {
  if (!isTokenValid.value) return
  loginError.value = null
  
  const success = await loginWithToken(tokenInput.value.trim())
  if (success) {
    currentStep.value = 3
  } else {
    loginError.value = error.value || 'Failed to validate token'
  }
}

async function handleRepoSubmit() {
  if (!isRepoValid.value) return
  await setRepo(repoInput.value.trim())
  emit('complete')
}

function openTokenPage() {
  window.open('https://github.com/settings/tokens/new?description=iMAGE%20Web&scopes=repo', '_blank')
}

function nextStep() {
  if (currentStep.value < 3) {
    currentStep.value++
  }
}

function prevStep() {
  if (currentStep.value > 1) {
    currentStep.value--
  }
}
</script>

<template>
  <div class="web-login-stepper">
    <PixelCard title="WEB LOGIN" :glow="true">
      <div class="stepper-intro">
        <div class="web-badge">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="2" y1="12" x2="22" y2="12"/>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
          </svg>
          <span>BROWSER MODE</span>
        </div>
        <p class="intro-text">
          You're using iMAGE in your browser. Follow these steps to connect your GitHub account.
        </p>
      </div>

      <!-- Step indicators -->
      <div class="step-indicators">
        <button 
          v-for="step in 3" 
          :key="step"
          class="step-indicator"
          :class="{ active: currentStep === step, complete: currentStep > step }"
          @click="currentStep = step"
        >
          <span v-if="currentStep > step">✓</span>
          <span v-else>{{ step }}</span>
        </button>
        <div class="step-connector" :class="{ filled: currentStep > 1 }"></div>
        <div class="step-connector" :class="{ filled: currentStep > 2 }"></div>
      </div>

      <!-- Step 1: Create Token -->
      <div v-if="currentStep === 1" class="step-content">
        <h3 class="step-title">
          <span class="step-number">1</span>
          Create GitHub Token
        </h3>
        <p class="step-desc">
          Generate a Personal Access Token with <code>repo</code> scope to allow iMAGE to access your repositories.
        </p>
        
        <div class="token-instructions">
          <div class="instruction-item">
            <span class="instruction-num">①</span>
            <span>Click the button below to open GitHub token settings</span>
          </div>
          <div class="instruction-item">
            <span class="instruction-num">②</span>
            <span>Select expiration (recommend: 90 days or No expiration)</span>
          </div>
          <div class="instruction-item">
            <span class="instruction-num">③</span>
            <span>Ensure <code>repo</code> scope is checked</span>
          </div>
          <div class="instruction-item">
            <span class="instruction-num">④</span>
            <span>Click "Generate token" and copy it</span>
          </div>
        </div>

        <PixelButton variant="primary" @click="openTokenPage">
          <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
          </svg>
          Open GitHub Settings
        </PixelButton>
        
        <div class="step-nav">
          <div></div>
          <PixelButton variant="primary" @click="nextStep">
            NEXT >
          </PixelButton>
        </div>
      </div>

      <!-- Step 2: Enter Token -->
      <div v-if="currentStep === 2" class="step-content">
        <h3 class="step-title">
          <span class="step-number">2</span>
          Enter Your Token
        </h3>
        <p class="step-desc">
          Paste the token you just created. It starts with <code>ghp_</code> or <code>github_pat_</code>
        </p>

        <div class="token-input-group">
          <PixelInput
            v-model="tokenInput"
            type="password"
            placeholder="ghp_xxxxxxxxxxxx"
            :disabled="validating"
          />
        </div>

        <div v-if="loginError" class="error-message">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          </svg>
          {{ loginError }}
        </div>

        <div v-if="user" class="success-message">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
            <polyline points="22 4 12 14.01 9 11.01"/>
          </svg>
          Connected as <strong>{{ user.login }}</strong>
        </div>

        <p class="security-note">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          </svg>
          Your token is stored locally in your browser and never sent to our servers.
        </p>
        
        <div class="step-nav">
          <PixelButton variant="ghost" @click="prevStep">
            < BACK
          </PixelButton>
          <PixelButton 
            variant="primary" 
            :disabled="!isTokenValid || validating"
            :loading="validating"
            @click="handleTokenSubmit"
          >
            {{ validating ? 'Validating...' : 'Verify Token' }}
          </PixelButton>
        </div>
      </div>

      <!-- Step 3: Select Repository -->
      <div v-if="currentStep === 3" class="step-content">
        <h3 class="step-title">
          <span class="step-number">3</span>
          Select Repository
        </h3>
        <p class="step-desc">
          Enter the repository where you want to store your photos. Format: <code>username/repo-name</code>
        </p>

        <div class="repo-input-group">
          <PixelInput
            v-model="repoInput"
            placeholder="username/my-photos"
          />
        </div>

        <div class="repo-tips">
          <p><strong>Tips:</strong></p>
          <ul>
            <li>Use a private repo for personal photos</li>
            <li>Create a new repo if you don't have one</li>
            <li>The repo will be created if it doesn't exist</li>
          </ul>
        </div>
        
        <div class="step-nav">
          <PixelButton variant="ghost" @click="prevStep">
            < BACK
          </PixelButton>
          <PixelButton 
            variant="primary" 
            :disabled="!isRepoValid"
            @click="handleRepoSubmit"
          >
            START!
          </PixelButton>
        </div>
      </div>

      <button class="close-btn" @click="emit('close')" aria-label="Close">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </PixelCard>
  </div>
</template>

<style scoped>
.web-login-stepper {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  z-index: 1000;
  padding: 1rem;
}

.web-login-stepper :deep(.pixel-card) {
  max-width: 500px;
  width: 100%;
  position: relative;
}

.stepper-intro {
  margin-bottom: 1.5rem;
  text-align: center;
}

.web-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: linear-gradient(135deg, var(--retro-accent-cyan, #00d4ff) 0%, var(--retro-accent-blue, #0066ff) 100%);
  border: 2px solid #000;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: #000;
  margin-bottom: 1rem;
}

.web-badge svg {
  width: 14px;
  height: 14px;
  stroke: #000;
}

.intro-text {
  font-size: 12px;
  color: var(--retro-text-muted, #9d8ec2);
  max-width: 400px;
  margin: 0 auto;
}

/* Step indicators */
.step-indicators {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
  margin-bottom: 1.5rem;
  position: relative;
}

.step-indicator {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  border: 3px solid #000;
  background: var(--retro-bg-dark, #0f0a1e);
  color: var(--retro-text-muted, #9d8ec2);
  cursor: pointer;
  transition: all 0.1s;
  box-shadow: 2px 2px 0 #000;
  z-index: 1;
}

.step-indicator.active {
  background: var(--retro-accent-pink, #ff2d95);
  color: #fff;
  box-shadow: 2px 2px 0 #000, 0 0 10px var(--retro-accent-pink);
}

.step-indicator.complete {
  background: var(--retro-accent-green, #00ff87);
  color: #000;
  box-shadow: 2px 2px 0 #000, 0 0 10px var(--retro-accent-green);
}

.step-connector {
  width: 40px;
  height: 4px;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 1px solid #000;
  transition: background 0.3s;
}

.step-connector.filled {
  background: var(--retro-accent-green, #00ff87);
}

.step-content {
  padding: 1rem 0;
}

.step-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-family: 'Press Start 2P', monospace;
  font-size: 11px;
  color: var(--retro-text-main, #fff);
  margin-bottom: 0.75rem;
}

.step-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--retro-accent-pink, #ff2d95);
  border: 2px solid #000;
  font-size: 10px;
}

.step-desc {
  font-size: 12px;
  color: var(--retro-text-muted, #9d8ec2);
  margin-bottom: 1.5rem;
  line-height: 1.6;
}

.step-desc code {
  background: var(--retro-bg-dark, #0f0a1e);
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--retro-border, #3d2a6d);
  font-family: monospace;
  font-size: 11px;
  color: var(--retro-accent-green, #00ff87);
}

.token-instructions {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-bottom: 1.5rem;
  padding: 1rem;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 2px solid var(--retro-border, #3d2a6d);
}

.instruction-item {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  font-size: 11px;
  color: var(--retro-text-muted, #9d8ec2);
}

.instruction-num {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--retro-accent-cyan, #00d4ff);
  color: #000;
  font-size: 10px;
  font-weight: bold;
}

.token-input-group,
.repo-input-group {
  margin-bottom: 1rem;
}

.step-nav {
  display: flex;
  justify-content: space-between;
  margin-top: 1.5rem;
  padding-top: 1rem;
  border-top: 2px solid var(--retro-border, #3d2a6d);
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  background: rgba(255, 45, 85, 0.1);
  border: 2px solid var(--retro-accent-pink, #ff2d95);
  color: var(--retro-accent-pink, #ff2d95);
  font-size: 11px;
  margin-bottom: 1rem;
}

.error-message svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.success-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  background: rgba(0, 255, 135, 0.1);
  border: 2px solid var(--retro-accent-green, #00ff87);
  color: var(--retro-accent-green, #00ff87);
  font-size: 11px;
  margin-bottom: 1rem;
}

.success-message svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.security-note {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 10px;
  color: var(--retro-text-muted, #9d8ec2);
  opacity: 0.7;
}

.security-note svg {
  width: 14px;
  height: 14px;
  stroke: var(--retro-accent-green, #00ff87);
}

.repo-tips {
  padding: 1rem;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 2px solid var(--retro-border, #3d2a6d);
  font-size: 11px;
}

.repo-tips p {
  color: var(--retro-text-main, #fff);
  margin-bottom: 0.5rem;
}

.repo-tips ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.repo-tips li {
  color: var(--retro-text-muted, #9d8ec2);
  padding-left: 1rem;
  position: relative;
  margin-bottom: 0.25rem;
}

.repo-tips li::before {
  content: '>';
  position: absolute;
  left: 0;
  color: var(--retro-accent-cyan, #00d4ff);
}

.close-btn {
  position: absolute;
  top: 1rem;
  right: 1rem;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--retro-bg-dark, #0f0a1e);
  border: 2px solid var(--retro-border, #3d2a6d);
  cursor: pointer;
  transition: all 0.1s;
}

.close-btn:hover {
  background: var(--retro-accent-pink, #ff2d95);
  border-color: #000;
}

.close-btn svg {
  width: 16px;
  height: 16px;
  stroke: var(--retro-text-muted, #9d8ec2);
}

.close-btn:hover svg {
  stroke: #fff;
}
</style>
