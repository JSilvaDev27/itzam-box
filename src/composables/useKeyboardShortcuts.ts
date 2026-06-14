// ItzamBox — Keyboard Shortcuts (customizable via T-092)
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

export interface ShortcutEntry {
  id: string
  label: string
  defaultKey: string
  currentKey: string
  description: string
}

const shortcuts = ref<ShortcutEntry[]>([])

async function loadShortcuts() {
  const defaults: ShortcutEntry[] = [
    { id: 'navigate-dashboard', label: 'Dashboard', defaultKey: '1', currentKey: '1', description: 'Navigate to Dashboard' },
    { id: 'navigate-containers', label: 'Containers', defaultKey: '2', currentKey: '2', description: 'Navigate to Containers' },
    { id: 'navigate-images', label: 'Images', defaultKey: '3', currentKey: '3', description: 'Navigate to Images' },
    { id: 'navigate-volumes', label: 'Volumes', defaultKey: '4', currentKey: '4', description: 'Navigate to Volumes' },
    { id: 'navigate-networks', label: 'Networks', defaultKey: '5', currentKey: '5', description: 'Navigate to Networks' },
    { id: 'reload', label: 'Reload', defaultKey: 'r', currentKey: 'r', description: 'Reload the application' },
    { id: 'settings', label: 'Settings', defaultKey: ',', currentKey: ',', description: 'Open Settings' },
  ]
  for (const s of defaults) {
    try {
      const saved = await invoke<string>('get_config', { key: 'shortcuts.' + s.id })
      if (saved) s.currentKey = saved
    } catch { /* use default */ }
  }
  shortcuts.value = defaults
}

async function saveShortcut(id: string, key: string) {
  const entry = shortcuts.value.find(s => s.id === id)
  if (entry) {
    entry.currentKey = key
    try { await invoke('set_config', { key: 'shortcuts.' + id, value: key }) } catch { /* ignore */ }
  }
}

export function useKeyboardShortcuts() {
  const router = useRouter()

  function handler(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey
    if (!ctrl) return

    for (const s of shortcuts.value) {
      if (e.key === s.currentKey) {
        e.preventDefault()
        switch (s.id) {
          case 'navigate-dashboard': router.push('/'); break
          case 'navigate-containers': router.push('/containers'); break
          case 'navigate-images': router.push('/images'); break
          case 'navigate-volumes': router.push('/volumes'); break
          case 'navigate-networks': router.push('/networks'); break
          case 'reload': window.location.reload(); break
          case 'settings': router.push('/settings'); break
        }
        break
      }
    }
  }

  onMounted(async () => {
    await loadShortcuts()
    window.addEventListener('keydown', handler)
  })
  onUnmounted(() => window.removeEventListener('keydown', handler))
}

export function useShortcutSettings() {
  return { shortcuts, loadShortcuts, saveShortcut }
}
