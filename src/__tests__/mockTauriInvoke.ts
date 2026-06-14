import { vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

/**
 * Setup a mock implementation for Tauri invoke calls.
 */
export function mockTauriInvoke(implementation?: (cmd: string, args?: any) => any) {
  const mock = vi.mocked(invoke)
  if (implementation) {
    mock.mockImplementation(async (cmd, args) => implementation(cmd, args))
  }
  return mock
}
