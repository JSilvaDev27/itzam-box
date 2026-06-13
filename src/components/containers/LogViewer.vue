<!-- ItzamBox — Log Viewer Component
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{ containerId: string; containerName: string }>()

const logs = ref('')
const tail = ref(500)
const timestamps = ref(true)
const follow = ref(false)
const filter = ref('')
const showStdout = ref(true)
const showStderr = ref(true)
const loading = ref(false)
const logContainer = ref<HTMLDivElement>()

async function fetchLogs() {
  loading.value = true
  try {
    const result = await invoke<string>('get_container_logs', { id: props.containerId, tail: tail.value, timestamps: timestamps.value })
    let filtered = result
    if (!showStdout.value) filtered = filtered.split('\n').filter(l => l.includes('[error]') || l.includes('[warn]')).join('\n')
    if (!showStderr.value) filtered = filtered.split('\n').filter(l => !l.includes('[error]')).join('\n')
    if (filter.value) {
      const re = new RegExp(filter.value, 'gi')
      filtered = filtered.split('\n').filter(l => re.test(l)).join('\n')
    }
    logs.value = filtered || 'No logs found'
  } catch (e: any) {
    logs.value = `Error: ${e.toString()}`
  }
  loading.value = false
  await nextTick()
  if (logContainer.value) logContainer.value.scrollTop = logContainer.value.scrollHeight
}

watch([tail, timestamps], () => fetchLogs())

function copyLogs() {
  navigator.clipboard.writeText(logs.value)
}
</script>

<template>
  <div class="log-viewer">
    <div class="log-toolbar" style="display:flex;align-items:center;justify-content:space-between;padding:8px 0;border-bottom:1px solid var(--border-light);margin-bottom:8px;gap:8px;flex-wrap:wrap">
      <div style="display:flex;gap:12px;align-items:center;font-size:11px;color:var(--text-muted)">
        <label style="display:flex;align-items:center;gap:4px;cursor:pointer"><input type="checkbox" v-model="showStdout" @change="fetchLogs"> STDOUT</label>
        <label style="display:flex;align-items:center;gap:4px;cursor:pointer"><input type="checkbox" v-model="showStderr" @change="fetchLogs"> STDERR</label>
        <label style="display:flex;align-items:center;gap:4px;cursor:pointer"><input type="checkbox" v-model="timestamps" @change="fetchLogs"> TS</label>
      </div>
      <div style="display:flex;gap:6px;align-items:center">
        <input v-model="filter" placeholder="Search..." @keyup.enter="fetchLogs"
          style="width:140px;background:var(--bg-tertiary);border:1px solid var(--border-color);border-radius:4px;padding:4px 8px;font-size:11px;color:var(--text-main);font-family:var(--font-sans);outline:none">
        <select v-model="tail" style="background:var(--bg-tertiary);border:1px solid var(--border-color);color:var(--text-main);font-size:11px;padding:3px 6px;border-radius:4px">
          <option :value="100">100</option><option :value="500">500</option><option :value="1000">1000</option>
        </select>
        <button class="btn btn-ghost btn-sm" @click="fetchLogs"><i class="fa-solid fa-rotate"></i> Refresh</button>
        <button class="btn btn-ghost btn-sm" @click="copyLogs"><i class="fa-solid fa-copy"></i></button>
      </div>
    </div>
    <div ref="logContainer"
      style="background:#000;border-radius:var(--radius-md);padding:12px 14px;font-family:var(--font-mono);font-size:11px;line-height:1.6;color:#e0e0e0;max-height:350px;overflow-y:auto;white-space:pre-wrap;word-break:break-all"
      v-html="logs.replace(/\[error\]/gi,'<span style=color:#ef4444>[error]</span>').replace(/\[warn\]/gi,'<span style=color:#f59e0b>[warn]</span>').replace(/\[info\]/gi,'<span style=color:#3b82f6>[info]</span>').replace(/\[debug\]/gi,'<span style=color:#9ca3af>[debug]</span>') || 'No logs'">
    </div>
  </div>
</template>
