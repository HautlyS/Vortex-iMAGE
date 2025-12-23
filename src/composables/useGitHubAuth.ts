import { ref, onUnmounted } from 'vue'

// Check if running in dev mock mode OR in browser without Tauri
const isTauriAvailable = typeof window !== 'undefined' && !!(window as any).__TAURI__
export const isDevMode = import.meta.env.DEV && (import.meta.env.VITE_MOCK_AUTH === 'true' || !isTauriAvailable)

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

export interface PublicBundle {
  pq_encap: number[]
  x25519: number[]
  pq_verify: number[]
  ed_verify: number[]
}

interface KeypairResult {
  public_bundle: PublicBundle
  keypair_bytes: number[]
}

const token = ref<string | null>(isDevMode ? 'mock-token-dev' : null)
const user = ref<GitHubUser | null>(isDevMode ? { login: 'dev-user', avatar_url: 'https://avatars.githubusercontent.com/u/0?v=4' } : null)
const repo = ref<string>(isDevMode ? 'dev-user/photos' : '')
const publicBundle = ref<PublicBundle | null>(null)
const keypairBytes = ref<number[] | null>(null)

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

      // Load keys
      const storedKeypair = await store.get<number[]>('keypair_bytes')
      const storedBundle = await store.get<PublicBundle>('public_bundle')

      if (storedKeypair && storedBundle) {
        keypairBytes.value = storedKeypair
        publicBundle.value = storedBundle
      } else if (token.value) {
        // If logged in but no keys, generate them
        await rotateKeys()
      }

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
      } catch { }
    }
  }

  async function rotateKeys() {
    if (isDevMode) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const { load } = await import('@tauri-apps/plugin-store')

      const res = await invoke<KeypairResult>('generate_keypair')
      keypairBytes.value = res.keypair_bytes
      publicBundle.value = res.public_bundle

      const store = await load('settings.json')
      await store.set('keypair_bytes', res.keypair_bytes)
      await store.set('public_bundle', res.public_bundle)
      await store.save()
    } catch (e) {
      console.error('Failed to rotate keys:', e)
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
      publicBundle.value = {
        pq_encap: Array(1184).fill(0),
        x25519: Array(32).fill(0),
        pq_verify: Array(1312).fill(0),
        ed_verify: Array(32).fill(0)
      }
      loading.value = false
      return
    }

    // Check loading BEFORE clearing to prevent race condition
    if (loading.value) return
    loading.value = true
    clearPolling()
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

            // Generate keys on new login
            if (!keypairBytes.value) {
              await rotateKeys()
            }

            try {
              const store = await load('settings.json')
              await store.set('token', t)
              await store.save()
            } catch { }

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
    // We don't necessarily clear keys on logout to allow decryption of existing local checks
    // but strict security might require it. For now, keep them to avoid data loss feeling.

    if (isDevMode) return

    try {
      const { load } = await import('@tauri-apps/plugin-store')
      const store = await load('settings.json')
      await store.delete('token')
      await store.save()
    } catch { }
  }

  async function setRepo(r: string) {
    repo.value = r

    if (isDevMode) return

    try {
      const { load } = await import('@tauri-apps/plugin-store')
      const store = await load('settings.json')
      await store.set('repo', r)
      await store.save()
    } catch { }
  }

  return {
    token,
    user,
    repo,
    publicBundle,
    keypairBytes,
    loading,
    userCode,
    error,
    init,
    startLogin,
    logout,
    setRepo
  }
}
