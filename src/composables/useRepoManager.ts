/**
 * TypeScript Module - 2 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 2 modules
 */

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'

export interface RepoConfig {
  name: string
  description: string
  visibility: 'public' | 'private'
}

export interface RepoInfo {
  name: string
  full_name: string
  private: boolean
  description: string | null
  html_url: string
  default_branch: string
}

const creating = ref(false)
const syncing = ref(false)
const error = ref<string | null>(null)
const currentRepo = ref<RepoInfo | null>(null)
let initialized = false

export function validateRepoName(name: string): { valid: boolean; error?: string } {
  if (!name || name.length === 0) {
    return { valid: false, error: 'Repository name is required' }
  }

  if (name.length > 100) {
    return { valid: false, error: 'Repository name must be 100 characters or less' }
  }

  const validChars = /^[a-zA-Z0-9._-]+$/
  if (!validChars.test(name)) {
    return {
      valid: false,
      error: 'Repository name can only contain letters, numbers, hyphens, underscores, and dots',
    }
  }

  if (name.startsWith('.') || name.startsWith('-') || name.endsWith('.') || name.endsWith('-')) {
    return { valid: false, error: 'Repository name cannot start or end with a dot or hyphen' }
  }

  if (name.includes('..')) {
    return { valid: false, error: 'Repository name cannot contain consecutive dots' }
  }

  return { valid: true }
}

export function useRepoManager() {
  async function loadCurrentRepo(): Promise<void> {
    if (initialized && currentRepo.value) return
    try {
      const store = await load('settings.json')
      const savedRepo = await store.get<string>('repo')
      const savedToken = await store.get<string>('token')

      if (savedRepo && savedToken) {
        
        const info = await getRepoInfo(savedRepo, savedToken)
        currentRepo.value = info
      }
      initialized = true
    } catch {
      
    }
  }

  async function saveRepoSetting(repoFullName: string): Promise<void> {
    try {
      const store = await load('settings.json')
      await store.set('repo', repoFullName)
      await store.save()
    } catch {
      
    }
  }

  async function createRepo(config: RepoConfig, token: string): Promise<RepoInfo> {
    const validation = validateRepoName(config.name)
    if (!validation.valid) {
      throw new Error(validation.error)
    }

    creating.value = true
    error.value = null

    try {
      const result = await invoke<RepoInfo>('create_repo', {
        token,
        name: config.name,
        description: config.description,
        private: config.visibility === 'private',
      })

      currentRepo.value = result
      await saveRepoSetting(result.full_name)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      creating.value = false
    }
  }

  async function getRepoInfo(repo: string, token: string): Promise<RepoInfo> {
    try {
      const result = await invoke<RepoInfo>('get_repo_info', {
        token,
        repo,
      })
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  async function updateVisibility(
    repo: string,
    isPrivate: boolean,
    token: string
  ): Promise<RepoInfo> {
    syncing.value = true
    error.value = null

    try {
      const result = await invoke<RepoInfo>('update_repo_visibility', {
        token,
        repo,
        private: isPrivate,
      })

      if (currentRepo.value && currentRepo.value.full_name === repo) {
        currentRepo.value = result
      }

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      syncing.value = false
    }
  }

  async function syncPrivacy(repo: string, token: string): Promise<boolean> {
    syncing.value = true
    error.value = null

    try {
      
      const remoteInfo = await getRepoInfo(repo, token)

      currentRepo.value = remoteInfo

      return remoteInfo.private
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      syncing.value = false
    }
  }

  function clearError(): void {
    error.value = null
  }

  return {
    creating,
    syncing,
    error,
    currentRepo,
    createRepo,
    getRepoInfo,
    updateVisibility,
    syncPrivacy,
    loadCurrentRepo,
    clearError,
    validateRepoName,
  }
}