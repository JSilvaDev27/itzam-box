<!-- ItzamBox — Docker Event Stream View
     Copyright (C) 2026 SodigTech — GPL-3.0
     Section 16 PRD: Real-time Docker events timeline with filtering,
     search, and live counter. -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  type DockerEvent,
  listenDockerEvents,
  startEventStream,
  stopEventStream,
  relativeTime,
} from '../composables/useDocker'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'

const router = useRouter()

// ─── Stream State ───────────────────────────────────────────────────────────

const isStreaming = ref(false)
const starting = ref(false)
const error = ref<string | null>(null)

// ─── Event Buffer (circular, max 500) ───────────────────────────────────────

const MAX_EVENTS = 500
const events = ref<DockerEvent[]>([])
const liveCount = ref(0)

// ─── Filter State ──────────────────────────────────────────────────────────

const filterType = ref<string>('all')
const filterAction = ref<string>('all')
const searchQuery = ref('')
const showTimestamps = ref(true)

const EVENT_TYPES = ['all', 'container', 'image', 'volume', 'network'] as const
const EVENT_ACTIONS = [
  'all', 'start', 'stop', 'create', 'destroy', 'die', 'kill',
  'pause', 'unpause', 'restart', 'rename', 'exec_create', 'exec_start',
  'attach', 'detach', 'connect', 'disconnect', 'mount', 'unmount',
  'pull', 'push', 'tag', 'untag', 'delete', 'import', 'export',
  'update', 'commit',
] as const

// ─── Computed ───────────────────────────────────────────────────────────────

const filteredEvents = computed(() => {
  return events.value.filter((e) => {
    if (filterType.value !== 'all' && e.event_type !== filterType.value) return false
    if (filterAction.value !== 'all' && e.action !== filterAction.value) return false
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      const nameMatch = e.actor_name.toLowerCase().includes(q)
      const idMatch = e.actor_id.toLowerCase().includes(q)
      if (!nameMatch && !idMatch) return false
    }
    return true
  })
})

// ─── Action Icon Map ────────────────────────────────────────────────────────

function typeIcon(eventType: string): string {
  const map: Record<string, string> = {
    container: 'fa-solid fa-cube',
    image: 'fa-solid fa-layer-group',
    volume: 'fa-solid fa-database',
    network: 'fa-solid fa-network-wired',
  }
  return map[eventType] || 'fa-solid fa-question'
}

function actionIcon(action: string): string {
  const map: Record<string, string> = {
    start: 'fa-solid fa-play',
    stop: 'fa-solid fa-stop',
    die: 'fa-solid fa-skull',
    create: 'fa-solid fa-sparkles',
    destroy: 'fa-solid fa-trash-can',
    kill: 'fa-solid fa-bolt',
    pause: 'fa-solid fa-pause',
    unpause: 'fa-solid fa-play',
    restart: 'fa-solid fa-rotate-right',
    rename: 'fa-solid fa-pen',
    pull: 'fa-solid fa-download',
    push: 'fa-solid fa-upload',
    tag: 'fa-solid fa-tag',
    untag: 'fa-solid fa-tag',
    delete: 'fa-solid fa-eraser',
    import: 'fa-solid fa-file-import',
    export: 'fa-solid fa-file-export',
    connect: 'fa-solid fa-plug',
    disconnect: 'fa-solid fa-plug-circle-xmark',
    mount: 'fa-solid fa-folder-open',
    unmount: 'fa-solid fa-folder',
    attach: 'fa-solid fa-paperclip',
    detach: 'fa-solid fa-paperclip',
    commit: 'fa-solid fa-floppy-disk',
    exec_create: 'fa-solid fa-terminal',
    exec_start: 'fa-solid fa-terminal',
    update: 'fa-solid fa-pen-to-square',
  }
  return map[action] || 'fa-solid fa-bolt'
}

function actionColor(action: string): string {
  const green = ['start', 'unpause', 'create', 'pull', 'mount', 'connect', 'attach']
  const red = ['stop', 'die', 'destroy', 'kill', 'untag', 'delete', 'detach', 'disconnect', 'unmount']
  const amber = ['pause', 'restart', 'rename', 'update', 'exec_create', 'exec_start']
  const blue = ['commit', 'push', 'export', 'import', 'tag']
  if (green.includes(action)) return 'var(--accent-green)'
  if (red.includes(action)) return 'var(--accent-red)'
  if (amber.includes(action)) return 'var(--accent-amber)'
  if (blue.includes(action)) return 'var(--accent-blue)'
  return 'var(--text-muted)'
}

// ─── Stream Management ──────────────────────────────────────────────────────

let unlistenEvents: (() => void) | null = null

async function startStream() {
  if (isStreaming.value) return
  starting.value = true
  error.value = null
  try {
    await startEventStream()
    isStreaming.value = true
    unlistenEvents = await listenDockerEvents(handleEvent)
    liveCount.value = 0
  } catch (e: any) {
    error.value = e.toString()
  } finally {
    starting.value = false
  }
}

async function stopStream() {
  if (!isStreaming.value) return
  try {
    await stopEventStream()
  } catch (e: any) {
    console.warn('Error stopping stream:', e.toString())
  }
  cleanupStream()
}

function cleanupStream() {
  isStreaming.value = false
  if (unlistenEvents) {
    unlistenEvents()
    unlistenEvents = null
  }
}

function handleEvent(evt: DockerEvent) {
  // Prepend newest events to the front (most recent first)
  events.value.unshift({ ...evt })
  if (events.value.length > MAX_EVENTS) {
    events.value.pop()
  }
  liveCount.value++
}

// ─── Copy Actor ID ──────────────────────────────────────────────────────────

const copiedId = ref<string | null>(null)

async function copyActorId(id: string) {
  try {
    await navigator.clipboard.writeText(id)
    copiedId.value = id
    setTimeout(() => {
      if (copiedId.value === id) copiedId.value = null
    }, 2000)
  } catch {
    // Fallback
    const ta = document.createElement('textarea')
    ta.value = id
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    copiedId.value = id
    setTimeout(() => {
      if (copiedId.value === id) copiedId.value = null
    }, 2000)
  }
}

// ─── Navigation ─────────────────────────────────────────────────────────────

function navigateToActor(event: DockerEvent) {
  if (event.event_type === 'container' && event.actor_id) {
    router.push(`/containers/${event.actor_id}`)
  }
}

// ─── Auto-refresh relative times every 10 seconds ───────────────────────────

const tick = ref(0)
let tickInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  tickInterval = setInterval(() => {
    tick.value++
    // Prune stale events older than 1 hour from the buffer only if we're
    // over the limit to keep memory in check
    if (events.value.length > MAX_EVENTS * 0.9) {
      const cutoff = Math.floor(Date.now() / 1000) - 3600
      events.value = events.value.filter((e) => e.timestamp >= cutoff)
    }
  }, 10000)
})

onUnmounted(() => {
  cleanupStream()
  if (tickInterval) {
    clearInterval(tickInterval)
    tickInterval = null
  }
})
</script>

<template>
  <!-- ═══ Breadcrumb ═══ -->
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i>
    <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span class="current">Events</span>
  </div>

  <!-- ═══ Header ═══ -->
  <div class="events-header">
    <div class="events-header-left">
      <h1 class="text-h1">Docker Events</h1>
      <div v-if="isStreaming" class="live-badge" title="Streaming live events">
        <span class="live-dot"></span>
        <span class="live-count">{{ liveCount }}</span>
        <span class="live-label">events</span>
      </div>
    </div>

    <div class="events-header-right">
      <div class="stream-indicator">
        <span
          :class="['stream-dot', isStreaming ? 'stream-dot--live' : 'stream-dot--stopped']"
        ></span>
        <span class="stream-status">{{ isStreaming ? 'Live' : 'Stopped' }}</span>
      </div>

      <button
        v-if="!isStreaming"
        class="btn btn-primary"
        :disabled="starting"
        @click="startStream"
      >
        <i :class="starting ? 'fa-solid fa-spinner fa-spin' : 'fa-solid fa-play'"></i>
        {{ starting ? 'Starting...' : 'Start Stream' }}
      </button>
      <button
        v-else
        class="btn btn-secondary btn-stop"
        @click="stopStream"
      >
        <i class="fa-solid fa-stop"></i>
        Stop Stream
      </button>

      <button
        class="btn btn-ghost"
        @click="events = []; liveCount = 0"
        title="Clear events"
      >
        <i class="fa-solid fa-eraser"></i>
      </button>
    </div>
  </div>

  <!-- ═══ Error State ═══ -->
  <ErrorState
    v-if="error && !isStreaming"
    :message="'Failed to start Docker event stream'"
    :suggestion="'Make sure Docker is installed and the daemon is running.'"
    :detail="error"
    @retry="startStream"
  />

  <!-- ═══ No stream — Empty State ═══ -->
  <EmptyState
    v-if="!error && !isStreaming && !starting && events.length === 0"
    icon="fa-solid fa-clock-rotate-left"
    title="Docker Event Stream"
    description="Start the event stream to monitor Docker activity in real time."
    action-label="Start Stream"
    @action="startStream"
  />

  <!-- ═══ Loading ═══ -->
  <SkeletonLoader v-if="starting" variant="text" :lines="5" />

  <!-- ═══ Stream Active ═══ -->
  <template v-if="isStreaming || events.length > 0">
    <!-- ─── Filters Bar ─── -->
    <div class="events-filters">
      <div class="filter-group">
        <label class="filter-label" for="filter-type">Type</label>
        <select id="filter-type" v-model="filterType" class="filter-select">
          <option v-for="t in EVENT_TYPES" :key="t" :value="t">
            {{ t === 'all' ? 'All Types' : t }}
          </option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label" for="filter-action">Action</label>
        <select id="filter-action" v-model="filterAction" class="filter-select">
          <option v-for="a in EVENT_ACTIONS" :key="a" :value="a">
            {{ a === 'all' ? 'All Actions' : a }}
          </option>
        </select>
      </div>

      <div class="filter-group filter-search">
        <label class="filter-label" for="filter-search">
          <i class="fa-solid fa-search"></i>
        </label>
        <input
          id="filter-search"
          v-model="searchQuery"
          type="text"
          class="filter-input"
          placeholder="Search name or ID..."
        />
        <button
          v-if="searchQuery"
          class="filter-clear"
          @click="searchQuery = ''"
          tabindex="-1"
        >
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <div class="filter-group filter-toggle">
        <label class="filter-label toggle-label">
          <input type="checkbox" v-model="showTimestamps" />
          <span>Timestamps</span>
        </label>
      </div>

      <div class="filter-group filter-count">
        <span class="filter-result-count">
          {{ filteredEvents.length }}
          {{ filteredEvents.length === 1 ? 'event' : 'events' }}
        </span>
      </div>
    </div>

    <!-- ─── Timeline ─── -->
    <div class="events-timeline">
      <!-- Empty within stream -->
      <div
        v-if="filteredEvents.length === 0"
        class="events-empty-stream"
      >
        <i class="fa-solid fa-ear-listen"></i>
        <p>Listening… waiting for Docker events</p>
      </div>

      <!-- Event rows -->
      <div
        v-for="(evt, idx) in filteredEvents"
        :key="evt.timestamp + '-' + evt.actor_id + '-' + idx"
        :class="[
          'event-row',
          evt.event_type === 'container' && evt.actor_id ? 'event-row--clickable' : '',
        ]"
        @click="navigateToActor(evt)"
      >
        <!-- Type icon -->
        <div class="event-type-icon" :title="evt.event_type">
          <i :class="typeIcon(evt.event_type)"></i>
        </div>

        <!-- Action icon + badge -->
        <div class="event-action-col">
          <i
            :class="actionIcon(evt.action)"
            class="event-action-icon"
            :style="{ color: actionColor(evt.action) }"
          ></i>
          <span class="event-action-badge">{{ evt.event_type }}.{{ evt.action }}</span>
        </div>

        <!-- Actor info -->
        <div class="event-actor">
          <span class="event-actor-name" :title="evt.actor_name">
            {{ evt.actor_name || '—' }}
          </span>
          <span class="event-actor-id">
            <code>{{ evt.actor_id.length > 12 ? evt.actor_id.substring(0, 12) + '…' : evt.actor_id || '—' }}</code>
            <button
              v-if="evt.actor_id"
              class="btn-copy-id"
              :class="{ 'btn-copy-id--copied': copiedId === evt.actor_id }"
              @click.stop="copyActorId(evt.actor_id)"
              :title="copiedId === evt.actor_id ? 'Copied!' : 'Copy ID'"
            >
              <i :class="copiedId === evt.actor_id ? 'fa-solid fa-check' : 'fa-solid fa-copy'"></i>
            </button>
          </span>
        </div>

        <!-- Timestamp -->
        <div class="event-time" :title="showTimestamps && evt.timestamp ? new Date(evt.timestamp * 1000).toLocaleString() : ''">
          <template v-if="showTimestamps">
            <span v-if="evt.timestamp" class="event-time-relative">{{ relativeTime(evt.timestamp) }}</span>
            <span v-else class="event-time-empty">—</span>
          </template>
        </div>
      </div>
    </div>
  </template>
</template>

<style scoped>
/* ─── Events Header ──────────────────────────────────────────────────── */
.events-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 12px;
}

.events-header-left {
  display: flex;
  align-items: center;
  gap: 14px;
}

.events-header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

/* ─── Live Badge ─────────────────────────────────────────────────────── */
.live-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent-green) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-green) 30%, transparent);
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-green);
}

.live-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-green);
  animation: live-pulse 1.5s ease-in-out infinite;
}

@keyframes live-pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.85); }
}

.live-count {
  font-variant-numeric: tabular-nums;
}

.live-label {
  font-weight: 400;
  opacity: 0.7;
}

/* ─── Stream Indicator ───────────────────────────────────────────────── */
.stream-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  padding: 4px 10px;
  border-radius: 6px;
  background: var(--bg-secondary);
}

.stream-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.stream-dot--live {
  background: var(--accent-green);
  box-shadow: 0 0 6px var(--accent-green);
}

.stream-dot--stopped {
  background: var(--text-muted);
}

.stream-status {
  color: var(--text-primary);
}

/* ─── Stop button ────────────────────────────────────────────────────── */
.btn-stop {
  border-color: color-mix(in srgb, var(--accent-red) 40%, transparent) !important;
  color: var(--accent-red) !important;
}
.btn-stop:hover {
  background: color-mix(in srgb, var(--accent-red) 12%, transparent) !important;
}

/* ─── Filters Bar ────────────────────────────────────────────────────── */
.events-filters {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  padding: 14px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}

.filter-select,
.filter-input {
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s;
  min-width: 0;
}

.filter-select:focus,
.filter-input:focus {
  border-color: var(--accent-blue);
}

.filter-search {
  position: relative;
  flex: 1;
  min-width: 160px;
}

.filter-search .filter-label {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  z-index: 1;
  color: var(--text-muted);
  font-size: 13px;
}

.filter-search .filter-input {
  padding-left: 28px;
  width: 100%;
}

.filter-clear {
  position: absolute;
  right: 6px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px 4px;
  font-size: 14px;
}

.filter-clear:hover {
  color: var(--text-primary);
}

.filter-toggle .toggle-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 13px;
  text-transform: none;
  letter-spacing: normal;
  font-weight: 400;
  color: var(--text-primary);
  padding-top: 16px;
}

.filter-toggle input[type="checkbox"] {
  accent-color: var(--accent-blue);
}

.filter-result-count {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  padding-top: 16px;
}

/* ─── Timeline ───────────────────────────────────────────────────────── */
.events-timeline {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--bg-primary);
}

.events-empty-stream {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-muted);
}

.events-empty-stream i {
  font-size: 40px;
  opacity: 0.4;
  animation: listen-pulse 2s ease-in-out infinite;
}

@keyframes listen-pulse {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 0.7; }
}

.events-empty-stream p {
  font-size: 14px;
}

/* ─── Event Row ──────────────────────────────────────────────────────── */
.event-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-color);
  transition: background 0.1s;
  min-height: 44px;
}

.event-row:last-child {
  border-bottom: none;
}

.event-row--clickable {
  cursor: pointer;
}

.event-row--clickable:hover {
  background: var(--bg-secondary);
}

.event-type-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-size: 15px;
  flex-shrink: 0;
}

.event-action-col {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 150px;
  flex-shrink: 0;
}

.event-action-icon {
  font-size: 14px;
  width: 18px;
  text-align: center;
}

.event-action-badge {
  font-size: 12px;
  font-weight: 600;
  font-family: ui-monospace, 'SFMono-Regular', 'SF Mono', Menlo, Consolas, monospace;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  white-space: nowrap;
}

.event-actor {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.event-actor-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.event-actor-id {
  display: flex;
  align-items: center;
  gap: 4px;
}

.event-actor-id code {
  font-size: 11px;
  color: var(--text-muted);
  font-family: ui-monospace, 'SFMono-Regular', 'SF Mono', Menlo, Consolas, monospace;
}

.btn-copy-id {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 1px 3px;
  font-size: 11px;
  border-radius: 3px;
  opacity: 0;
  transition: opacity 0.1s, color 0.1s;
}

.event-row:hover .btn-copy-id {
  opacity: 1;
}

.btn-copy-id:hover {
  color: var(--accent-blue);
  background: var(--bg-secondary);
}

.btn-copy-id--copied {
  opacity: 1 !important;
  color: var(--accent-green) !important;
}

.event-time {
  flex-shrink: 0;
  min-width: 70px;
  text-align: right;
}

.event-time-relative {
  font-size: 12px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.event-time-empty {
  font-size: 12px;
  color: var(--text-muted);
  opacity: 0.4;
}
</style>
