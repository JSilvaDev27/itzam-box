import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useNotifications } from '../../composables/useNotifications'
import { invoke } from '@tauri-apps/api/core'

describe('useNotifications composable', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    const { notifications, toasts } = useNotifications()
    notifications.value = []
    toasts.value = []
  })

  it('should load notifications from backend db', async () => {
    const { notifications, loadNotifications } = useNotifications()
    const mockDbNotifications = [
      {
        id: '1',
        type_str: 'success',
        title: 'Test Title',
        message: 'Test Message',
        read: false,
        created_at: 1718320000,
      },
    ]
    vi.mocked(invoke).mockResolvedValueOnce(mockDbNotifications)

    await loadNotifications()
    expect(notifications.value.length).toBe(1)
    expect(notifications.value[0].title).toBe('Test Title')
    expect(notifications.value[0].read).toBe(false)
  })

  it('should notify and trigger save_notification', async () => {
    const { toasts, notify } = useNotifications()
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_notifications') return []
      return true
    })

    await notify('success', 'Hello', 'World')
    expect(toasts.value.length).toBe(1)
    expect(toasts.value[0].title).toBe('Hello')
    expect(invoke).toHaveBeenCalledWith('save_notification', expect.any(Object))
  })

  it('should support clearing all notifications', async () => {
    const { notifications, clearAll } = useNotifications()
    notifications.value = [{ id: '1', type: 'success', title: 'A', message: 'B', timestamp: 123, read: false }]
    vi.mocked(invoke).mockResolvedValueOnce(true)

    await clearAll()
    expect(notifications.value.length).toBe(0)
    expect(invoke).toHaveBeenCalledWith('clear_notifications')
  })

  it('should fallback to local list on save failure', async () => {
    const { notifications, notify } = useNotifications()
    vi.mocked(invoke).mockRejectedValueOnce(new Error('DB save error'))

    await notify('error', 'Failed Save', 'Fallback')
    await new Promise(resolve => setTimeout(resolve, 10))
    expect(notifications.value.length).toBe(1)
    expect(notifications.value[0].title).toBe('Failed Save')
  })

  it('should remove toast after timeout', async () => {
    vi.useFakeTimers()
    const { toasts, notify } = useNotifications()
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_notifications') return []
      return true
    })

    await notify('success', 'Hello', 'World')
    expect(toasts.value.length).toBe(1)

    vi.runAllTimers()
    expect(toasts.value.length).toBe(0)
    vi.useRealTimers()
  })

  it('should dismiss a toast manually', async () => {
    const { toasts, notify, dismissToast } = useNotifications()
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_notifications') return []
      return true
    })

    await notify('info', 'Title', 'Msg')
    const id = toasts.value[0].id
    dismissToast(id)
    expect(toasts.value.length).toBe(0)
  })

  it('should mark notification as read and handle error', async () => {
    const { notifications, markRead } = useNotifications()
    notifications.value = [{ id: '123', type: 'info', title: 'T', message: 'M', timestamp: 123, read: false }]
    
    vi.mocked(invoke).mockResolvedValueOnce(true)
    await markRead('123')
    expect(notifications.value[0].read).toBe(true)

    // Error path
    notifications.value[0].read = false
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Mark read error'))
    await markRead('123')
    expect(notifications.value[0].read).toBe(false)
  })

  it('should mark all as read and handle error', async () => {
    const { notifications, markAllRead } = useNotifications()
    notifications.value = [{ id: '123', type: 'info', title: 'T', message: 'M', timestamp: 123, read: false }]
    
    vi.mocked(invoke).mockResolvedValueOnce(true)
    await markAllRead()
    expect(notifications.value[0].read).toBe(true)

    // Error path
    notifications.value[0].read = false
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Mark all read error'))
    await markAllRead()
    expect(notifications.value[0].read).toBe(false)
  })

  it('should handle error when clearing all', async () => {
    const { notifications, clearAll } = useNotifications()
    notifications.value = [{ id: '123', type: 'info', title: 'T', message: 'M', timestamp: 123, read: false }]
    
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Clear error'))
    await clearAll()
    expect(notifications.value.length).toBe(1)
  })
})
