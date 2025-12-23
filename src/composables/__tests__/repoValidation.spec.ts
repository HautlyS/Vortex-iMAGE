/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'

export function validateRepoName(name: string): { valid: boolean; error?: string } {
  if (!name || name.length === 0 || name.length > 100) {
    return { valid: false, error: 'Repository name must be 1-100 characters' }
  }

  const validChars = /^[a-zA-Z0-9._-]+$/
  if (!validChars.test(name)) {
    return {
      valid: false,
      error:
        'Repository name can only contain letters, numbers, hyphens, underscores, and dots',
    }
  }

  if (
    name.startsWith('.') ||
    name.startsWith('-') ||
    name.endsWith('.') ||
    name.endsWith('-')
  ) {
    return {
      valid: false,
      error: 'Repository name cannot start or end with a dot or hyphen',
    }
  }

  if (name.includes('..')) {
    return { valid: false, error: 'Repository name cannot contain consecutive dots' }
  }

  return { valid: true }
}

const validRepoChars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-.'
const alphanumeric = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'

function genStringFromChars(chars: string, minLen: number, maxLen: number) {
  return fc
    .array(fc.constantFrom(...chars.split('')), { minLength: minLen, maxLength: maxLen })
    .map((arr) => arr.join(''))
}

describe('Repository Name Validation', () => {
  
  describe('Property 4: Repository Name Validation', () => {
    it('Property 4: valid names (alphanumeric only) are accepted', () => {
      fc.assert(
        fc.property(genStringFromChars(alphanumeric, 1, 50), (name) => {
          const result = validateRepoName(name)
          return result.valid === true
        }),
        { numRuns: 100 }
      )
    })

    it('Property 4: names with valid chars but invalid structure are rejected', () => {
      fc.assert(
        fc.property(genStringFromChars(validRepoChars, 1, 50), (name) => {
          const result = validateRepoName(name)

          const hasInvalidStructure =
            name.startsWith('.') ||
            name.startsWith('-') ||
            name.endsWith('.') ||
            name.endsWith('-') ||
            name.includes('..')

          if (hasInvalidStructure) {
            return !result.valid
          }
          return result.valid
        }),
        { numRuns: 100 }
      )
    })

    it('Property 4: empty names are rejected', () => {
      const result = validateRepoName('')
      expect(result.valid).toBe(false)
    })

    it('Property 4: names over 100 characters are rejected', () => {
      fc.assert(
        fc.property(genStringFromChars(alphanumeric, 101, 150), (name) => {
          const result = validateRepoName(name)
          return !result.valid
        }),
        { numRuns: 100 }
      )
    })

    it('Property 4: names with invalid characters are rejected', () => {
      const invalidChars = ' !@#$%^&*()/\\|<>?'
      fc.assert(
        fc.property(
          genStringFromChars(alphanumeric, 1, 20),
          fc.constantFrom(...invalidChars.split('')),
          (base, invalidChar) => {
            const name = base + invalidChar
            const result = validateRepoName(name)
            return !result.valid
          }
        ),
        { numRuns: 100 }
      )
    })

    it('Property 4: names starting with dot are rejected', () => {
      fc.assert(
        fc.property(genStringFromChars(alphanumeric, 1, 20), (rest) => {
          const name = '.' + rest
          const result = validateRepoName(name)
          return !result.valid
        }),
        { numRuns: 100 }
      )
    })

    it('Property 4: names starting with hyphen are rejected', () => {
      fc.assert(
        fc.property(genStringFromChars(alphanumeric, 1, 20), (rest) => {
          const name = '-' + rest
          const result = validateRepoName(name)
          return !result.valid
        }),
        { numRuns: 100 }
      )
    })

    it('Property 4: names ending with dot are rejected', () => {
      fc.assert(
        fc.property(genStringFromChars(alphanumeric, 1, 20), (base) => {
          const name = base + '.'
          const result = validateRepoName(name)
          return !result.valid
        }),
        { numRuns: 100 }
      )
    })

    it('Property 4: names ending with hyphen are rejected', () => {
      fc.assert(
        fc.property(genStringFromChars(alphanumeric, 1, 20), (base) => {
          const name = base + '-'
          const result = validateRepoName(name)
          return !result.valid
        }),
        { numRuns: 100 }
      )
    })

    it('Property 4: names with consecutive dots are rejected', () => {
      fc.assert(
        fc.property(
          genStringFromChars(alphanumeric, 1, 10),
          genStringFromChars(alphanumeric, 1, 10),
          (before, after) => {
            const name = before + '..' + after
            const result = validateRepoName(name)
            return !result.valid
          }
        ),
        { numRuns: 100 }
      )
    })
  })

  describe('Example cases', () => {
    it('accepts valid repository names', () => {
      expect(validateRepoName('my-repo').valid).toBe(true)
      expect(validateRepoName('my_repo').valid).toBe(true)
      expect(validateRepoName('MyRepo123').valid).toBe(true)
      expect(validateRepoName('repo.name').valid).toBe(true)
      expect(validateRepoName('a').valid).toBe(true)
    })

    it('rejects invalid repository names', () => {
      expect(validateRepoName('').valid).toBe(false)
      expect(validateRepoName('.repo').valid).toBe(false)
      expect(validateRepoName('-repo').valid).toBe(false)
      expect(validateRepoName('repo.').valid).toBe(false)
      expect(validateRepoName('repo-').valid).toBe(false)
      expect(validateRepoName('repo..name').valid).toBe(false)
      expect(validateRepoName('repo name').valid).toBe(false)
      expect(validateRepoName('repo/name').valid).toBe(false)
    })
  })
})