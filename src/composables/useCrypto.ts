/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 2 modules
 */

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'

export interface PublicBundle {
  pq: number[]
  x25519: number[]
  signing: number[]
}

export interface KeypairResult {
  public_bundle: PublicBundle
  keypair_bytes: number[]
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
  features: string[]
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

const keypair = ref<KeypairResult | null>(null)
const encryptedKeypair = ref<number[] | null>(null)
const isUnlocked = ref(false)
const cryptoInfo = ref<CryptoInfo | null>(null)
let initialized = false

export function useCrypto() {
  const hasKeypair = computed(() => keypair.value !== null)
  const hasStoredKeypair = computed(() => encryptedKeypair.value !== null)
  const publicBundle = computed(() => keypair.value?.public_bundle || null)

  async function initialize(): Promise<void> {
    if (initialized) return
    try {
      const store = await load('settings.json')
      const stored = await store.get<number[]>('encryptedKeypair')
      if (stored) {
        encryptedKeypair.value = stored
      }
      cryptoInfo.value = await invoke<CryptoInfo>('get_crypto_info')
      initialized = true
    } catch (e) {
      console.error('Failed to initialize crypto:', e)
    }
  }

  async function generateKeypair(): Promise<KeypairResult> {
    const result = await invoke<KeypairResult>('generate_keypair')
    keypair.value = result
    isUnlocked.value = true
    return result
  }

  async function saveKeypair(password: string): Promise<void> {
    if (!keypair.value) throw new Error('No keypair to save')
    
    const encrypted = await invoke<number[]>('encrypt_keypair', {
      keypairBytes: keypair.value.keypair_bytes,
      password
    })
    
    encryptedKeypair.value = encrypted
    const store = await load('settings.json')
    await store.set('encryptedKeypair', encrypted)
    await store.save()
  }

  async function unlockKeypair(password: string): Promise<boolean> {
    if (!encryptedKeypair.value) return false
    
    try {
      const result = await invoke<KeypairResult>('decrypt_keypair', {
        encryptedBytes: encryptedKeypair.value,
        password
      })
      keypair.value = result
      isUnlocked.value = true
      return true
    } catch {
      return false
    }
  }

  function lockKeypair(): void {
    keypair.value = null
    isUnlocked.value = false
  }

  async function encryptForRecipient(data: Uint8Array, recipientBundle: PublicBundle): Promise<Uint8Array> {
    const result = await invoke<number[]>('encrypt_hybrid', {
      data: Array.from(data),
      recipientBundle
    })
    return new Uint8Array(result)
  }

  async function decryptFromSender(encryptedData: Uint8Array): Promise<Uint8Array> {
    if (!keypair.value) throw new Error('Keypair not unlocked')
    
    const result = await invoke<number[]>('decrypt_hybrid', {
      encryptedData: Array.from(encryptedData),
      keypairBytes: keypair.value.keypair_bytes
    })
    return new Uint8Array(result)
  }

  async function signData(data: Uint8Array): Promise<Uint8Array> {
    if (!keypair.value) throw new Error('Keypair not unlocked')
    
    const result = await invoke<number[]>('sign_data', {
      data: Array.from(data),
      keypairBytes: keypair.value.keypair_bytes
    })
    return new Uint8Array(result)
  }

  async function verifySignature(data: Uint8Array, signature: Uint8Array, bundle: PublicBundle): Promise<boolean> {
    return await invoke<boolean>('verify_signature', {
      data: Array.from(data),
      signature: Array.from(signature),
      publicBundle: bundle
    })
  }

  async function encryptWithPassword(data: Uint8Array, password: string): Promise<Uint8Array> {
    const result = await invoke<number[]>('encrypt_data_password', {
      data: Array.from(data),
      password
    })
    return new Uint8Array(result)
  }

  async function decryptWithPassword(data: Uint8Array, password: string): Promise<Uint8Array> {
    const result = await invoke<number[]>('decrypt_data_password', {
      data: Array.from(data),
      password
    })
    return new Uint8Array(result)
  }

  async function hashData(data: Uint8Array): Promise<Uint8Array> {
    const result = await invoke<number[]>('hash_data_blake3', {
      data: Array.from(data)
    })
    return new Uint8Array(result)
  }

  async function encryptFile(
    data: Uint8Array,
    settings: EncryptionSettings,
    password?: string
  ): Promise<EncryptedFileData> {
    return await invoke<EncryptedFileData>('encrypt_file', {
      data: Array.from(data),
      settings,
      password: password || null,
      keypairBytes: keypair.value?.keypair_bytes || null
    })
  }

  async function decryptFile(
    encrypted: EncryptedFileData,
    password?: string
  ): Promise<Uint8Array> {
    const result = await invoke<number[]>('decrypt_file', {
      encrypted,
      password: password || null,
      keypairBytes: keypair.value?.keypair_bytes || null
    })
    return new Uint8Array(result)
  }

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

  async function deleteKeypair(): Promise<void> {
    keypair.value = null
    encryptedKeypair.value = null
    isUnlocked.value = false
    
    const store = await load('settings.json')
    await store.delete('encryptedKeypair')
    await store.save()
  }

  return {
    keypair,
    hasKeypair,
    hasStoredKeypair,
    isUnlocked,
    publicBundle,
    cryptoInfo,
    initialize,
    generateKeypair,
    saveKeypair,
    unlockKeypair,
    lockKeypair,
    encryptForRecipient,
    decryptFromSender,
    signData,
    verifySignature,
    encryptWithPassword,
    decryptWithPassword,
    hashData,
    encryptFile,
    decryptFile,
    createEncryptionSettings,
    deleteKeypair
  }
}