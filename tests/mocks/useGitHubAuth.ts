/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref } from 'vue'

const getMockState = () => (window as any).__MOCK_AUTH__ || {}

const user = ref<any>(getMockState().user || null)
const loading = ref(false)
const userCode = ref<string | null>(getMockState().userCode || null)
const error = ref<string | null>(null)
const repo = ref<string>('test/repo')

export function useGitHubAuth() {
    
    const state = getMockState()
    console.log('[MockAuth] State from window:', state)
    user.value = state.user || null
    userCode.value = state.userCode || null

        ; (window as any).__SET_MOCK_USER__ = (u: any) => { user.value = u }
        ; (window as any).__SET_MOCK_CODE__ = (c: string) => { userCode.value = c }

    return {
        user,
        loading,
        userCode,
        error,
        repo,
        init: async () => { },
        startLogin: async () => {
            loading.value = true
            setTimeout(() => {
                loading.value = false
                userCode.value = "1234-5678"
            }, 100)
        },
        logout: async () => { user.value = null },
        setRepo: (r: string) => { repo.value = r }
    }
}