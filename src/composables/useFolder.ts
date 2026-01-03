import { ref } from 'vue'
import { useGitHubAuth, isDevMode, addMockAlbum } from './useGitHubAuth'

export interface FolderInfo {
  path: string
  name: string
  parentPath: string | null
}

const creating = ref(false)
const error = ref<string | null>(null)

export function validateFolderName(name: string): { valid: boolean; error?: string } {
  if (!name || name.trim().length === 0) {
    return { valid: false, error: 'Nome da pasta é obrigatório' }
  }

  const trimmed = name.trim()

  if (trimmed.length > 100) {
    return { valid: false, error: 'Nome deve ter no máximo 100 caracteres' }
  }

  if (/[<>:"/\\|?*]/.test(trimmed)) {
    return { valid: false, error: 'Nome contém caracteres inválidos' }
  }

  if (trimmed.startsWith('.') || trimmed.startsWith('-')) {
    return { valid: false, error: 'Nome não pode começar com ponto ou hífen' }
  }

  if (trimmed.includes('..')) {
    return { valid: false, error: 'Nome não pode conter pontos consecutivos' }
  }

  return { valid: true }
}

export function useFolder() {
  const { token, repo } = useGitHubAuth()

  async function createFolder(folderPath: string): Promise<string> {
    const pathParts = folderPath.split('/').filter(Boolean)
    
    for (const part of pathParts) {
      const validation = validateFolderName(part)
      if (!validation.valid) {
        throw new Error(validation.error)
      }
    }

    creating.value = true
    error.value = null

    try {
      // Dev mode: simulate folder creation
      if (isDevMode) {
        await new Promise(r => setTimeout(r, 400))
        const name = pathParts[pathParts.length - 1]
        const parentPath = pathParts.length > 1 ? `photos/${pathParts.slice(0, -1).join('/')}` : undefined
        addMockAlbum(name, parentPath)
        return `photos/${folderPath}`
      }

      if (!token.value || !repo.value) {
        throw new Error('Não autenticado')
      }

      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke<string>('create_folder', {
        folderPath,
        repo: repo.value,
        token: token.value
      })
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      creating.value = false
    }
  }

  async function createSubfolder(parentPath: string, folderName: string): Promise<string> {
    const fullPath = parentPath ? `${parentPath}/${folderName}` : folderName
    return createFolder(fullPath)
  }

  function clearError(): void {
    error.value = null
  }

  function parseFolderPath(fullPath: string): FolderInfo {
    const cleanPath = fullPath.replace(/^photos\//, '')
    const parts = cleanPath.split('/')
    const name = parts.pop() || ''
    const parentPath = parts.length > 0 ? parts.join('/') : null

    return {
      path: cleanPath,
      name,
      parentPath
    }
  }

  return {
    creating,
    error,
    createFolder,
    createSubfolder,
    clearError,
    parseFolderPath,
    validateFolderName
  }
}
