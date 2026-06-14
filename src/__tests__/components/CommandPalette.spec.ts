import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import CommandPalette from '../../components/shared/CommandPalette.vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

vi.mock('vue-router', () => ({
  useRouter: vi.fn(),
}))

describe('CommandPalette component', () => {
  let pushMock = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    pushMock = vi.fn()
    vi.mocked(useRouter).mockReturnValue({
      push: pushMock,
    } as any)
    
    // Clean up body
    document.body.innerHTML = ''
  })

  it('toggles visibility on Ctrl+K key event and processes input and list navigation', async () => {
    const wrapper = mount(CommandPalette)
    
    // Allow onMounted to run its async fetches and register listener
    await new Promise(resolve => setTimeout(resolve, 20))

    // Simulate Ctrl+k keydown
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }))
    await wrapper.vm.$nextTick()
    
    expect((wrapper.vm as any).visible).toBe(true)

    const input = document.querySelector('input')
    expect(input).not.toBeNull()

    // Type query
    input!.value = 'Dashboard'
    input!.dispatchEvent(new Event('input'))
    await wrapper.vm.$nextTick()

    // Navigate with ArrowDown
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }))
    await wrapper.vm.$nextTick()
    expect((wrapper.vm as any).selectedIdx).toBe(0)

    // Navigate with ArrowUp
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp' }))
    await wrapper.vm.$nextTick()

    // Press Escape to close
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await wrapper.vm.$nextTick()
    expect((wrapper.vm as any).visible).toBe(false)

    wrapper.unmount()
  })

  it('searches actions and navigates when enter is pressed', async () => {
    const wrapper = mount(CommandPalette)
    ;(wrapper.vm as any).visible = true
    await wrapper.vm.$nextTick()

    // Type query
    ;(wrapper.vm as any).query = 'Dashboard'
    await wrapper.vm.$nextTick()

    expect((wrapper.vm as any).results.length).toBeGreaterThan(0)

    // Simulate Enter key
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    await wrapper.vm.$nextTick()

    expect(pushMock).toHaveBeenCalledWith('/')
    expect((wrapper.vm as any).visible).toBe(false)

    wrapper.unmount()
  })

  it('executes item on click', async () => {
    const wrapper = mount(CommandPalette)
    ;(wrapper.vm as any).visible = true
    await wrapper.vm.$nextTick()

    ;(wrapper.vm as any).query = 'Dashboard'
    await wrapper.vm.$nextTick()

    const item = document.querySelector('.palette-item')
    expect(item).not.toBeNull()
    item!.dispatchEvent(new MouseEvent('click'))
    await wrapper.vm.$nextTick()

    expect(pushMock).toHaveBeenCalledWith('/')
    wrapper.unmount()
  })

  it('covers pullImageAction, pruneAllAction and startAllStoppedAction', async () => {
    const wrapper = mount(CommandPalette)
    ;(wrapper.vm as any).visible = true
    await wrapper.vm.$nextTick()

    // Mock prompt and confirm
    const promptSpy = vi.spyOn(window, 'prompt').mockReturnValue('ubuntu:latest')
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true)
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'list_containers' || cmd === 'list_images') return []
      if (cmd === 'prune_containers' || cmd === 'prune_images' || cmd === 'prune_volumes' || cmd === 'prune_networks') return 1
      return true
    })

    // Find and execute pull image action
    ;(wrapper.vm as any).query = 'Pull'
    await wrapper.vm.$nextTick()
    const pullImageItem = (wrapper.vm as any).results.find((r: any) => r.label === 'Pull Image')
    expect(pullImageItem).toBeDefined()
    await pullImageItem?.action?.()
    expect(invoke).toHaveBeenCalledWith('pull_image', { imageName: 'ubuntu:latest' })

    // Find and execute prune all action
    ;(wrapper.vm as any).query = 'Prune'
    await wrapper.vm.$nextTick()
    const pruneAllItem = (wrapper.vm as any).results.find((r: any) => r.label === 'Prune All')
    expect(pruneAllItem).toBeDefined()
    await pruneAllItem?.action?.()
    expect(invoke).toHaveBeenCalledWith('prune_containers')

    // Find and execute toggle theme action
    ;(wrapper.vm as any).query = 'Theme'
    await wrapper.vm.$nextTick()
    const themeItem = (wrapper.vm as any).results.find((r: any) => r.label === 'Toggle Theme')
    expect(themeItem).toBeDefined()
    themeItem?.action?.()

    promptSpy.mockRestore()
    confirmSpy.mockRestore()
    wrapper.unmount()
  })
})
