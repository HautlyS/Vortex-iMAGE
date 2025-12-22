import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { load } from '@tauri-apps/plugin-store'

interface DeviceCodeResponse {
  device_code: string
  user_code: string
  verification_uri: string
  interval: number
  expires_in: number
}

interface GitHubUser {
  login: string
  avatar_url: string
}

const token = ref<string | null>(null)
const user = ref<GitHubUser | null>(null)
const repo = ref<string>('')
const tokenSecured = ref(false)

let pollInterval: ReturnType<typeof setInterval> | null = null
let pollTimeout: ReturnType<typeof setTimeout> | null = null

function clearPolling() {
  if (pollInterval) {
    clearInterval(pollInterval)
    pollInterval = null
  }
  if (pollTimeout) {
    clearTimeout(pollTimeout)
    pollTimeout = null
  }
}

// Secure token storage helpers
async function secureStoreToken(plainToken: string): Promise<void> {
  try {
    const encrypted = await invoke<number[]>('secure_store_token', { token: plainToken })
    const store = await load('settings.json')
    await store.set('encryptedToken', encrypted)
    await store.delete('token') // Remove old plaintext token
    await store.save()
    tokenSecured.value = true
  } catch (e) {
    console.error('Failed to securely store token:', e)
    // Fallback to regular storage
    const store = await load('settings.json')
    await store.set('token', plainToken)
    await store.save()
    tokenSecured.value = false
  }
}

async function secureRetrieveToken(): Promise<string | null> {
  try {
    const store = await load('settings.json')
    
    // Try encrypted token first
    const encrypted = await store.get<number[]>('encryptedToken')
    if (encrypted && encrypted.length > 0) {
      const plainToken = await invoke<string>('secure_retrieve_token', { encrypted })
      tokenSecured.value = true
      return plainToken
    }
    
    // Fallback to old plaintext token and migrate it
    const plainToken = await store.get<string>('token')
    if (plainToken) {
      // Migrate to secure storage
      await secureStoreToken(plainToken)
      return plainToken
    }
    
    return null
  } catch (e) {
    console.error('Failed to retrieve token:', e)
    // Try plaintext fallback
    const store = await load('settings.json')
    return await store.get<string>('token') || null
  }
}

export function useGitHubAuth() {
  const loading = ref(false)
  const userCode = ref('')
  const error = ref<string | null>(null)

  onUnmounted(clearPolling)

  async function init() {
    try {
      const store = await load('settings.json')
      token.value = await secureRetrieveToken()
      repo.value = await store.get<string>('repo') || ''
      if (token.value) {
        user.value = await invoke<GitHubUser>('get_user', { token: token.value })
      }
    } catch {
      token.value = null
      user.value = null
      const store = await load('settings.json')
      await store.delete('token')
      await store.delete('encryptedToken')
      await store.save()
    }
  }

  async function startLogin() {
    if (loading.value) return // Prevent multiple calls
    
    clearPolling()
    loading.value = true
    error.value = null

    try {
      const res = await invoke<DeviceCodeResponse>('start_oauth')
      userCode.value = res.user_code
      await open(res.verification_uri)

      // Set timeout for expiration
      pollTimeout = setTimeout(() => {
        clearPolling()
        loading.value = false
        userCode.value = ''
        error.value = 'Authentication expired. Please try again.'
      }, res.expires_in * 1000)

      // Poll every 5 seconds (GitHub's minimum interval)
      const pollIntervalMs = Math.max(res.interval * 1000, 5000)
      
      pollInterval = setInterval(async () => {
        try {
          const result = await invoke<string | null>('poll_oauth', { deviceCode: res.device_code })
          
          // Check if we got a token (non-null, non-empty string)
          if (result && typeof result === 'string' && result.length > 0) {
            clearPolling()
            token.value = result
            
            try {
              const fetchedUser = await invoke<GitHubUser>('get_user', { token: result })
              user.value = fetchedUser
            } catch (userErr) {
              console.error('Failed to fetch user:', userErr)
            }
            
            // Store token securely
            await secureStoreToken(result)
            
            userCode.value = ''
            loading.value = false
          }
        } catch (e) {
          const errStr = String(e)
          // Only stop polling on real errors, not pending states
          if (!errStr.includes('pending') && !errStr.includes('slow_down')) {
            clearPolling()
            loading.value = false
            userCode.value = ''
            error.value = errStr
          }
        }
      }, pollIntervalMs)
    } catch (e) {
      loading.value = false
      error.value = String(e)
    }
  }

  async function logout() {
    clearPolling()
    token.value = null
    user.value = null
    tokenSecured.value = false
    const store = await load('settings.json')
    await store.delete('token')
    await store.delete('encryptedToken')
    await store.save()
  }

  async function setRepo(r: string) {
    repo.value = r
    const store = await load('settings.json')
    await store.set('repo', r)
    await store.save()
  }

  return { 
    token, 
    user, 
    repo, 
    loading, 
    userCode, 
    error, 
    tokenSecured,
    init, 
    startLogin, 
    logout, 
    setRepo 
  }
}
