import { vi } from 'vitest'

// Mock Tauri core API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd, args) => {
    // Basic mock router for fallback values
    if (cmd === 'get_config') {
      if (args?.key === 'theme') return Promise.resolve('dark')
      if (args?.key === 'lang') return Promise.resolve('es')
      return Promise.resolve('')
    }
    return Promise.resolve([])
  }),
}))

// Mock Tauri event API
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))
