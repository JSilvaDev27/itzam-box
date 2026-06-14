import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  useContextMenu,
  containerContextMenu,
  imageContextMenu,
  volumeContextMenu,
  networkContextMenu,
  terminalContextMenu,
  genericCopyMenu
} from '../../composables/useContextMenu'

describe('useContextMenu composable', () => {
  beforeEach(() => {
    // Mock navigator.clipboard
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
      writable: true,
      configurable: true,
    })
  })

  it('should initialize with visible false', () => {
    const { state } = useContextMenu()
    expect(state.value.visible).toBe(false)
  })

  it('should show the menu at the correct coordinate', () => {
    const { state, show } = useContextMenu()
    const mockEvent = {
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
      clientX: 100,
      clientY: 150,
    } as unknown as MouseEvent

    const items = [{ id: 'test', label: 'Test Item' }]
    show(mockEvent, items)

    expect(mockEvent.preventDefault).toHaveBeenCalled()
    expect(mockEvent.stopPropagation).toHaveBeenCalled()
    expect(state.value.visible).toBe(true)
    expect(state.value.x).toBe(100)
    expect(state.value.y).toBe(150)
    expect(state.value.items).toEqual(items)
  })

  it('should hide the menu', () => {
    const { state, show, hide } = useContextMenu()
    show({ preventDefault: () => {}, stopPropagation: () => {}, clientX: 0, clientY: 0 } as MouseEvent, [])
    expect(state.value.visible).toBe(true)

    hide()
    expect(state.value.visible).toBe(false)
  })

  it('should handle item action and hide', () => {
    const { handleAction } = useContextMenu()
    const actionSpy = vi.fn()
    const item = { id: 'act', label: 'Act', action: actionSpy }
    handleAction(item)
    expect(actionSpy).toHaveBeenCalled()
  })

  it('should construct container context menu items', () => {
    const container = { id: '123', name: 'my-nginx', image: 'nginx', state: 'running' }
    const menu = containerContextMenu(container)
    expect(menu.length).toBeGreaterThan(0)
    expect(menu.find(m => m.id === 'start')?.disabled).toBe(true)
    expect(menu.find(m => m.id === 'stop')?.disabled).toBe(false)
  })

  it('should copy to clipboard when action is triggered', async () => {
    const container = { id: '123', name: 'my-nginx', image: 'nginx', state: 'running' }
    const menu = containerContextMenu(container)
    const copyIdItem = menu.find(m => m.id === 'copy-id')
    expect(copyIdItem).toBeDefined()
    
    // Trigger action
    await copyIdItem?.action?.()
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('123')
  })

  it('should construct image, volume, network, terminal and generic context menu items', async () => {
    const imgMenu = imageContextMenu({ id: 'img-1', repository: 'node', tag: 'latest' })
    expect(imgMenu.length).toBeGreaterThan(0)
    await imgMenu.find(m => m.id === 'copy-repo')?.action?.()
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('node')

    const volMenu = volumeContextMenu({ name: 'my-vol', driver: 'local', mountpoint: '/var' })
    expect(volMenu.length).toBeGreaterThan(0)

    const netMenu = networkContextMenu({ id: 'net-1', name: 'my-net', driver: 'bridge', scope: 'local' })
    expect(netMenu.length).toBeGreaterThan(0)

    const termMenu = terminalContextMenu({ onCopy: vi.fn(), onPaste: vi.fn(), onClear: vi.fn() })
    expect(termMenu.length).toBeGreaterThan(0)

    const genMenu = genericCopyMenu('some-text', 'Text')
    expect(genMenu.length).toBe(1)
  })

  it('should fall back to document.execCommand when clipboard API fails', async () => {
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: vi.fn().mockRejectedValue(new Error('Blocked')),
      },
      writable: true,
      configurable: true,
    })
    const execMock = vi.fn().mockReturnValue(true)
    Object.defineProperty(document, 'execCommand', {
      value: execMock,
      writable: true,
      configurable: true,
    })
    const { copyToClipboard } = useContextMenu()
    await copyToClipboard('text', 'label')
    expect(execMock).toHaveBeenCalledWith('copy')
  })
})
