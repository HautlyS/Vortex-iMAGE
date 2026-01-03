import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import PhotoEditor from '../PhotoEditor.vue'

describe('PhotoEditor.vue', () => {
    it('renders when visible', () => {
        const wrapper = mount(PhotoEditor, {
            props: { visible: true, imageUrl: 'test.jpg' }
        })
        expect(wrapper.find('.editor-overlay').exists()).toBe(true)
    })

    it('does not render when hidden', () => {
        const wrapper = mount(PhotoEditor, {
            props: { visible: false, imageUrl: 'test.jpg' }
        })
        expect(wrapper.find('.editor-overlay').exists()).toBe(false)
    })

    it('emits close event', async () => {
        const wrapper = mount(PhotoEditor, {
            props: { visible: true, imageUrl: 'test.jpg' }
        })
        await wrapper.find('.close-btn').trigger('click')
        expect(wrapper.emitted('close')).toBeTruthy()
    })

    // Canvas mocking is tricky in jsdom/happydom without setup, skipping canvas interaction tests 
    // unless we mock getContext efficiently.
    // For now, basic rendering and event emission is enough to verify component structure.
})
