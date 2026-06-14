import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useTheme } from '../../composables/useTheme'
import { invoke } from '@tauri-apps/api/core'

describe('useTheme composable', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    document.documentElement.className = ''
  })

  it('should initialize with dark theme default', async () => {
    const { isDark, init } = useTheme()
    
    // Mock get_config to return 'dark'
    vi.mocked(invoke).mockResolvedValueOnce('dark')

    await init()
    expect(isDark.value).toBe(true)
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
  })

  it('should toggle theme and invoke set_config', async () => {
    const { isDark, init, toggleTheme } = useTheme()
    
    vi.mocked(invoke).mockResolvedValueOnce('dark')
    await init()

    await toggleTheme()
    expect(isDark.value).toBe(false)
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
    expect(invoke).toHaveBeenCalledWith('set_config', { key: 'theme', value: 'light' })
  })
})
