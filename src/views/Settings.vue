<!-- ItzamBox — Settings View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTheme } from '../composables/useTheme'
import { useI18n } from '../composables/useI18n'
import { useShortcutSettings } from '../composables/useKeyboardShortcuts'
import { useNotifications } from '../composables/useNotifications'

const { isDark, toggleTheme } = useTheme()
const { locale, setLocale, t } = useI18n()
const { shortcuts, loadShortcuts, saveShortcut } = useShortcutSettings()
const { success, error: notifyError } = useNotifications()
const editingShortcut = ref<string | null>(null)
const editKey = ref('')

function setDarkTheme() { toggleTheme(); }
function setLightTheme() { toggleTheme(); }

onMounted(() => { loadShortcuts() })

function onShortcutClick(id: string) {
  editingShortcut.value = id
  const s = shortcuts.value.find(s => s.id === id)
  editKey.value = s?.currentKey ?? ''
}

function onShortcutKeydown(e: KeyboardEvent) {
  if (editingShortcut.value) {
    e.preventDefault()
    const key = e.key.length === 1 ? e.key.toLowerCase() : e.key
    if (key === 'Escape') { editingShortcut.value = null; return }
    saveShortcut(editingShortcut.value, key)
    editingShortcut.value = null
  }
}

async function handleExportSettings() {
  const keys = ['theme', 'lang', 'shortcuts.navigate-dashboard', 'shortcuts.navigate-containers', 'shortcuts.navigate-images', 'shortcuts.navigate-volumes', 'shortcuts.navigate-networks', 'shortcuts.reload', 'shortcuts.settings']
  const config: Record<string, string> = {}
  for (const key of keys) {
    try {
      const val = await invoke<string>('get_config', { key })
      if (val) config[key] = val
    } catch { /* skip */ }
  }
  const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a'); a.href = url; a.download = 'itzambox-settings.json'
  a.click(); URL.revokeObjectURL(url)
  success('Settings exported', 'Settings saved as itzambox-settings.json')
}

async function handleImportSettings() {
  const input = document.createElement('input'); input.type = 'file'; input.accept = '.json'
  input.onchange = async () => {
    const file = input.files?.[0]
    if (!file) return
    try {
      const text = await file.text()
      const config = JSON.parse(text)
      for (const [key, value] of Object.entries(config)) {
        try { await invoke('set_config', { key, value }) } catch { /* skip */ }
      }
      success('Settings imported', 'Configuration has been updated.')
      await loadShortcuts()
    } catch (e: any) { notifyError('Import failed', e?.toString() ?? 'Invalid file') }
  }
  input.click()
}
</script>

<template>
  <div class="view-root">
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">{{ t.settings.title }}</span>
  </div>
  <h1 class="text-h1" data-testid="settings-title">{{ t.settings.title }}</h1>

  <div style="display:flex;flex-direction:column;gap:16px;max-width:600px">
    <!-- Theme -->
    <section class="section" style="padding:20px">
      <div class="section-header" style="margin:-20px -20px 0 -20px;padding:12px 20px"><span class="section-title"><i class="fa-solid fa-palette" style="color:var(--accent-cyan);margin-right:8px"></i>{{ t.settings.theme }}</span></div>
      <div data-testid="theme-card-dark" @click="setDarkTheme"
        :style="{marginTop:'16px',padding:'20px',borderRadius:'var(--radius-lg)',cursor:'pointer',border:'2px solid ' + (isDark ? 'var(--accent-cyan)' : 'var(--border-color)'),background:'#0a0c10',textAlign:'center'}">
        <i class="fa-solid fa-moon" style="font-size:32px;color:var(--accent-cyan);margin-bottom:8px;display:block"></i>
        <div style="fontWeight:600;fontSize:14px;color:var(--text-main)">Dark</div>
        <div v-if="isDark" style="color:var(--accent-cyan);fontSize:11px;marginTop:4px">✓ Active</div>
      </div>
      <div data-testid="theme-card-light" @click="setLightTheme"
        :style="{marginTop:'8px',padding:'20px',borderRadius:'var(--radius-lg)',cursor:'pointer',border:'2px solid ' + (!isDark ? 'var(--accent-cyan)' : 'var(--border-color)'),background:'#f8fafc',textAlign:'center'}">
        <i class="fa-solid fa-sun" style="font-size:32px;color:'#0284c7';margin-bottom:8px;display:block"></i>
        <div style="fontWeight:600;fontSize:14px;color:'#0f172a'">Light</div>
        <div v-if="!isDark" style="color:'#0284c7';fontSize:11px;marginTop:4px">✓ Active</div>
      </div>
    </section>

    <!-- Language -->
    <section class="section" style="padding:20px">
      <div class="section-header" style="margin:-20px -20px 0 -20px;padding:12px 20px"><span class="section-title"><i class="fa-solid fa-language" style="color:var(--accent-purple);margin-right:8px"></i>{{ t.settings.language }}</span></div>
      <div @click="setLocale('es')" data-testid="locale-es"
        :style="{marginTop:'16px',padding:'16px 24px',borderRadius:'var(--radius-md)',cursor:'pointer',border:'2px solid ' + (locale === 'es' ? 'var(--accent-cyan)' : 'var(--border-color)'),background:locale === 'es' ? 'rgba(0,229,255,0.05)' : 'var(--bg-tertiary)',display:'flex',alignItems:'center',gap:'10px'}">
        <span style="fontSize:20px">🇪🇸</span>
        <span style="fontWeight:600;fontSize:14px">{{ t.settings.spanish }}</span>
      </div>
      <div @click="setLocale('en')" data-testid="locale-en"
        :style="{marginTop:'8px',padding:'16px 24px',borderRadius:'var(--radius-md)',cursor:'pointer',border:'2px solid ' + (locale === 'en' ? 'var(--accent-cyan)' : 'var(--border-color)'),background:locale === 'en' ? 'rgba(0,229,255,0.05)' : 'var(--bg-tertiary)',display:'flex',alignItems:'center',gap:'10px'}">
        <span style="fontSize:20px">🇺🇸</span>
        <span style="fontWeight:600;fontSize:14px">{{ t.settings.english }}</span>
      </div>
    </section>

    <!-- Shortcuts (T-092) -->
    <section class="section" style="padding:20px">
      <div class="section-header" style="margin:-20px -20px 0 -20px;padding:12px 20px"><span class="section-title"><i class="fa-solid fa-keyboard" style="color:var(--accent-cyan);margin-right:8px"></i>Keyboard Shortcuts</span></div>
      <div style="marginTop:16px;display:flex;flex-direction:column;gap:6px" @keydown="onShortcutKeydown">
        <div v-for="s in shortcuts" :key="s.id" style="display:flex;align-items:center;justify-content:space-between;padding:8px 12px;background:var(--bg-tertiary);border-radius:var(--radius-md);border:1px solid var(--border-light)">
          <div>
            <div style="font-size:13px;font-weight:500">{{ s.label }}</div>
            <div style="font-size:11px;color:var(--text-muted)">{{ s.description }}</div>
          </div>
          <div style="display:flex;align-items:center;gap:8px">
            <kbd v-if="editingShortcut !== s.id" @click="onShortcutClick(s.id)" style="cursor:pointer;padding:4px 10px;background:var(--bg-secondary);border:1px solid var(--border-color);border-radius:4px;font-family:var(--font-mono);font-size:12px;min-width:40px;text-align:center;transition:border-color var(--transition-fast)" :style="{ borderColor: editingShortcut === s.id ? 'var(--accent-cyan)' : '' }">
              Ctrl+{{ s.currentKey.toUpperCase() }}
            </kbd>
            <input v-else ref="shortcutInput" v-model="editKey" style="width:80px;padding:4px 8px;background:var(--bg-secondary);border:2px solid var(--accent-cyan);border-radius:4px;font-family:var(--font-mono);font-size:12px;color:var(--text-main);text-align:center;outline:none" placeholder="Press a key" @blur="editingShortcut = null" autofocus />
          </div>
        </div>
        <div style="font-size:10px;color:var(--text-disabled);margin-top:4px">Click a shortcut to remap it, then press the new key combination.</div>
      </div>
    </section>

    <!-- Export / Import Settings (T-093) -->
    <section class="section" style="padding:20px">
      <div class="section-header" style="margin:-20px -20px 0 -20px;padding:12px 20px"><span class="section-title"><i class="fa-solid fa-file-export" style="color:var(--accent-purple);margin-right:8px"></i>Settings Export / Import</span></div>
      <div style="marginTop:16px;display:flex;gap:12px">
        <button class="btn btn-secondary" @click="handleExportSettings">
          <i class="fa-solid fa-download"></i> Export Settings
        </button>
        <button class="btn btn-secondary" @click="handleImportSettings">
          <i class="fa-solid fa-upload"></i> Import Settings
        </button>
      </div>
    </section>

    <!-- About -->
    <section class="section">
      <div class="section-header"><span class="section-title"><i class="fa-solid fa-circle-info" style="color:var(--accent-cyan);margin-right:8px"></i>Version</span></div>
      <div style="padding:16px 20px;fontSize:13px;display:flex;flexDirection:column;gap:6px">
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">ItzamBox</span><span style="fontFamily:'var(--font-mono)'">v1.3.0</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Tauri</span><span style="fontFamily:'var(--font-mono)'">v2.11</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">License</span><span>GNU GPL v3.0</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">© 2026</span><span>SodigTech</span></div>
      </div>
    </section>
  </div>
  </div>
</template>
