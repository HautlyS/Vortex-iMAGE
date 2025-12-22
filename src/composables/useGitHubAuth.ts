import { ref, onUnmounted } from 'vue'

// Check if running in dev mock mode
export const isDevMode = import.meta.env.DEV && import.meta.env.VITE_MOCK_AUTH === 'true'

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

const token = ref<string | null>(isDevMode ? 'mock-token-dev' : null)
const user = ref<GitHubUser | null>(isDevMode ? { login: 'dev-user', avatar_url: 'https://avatars.githubusercontent.com/u/0?v=4' } : null)
const repo = ref<string>(isDevMode ? 'dev-user/photos' : '')

export function useGitHubAuth() {
  const loading = ref(false)
  const userCode = ref('')
  const error = ref<string | null>(null)
  
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
    // In dev mode, already initialized with mock data
    if (isDevMode) return

    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const { load } = await import('@tauri-apps/plugin-store')
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
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('settings.json')
        await store.delete('token')
        await store.save()
      } catch {}
    }
  }

  async function startLogin() {
    if (isDevMode) {
      // Mock instant login
      loading.value = true
      await new Promise(r => setTimeout(r, 500))
      token.value = 'mock-token-dev'
      user.value = { login: 'dev-user', avatar_url: 'https://avatars.githubusercontent.com/u/0?v=4' }
      repo.value = 'dev-user/photos'
      loading.value = false
      return
    }

    if (loading.value) return
    clearPolling()
    loading.value = true
    error.value = null
    
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const { open } = await import('@tauri-apps/plugin-shell')
      const { load } = await import('@tauri-apps/plugin-store')
      
      const res = await invoke<DeviceCodeResponse>('start_oauth')
      userCode.value = res.user_code
      await open(res.verification_uri)
      
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
            } catch {}
            
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
    
    if (isDevMode) return

    try {
      const { load } = await import('@tauri-apps/plugin-store')
      const store = await load('settings.json')
      await store.delete('token')
      await store.save()
    } catch {}
  }

  async function setRepo(r: string) {
    repo.value = r
    
    if (isDevMode) return

    try {
      const { load } = await import('@tauri-apps/plugin-store')
      const store = await load('settings.json')
      await store.set('repo', r)
      await store.save()
    } catch {}
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
