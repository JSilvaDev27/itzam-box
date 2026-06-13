<!-- ItzamBox — Terminal Panel (xterm.js + WebGL)
     Multi-session with independent tabs, container dropdown, and context menu.
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Terminal } from 'xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebglAddon } from '@xterm/addon-webgl'
import { useContextMenu, terminalContextMenu } from '../../composables/useContextMenu'
import { useDocker } from '../../composables/useDocker'
import { useI18n } from '../../composables/useI18n'
import 'xterm/css/xterm.css'

// ─── Types ──────────────────────────────────────────────────────────────────

interface TerminalSession {
  id: string
  name: string
  sessionId: string | null
  term: Terminal | null
  fitAddon: FitAddon | null
  element: HTMLDivElement | null
}

// ─── Composables ────────────────────────────────────────────────────────────

const { show } = useContextMenu()
const { containers, fetchContainers } = useDocker()
const { t } = useI18n()

// ─── Props / Emits ──────────────────────────────────────────────────────────

const emit = defineEmits<{ close: [] }>()
const props = defineProps<{ containerId?: string; containerName?: string }>()

// ─── Session State ──────────────────────────────────────────────────────────

const sessions = new Map<string, TerminalSession>()
const tabs = ref<{ id: string; name: string }[]>([{ id: 'host', name: 'Host' }])
const activeTab = ref('host')
const isOpen = ref(false)
const showContainerDropdown = ref(false)

// ─── Computed ───────────────────────────────────────────────────────────────

const runningContainers = computed(() =>
  containers.value.filter(c => c.state === 'running')
)

// ─── Session Management ─────────────────────────────────────────────────────

function getSession(tabId: string): TerminalSession {
  let session = sessions.get(tabId)
  if (!session) {
    const tab = tabs.value.find(t => t.id === tabId)
    session = {
      id: tabId,
      name: tab?.name ?? tabId,
      sessionId: null,
      term: null,
      fitAddon: null,
      element: null,
    }
    sessions.set(tabId, session)
  }
  return session
}

function setTerminalRef(tabId: string, el: HTMLDivElement | null): void {
  getSession(tabId).element = el
}

async function initSessionTerminal(session: TerminalSession): Promise<void> {
  // Already initialized
  if (session.term) return
  // Element not yet in DOM — wait one tick
  if (!session.element) {
    await nextTick()
  }
  if (!session.element) return

  const fitAddon = new FitAddon()
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

  t.open(session.element)
  fitAddon.fit()

  // Wire xterm data → PTY write
  t.onData((data: string) => {
    if (session.sessionId) {
      invoke('pty_write', { id: session.sessionId, data })
    }
  })

  session.term = t
  session.fitAddon = fitAddon
}

async function openSession(tabId: string, containerName?: string): Promise<void> {
  showContainerDropdown.value = false
  isOpen.value = true

  // Add tab entry if new
  const existing = tabs.value.find(t => t.id === tabId)
  if (!existing) {
    const displayName = tabId === 'host'
      ? 'Host'
      : containerName
        ? (containerName.length > 12 ? containerName.substring(0, 12) + '…' : containerName)
        : tabId.substring(0, 12)
    tabs.value.push({ id: tabId, name: displayName })
  }

  activeTab.value = tabId
  await nextTick()

  const session = getSession(tabId)
  await initSessionTerminal(session)

  // If session already has a PTY backend connected, just re-fit
  if (session.sessionId) {
    session.fitAddon?.fit()
    return
  }

  // Spawn new PTY session
  try {
    const sid = tabId === 'host'
      ? await invoke<string>('spawn_host_terminal')
      : await invoke<string>('spawn_container_terminal', { containerId: tabId })
    session.sessionId = sid
    session.fitAddon?.fit()
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    session.term?.writeln(`\x1b[31mError: ${msg}\x1b[0m`)
  }
}

function switchTab(tabId: string): void {
  activeTab.value = tabId
  nextTick(() => {
    getSession(tabId).fitAddon?.fit()
  })
}

async function closeTab(tabId: string): Promise<void> {
  if (tabId === 'host') return

  const session = sessions.get(tabId)
  if (session) {
    // Close the backend PTY session
    if (session.sessionId) {
      try { await invoke('pty_close', { id: session.sessionId }) } catch { /* ignore */ }
    }
    // Dispose xterm instance
    session.term?.dispose()
    sessions.delete(tabId)
  }

  // Remove from tabs array
  tabs.value = tabs.value.filter(t => t.id !== tabId)

  // If the closed tab was active, switch to host
  if (activeTab.value === tabId) {
    activeTab.value = 'host'
    nextTick(() => {
      getSession('host').fitAddon?.fit()
    })
  }
}

function closePanel(): void {
  isOpen.value = false
}

// ─── Context Menu ───────────────────────────────────────────────────────────

function onTerminalContextMenu(e: MouseEvent): void {
  const session = sessions.get(activeTab.value)
  if (!session) return

  const sel = session.term?.getSelection()
  show(e, terminalContextMenu({
    onCopy: () => {
      if (sel) {
        navigator.clipboard.writeText(sel)
      } else {
        session.term?.selectAll()
      }
    },
    onPaste: () => {
      navigator.clipboard.readText().then(text => {
        if (session.sessionId) {
          invoke('pty_write', { id: session.sessionId, data: text })
        }
      })
    },
    onClear: () => session.term?.clear(),
  }))
}

// ─── Dropdown Dismiss ───────────────────────────────────────────────────────

function handleDocumentClick(e: MouseEvent): void {
  if (showContainerDropdown.value) {
    const target = e.target as HTMLElement
    if (!target.closest('.terminal-tab-add')) {
      showContainerDropdown.value = false
    }
  }
}

// ─── Lifecycle ──────────────────────────────────────────────────────────────

let ptyListener: (() => void) | null = null
let resizeHandler: (() => void) | null = null

onMounted(async () => {
  document.addEventListener('click', handleDocumentClick)
  fetchContainers()

  // Global resize handler — fit the active session
  resizeHandler = () => {
    const session = sessions.get(activeTab.value)
    session?.fitAddon?.fit()
  }
  window.addEventListener('resize', resizeHandler)

  // Single global PTY-output listener dispatches to the matching session
  ptyListener = await listen<{ id: string; data: string }>('pty-output', (event) => {
    for (const [, session] of sessions) {
      if (event.payload.id === session.sessionId && session.term) {
        session.term.write(event.payload.data)
      }
    }
  })

  // Auto-open terminal if containerId was passed in props
  if (props.containerId) {
    openSession(props.containerId, props.containerName)
  }
})

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick)
  if (resizeHandler) window.removeEventListener('resize', resizeHandler)

  // Tear down the single PTY listener
  ptyListener?.()

  // Dispose every session
  for (const [, session] of sessions) {
    session.term?.dispose()
    if (session.sessionId) {
      invoke('pty_close', { id: session.sessionId }).catch(() => {})
    }
  }
  sessions.clear()
})

// ─── Expose Public API ──────────────────────────────────────────────────────

defineExpose({ openSession })
</script>

<template>
  <!-- ── Minimized bar ──────────────────────────────────────────── -->
  <div v-if="!isOpen" class="terminal-min">
    <span class="terminal-min-label" @click="openSession('host')">
      <i class="fa-solid fa-chevron-up"></i> {{ t.terminal.host || 'Terminal' }}
    </span>
    <span class="terminal-min-label" @click="openSession('host')">
      <i class="fa-solid fa-terminal"></i> {{ t.terminal.host_tab || 'Host' }}
    </span>
  </div>

  <!-- ── Expanded panel ─────────────────────────────────────────── -->
  <div v-if="isOpen" class="terminal-expanded">
    <!-- Drag handle at top -->
    <div class="terminal-drag-handle"></div>

    <!-- Tab bar -->
    <div class="terminal-tabs-bar">
      <!-- Scrollable tab list -->
      <div class="terminal-tabs-scroll">
        <span
          v-for="tab in tabs"
          :key="tab.id"
          :class="['terminal-tab', { 'terminal-tab--active': activeTab === tab.id }]"
          @click="switchTab(tab.id)"
          role="tab"
          :aria-selected="activeTab === tab.id"
          :aria-controls="'terminal-panel-' + tab.id"
        >
          <i :class="tab.id === 'host' ? 'fa-solid fa-server' : 'fa-solid fa-cube'" class="terminal-tab-icon"></i>
          <span class="terminal-tab-name">{{ tab.name }}</span>
          <button
            v-if="tab.id !== 'host'"
            class="terminal-tab-close"
            @click.stop="closeTab(tab.id)"
            :title="t.terminal.close_tab || 'Close terminal'"
            :aria-label="(t.terminal.close_tab || 'Close') + ' ' + tab.name"
          >&times;</button>
        </span>
      </div>

      <!-- Add-terminal dropdown -->
      <div class="terminal-tab-add" @click.stop>
        <button
          class="terminal-tab-add-btn"
          @click="showContainerDropdown = !showContainerDropdown"
          :title="t.terminal.new_terminal || 'Open new terminal\u2026'"
          :aria-label="t.terminal.new_terminal || 'Open new terminal'"
        >+</button>
        <div v-if="showContainerDropdown" class="terminal-dropdown">
          <div class="terminal-dropdown-item" @click="openSession('host')">
            <i class="fa-solid fa-server"></i> {{ t.terminal.host || 'Host Terminal' }}
          </div>
          <div class="terminal-dropdown-divider"></div>
          <template v-if="runningContainers.length > 0">
            <div
              v-for="c in runningContainers"
              :key="c.id"
              class="terminal-dropdown-item"
              @click="openSession(c.id, c.name)"
            >
              <i class="fa-solid fa-cube"></i>
              <span class="terminal-dropdown-item-label">{{ c.name }}</span>
            </div>
          </template>
          <div v-else class="terminal-dropdown-item terminal-dropdown-item--empty">
            {{ t.terminal.no_running || 'No running containers' }}
          </div>
        </div>
      </div>

      <!-- Spacer to push close button right -->
      <div class="terminal-tabs-spacer"></div>

      <!-- Close-panel button -->
      <button class="terminal-tabs-minimize" @click="closePanel" :title="t.terminal.close_panel || 'Close terminal panel'">
        <i class="fa-solid fa-chevron-down"></i>
      </button>
    </div>

    <!-- Terminal body — one container per session, only the active one visible -->
    <div class="terminal-body">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        v-show="activeTab === tab.id"
        :ref="(el: any) => setTerminalRef(tab.id, el as HTMLDivElement | null)"
        class="terminal-container"
        :id="'terminal-panel-' + tab.id"
        role="tabpanel"
        @contextmenu="onTerminalContextMenu"
      ></div>
    </div>
  </div>
</template>

<style scoped>
/* ─── Minimized bar ──────────────────────────────────────────── */
.terminal-min {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 32px;
  padding: 0 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}
.terminal-min-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
  cursor: pointer;
  user-select: none;
  transition: color var(--transition-fast);
}
.terminal-min-label:hover {
  color: var(--accent-cyan);
}

/* ─── Expanded panel ──────────────────────────────────────────── */
.terminal-expanded {
  display: flex;
  flex-direction: column;
  height: 280px;
  flex-shrink: 0;
}
.terminal-drag-handle {
  height: 3px;
  background: var(--border-color);
  cursor: ns-resize;
  flex-shrink: 0;
}

/* ─── Tab bar ──────────────────────────────────────────────────── */
.terminal-tabs-bar {
  display: flex;
  align-items: center;
  height: 34px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  padding: 0 4px;
  gap: 0;
  flex-shrink: 0;
}
.terminal-tabs-scroll {
  display: flex;
  align-items: center;
  gap: 2px;
  overflow-x: auto;
  overflow-y: hidden;
  white-space: nowrap;
  flex: 1;
  scrollbar-width: thin;
}
.terminal-tabs-scroll::-webkit-scrollbar {
  height: 3px;
}

.terminal-tab {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  color: var(--text-muted);
  border: 1px solid transparent;
  border-bottom-color: transparent;
  flex-shrink: 0;
  user-select: none;
  transition: background var(--transition-fast), color var(--transition-fast);
}
.terminal-tab:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}
.terminal-tab--active {
  background: var(--terminal-bg);
  color: var(--terminal-fg);
  border-color: var(--border-color);
  border-bottom-color: var(--terminal-bg);
}
.terminal-tab-icon {
  font-size: 11px;
}
.terminal-tab-name {
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
}
.terminal-tab-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 3px;
  font-size: 14px;
  line-height: 1;
  padding: 0;
  transition: background var(--transition-fast), color var(--transition-fast);
}
.terminal-tab-close:hover {
  background: var(--accent-red);
  color: #fff;
}

/* ─── Add-terminal button & dropdown ──────────────────────────── */
.terminal-tab-add {
  position: relative;
  flex-shrink: 0;
}
.terminal-tab-add-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px;
  font-size: 18px;
  font-weight: 600;
  line-height: 1;
  transition: background var(--transition-fast), color var(--transition-fast);
}
.terminal-tab-add-btn:hover {
  background: var(--bg-hover);
  color: var(--accent-cyan);
}

.terminal-dropdown {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 4px;
  min-width: 220px;
  max-height: 280px;
  overflow-y: auto;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  z-index: 1000;
  padding: 4px 0;
}
.terminal-dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-main);
  cursor: pointer;
  transition: background var(--transition-fast);
  white-space: nowrap;
}
.terminal-dropdown-item:hover {
  background: var(--bg-hover);
}
.terminal-dropdown-item i {
  width: 16px;
  text-align: center;
  font-size: 13px;
  color: var(--accent-cyan);
}
.terminal-dropdown-item--empty {
  color: var(--text-disabled);
  cursor: default;
}
.terminal-dropdown-item--empty:hover {
  background: transparent;
}
.terminal-dropdown-item-label {
  overflow: hidden;
  text-overflow: ellipsis;
}
.terminal-dropdown-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}

/* ─── Spacer & minimize ───────────────────────────────────────── */
.terminal-tabs-spacer {
  flex: 0 0 auto;
  min-width: 4px;
}
.terminal-tabs-minimize {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px;
  flex-shrink: 0;
  transition: background var(--transition-fast), color var(--transition-fast);
}
.terminal-tabs-minimize:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

/* ─── Terminal body ────────────────────────────────────────────── */
.terminal-body {
  flex: 1;
  position: relative;
  overflow: hidden;
}
.terminal-container {
  position: absolute;
  inset: 0;
}
</style>
