<!-- ItzamBox — Swarm Node/Service Inspector Drawer
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, watch } from 'vue'
import type { SwarmNode, SwarmService } from '../../composables/useSwarm'

const props = defineProps<{
  node: SwarmNode | null
  service: SwarmService | null
  nodeInspect: Record<string, unknown> | null
  serviceInspect: Record<string, unknown> | null
}>()

defineEmits<{
  (e: 'close'): void
}>()

const activeTab = ref<'details' | 'resources' | 'labels' | 'json'>('details')

watch(
  () => [props.node?.id, props.service?.id],
  () => {
    activeTab.value = 'details'
  },
)

function isNodeMode(): boolean {
  return props.node !== null
}

function getEntityName(): string {
  if (props.node) return props.node.hostname
  if (props.service) return props.service.name
  return ''
}

function getEntityType(): string {
  if (props.node) return 'Node'
  if (props.service) return 'Service'
  return ''
}

function formatJSON(data: Record<string, unknown> | null): string {
  if (!data) return '{}'
  try {
    return JSON.stringify(data, null, 2)
  } catch {
    return '{}'
  }
}

function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null || bytes <= 0) return '—'
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB']
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const val = bytes / Math.pow(1024, i)
  return val.toFixed(1) + ' ' + units[i]
}
</script>

<template>
  <Teleport to="body">
    <div v-if="node || service" class="drawer-overlay" @click.self="$emit('close')"></div>
    <aside
      v-if="node || service"
      class="drawer"
      role="dialog"
      :aria-label="'Inspector: ' + getEntityName()"
    >
      <div class="drawer__header">
        <div>
          <div class="drawer__title">
            <i
              :class="isNodeMode() ? 'fa-solid fa-server' : 'fa-solid fa-diagram-project'"
              style="margin-right: 8px; color: var(--accent-cyan)"
            ></i>
            {{ getEntityName() }}
          </div>
          <div class="drawer__subtitle">
            <span class="badge" :class="isNodeMode() ? 'badge--manager' : 'badge--replicated'">
              {{ getEntityType() }}
            </span>
            <span v-if="node" style="margin-left: 8px; font-family: var(--font-mono); font-size: 11px">
              {{ node.ip_address }}
            </span>
          </div>
        </div>
        <button class="drawer__close" aria-label="Close inspector" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Tabs -->
      <div class="drawer__tabs" role="tablist">
        <button
          class="drawer__tab"
          :class="{ active: activeTab === 'details' }"
          role="tab"
          :aria-selected="activeTab === 'details'"
          @click="activeTab = 'details'"
        >
          Details
        </button>
        <button
          v-if="isNodeMode()"
          class="drawer__tab"
          :class="{ active: activeTab === 'resources' }"
          role="tab"
          :aria-selected="activeTab === 'resources'"
          @click="activeTab = 'resources'"
        >
          Resources
        </button>
        <button
          v-if="isNodeMode()"
          class="drawer__tab"
          :class="{ active: activeTab === 'labels' }"
          role="tab"
          :aria-selected="activeTab === 'labels'"
          @click="activeTab = 'labels'"
        >
          Labels
        </button>
        <button
          class="drawer__tab"
          :class="{ active: activeTab === 'json' }"
          role="tab"
          :aria-selected="activeTab === 'json'"
          @click="activeTab = 'json'"
        >
          JSON
        </button>
      </div>

      <!-- Body -->
      <div class="drawer__body">
        <!-- Details Tab (Node) -->
        <div v-if="activeTab === 'details' && node" class="inspector-panel">
          <div class="detail-grid">
            <span class="detail-grid__label">ID</span>
            <span class="detail-grid__value text-mono">{{ node.id.slice(0, 24) }}...</span>

            <span class="detail-grid__label">Hostname</span>
            <span class="detail-grid__value">{{ node.hostname }}</span>

            <span class="detail-grid__label">Role</span>
            <span class="detail-grid__value" :style="{ color: node.role === 'Manager' ? 'var(--accent-cyan)' : 'var(--text-muted)' }">
              {{ node.role }}
            </span>

            <span class="detail-grid__label">Status</span>
            <span class="detail-grid__value" :style="{ color: node.status === 'Ready' ? 'var(--accent-green)' : 'var(--accent-red)' }">
              <span class="status-dot" :class="node.status === 'Ready' ? 'status-dot--running' : 'status-dot--error'"></span>
              {{ node.status }}
            </span>

            <span class="detail-grid__label">Availability</span>
            <span class="detail-grid__value">{{ node.availability }}</span>

            <span class="detail-grid__label">Engine Version</span>
            <span class="detail-grid__value text-mono">{{ node.engine_version || '—' }}</span>

            <span class="detail-grid__label">IP Address</span>
            <span class="detail-grid__value text-mono">{{ node.ip_address || '—' }}</span>
          </div>
        </div>

        <!-- Details Tab (Service) -->
        <div v-if="activeTab === 'details' && service" class="inspector-panel">
          <div class="detail-grid">
            <span class="detail-grid__label">ID</span>
            <span class="detail-grid__value text-mono">{{ service.id.slice(0, 24) }}...</span>

            <span class="detail-grid__label">Name</span>
            <span class="detail-grid__value">{{ service.name }}</span>

            <span class="detail-grid__label">Mode</span>
            <span class="detail-grid__value">
              <span :class="['badge', service.mode === 'Global' ? 'badge--global' : 'badge--replicated']">
                {{ service.mode }}
              </span>
            </span>

            <span class="detail-grid__label">Replicas</span>
            <span class="detail-grid__value">
              <span :class="['replicas', service.replicas]">{{ service.replicas }}</span>
            </span>

            <span class="detail-grid__label">Image</span>
            <span class="detail-grid__value text-mono" style="font-size: 12px">{{ service.image }}</span>

            <span class="detail-grid__label">Ports</span>
            <span class="detail-grid__value text-mono" style="font-size: 12px">
              {{ service.ports.length > 0 ? service.ports.join(', ') : 'Internal only' }}
            </span>
          </div>
        </div>

        <!-- Resources Tab (Node only) -->
        <div v-if="activeTab === 'resources' && node" class="inspector-panel">
          <div class="detail-grid">
            <span class="detail-grid__label">CPU Cores</span>
            <span class="detail-grid__value">{{ node.cpu_cores ?? '—' }}</span>

            <span class="detail-grid__label">Memory</span>
            <span class="detail-grid__value">{{ formatBytes(node.memory_bytes) }}</span>
          </div>
        </div>

        <!-- Labels Tab (Node only) -->
        <div v-if="activeTab === 'labels' && node" class="inspector-panel">
          <div v-if="Object.keys(node.labels).length === 0" class="inspector-empty">
            <i class="fa-solid fa-tag" style="font-size: 20px; color: var(--text-disabled)"></i>
            <p style="font-size: 12px; color: var(--text-muted); margin-top: 8px">No labels assigned</p>
          </div>
          <div v-else class="labels-grid">
            <div v-for="(val, key) in node.labels" :key="key" class="label-chip">
              <span class="label-chip__key">{{ key }}</span>
              <span class="label-chip__eq">=</span>
              <span class="label-chip__val">{{ val }}</span>
            </div>
          </div>
        </div>

        <!-- JSON Tab -->
        <div v-if="activeTab === 'json'" class="inspector-panel">
          <pre class="json-viewer">{{ isNodeMode() ? formatJSON(nodeInspect) : formatJSON(serviceInspect) }}</pre>
        </div>
      </div>
    </aside>
  </Teleport>
</template>

<style scoped>
.drawer-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 90;
}

.drawer {
  position: fixed;
  top: 0;
  right: 0;
  width: 420px;
  height: 100vh;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
  box-shadow: var(--shadow-lg);
  z-index: 100;
  display: flex;
  flex-direction: column;
  animation: slideInRight 0.2s cubic-bezier(0, 0, 0.2, 1);
}

@keyframes slideInRight {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

.drawer__header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-light);
}

.drawer__title {
  font-size: 15px;
  font-weight: 600;
}

.drawer__subtitle {
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-muted);
}

.drawer__close {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  transition: all var(--transition-fast);
}

.drawer__close:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

.drawer__tabs {
  display: flex;
  border-bottom: 1px solid var(--border-light);
  padding: 0 16px;
  gap: 0;
}

.drawer__tab {
  padding: 10px 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  border: none;
  background: none;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
  font-family: var(--font-sans);
}

.drawer__tab:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.drawer__tab.active {
  color: var(--accent-cyan);
  border-bottom-color: var(--accent-cyan);
}

.drawer__body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.inspector-panel {
  animation: fadeIn 0.15s ease-out;
}

.inspector-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px;
}

.detail-grid {
  display: grid;
  grid-template-columns: 120px 1fr;
  gap: 3px 16px;
  font-size: 13px;
}

.detail-grid__label {
  color: var(--text-disabled);
  font-size: 11px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  padding: 6px 0;
}

.detail-grid__value {
  padding: 6px 0;
  display: flex;
  align-items: center;
  gap: 6px;
}

.labels-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.label-chip {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 11px;
  font-family: var(--font-mono);
  gap: 2px;
}

.label-chip__key {
  color: var(--accent-cyan);
}

.label-chip__eq {
  color: var(--text-disabled);
}

.label-chip__val {
  color: var(--text-main);
}

.json-viewer {
  background: var(--terminal-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 16px;
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.6;
  color: var(--terminal-fg);
  overflow-x: auto;
  white-space: pre;
  max-height: 60vh;
  overflow-y: auto;
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

.badge--replicated {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 229, 255, 0.2);
}

.badge--global {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.replicas {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 600;
}
</style>
