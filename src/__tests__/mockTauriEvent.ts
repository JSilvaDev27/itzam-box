import { vi } from 'vitest'
import { listen } from '@tauri-apps/api/event'

/**
 * Setup a mock implementation for Tauri events.
 * Returns tools to simulate event emission from Tauri.
 */
export function mockTauriEvent() {
  const listeners: Record<string, ((event: any) => void)[]> = {}
  
  const mockListen = vi.mocked(listen).mockImplementation(async (event, callback) => {
    if (!listeners[event]) {
      listeners[event] = []
    }
    listeners[event].push(callback as any)
    return () => {
      listeners[event] = listeners[event].filter(cb => cb !== callback)
    }
  })

  function emitMock(event: string, payload: any) {
    if (listeners[event]) {
      listeners[event].forEach(cb => cb({ payload }))
    }
  }

  return { listen: mockListen, emit: emitMock, listeners }
}
