/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 2 modules
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import * as fc from 'fast-check'
import { validateRepoName } from '../useRepoManager'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(() =>
    Promise.resolve({
      get: vi.fn(),
      set: vi.fn(),
      save: vi.fn(),
    })
  ),
}))

const validRepoNameArb = fc
  .array(fc.constantFrom(...'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.'.split('')), { minLength: 1, maxLength: 100 })
  .map(chars => chars.join(''))
  .filter(s => {
    if (s.startsWith('.') || s.startsWith('-')) return false
    if (s.endsWith('.') || s.endsWith('-')) return false
    if (s.includes('..')) return false
    return s.length >= 1 && s.length <= 100
  })

const alphanumericArb = (minLength: number, maxLength: number) =>
  fc.array(fc.constantFrom(...'abcdefghijklmnopqrstuvwxyz0123456789'.split('')), { minLength, maxLength })
    .map(chars => chars.join(''))

describe('useRepoManager', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.resetModules()
  })

  describe('validateRepoName', () => {
    
    it('accepts valid repository names', () => {
      fc.assert(
        fc.property(validRepoNameArb, (name) => {
          const result = validateRepoName(name)
          return result.valid === true
        }),
        { numRuns: 100 }
      )
    })

    it('rejects empty names', () => {
      const result = validateRepoName('')
      expect(result.valid).toBe(false)
      expect(result.error).toContain('required')
    })

    it('rejects names over 100 characters', () => {
      fc.assert(
        fc.property(
          fc.string({ minLength: 101, maxLength: 200 }),
          (name) => {
            const result = validateRepoName(name)
            return result.valid === false
          }
        ),
        { numRuns: 50 }
      )
    })

    it('rejects names with invalid characters', () => {
      const invalidChars = ['!', '@', '#', '$', '%', '^', '&', '*', '(', ')', ' ', '/', '\\', '?', '<', '>']
      for (const char of invalidChars) {
        const result = validateRepoName(`test${char}repo`)
        expect(result.valid).toBe(false)
      }
    })

    it('rejects names starting with dot or hyphen', () => {
      fc.assert(
        fc.property(
          fc.constantFrom('.', '-'),
          alphanumericArb(1, 10),
          (prefix, rest) => {
            const result = validateRepoName(prefix + rest)
            return result.valid === false
          }
        ),
        { numRuns: 50 }
      )
    })

    it('rejects names ending with dot or hyphen', () => {
      fc.assert(
        fc.property(
          alphanumericArb(1, 10),
          fc.constantFrom('.', '-'),
          (start, suffix) => {
            const result = validateRepoName(start + suffix)
            return result.valid === false
          }
        ),
        { numRuns: 50 }
      )
    })

    it('rejects names with consecutive dots', () => {
      fc.assert(
        fc.property(
          alphanumericArb(1, 10),
          alphanumericArb(1, 10),
          (before, after) => {
            const result = validateRepoName(`${before}..${after}`)
            return result.valid === false
          }
        ),
        { numRuns: 50 }
      )
    })
  })

  describe('Property 5: Repository Creation State Update', () => {
    it('app repo setting matches new repo after creation', async () => {
      
      const testCases = [
        { name: 'myrepo', description: 'Test repo', visibility: 'public' as const, owner: 'testuser' },
        { name: 'private-repo', description: '', visibility: 'private' as const, owner: 'anotheruser' },
        { name: 'repo123', description: 'Numbers', visibility: 'public' as const, owner: 'user123' },
      ]

      for (const testCase of testCases) {
        vi.resetModules()
        vi.clearAllMocks()

        const { invoke } = await import('@tauri-apps/api/core')
        const { load } = await import('@tauri-apps/plugin-store')

        let savedRepo: string | null = null
        const mockStore = {
          get: vi.fn(),
          set: vi.fn((key: string, value: unknown) => {
            if (key === 'repo') savedRepo = value as string
          }),
          save: vi.fn(),
        }

        vi.mocked(load).mockResolvedValue(mockStore as never)

        const fullName = `${testCase.owner}/${testCase.name}`

        vi.mocked(invoke).mockResolvedValueOnce({
          name: testCase.name,
          full_name: fullName,
          private: testCase.visibility === 'private',
          description: testCase.description,
          html_url: `https://github.com/${fullName}`,
          default_branch: 'main',
        })

        const { useRepoManager } = await import('../useRepoManager')
        const { createRepo, currentRepo } = useRepoManager()
        const result = await createRepo(
          { name: testCase.name, description: testCase.description, visibility: testCase.visibility },
          'test-token'
        )

        expect(result.full_name).toBe(fullName)
        expect(currentRepo.value?.full_name).toBe(fullName)
        expect(savedRepo).toBe(fullName)
      }
    })
  })

  describe('Property 6: Privacy Sync Bidirectional Consistency', () => {
    it('local and remote privacy match after sync', async () => {
      const testCases = [
        { owner: 'user1', repoName: 'repo1', remotePrivate: true },
        { owner: 'user2', repoName: 'repo2', remotePrivate: false },
        { owner: 'org', repoName: 'project', remotePrivate: true },
      ]

      for (const testCase of testCases) {
        vi.resetModules()
        vi.clearAllMocks()

        const { invoke } = await import('@tauri-apps/api/core')

        const fullName = `${testCase.owner}/${testCase.repoName}`

        vi.mocked(invoke).mockResolvedValueOnce({
          name: testCase.repoName,
          full_name: fullName,
          private: testCase.remotePrivate,
          description: null,
          html_url: `https://github.com/${fullName}`,
          default_branch: 'main',
        })

        const { useRepoManager } = await import('../useRepoManager')
        const { syncPrivacy, currentRepo } = useRepoManager()
        const result = await syncPrivacy(fullName, 'test-token')

        expect(result).toBe(testCase.remotePrivate)
        expect(currentRepo.value?.private).toBe(testCase.remotePrivate)
      }
    })

    it('local state matches remote after visibility update', async () => {
      const testCases = [
        { owner: 'user1', repoName: 'repo1', initialPrivate: true, newPrivate: false },
        { owner: 'user2', repoName: 'repo2', initialPrivate: false, newPrivate: true },
        { owner: 'org', repoName: 'project', initialPrivate: false, newPrivate: false },
      ]

      for (const testCase of testCases) {
        vi.resetModules()
        vi.clearAllMocks()

        const { invoke } = await import('@tauri-apps/api/core')

        const fullName = `${testCase.owner}/${testCase.repoName}`

        const { useRepoManager } = await import('../useRepoManager')
        const { updateVisibility, currentRepo } = useRepoManager()

        currentRepo.value = {
          name: testCase.repoName,
          full_name: fullName,
          private: testCase.initialPrivate,
          description: null,
          html_url: `https://github.com/${fullName}`,
          default_branch: 'main',
        }

        vi.mocked(invoke).mockResolvedValueOnce({
          name: testCase.repoName,
          full_name: fullName,
          private: testCase.newPrivate,
          description: null,
          html_url: `https://github.com/${fullName}`,
          default_branch: 'main',
        })

        const result = await updateVisibility(fullName, testCase.newPrivate, 'test-token')

        expect(result.private).toBe(testCase.newPrivate)
        expect(currentRepo.value?.private).toBe(testCase.newPrivate)
      }
    })
  })
})