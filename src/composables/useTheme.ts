// ItzamBox — Theme Toggle Composable
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const isDark = ref(true)
const loaded = ref(false)

export function useTheme() {
  async function init() {
    try {
      const saved = await invoke<string>('get_config', { key: 'theme' })
      isDark.value = saved === 'dark'
      applyTheme()
    } catch {
      isDark.value = true
      applyTheme()
    }
    loaded.value = true
  }

  function applyTheme() {
    document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  }

  async function toggleTheme() {
    isDark.value = !isDark.value
    applyTheme()
    try {
      await invoke('set_config', { key: 'theme', value: isDark.value ? 'dark' : 'light' })
    } catch { /* non-critical */ }
  }

  watch(isDark, applyTheme)

  return { isDark, loaded, init, toggleTheme }
}
