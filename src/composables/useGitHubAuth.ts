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

export function useGitHubAuth() {
  const loading = ref(false)
  const userCode = ref('')
  const error = ref<string | null>(null)
  
  // Instance-specific polling variables to prevent conflicts
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

  onUnmounted(clearPolling)

  async function init() {
    try {
      const store = await load('settings.json')
      token.value = await store.get<string>('token') || null
      repo.value = await store.get<string>('repo') || ''
      if (token.value) {
        user.value = await invoke<GitHubUser>('get_user', { token: token.value })
      }
    } catch {
      token.value = null
      user.value = null
      try {
        const store = await load('settings.json')
        await store.delete('token')
        await store.save()
      } catch {
        // Silent fail on cleanup
      }
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

      pollInterval = setInterval(async () => {
        try {
          const t = await invoke<string | null>('poll_oauth', { deviceCode: res.device_code })
          if (t) {
            clearPolling()
            token.value = t
            user.value = await invoke<GitHubUser>('get_user', { token: t })
            
            try {
              const store = await load('settings.json')
              await store.set('token', t)
              await store.save()
            } catch {
              // Continue even if save fails
            }
            
            loading.value = false
            userCode.value = ''
          }
        } catch (e) {
          clearPolling()
          loading.value = false
          userCode.value = ''
          error.value = String(e)
        }
      }, res.interval * 1000)
    } catch (e) {
      loading.value = false
      error.value = String(e)
    }
  }

  async function logout() {
    clearPolling()
    token.value = null
    user.value = null
    
    try {
      const store = await load('settings.json')
      await store.delete('token')
      await store.save()
    } catch {
      // Silent fail
    }
  }

  async function setRepo(r: string) {
    repo.value = r
    try {
      const store = await load('settings.json')
      await store.set('repo', r)
      await store.save()
    } catch {
      // Silent fail
    }
  }

  return {
    token,
    user,
    repo,
    loading,
    userCode,
    error,
    init,
    startLogin,
    logout,
    setRepo
  }
}
