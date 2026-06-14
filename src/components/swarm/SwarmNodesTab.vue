<!-- ItzamBox — Swarm Nodes Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref } from 'vue'
import type { SwarmNode } from '../../composables/useSwarm'

defineProps<{
  nodes: SwarmNode[]
  loading: boolean
}>()

defineEmits<{
  (e: 'inspect', nodeId: string): void
  (e: 'refresh'): void
}>()

const filter = ref('')

function filteredNodes(nodes: SwarmNode[]): SwarmNode[] {
  if (!filter.value.trim()) return nodes
  const q = filter.value.toLowerCase()
  return nodes.filter(
    n =>
      n.hostname.toLowerCase().includes(q) ||
      n.role.toLowerCase().includes(q) ||
      n.status.toLowerCase().includes(q) ||
      n.ip_address.includes(q),
  )
}

function statusDotClass(status: string): string {
  switch (status) {
    case 'Ready':
      return 'status-dot--running'
    case 'Down':
      return 'status-dot--error'
    default:
      return 'status-dot--stopped'
  }
}

function availabilityPillClass(avail: string): string {
  switch (avail) {
    case 'Active':
      return 'pill-active'
    case 'Pause':
      return 'pill-pause'
    case 'Drain':
      return 'pill-drain'
    default:
      return 'pill-drain'
  }
}

function safeEngineVersion(v: string): string {
  return v && v !== '' ? v : '—'
}

function safeIp(ip: string): string {
  return ip && ip !== '' ? ip : '—'
}
</script>

<template>
  <div class="swarm-table-section">
    <div class="table-toolbar">
      <span class="section-title" style="font-size: 14px">
        <i class="fa-solid fa-server" style="margin-right: 6px; color: var(--accent-cyan)"></i>
        Cluster Nodes
      </span>
      <div class="table-toolbar__actions">
        <div class="table-filter">
          <i class="fa-solid fa-magnifying-glass"></i>
          <input
            v-model="filter"
            type="text"
            placeholder="Filter nodes..."
            aria-label="Filter nodes by name, role, or IP"
          />
        </div>
        <button class="btn btn-ghost btn-sm" aria-label="Refresh nodes" @click="$emit('refresh')">
          <i class="fa-solid fa-arrows-rotate" :class="{ 'fa-spin': loading }"></i>
        </button>
      </div>
    </div>

    <!-- Skeleton loading -->
    <template v-if="loading && nodes.length === 0">
      <div v-for="i in 4" :key="i" class="data-row skeleton-row">
        <div class="skeleton skeleton-dot" style="margin: 0 8px"></div>
        <div class="row-info">
          <div class="skeleton skeleton-text-md" style="width: 140px"></div>
        </div>
        <div class="skeleton skeleton-tag"></div>
        <div class="skeleton skeleton-tag"></div>
        <div class="skeleton skeleton-text-xs"></div>
        <div class="skeleton skeleton-text-xs"></div>
      </div>
    </template>

    <!-- Empty state -->
    <div v-else-if="nodes.length === 0" class="empty-state" style="padding: 32px">
      <div class="empty-state-icon" style="width: 56px; height: 56px; font-size: 24px">
        <i class="fa-solid fa-server"></i>
      </div>
      <p class="empty-state-title">No nodes found</p>
      <p class="empty-state-desc">Swarm is active but no nodes are registered yet.</p>
    </div>

    <!-- Table -->
    <div v-else class="swarm-table">
      <div class="col-headers">
        <span class="col-header" style="flex: 2">Hostname</span>
        <span class="col-header" style="flex: 1">Role</span>
        <span class="col-header" style="flex: 1">Status</span>
        <span class="col-header" style="flex: 1.2">Availability</span>
        <span class="col-header" style="flex: 1.2">Engine Version</span>
        <span class="col-header" style="flex: 1.2">IP Address</span>
      </div>

      <div
        v-for="node in filteredNodes(nodes)"
        :key="node.id"
        class="data-row"
        :class="{ 'skeleton-row': loading }"
        @click="$emit('inspect', node.id)"
        role="button"
        :tabindex="0"
        :aria-label="'Inspect node ' + node.hostname"
        @keydown.enter="$emit('inspect', node.id)"
      >
        <div class="row-info" style="flex: 2">
          <div class="row-name">
            <i
              v-if="node.role === 'Manager'"
              class="fa-solid fa-star"
              style="color: var(--accent-cyan); font-size: 10px; margin-right: 4px"
              aria-label="Manager node"
            ></i>
            {{ node.hostname }}
          </div>
          <div class="row-meta">{{ node.id.slice(0, 12) }}...</div>
        </div>

        <div style="flex: 1">
          <span :class="['badge', node.role === 'Manager' ? 'badge--manager' : 'badge--worker']">
            <i v-if="node.role === 'Manager'" class="fa-solid fa-star" style="font-size: 8px; margin-right: 2px"></i>
            {{ node.role }}
          </span>
        </div>

        <div style="flex: 1; display: flex; align-items: center; gap: 6px">
          <span :class="['status-dot', statusDotClass(node.status)]"></span>
          <span style="font-size: 13px">
            {{ node.status === 'Unknown' ? '—' : node.status }}
          </span>
        </div>

        <div style="flex: 1.2">
          <span :class="['availability-pill', availabilityPillClass(node.availability)]">
            {{ node.availability }}
          </span>
        </div>

        <div style="flex: 1.2; font-size: 12px" class="text-mono">
          {{ safeEngineVersion(node.engine_version) }}
        </div>

        <div style="flex: 1.2; font-size: 12px" class="text-mono">
          {{ safeIp(node.ip_address) }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.swarm-table-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border-light);
}

.table-toolbar__actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.badge--manager {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 229, 255, 0.2);
}

.badge--worker {
  background: rgba(156, 163, 175, 0.08);
  color: var(--text-muted);
  border: 1px solid rgba(156, 163, 175, 0.15);
}

.availability-pill {
  display: inline-flex;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}

.pill-active {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
}

.pill-pause {
  background: rgba(245, 158, 11, 0.1);
  color: var(--accent-yellow);
}

.pill-drain {
  background: rgba(107, 114, 128, 0.08);
  color: var(--text-muted);
}
</style>
