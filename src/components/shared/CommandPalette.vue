<!-- ItzamBox — Command Palette (Ctrl+K)
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useDocker } from '../../composables/useDocker'

const router = useRouter()
const visible = ref(false)
const query = ref('')
const selectedIdx = ref(0)
const { containers, images, fetchContainers, fetchImages } = useDocker()

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
  else if (e.key === 'Enter') { const r = results.value[selectedIdx.value]; if (r) { if (r.to) router.push(r.to); visible.value = false; query.value = '' } }
}

const results = computed(() => {
  const q = query.value.toLowerCase()
  if (!q) return []
  const items: { label: string; meta?: string; icon: string; to?: string; group: string }[] = []
  for (const c of containers.value) {
    if (c.name.toLowerCase().includes(q) || c.image.toLowerCase().includes(q))
      items.push({ label: c.name, meta: c.image, icon: 'fa-cube', to: '/containers', group: 'Containers' })
  }
  for (const img of images.value) {
    if (img.repository.toLowerCase().includes(q) || img.tag.toLowerCase().includes(q))
      items.push({ label: img.repository, meta: img.tag, icon: 'fa-layer-group', to: '/images', group: 'Images' })
  }
  // Actions
  const actions = [
    { label: 'Pull Image', meta: 'Download from registry', icon: 'fa-cloud-arrow-down', group: 'Actions' },
    { label: 'System Prune', meta: 'Clean up unused resources', icon: 'fa-broom', to: '/cleanup', group: 'Actions' },
    { label: 'Open Terminal', meta: 'New host terminal', icon: 'fa-terminal', group: 'Actions' },
    { label: 'Dashboard', meta: 'Go to Dashboard', icon: 'fa-chart-line', to: '/', group: 'Navigation' },
    { label: 'Containers', meta: 'View all containers', icon: 'fa-cubes', to: '/containers', group: 'Navigation' },
    { label: 'Images', meta: 'View all images', icon: 'fa-layer-group', to: '/images', group: 'Navigation' },
    { label: 'Settings', meta: 'App preferences', icon: 'fa-gear', to: '/settings', group: 'Navigation' },
    { label: 'Toggle Theme', meta: 'Dark/Light mode', icon: 'fa-moon', group: 'Actions' },
  ]
  for (const a of actions) {
    if (a.label.toLowerCase().includes(q) || a.meta?.toLowerCase().includes(q))
      items.push(a)
  }
  return items
})

watch(visible, (v) => { if (v) { selectedIdx.value = 0; if (!containers.value.length) fetchContainers(true) } })
import { watch } from 'vue'
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
              @click="r.to ? router.push(r.to) : null; visible = false; query = ''"
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
