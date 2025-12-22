import { ref } from 'vue'

const getMockState = () => (window as any).__MOCK_ACCENT__ || {}

const accent = ref(getMockState().accent || 'cyber-blue')
const accentHex = ref(getMockState().accentHex || '#00f0ff')
const colors = {
    'cyber-pink': '#ff2d6a',
    'cyber-cyan': '#00f0ff',
    'cyber-purple': '#b026ff',
}

export function useAccentColor() {
    return {
        accent,
        accentHex,
        colors,
        init: async () => { },
        setAccent: (name: string) => {
            accent.value = name
            // @ts-ignore
            accentHex.value = colors[name] || '#00f0ff'
        }
    }
}
