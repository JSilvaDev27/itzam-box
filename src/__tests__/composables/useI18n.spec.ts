import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useI18n } from '../../composables/useI18n'
import { invoke } from '@tauri-apps/api/core'

describe('useI18n composable', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should initialize with default es translations', async () => {
    const { t, init } = useI18n()
    vi.mocked(invoke).mockResolvedValueOnce('es')

    await init()
    expect(t.value.dashboard.title).toBe('Panel General')
  })

  it('should support switching locale to English', async () => {
    const { t, setLocale } = useI18n()

    await setLocale('en')
    expect(t.value.dashboard.title).toBe('Dashboard')
    expect(invoke).toHaveBeenCalledWith('set_config', { key: 'lang', value: 'en' })
  })
})
