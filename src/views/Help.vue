<!-- ItzamBox — Help View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const dockerVersion = ref('Checking...')
const hostname = ref('...')

onMounted(async () => {
  try { dockerVersion.value = await invoke<string>('get_engine_version') } catch { dockerVersion.value = 'Not connected' }
  try { const m: any = await invoke('get_host_metrics'); hostname.value = m.hostname } catch { /* ok */ }
})
</script>
<template>
  <div class="breadcrumb"><i class="fa-solid fa-house"></i> <span>Home</span> <i class="fa-solid fa-chevron-right"></i> <span class="current">Help</span></div>
  <h1 class="text-h1">Help</h1>
  <div style="display:flex;flex-direction:column;gap:16px;max-width:700px">

    <div class="section">
      <div class="section-header"><span class="section-title"><i class="fa-solid fa-circle-info" style="color:var(--accent-cyan);margin-right:8px"></i> About ItzamBox</span></div>
      <div style="padding:16px 20px;font-size:13px;display:flex;flex-direction:column;gap:6px">
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Version</span><span style="font-family:var(--font-mono)">1.0.0</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Tauri</span><span style="font-family:var(--font-mono)">v2.11</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Docker Engine</span><span style="font-family:var(--font-mono)">{{ dockerVersion }}</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Host</span><span style="font-family:var(--font-mono)">{{ hostname }}</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">License</span><span>GNU GPL v3.0</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">© 2026</span><span>SodigTech</span></div>
      </div>
    </div>

    <div class="section">
      <div class="section-header"><span class="section-title"><i class="fa-solid fa-keyboard" style="color:var(--accent-purple);margin-right:8px"></i> Keyboard Shortcuts</span></div>
      <div style="padding:12px 20px;font-size:12px;display:flex;flex-direction:column;gap:4px">
        <div class="data-row" style="padding:6px 0;border:none"><span>Ctrl+K</span><span style="margin-left:auto;color:var(--text-muted)">Command palette</span></div>
        <div class="data-row" style="padding:6px 0;border:none"><span>Ctrl+1..5</span><span style="margin-left:auto;color:var(--text-muted)">Navigate sections</span></div>
        <div class="data-row" style="padding:6px 0;border:none"><span>Ctrl+T</span><span style="margin-left:auto;color:var(--text-muted)">Host terminal</span></div>
        <div class="data-row" style="padding:6px 0;border:none"><span>Ctrl+Shift+T</span><span style="margin-left:auto;color:var(--text-muted)">Toggle theme</span></div>
        <div class="data-row" style="padding:6px 0;border:none"><span>Ctrl+R</span><span style="margin-left:auto;color:var(--text-muted)">Refresh</span></div>
        <div class="data-row" style="padding:6px 0;border:none"><span>Ctrl+,</span><span style="margin-left:auto;color:var(--text-muted)">Settings</span></div>
        <div class="data-row" style="padding:6px 0;border:none"><span>Escape</span><span style="margin-left:auto;color:var(--text-muted)">Close modal/panel</span></div>
      </div>
    </div>
  </div>
</template>
