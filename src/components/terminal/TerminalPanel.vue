<!-- ItzamBox — Terminal Panel (xterm.js + WebGL)
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Terminal } from 'xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebglAddon } from '@xterm/addon-webgl'
import 'xterm/css/xterm.css'

const emit = defineEmits<{ close: [] }>()

const terminalRef = ref<HTMLDivElement>()
const isOpen = ref(false)
const tabs = ref<{ id: string; name: string; sessionId?: string }[]>([
  { id: 'host', name: 'Host' }
])
const activeTab = ref('host')
const term = ref<Terminal | null>(null)
const fitAddon = new FitAddon()
let unlisten: (() => void) | null = null
let activeSessionId: string | null = null

async function openTerminal(tabId: string) {
  if (isOpen.value && activeTab.value === tabId) {
    isOpen.value = false
    return
  }
  isOpen.value = true
  activeTab.value = tabId
  await nextTick()
  await initXterm()
  if (tabId === 'host' && !tabs.value.find(t => t.id === 'host')?.sessionId) {
    try {
      const sid = await invoke<string>('spawn_host_terminal')
      const hostTab = tabs.value.find(t => t.id === 'host')
      if (hostTab) hostTab.sessionId = sid
      activeSessionId = sid
      setupPtyListener(sid)
    } catch (e: any) {
      term.value?.writeln(`\x1b[31mError: ${e}\x1b[0m`)
    }
  }
}

async function initXterm() {
  if (term.value) return
  await nextTick()
  if (!terminalRef.value) return

  const t = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    theme: {
      background: '#000000',
      foreground: '#00e5ff',
      cursor: '#00e5ff',
      selectionBackground: 'rgba(0,229,255,0.25)',
    },
    allowProposedApi: true,
  })

  t.loadAddon(fitAddon)
  try { t.loadAddon(new WebglAddon()) } catch { /* fallback to canvas */ }

  t.open(terminalRef.value)
  fitAddon.fit()

  t.onData((data) => {
    if (activeSessionId) {
      invoke('pty_write', { id: activeSessionId, data })
    }
  })

  term.value = t

  window.addEventListener('resize', () => fitAddon.fit())
}

function setupPtyListener(sid: string) {
  if (unlisten) unlisten()
  listen<{ id: string; data: string }>('pty-output', (event) => {
    if (event.payload.id === sid && term.value) {
      term.value.write(event.payload.data)
    }
  }).then(fn => { unlisten = fn })
}

function closeTerminal() {
  if (activeSessionId) {
    invoke('pty_close', { id: activeSessionId })
    activeSessionId = null
  }
  isOpen.value = false
  if (unlisten) { unlisten(); unlisten = null }
}

onUnmounted(() => {
  if (unlisten) unlisten()
  term.value?.dispose()
})
</script>

<template>
  <!-- Minimized bar -->
  <div v-if="!isOpen" class="terminal-min">
    <span class="terminal-min-label" @click="openTerminal('host')">
      <i class="fa-solid fa-chevron-up"></i> Terminal
    </span>
    <span class="terminal-min-label" @click="openTerminal('host')">
      <i class="fa-solid fa-terminal"></i> Host
    </span>
  </div>

  <!-- Expanded panel -->
  <div v-if="isOpen" style="display:flex;flex-direction:column;height:280px;flex-shrink:0">
    <div class="drag-handle" style="height:3px;background:var(--border-color);cursor:ns-resize"></div>
    <div style="display:flex;align-items:center;height:34px;background:var(--bg-secondary);border-top:1px solid var(--border-color);padding:0 8px;gap:2px">
      <span
        v-for="tab in tabs" :key="tab.id"
        @click="activeTab = tab.id"
        :style="{
          padding:'6px 14px', fontSize:'12px', fontWeight:500,
          borderRadius:'4px 4px 0 0', cursor:'pointer',
          background: activeTab === tab.id ? 'var(--terminal-bg)' : 'transparent',
          color: activeTab === tab.id ? 'var(--terminal-fg)' : 'var(--text-muted)',
          border: activeTab === tab.id ? '1px solid var(--border-color)' : '1px solid transparent',
          borderBottomColor: activeTab === tab.id ? 'var(--terminal-bg)' : 'transparent',
        }"
      >
        <i class="fa-solid fa-terminal" style="fontSize:11px;marginRight:6px"></i>{{ tab.name }}
      </span>
      <button style="marginLeft:auto;width:28px;height:28px;border:none;background:transparent;color:var(--text-muted);cursor:pointer;borderRadius:4px"
        @click="closeTerminal" title="Close terminal">
        <i class="fa-solid fa-chevron-down"></i>
      </button>
    </div>
    <div ref="terminalRef" style="flex:1;overflow:hidden"></div>
  </div>
</template>
