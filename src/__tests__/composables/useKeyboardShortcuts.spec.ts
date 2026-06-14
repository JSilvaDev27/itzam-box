import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { useKeyboardShortcuts } from '../../composables/useKeyboardShortcuts'
import { useRouter } from 'vue-router'
import { defineComponent } from 'vue'

vi.mock('vue-router', () => ({
  useRouter: vi.fn(),
}))

describe('useKeyboardShortcuts composable', () => {
  let pushMock = vi.fn()
  
  beforeEach(() => {
    vi.clearAllMocks()
    pushMock = vi.fn()
    vi.mocked(useRouter).mockReturnValue({
      push: pushMock,
    } as any)
  })

  it('should navigate to routes on shortcut key combinations', async () => {
    // Create dummy component to consume useKeyboardShortcuts
    const TestComponent = defineComponent({
      setup() {
        useKeyboardShortcuts()
        return {}
      },
      template: '<div></div>'
    })

    const wrapper = mount(TestComponent)
    await flushPromises()

    // Simulate Keyboard events
    const simulateShortcut = (key: string, ctrl = true) => {
      const event = new KeyboardEvent('keydown', {
        key,
        ctrlKey: ctrl,
        bubbles: true,
        cancelable: true,
      })
      window.dispatchEvent(event)
    }

    // Ctrl+1 -> Home
    simulateShortcut('1')
    expect(pushMock).toHaveBeenCalledWith('/')

    // Ctrl+2 -> Containers
    simulateShortcut('2')
    expect(pushMock).toHaveBeenCalledWith('/containers')

    // Ctrl+3 -> Images
    simulateShortcut('3')
    expect(pushMock).toHaveBeenCalledWith('/images')

    // Ctrl+4 -> Volumes
    simulateShortcut('4')
    expect(pushMock).toHaveBeenCalledWith('/volumes')

    // Ctrl+5 -> Networks
    simulateShortcut('5')
    expect(pushMock).toHaveBeenCalledWith('/networks')

    // Ctrl+, -> Settings
    simulateShortcut(',')
    expect(pushMock).toHaveBeenCalledWith('/settings')

    wrapper.unmount()
  })
})
