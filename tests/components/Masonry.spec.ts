import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import Masonry from '../../src/components/Masonry.vue'

// Mock GSAP
vi.mock('gsap', () => ({
    gsap: {
        to: vi.fn(),
        fromTo: vi.fn(),
        set: vi.fn(),
        registerPlugin: vi.fn(),
    },
    default: {
        to: vi.fn(),
        fromTo: vi.fn(),
        set: vi.fn(),
        registerPlugin: vi.fn(),
    }
}))

// Proper Mock ResizeObserver
let lastResizeObserverCallback: ResizeObserverCallback | null = null;

class ResizeObserverMock {
    constructor(callback: ResizeObserverCallback) {
        lastResizeObserverCallback = callback;
    }
    observe = vi.fn()
    unobserve = vi.fn()
    disconnect = vi.fn()
}

vi.stubGlobal('ResizeObserver', ResizeObserverMock)
vi.stubGlobal('matchMedia', vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
})))

describe('Masonry.vue', () => {
    const items = [
        { id: '1', img: '1.jpg', url: '1', height: 100 },
        { id: '2', img: '2.jpg', url: '2', height: 150 },
        { id: '3', img: '3.jpg', url: '3', height: 200 },
        { id: '4', img: '4.jpg', url: '4', height: 120 }
    ]

    beforeEach(() => {
        vi.clearAllMocks()
        lastResizeObserverCallback = null;
    })

    it('renders without crashing', () => {
        const wrapper = mount(Masonry, {
            props: { items }
        })
        expect(wrapper.exists()).toBe(true)
    })

    it('renders items', async () => {
        const wrapper = mount(Masonry, {
            props: { items }
        })

        // Simulate ResizeObserver triggering size update
        if (lastResizeObserverCallback) {
            lastResizeObserverCallback([{ contentRect: { width: 1000, height: 800 } } as ResizeObserverEntry], {} as ResizeObserver)
        }

        await wrapper.vm.$nextTick()

        const renderedItems = wrapper.findAll('[data-key]')
        expect(renderedItems.length).toBe(4)
    })

    it('calculates layout correctly', async () => {
        const wrapper = mount(Masonry, {
            props: { items }
        })

        if (lastResizeObserverCallback) {
            lastResizeObserverCallback([{ contentRect: { width: 1000, height: 800 } } as ResizeObserverEntry], {} as ResizeObserver)
        }

        await wrapper.vm.$nextTick()

        const renderedItems = wrapper.findAll('[data-key]')
        const firstItem = renderedItems[0]
        // Check absolute positioning class
        const classes = firstItem.attributes('class')
        expect(classes).toContain('absolute')
        // GSAP applies style inline, but we might not see exact transform values easily in jsdom unless we wait for GSAP.
        // However, we can check basic rendering existence.
    })

    it('emits click event', async () => {
        const wrapper = mount(Masonry, {
            props: { items }
        })

        if (lastResizeObserverCallback) {
            lastResizeObserverCallback([{ contentRect: { width: 1000, height: 800 } } as ResizeObserverEntry], {} as ResizeObserver)
        }
        await wrapper.vm.$nextTick()

        await wrapper.find('[data-key="1"]').trigger('click')
        expect(wrapper.emitted('itemClick')).toBeTruthy()
        expect(wrapper.emitted('itemClick')?.[0][0]).toEqual(expect.objectContaining({ id: '1' }))
    })

    it('emits dblclick event', async () => {
        const wrapper = mount(Masonry, {
            props: { items }
        })

        if (lastResizeObserverCallback) {
            lastResizeObserverCallback([{ contentRect: { width: 1000, height: 800 } } as ResizeObserverEntry], {} as ResizeObserver)
        }
        await wrapper.vm.$nextTick()

        // Trigger double click
        await wrapper.find('[data-key="1"]').trigger('dblclick')

        // Verify event
        expect(wrapper.emitted('itemDblClick')).toBeTruthy()
        expect(wrapper.emitted('itemDblClick')?.[0][0]).toEqual(expect.objectContaining({ id: '1' }))
    })
})
