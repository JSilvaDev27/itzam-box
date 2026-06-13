<!-- ItzamBox — Cleanup View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface DiskUsage { containers_count: number; containers_size_bytes: number; images_count: number; images_size_bytes: number; volumes_count: number; volumes_size_bytes: number; build_cache_size_bytes: number; total_reclaimable_bytes: number }
const usage = ref<DiskUsage | null>(null)

onMounted(fetchUsage)
async function fetchUsage() { try { usage.value = await invoke<DiskUsage>('get_disk_usage') } catch(e: any) { alert(e.toString()) } }
function fmt(b: number) { if (b > 1e9) return (b/1e9).toFixed(1)+' GB'; return Math.round(b/1e6)+' MB' }
async function prune(type: string) {
  if (!confirm(`Prune ${type}? This cannot be undone.`)) return
  try { await invoke(`prune_${type}`); await fetchUsage() } catch(e: any) { alert(e.toString()) }
}
</script>
<template>
  <div class="breadcrumb"><i class="fa-solid fa-house"></i> <span>Home</span> <i class="fa-solid fa-chevron-right"></i> <span class="current">Cleanup</span></div>
  <h1 class="text-h1">System Cleanup</h1>
  <div class="metrics-grid" v-if="usage">
    <div class="metric-card">
      <div class="metric-icon cyan"><i class="fa-solid fa-cubes"></i></div>
      <div class="metric-label">Containers</div>
      <div class="metric-value" style="font-size:1.2rem">{{ usage.containers_count }}</div>
      <div class="metric-delta">{{ fmt(usage.containers_size_bytes) }}</div>
      <button class="btn btn-secondary btn-sm" @click="prune('containers')"><i class="fa-solid fa-broom"></i> Prune</button>
    </div>
    <div class="metric-card">
      <div class="metric-icon purple"><i class="fa-solid fa-layer-group"></i></div>
      <div class="metric-label">Images</div>
      <div class="metric-value" style="font-size:1.2rem">{{ usage.images_count }}</div>
      <div class="metric-delta">{{ fmt(usage.images_size_bytes) }}</div>
      <button class="btn btn-secondary btn-sm" @click="prune('images')"><i class="fa-solid fa-broom"></i> Prune</button>
    </div>
    <div class="metric-card">
      <div class="metric-icon purple"><i class="fa-solid fa-database"></i></div>
      <div class="metric-label">Volumes</div>
      <div class="metric-value" style="font-size:1.2rem">{{ usage.volumes_count }}</div>
      <div class="metric-delta">{{ fmt(usage.volumes_size_bytes) }}</div>
      <button class="btn btn-secondary btn-sm" @click="prune('volumes')"><i class="fa-solid fa-broom"></i> Prune</button>
    </div>
    <div class="metric-card" style="border-color:rgba(239,68,68,0.3)">
      <div class="metric-icon" style="background:rgba(239,68,68,0.1);color:var(--accent-red)"><i class="fa-solid fa-recycle"></i></div>
      <div class="metric-label">Reclaimable</div>
      <div class="metric-value" style="font-size:1.5rem;color:var(--accent-red)">{{ fmt(usage.total_reclaimable_bytes) }}</div>
      <button class="btn btn-danger btn-sm" @click="prune('containers');prune('images');prune('volumes');prune('networks')"><i class="fa-solid fa-broom"></i> Prune All</button>
    </div>
  </div>
  <div v-else class="section" style="padding:40px;text-align:center"><span style="color:var(--text-muted)">Loading disk usage...</span></div>
</template>
