<!-- ItzamBox — Command Palette (Ctrl+K)
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useDocker } from '../../composables/useDocker'
import { useTheme } from '../../composables/useTheme'
import { useNotifications } from '../../composables/useNotifications'

const router = useRouter()
const visible = ref(false)
const query = ref('')
const selectedIdx = ref(0)
const { containers, images, fetchContainers, fetchImages } = useDocker()
const { toggleTheme, isDark } = useTheme()
const { success, error, info, warning } = useNotifications()

// ── Executable action functions ──────────────────────────────────

async function pullImageAction() {
  const name = prompt('Image name to pull (e.g., nginx:latest):')
  if (!name || !name.trim()) return
  const imageName = name.trim()
  info('Pulling image…', `Downloading ${imageName}`)
  try {
    await invoke('pull_image', { imageName })
    await fetchImages()
    success('Image pulled', `${imageName} downloaded successfully`)
  } catch (e: any) {
    error('Pull failed', e?.toString?.() || String(e))
  }
}

async function pruneAllAction() {
  if (!confirm('This will remove all unused containers, images, volumes, and networks. Continue?')) return
  info('Pruning…', 'Cleaning up unused Docker resources')
  let summary: string[] = []
  try {
    const c = await invoke<number>('prune_containers')
    if (c > 0) summary.push(`${c} container(s)`)
  } catch (e: any) { warning('Prune containers', e?.toString?.() || String(e)) }
  try {
    const i = await invoke<number>('prune_images', { danglingOnly: false })
    if (i > 0) summary.push(`${i} image(s)`)
  } catch (e: any) { warning('Prune images', e?.toString?.() || String(e)) }
  try {
    const v = await invoke<number>('prune_volumes')
    if (v > 0) summary.push(`${v} volume(s)`)
  } catch (e: any) { warning('Prune volumes', e?.toString?.() || String(e)) }
  try {
    const n = await invoke<number>('prune_networks')
    if (n > 0) summary.push(`${n} network(s)`)
  } catch (e: any) { warning('Prune networks', e?.toString?.() || String(e)) }
  if (summary.length) {
    success('Prune complete', `Reclaimed: ${summary.join(', ')}`)
  } else {
    info('Prune complete', 'Nothing to prune — system already clean')
  }
  await fetchContainers(true)
}

async function startAllStoppedAction() {
  const stopped = containers.value.filter(c => c.state === 'exited')
  if (!stopped.length) {
    info('No stopped containers', 'All containers are already running')
    return
  }
  const total = stopped.length
  let ok = 0
  info('Starting containers…', `Restoring ${total} stopped container(s)`)
  for (const c of stopped) {
    try {
      await invoke('start_container', { id: c.id })
      ok++
    } catch {
      /* individual failure — continue with next */
    }
  }
  await fetchContainers(true)
  if (ok === total) {
    success('All containers started', `${total} container(s) are now running`)
  } else {
    warning('Partial start', `${ok}/${total} container(s) started; check logs for details`)
  }
}

function executeItem(item: PaletteItem) {
  if (item.action) item.action()
  else if (item.to) router.push(item.to)
}

function toggleThemeAction() {
  toggleTheme()
  success('Theme toggled', isDark.value ? 'Switched to dark mode' : 'Switched to light mode')
}

onMounted(async () => {
  await Promise.all([fetchContainers(true), fetchImages()])
  window.addEventListener('keydown', onKeyDown)
})
onUnmounted(() => window.removeEventListener('keydown', onKeyDown))

function onKeyDown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') { e.preventDefault(); visible.value = !visible.value; query.value = '' }
  if (!visible.value) return
  if (e.key === 'Escape') { visible.value = false }
  else if (e.key === 'ArrowDown') { e.preventDefault(); selectedIdx.value = Math.min(selectedIdx.value + 1, results.value.length - 1) }
  else if (e.key === 'ArrowUp') { e.preventDefault(); selectedIdx.value = Math.max(selectedIdx.value - 1, 0) }
  else if (e.key === 'Enter') {
    const r = results.value[selectedIdx.value]
    if (r) {
      if (r.action) { r.action(); visible.value = false; query.value = '' }
      else if (r.to) { router.push(r.to); visible.value = false; query.value = '' }
    }
  }
}

interface PaletteItem {
  label: string
  meta?: string
  icon: string
  to?: string
  action?: () => void
  group: string
}

const results = computed(() => {
  const q = query.value.toLowerCase()
  if (!q) return []
  const items: PaletteItem[] = []
  for (const c of containers.value) {
    if (c.name.toLowerCase().includes(q) || c.image.toLowerCase().includes(q))
      items.push({ label: c.name, meta: c.image, icon: 'fa-cube', to: '/containers', group: 'Containers' })
  }
  for (const img of images.value) {
    if (img.repository.toLowerCase().includes(q) || img.tag.toLowerCase().includes(q))
      items.push({ label: img.repository, meta: img.tag, icon: 'fa-layer-group', to: '/images', group: 'Images' })
  }
  // Actions & Navigation
  const actions: PaletteItem[] = [
    // ── Docker Actions ──
    { label: 'Pull Image', meta: 'Download from registry', icon: 'fa-cloud-arrow-down', group: 'Docker Actions', action: pullImageAction },
    { label: 'Prune All', meta: 'Clean unused containers, images, volumes, networks', icon: 'fa-broom', group: 'Docker Actions', action: pruneAllAction },
    { label: 'Start All Stopped', meta: 'Restart all exited containers', icon: 'fa-play', group: 'Docker Actions', action: startAllStoppedAction },
    // ── Navigation ──
    { label: 'Dashboard', meta: 'Go to Dashboard', icon: 'fa-chart-line', group: 'Navigation', to: '/' },
    { label: 'Containers', meta: 'View all containers', icon: 'fa-cubes', group: 'Navigation', to: '/containers' },
    { label: 'Images', meta: 'View all images', icon: 'fa-layer-group', group: 'Navigation', to: '/images' },
    { label: 'Volumes', meta: 'View all volumes', icon: 'fa-database', group: 'Navigation', to: '/volumes' },
    { label: 'Networks', meta: 'View all networks', icon: 'fa-network-wired', group: 'Navigation', to: '/networks' },
    { label: 'Settings', meta: 'App preferences', icon: 'fa-gear', group: 'Navigation', to: '/settings' },
    // ── App ──
    { label: 'Toggle Theme', meta: 'Dark/Light mode', icon: 'fa-moon', group: 'App', action: toggleThemeAction },
  ]
  for (const a of actions) {
    if (a.label.toLowerCase().includes(q) || a.meta?.toLowerCase().includes(q))
      items.push(a)
  }
  return items
})

watch(visible, (v) => { if (v) { selectedIdx.value = 0; if (!containers.value.length) fetchContainers(true) } })
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-backdrop" @click="visible = false" style="z-index:250;align-items:flex-start;padding-top:120px">
      <div class="palette" @click.stop :style="{ width:'560px', background:'var(--bg-secondary)', border:'1px solid var(--border-color)', borderRadius:'var(--radius-lg)', boxShadow:'var(--shadow-lg), 0 0 60px rgba(0,229,255,0.05)', overflow:'hidden', animation:'ctxEnter 0.15s ease-out' }">
        <div style="display:flex;align-items:center;gap:12px;padding:14px 18px;border-bottom:1px solid var(--border-color)">
          <i class="fa-solid fa-magnifying-glass" style="color:var(--accent-cyan)"></i>
          <input v-model="query" placeholder="Search containers, images, actions..." ref="searchInput"
            style="flex:1;background:none;border:none;outline:none;color:var(--text-main);font-size:14px;font-family:var(--font-sans)"
            @keydown.stop>
          <kbd style="font-size:10px;background:var(--bg-tertiary);border:1px solid var(--border-color);border-radius:3px;padding:2px 6px;font-family:var(--font-mono);color:var(--text-muted)">Esc</kbd>
        </div>
        <div v-if="results.length" style="max-height:360px;overflow-y:auto;padding:6px">
          <template v-for="(r, i) in results" :key="i">
            <div v-if="i === 0 || results[i-1].group !== r.group"
              style="font-size:10px;font-weight:600;color:var(--text-disabled);text-transform:uppercase;letter-spacing:.05em;padding:8px 10px 4px">
              {{ r.group }}
            </div>
            <div :class="['palette-item', { active: i === selectedIdx }]"
              @click="executeItem(r); visible = false; query = ''"
              @mouseenter="selectedIdx = i"
              style="display:flex;align-items:center;gap:10px;padding:7px 10px;border-radius:4px;cursor:pointer;font-size:13px;"
              :style="{ background: i === selectedIdx ? 'var(--bg-hover)' : 'transparent' }">
              <i :class="'fa-solid ' + r.icon" style="width:18px;text-align:center;font-size:13px;color:var(--text-muted)"></i>
              <span style="flex:1;font-weight:500">{{ r.label }}</span>
              <span v-if="r.meta" style="font-size:11px;color:var(--text-muted);font-family:var(--font-mono)">{{ r.meta }}</span>
            </div>
          </template>
        </div>
        <div v-else style="padding:30px;text-align:center;color:var(--text-muted);font-size:13px">
          <span v-if="!query">Start typing to search...</span>
          <span v-else>No results for "{{ query }}"</span>
        </div>
        <div style="display:flex;align-items:center;justify-content:space-between;padding:6px 14px;border-top:1px solid var(--border-light);font-size:10px;color:var(--text-disabled)">
          <span><kbd style="font-size:9px;background:var(--bg-tertiary);border:1px solid var(--border-color);border-radius:2px;padding:1px 4px;font-family:var(--font-mono)">↑↓</kbd> Navigate  <kbd style="font-size:9px;background:var(--bg-tertiary);border:1px solid var(--border-color);border-radius:2px;padding:1px 4px;font-family:var(--font-mono);margin-left:6px">↵</kbd> Select  <kbd style="font-size:9px;background:var(--bg-tertiary);border:1px solid var(--border-color);border-radius:2px;padding:1px 4px;font-family:var(--font-mono);margin-left:6px">Esc</kbd> Close</span>
          <span>ItzamBox v1.0</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>
