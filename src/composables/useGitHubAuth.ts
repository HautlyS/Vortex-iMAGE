/**
 * GitHub Auth Composable - Multi-platform authentication
 * Supports: Device Flow (Tauri), Manual Token (Web/Tauri)
 */

import { ref, computed, onUnmounted } from 'vue'

// Platform detection
const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI__
export const isDevMode = import.meta.env.DEV && import.meta.env.VITE_MOCK_AUTH === 'true'
export const isWebMode = !isTauri && !isDevMode

// Mock data for dev mode
const MOCK_USER: GitHubUser = {
  login: 'dev-user',
  avatar_url: 'https://api.dicebear.com/7.x/shapes/svg?seed=dev'
}

const MOCK_REPO = 'dev-user/photos'

const MOCK_PUBLIC_BUNDLE: PublicBundle = {
  pq_encap: Array(1184).fill(0),
  x25519: Array(32).fill(0),
  pq_verify: Array(1312).fill(0),
  ed_verify: Array(32).fill(0)
}

// Mock albums for dev mode
export interface MockAlbum {
  name: string
  path: string
  photo_count: number
  children: MockAlbum[]
}

const mockAlbums: MockAlbum[] = [
  { name: 'Vacation 2024', path: 'photos/vacation-2024', photo_count: 24, children: [
    { name: 'Beach', path: 'photos/vacation-2024/beach', photo_count: 12, children: [] },
    { name: 'Mountains', path: 'photos/vacation-2024/mountains', photo_count: 8, children: [] }
  ]},
  { name: 'Family', path: 'photos/family', photo_count: 45, children: [] },
  { name: 'Work', path: 'photos/work', photo_count: 15, children: [] }
]

// Mock photos for dev mode
export interface MockPhoto {
  name: string
  sha: string
  url: string
  path: string
  size: number
}

const generateMockPhotos = (count: number): MockPhoto[] => {
  return Array.from({ length: count }, (_, i) => ({
    name: `photo_${i + 1}.jpg`,
    sha: `mock-sha-${i + 1}-${Date.now()}`,
    url: `https://picsum.photos/seed/${i + 1}/800/600`,
    path: `photos/photo_${i + 1}.jpg`,
    size: Math.floor(Math.random() * 5000000) + 500000
  }))
}

let mockPhotosCache: MockPhoto[] | null = null

export function getMockPhotos(): MockPhoto[] {
  if (!mockPhotosCache) {
    mockPhotosCache = generateMockPhotos(24)
  }
  return mockPhotosCache
}

export function getMockAlbums(): MockAlbum[] {
  return mockAlbums
}

export function addMockAlbum(name: string, parentPath?: string): MockAlbum {
  const path = parentPath ? `${parentPath}/${name.toLowerCase().replace(/\s+/g, '-')}` : `photos/${name.toLowerCase().replace(/\s+/g, '-')}`
  const newAlbum: MockAlbum = { name, path, photo_count: 0, children: [] }
  
  if (parentPath) {
    const parent = findAlbumByPath(mockAlbums, parentPath)
    if (parent) parent.children.push(newAlbum)
  } else {
    mockAlbums.push(newAlbum)
  }
  
  return newAlbum
}

function findAlbumByPath(albums: MockAlbum[], path: string): MockAlbum | null {
  for (const album of albums) {
    if (album.path === path) return album
    const found = findAlbumByPath(album.children, path)
    if (found) return found
  }
  return null
}

export function addMockPhoto(photo: MockPhoto): void {
  if (!mockPhotosCache) mockPhotosCache = []
  mockPhotosCache.unshift(photo)
}

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

interface TokenValidation {
  valid: boolean
  user: GitHubUser | null
  scopes: string[]
  error: string | null
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

// Shared state
const token = ref<string | null>(isDevMode ? 'mock-token-dev' : null)
const user = ref<GitHubUser | null>(isDevMode ? MOCK_USER : null)
const repo = ref<string>(isDevMode ? MOCK_REPO : '')
const publicBundle = ref<PublicBundle | null>(isDevMode ? MOCK_PUBLIC_BUNDLE : null)
const keypairBytes = ref<number[] | null>(isDevMode ? Array(64).fill(0) : null)

// Web storage key
const WEB_TOKEN_KEY = 'vortex_gh_token'
const WEB_USER_KEY = 'vortex_gh_user'
const WEB_REPO_KEY = 'vortex_gh_repo'

export function useGitHubAuth() {
  const loading = ref(false)
  const userCode = ref('')
  const error = ref<string | null>(null)
  const validating = ref(false)

  let pollInterval: ReturnType<typeof setInterval> | null = null
  let pollTimeout: ReturnType<typeof setTimeout> | null = null

  function clearPolling() {
    if (pollInterval) { clearInterval(pollInterval); pollInterval = null }
    if (pollTimeout) { clearTimeout(pollTimeout); pollTimeout = null }
  }

  onUnmounted(clearPolling)

  // Check if device flow is available (Tauri only)
  const canUseDeviceFlow = computed(() => isTauri && !isDevMode)

  async function init() {
    if (isDevMode) return

    try {
      if (isTauri) {
        // Tauri: use plugin-store
        const { invoke } = await import('@tauri-apps/api/core')
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('settings.json')
        
        token.value = await store.get<string>('token') || null
        repo.value = await store.get<string>('repo') || ''

        const storedKeypair = await store.get<number[]>('keypair_bytes')
        const storedBundle = await store.get<PublicBundle>('public_bundle')

        if (storedKeypair && storedBundle) {
          keypairBytes.value = storedKeypair
          publicBundle.value = storedBundle
        } else if (token.value) {
          await rotateKeys()
        }

        if (token.value) {
          user.value = await invoke<GitHubUser>('get_user', { token: token.value })
        }
      } else {
        // Web: use localStorage
        const storedToken = localStorage.getItem(WEB_TOKEN_KEY)
        const storedUser = localStorage.getItem(WEB_USER_KEY)
        const storedRepo = localStorage.getItem(WEB_REPO_KEY)

        if (storedToken) {
          token.value = storedToken
          repo.value = storedRepo || ''
          if (storedUser) {
            user.value = JSON.parse(storedUser)
          }
          // Validate token on init
          const validation = await validateTokenWeb(storedToken)
          if (!validation.valid) {
            await logout()
            error.value = validation.error || 'Token expired'
          } else if (validation.user) {
            user.value = validation.user
          }
        }
      }
    } catch {
      token.value = null
      user.value = null
      if (isTauri) {
        try {
          const { load } = await import('@tauri-apps/plugin-store')
          const store = await load('settings.json')
          await store.delete('token')
          await store.save()
        } catch {}
      } else {
        localStorage.removeItem(WEB_TOKEN_KEY)
        localStorage.removeItem(WEB_USER_KEY)
      }
    }
  }

  async function rotateKeys() {
    if (isDevMode || !isTauri) return
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

  // Validate token via GitHub API (web-compatible)
  async function validateTokenWeb(tokenToValidate: string): Promise<TokenValidation> {
    try {
      const res = await fetch('https://api.github.com/user', {
        headers: {
          'Authorization': `Bearer ${tokenToValidate}`,
          'User-Agent': 'vortex-image'
        }
      })

      if (!res.ok) {
        return { valid: false, user: null, scopes: [], error: 'Invalid or expired token' }
      }

      const scopes = res.headers.get('x-oauth-scopes')?.split(', ') || []
      const userData: GitHubUser = await res.json()

      const hasRepo = scopes.some(s => s === 'repo' || s === 'public_repo')
      if (!hasRepo) {
        return {
          valid: false,
          user: userData,
          scopes,
          error: "Token missing 'repo' scope. Generate a new token with repo access."
        }
      }

      return { valid: true, user: userData, scopes, error: null }
    } catch (e) {
      return { valid: false, user: null, scopes: [], error: String(e) }
    }
  }

  // Validate token (uses Tauri command if available, otherwise web fetch)
  async function validateToken(tokenToValidate: string): Promise<TokenValidation> {
    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        return await invoke<TokenValidation>('validate_token', { token: tokenToValidate })
      } catch {
        return validateTokenWeb(tokenToValidate)
      }
    }
    return validateTokenWeb(tokenToValidate)
  }

  // Manual token login (works on both Tauri and Web)
  async function loginWithToken(manualToken: string): Promise<boolean> {
    if (!manualToken.trim()) {
      error.value = 'Please enter a token'
      return false
    }

    validating.value = true
    error.value = null

    try {
      const validation = await validateToken(manualToken.trim())

      if (!validation.valid) {
        error.value = validation.error || 'Invalid token'
        validating.value = false
        return false
      }

      token.value = manualToken.trim()
      user.value = validation.user

      if (isTauri) {
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('settings.json')
        await store.set('token', token.value)
        await store.save()

        if (!keypairBytes.value) {
          await rotateKeys()
        }
      } else {
        localStorage.setItem(WEB_TOKEN_KEY, token.value)
        if (user.value) {
          localStorage.setItem(WEB_USER_KEY, JSON.stringify(user.value))
        }
      }

      validating.value = false
      return true
    } catch (e) {
      error.value = String(e)
      validating.value = false
      return false
    }
  }

  // Device Flow login (Tauri only)
  async function startLogin() {
    if (isDevMode) {
      loading.value = true
      await new Promise(r => setTimeout(r, 800))
      token.value = 'mock-token-dev'
      user.value = MOCK_USER
      repo.value = MOCK_REPO
      publicBundle.value = MOCK_PUBLIC_BUNDLE
      keypairBytes.value = Array(64).fill(0)
      loading.value = false
      return
    }

    if (!isTauri) {
      error.value = 'Device Flow not available in web mode. Use manual token.'
      return
    }

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

            if (!keypairBytes.value) {
              await rotateKeys()
            }

            const store = await load('settings.json')
            await store.set('token', t)
            await store.save()

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

    if (isTauri) {
      try {
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('settings.json')
        await store.delete('token')
        await store.save()
      } catch {}
    } else {
      localStorage.removeItem(WEB_TOKEN_KEY)
      localStorage.removeItem(WEB_USER_KEY)
    }
  }

  async function setRepo(r: string) {
    repo.value = r

    if (isDevMode) return

    if (isTauri) {
      try {
        const { load } = await import('@tauri-apps/plugin-store')
        const store = await load('settings.json')
        await store.set('repo', r)
        await store.save()
      } catch {}
    } else {
      localStorage.setItem(WEB_REPO_KEY, r)
    }
  }

  return {
    // State
    token,
    user,
    repo,
    publicBundle,
    keypairBytes,
    loading,
    userCode,
    error,
    validating,
    // Computed
    canUseDeviceFlow,
    isWebMode: computed(() => isWebMode),
    // Methods
    init,
    startLogin,
    loginWithToken,
    validateToken,
    logout,
    setRepo,
    // Keypair sync
    checkKeypairSync,
    uploadKeypairSync,
    downloadKeypairSync,
    exportKeypairBackup,
    importKeypairBackup
  }
}

// ============================================================================
// Keypair Sync Functions (Cross-Device Support)
// ============================================================================

interface KeypairSyncInfo {
  exists: boolean
  sha: string | null
  updated_at: string | null
}

async function checkKeypairSync(): Promise<KeypairSyncInfo | null> {
  if (isDevMode || !isTauri || !token.value || !repo.value) return null
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<KeypairSyncInfo>('check_keypair_sync', {
      repo: repo.value,
      token: token.value
    })
  } catch {
    return null
  }
}

async function uploadKeypairSync(password: string): Promise<boolean> {
  if (isDevMode || !isTauri || !keypairBytes.value || !token.value || !repo.value) return false
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    
    // Encrypt keypair with password
    const encrypted = await invoke<number[]>('encrypt_data_password', {
      data: keypairBytes.value,
      password
    })
    
    // Upload to repo
    await invoke('upload_keypair_sync', {
      encryptedKeypair: encrypted,
      repo: repo.value,
      token: token.value
    })
    
    return true
  } catch (e) {
    console.error('Failed to upload keypair sync:', e)
    return false
  }
}

async function downloadKeypairSync(password: string): Promise<boolean> {
  if (isDevMode || !isTauri || !token.value || !repo.value) return false
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const { load } = await import('@tauri-apps/plugin-store')
    
    // Download encrypted keypair
    const encrypted = await invoke<number[]>('download_keypair_sync', {
      repo: repo.value,
      token: token.value
    })
    
    // Decrypt with password
    const decrypted = await invoke<number[]>('decrypt_data_password', {
      data: encrypted,
      password
    })
    
    // Validate and extract public bundle
    const result = await invoke<{ public_bundle: PublicBundle }>('validate_keypair_handle', {
      keypairBytes: decrypted
    })
    
    // Store locally
    keypairBytes.value = decrypted
    publicBundle.value = result.public_bundle
    
    const store = await load('settings.json')
    await store.set('keypair_bytes', decrypted)
    await store.set('public_bundle', result.public_bundle)
    await store.save()
    
    return true
  } catch (e) {
    console.error('Failed to download keypair sync:', e)
    return false
  }
}

// Export keypair as encrypted backup (for manual backup)
async function exportKeypairBackup(password: string): Promise<string | null> {
  if (isDevMode || !isTauri || !keypairBytes.value) return null
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    
    const encrypted = await invoke<number[]>('encrypt_data_password', {
      data: keypairBytes.value,
      password
    })
    
    // Return as base64 for easy copy/paste
    const bytes = new Uint8Array(encrypted)
    return btoa(String.fromCharCode(...bytes))
  } catch {
    return null
  }
}

// Import keypair from encrypted backup
async function importKeypairBackup(backup: string, password: string): Promise<boolean> {
  if (isDevMode || !isTauri) return false
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const { load } = await import('@tauri-apps/plugin-store')
    
    // Decode base64
    const bytes = Uint8Array.from(atob(backup), c => c.charCodeAt(0))
    
    // Decrypt
    const decrypted = await invoke<number[]>('decrypt_data_password', {
      data: Array.from(bytes),
      password
    })
    
    // Validate
    const result = await invoke<{ public_bundle: PublicBundle }>('validate_keypair_handle', {
      keypairBytes: decrypted
    })
    
    // Store
    keypairBytes.value = decrypted
    publicBundle.value = result.public_bundle
    
    const store = await load('settings.json')
    await store.set('keypair_bytes', decrypted)
    await store.set('public_bundle', result.public_bundle)
    await store.save()
    
    return true
  } catch {
    return false
  }
}
