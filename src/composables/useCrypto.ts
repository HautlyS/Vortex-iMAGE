/**
 * Crypto Composable - Security Hardened v4
 * 
 * Uses opaque keypair handles instead of raw bytes for improved security.
 * Implements session timeout to auto-lock after inactivity.
 * 
 * Security Features:
 * - Keypair bytes never exposed to frontend
 * - Automatic session timeout (default 15 minutes)
 * - Activity tracking for timeout reset
 */

import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'

// ============================================================================
// Types
// ============================================================================

export interface PublicBundle {
  pq_encap: number[]
  x25519: number[]
  pq_verify: number[]
  ed_verify: number[]
  created_at?: number
  key_id?: string
}

/**
 * Keypair info returned from backend - contains handle, NOT bytes
 */
export interface KeypairInfo {
  handle: number
  public_bundle: PublicBundle
  created_at: number
  key_id: string
}

export interface SessionKeysResult {
  encryption_key: number[]
  hmac_key: number[]
  iv: number[]
}

export interface CryptoInfo {
  key_exchange: string
  signatures: string
  symmetric: string
  kdf: string
  hash: string
  pq_security_level: string
  classical_security_level: string
  token_version: string
  keychain_status: string
  features: string[]
  backend: string
  pinned_versions: {
    pqc_kyber: string
    pqc_dilithium: string
  }
}

export type EncryptionMethod = 'None' | 'Password' | 'HybridPQ'

export interface EncryptionSettings {
  enabled: boolean
  use_password: boolean
  use_keypair: boolean
  recipient_bundle: PublicBundle | null
}

export interface EncryptedFileData {
  data: number[]
  encrypted: boolean
  method: EncryptionMethod
  metadata: Record<string, unknown> | null
}

export interface EncryptedPayload {
  nonce: number[]
  ciphertext: number[]
  encap: {
    pq_ciphertext: number[]
    x25519_ephemeral: number[]
  }
  aad_hash?: number[]
}

// ============================================================================
// State
// ============================================================================

// Opaque handle to keypair in backend - NOT the actual bytes
const keypairHandle = ref<number | null>(null)
const publicBundle = ref<PublicBundle | null>(null)
const isUnlocked = ref(false)
const cryptoInfo = ref<CryptoInfo | null>(null)
const lastActivity = ref<number>(Date.now())

// Session timeout configuration
const SESSION_TIMEOUT_MS = 15 * 60 * 1000 // 15 minutes default
let timeoutCheckInterval: ReturnType<typeof setInterval> | null = null
let initialized = false

// ============================================================================
// Composable
// ============================================================================

export function useCrypto() {
  const hasKeypair = computed(() => keypairHandle.value !== null)
  const hasStoredKeypair = ref(false)

  /**
   * Reset activity timer - called on crypto operations
   */
  function resetActivityTimer(): void {
    lastActivity.value = Date.now()
  }

  /**
   * Check if session has timed out
   */
  function checkSessionTimeout(): void {
    if (!isUnlocked.value) return
    
    const elapsed = Date.now() - lastActivity.value
    if (elapsed >= SESSION_TIMEOUT_MS) {
      console.log('Session timeout - auto-locking keypair')
      lockKeypair()
    }
  }

  /**
   * Start session timeout checker
   */
  function startTimeoutChecker(): void {
    if (timeoutCheckInterval) return
    timeoutCheckInterval = setInterval(checkSessionTimeout, 60000) // Check every minute
  }

  /**
   * Stop session timeout checker
   */
  function stopTimeoutChecker(): void {
    if (timeoutCheckInterval) {
      clearInterval(timeoutCheckInterval)
      timeoutCheckInterval = null
    }
  }

  /**
   * Initialize crypto module
   */
  async function initialize(): Promise<void> {
    if (initialized) return
    try {
      const store = await load('settings.json')
      const storedHandle = await store.get<number>('keypairHandle')
      hasStoredKeypair.value = storedHandle !== null && storedHandle !== undefined
      
      cryptoInfo.value = await invoke<CryptoInfo>('get_crypto_info')
      initialized = true
      
      // Start timeout checker
      startTimeoutChecker()
    } catch (e) {
      console.error('Failed to initialize crypto:', e)
    }
  }

  /**
   * Generate a new keypair - returns handle and public bundle, NOT bytes
   */
  async function generateKeypair(): Promise<KeypairInfo> {
    resetActivityTimer()
    
    const result = await invoke<KeypairInfo>('generate_keypair')
    keypairHandle.value = result.handle
    publicBundle.value = result.public_bundle
    isUnlocked.value = true
    
    return result
  }

  /**
   * Save keypair handle to persistent storage
   * Note: The actual keypair is encrypted and stored by the backend
   * @param _password - Reserved for future password-based encryption
   */
  async function saveKeypair(_password: string): Promise<void> {
    if (keypairHandle.value === null) throw new Error('No keypair to save')
    resetActivityTimer()
    
    // Store the encrypted keypair using secure token storage
    await invoke('secure_store_token', {
      key: 'keypair_handle',
      value: keypairHandle.value.toString()
    })
    
    const store = await load('settings.json')
    await store.set('keypairHandle', keypairHandle.value)
    await store.set('publicBundle', publicBundle.value)
    await store.save()
    hasStoredKeypair.value = true
  }

  /**
   * Unlock a stored keypair
   * @param _password - Reserved for future password-based decryption
   */
  async function unlockKeypair(_password: string): Promise<boolean> {
    if (!hasStoredKeypair.value) return false
    resetActivityTimer()
    
    try {
      // Retrieve the stored handle
      const storedValue = await invoke<string>('secure_retrieve_token', {
        key: 'keypair_handle'
      })
      
      const handle = parseInt(storedValue, 10)
      if (isNaN(handle)) return false
      
      // Validate that the handle is still valid in the backend
      const isValid = await invoke<boolean>('validate_keypair_handle', { handle })
      if (!isValid) {
        // Handle is stale - clear stored data and return false
        console.warn('Stored keypair handle is no longer valid in backend')
        hasStoredKeypair.value = false
        const store = await load('settings.json')
        await store.delete('keypairHandle')
        await store.delete('publicBundle')
        await store.save()
        return false
      }
      
      keypairHandle.value = handle
      
      // Get the public bundle for this handle
      const store = await load('settings.json')
      const savedBundle = await store.get<PublicBundle>('publicBundle')
      if (savedBundle) {
        publicBundle.value = savedBundle
      }
      
      isUnlocked.value = true
      return true
    } catch {
      return false
    }
  }

  /**
   * Lock the keypair (clear from memory)
   */
  function lockKeypair(): void {
    // Release the keypair from backend memory
    if (keypairHandle.value !== null) {
      invoke('release_keypair', { handle: keypairHandle.value }).catch(console.error)
    }
    
    keypairHandle.value = null
    publicBundle.value = null
    isUnlocked.value = false
  }

  /**
   * Rotate the keypair (generate new, keep old for decryption)
   */
  async function rotateKeypair(): Promise<PublicBundle> {
    if (keypairHandle.value === null) throw new Error('No keypair to rotate')
    resetActivityTimer()
    
    const newBundle = await invoke<PublicBundle>('rotate_keypair', {
      handle: keypairHandle.value
    })
    
    publicBundle.value = newBundle
    
    // Save the new public bundle
    const store = await load('settings.json')
    await store.set('publicBundle', newBundle)
    await store.save()
    
    return newBundle
  }

  /**
   * Encrypt data for a recipient
   */
  async function encryptForRecipient(
    data: Uint8Array,
    recipientBundle: PublicBundle,
    aad?: Uint8Array
  ): Promise<EncryptedPayload> {
    resetActivityTimer()
    
    return await invoke<EncryptedPayload>('encrypt_hybrid', {
      data: Array.from(data),
      recipientBundle,
      aad: aad ? Array.from(aad) : null
    })
  }

  /**
   * Decrypt data using the current keypair handle
   */
  async function decryptFromSender(
    encryptedData: EncryptedPayload,
    aad?: Uint8Array
  ): Promise<Uint8Array> {
    if (keypairHandle.value === null) throw new Error('Keypair not unlocked')
    resetActivityTimer()
    
    const result = await invoke<number[]>('decrypt_hybrid', {
      encryptedData,
      handle: keypairHandle.value,
      aad: aad ? Array.from(aad) : null
    })
    return new Uint8Array(result)
  }

  /**
   * Sign data using the current keypair handle
   */
  async function signData(data: Uint8Array): Promise<Uint8Array> {
    if (keypairHandle.value === null) throw new Error('Keypair not unlocked')
    resetActivityTimer()
    
    const result = await invoke<number[]>('sign_data', {
      data: Array.from(data),
      handle: keypairHandle.value
    })
    return new Uint8Array(result)
  }

  /**
   * Verify a signature using a public bundle
   */
  async function verifySignature(
    data: Uint8Array,
    signature: Uint8Array,
    bundle: PublicBundle
  ): Promise<boolean> {
    resetActivityTimer()
    
    return await invoke<boolean>('verify_signature', {
      data: Array.from(data),
      signature: Array.from(signature),
      publicBundle: bundle
    })
  }

  /**
   * Encrypt data with a password
   */
  async function encryptWithPassword(data: Uint8Array, password: string): Promise<Uint8Array> {
    resetActivityTimer()
    
    const result = await invoke<number[]>('encrypt_data_password', {
      data: Array.from(data),
      password
    })
    return new Uint8Array(result)
  }

  /**
   * Decrypt data with a password
   */
  async function decryptWithPassword(data: Uint8Array, password: string): Promise<Uint8Array> {
    resetActivityTimer()
    
    const result = await invoke<number[]>('decrypt_data_password', {
      data: Array.from(data),
      password
    })
    return new Uint8Array(result)
  }

  /**
   * Hash data using BLAKE3
   */
  async function hashData(data: Uint8Array): Promise<Uint8Array> {
    const result = await invoke<number[]>('hash_data_blake3', {
      data: Array.from(data)
    })
    return new Uint8Array(result)
  }

  /**
   * Encrypt a file with settings
   */
  async function encryptFile(
    data: Uint8Array,
    settings: EncryptionSettings,
    password?: string
  ): Promise<EncryptedFileData> {
    resetActivityTimer()
    
    return await invoke<EncryptedFileData>('encrypt_file', {
      data: Array.from(data),
      settings,
      password: password || null,
      handle: keypairHandle.value
    })
  }

  /**
   * Decrypt a file
   */
  async function decryptFile(
    encrypted: EncryptedFileData,
    password?: string
  ): Promise<Uint8Array> {
    resetActivityTimer()
    
    const result = await invoke<number[]>('decrypt_file', {
      encrypted,
      password: password || null,
      handle: keypairHandle.value
    })
    return new Uint8Array(result)
  }

  /**
   * Create encryption settings helper
   */
  function createEncryptionSettings(options: {
    enabled?: boolean
    usePassword?: boolean
    useKeypair?: boolean
    recipientBundle?: PublicBundle | null
  } = {}): EncryptionSettings {
    return {
      enabled: options.enabled ?? false,
      use_password: options.usePassword ?? false,
      use_keypair: options.useKeypair ?? false,
      recipient_bundle: options.recipientBundle ?? null
    }
  }

  /**
   * Delete the stored keypair completely
   * Removes from backend memory, secure storage, and local settings
   */
  async function deleteKeypair(): Promise<void> {
    // Release from backend memory
    if (keypairHandle.value !== null) {
      await invoke('release_keypair', { handle: keypairHandle.value }).catch(console.error)
    }
    
    // Delete from secure storage (keychain or encrypted file)
    try {
      await invoke('secure_delete_token', { key: 'keypair_handle' })
    } catch (e) {
      // Ignore if not found
      console.debug('No secure token to delete:', e)
    }
    
    keypairHandle.value = null
    publicBundle.value = null
    isUnlocked.value = false
    hasStoredKeypair.value = false
    
    // Clear from local settings
    const store = await load('settings.json')
    await store.delete('keypairHandle')
    await store.delete('publicBundle')
    await store.save()
  }

  /**
   * Get session timeout remaining (in ms)
   */
  function getSessionTimeoutRemaining(): number {
    if (!isUnlocked.value) return 0
    const elapsed = Date.now() - lastActivity.value
    return Math.max(0, SESSION_TIMEOUT_MS - elapsed)
  }

  // Cleanup on unmount
  onUnmounted(() => {
    stopTimeoutChecker()
  })

  return {
    // State
    keypairHandle,
    hasKeypair,
    hasStoredKeypair,
    isUnlocked,
    publicBundle,
    cryptoInfo,
    lastActivity,
    
    // Lifecycle
    initialize,
    
    // Keypair operations
    generateKeypair,
    saveKeypair,
    unlockKeypair,
    lockKeypair,
    rotateKeypair,
    deleteKeypair,
    
    // Encryption/Decryption
    encryptForRecipient,
    decryptFromSender,
    encryptWithPassword,
    decryptWithPassword,
    encryptFile,
    decryptFile,
    
    // Signing
    signData,
    verifySignature,
    
    // Utilities
    hashData,
    createEncryptionSettings,
    
    // Session management
    resetActivityTimer,
    getSessionTimeoutRemaining
  }
}
