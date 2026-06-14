<!-- ItzamBox — K8s Context & Namespace Toolbar
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { K8sContext, ConnectionStatus } from '../../composables/useKubernetes'

const props = defineProps<{
  contexts: K8sContext[]
  activeContext: string | null
  namespaces: string[]
  activeNamespace: string | null
  connectionStatus: ConnectionStatus
}>()

const emit = defineEmits<{
  'switch-context': [context: string]
  'switch-namespace': [namespace: string | null]
  'refresh': []
  'retry': []
}>()

const contextOpen = ref(false)
const namespaceOpen = ref(false)
const namespaceSearch = ref('')
const contextSearch = ref('')

const filteredNamespaces = computed(() => {
  if (!namespaceSearch.value) return props.namespaces
  const q = namespaceSearch.value.toLowerCase()
  return props.namespaces.filter(ns => ns.toLowerCase().includes(q))
})

const filteredContexts = computed(() => {
  if (!contextSearch.value) return props.contexts
  const q = contextSearch.value.toLowerCase()
  return props.contexts.filter(c => c.name.toLowerCase().includes(q) || c.cluster.toLowerCase().includes(q))
})

const statusInfo = computed(() => {
  const map: Record<ConnectionStatus, { color: string; label: string; dot: string }> = {
    connected: { color: 'var(--accent-green)', label: 'Connected', dot: '●' },
    loading: { color: 'var(--accent-yellow)', label: 'Loading', dot: '●' },
    offline: { color: 'var(--accent-red)', label: 'Offline', dot: '●' },
    'no-kubectl': { color: 'var(--accent-red)', label: 'kubectl not found', dot: '●' },
    'no-kubeconfig': { color: 'var(--accent-red)', label: 'No kubeconfig', dot: '●' },
  }
  return map[props.connectionStatus] ?? map.offline
})

function selectContext(ctx: string) {
  emit('switch-context', ctx)
  contextOpen.value = false
  contextSearch.value = ''
}

function selectNamespace(ns: string | null) {
  emit('switch-namespace', ns)
  namespaceOpen.value = false
  namespaceSearch.value = ''
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.k8s-toolbar')) {
    contextOpen.value = false
    namespaceOpen.value = false
  }
}

// Close on Escape
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    contextOpen.value = false
    namespaceOpen.value = false
  }
}

// Global listeners
if (typeof window !== 'undefined') {
  window.addEventListener('click', handleClickOutside)
  window.addEventListener('keydown', handleKeydown)
}
</script>

<template>
  <div class="k8s-toolbar" @click.stop>
    <!-- Connection Status Badge -->
    <div class="k8s-connection-badge" :title="`Cluster: ${statusInfo.label}`">
      <span class="k8s-status-dot" :style="{ color: statusInfo.color }">{{ statusInfo.dot }}</span>
      <span class="k8s-status-label">{{ statusInfo.label }}</span>
    </div>

    <!-- Context Selector -->
    <div class="k8s-selector-wrapper" v-click-outside="() => { contextOpen = false }">
      <button class="k8s-selector-btn" @click.stop="contextOpen = !contextOpen" title="Switch Kubernetes context">
        <i class="fa-solid fa-ship"></i>
        <span class="k8s-selector-label">{{ activeContext ?? 'No context' }}</span>
        <i class="fa-solid fa-chevron-down k8s-chevron" :class="{ open: contextOpen }"></i>
      </button>
      <div v-if="contextOpen" class="k8s-dropdown k8s-context-dropdown" @click.stop>
        <div class="k8s-dropdown-header">
          <i class="fa-solid fa-ship"></i>
          <span>Kubernetes Contexts</span>
        </div>
        <div class="k8s-dropdown-search">
          <i class="fa-solid fa-search"></i>
          <input v-model="contextSearch" placeholder="Search contexts..." autofocus />
        </div>
        <div class="k8s-dropdown-list">
          <button
            v-for="ctx in filteredContexts"
            :key="ctx.name"
            class="k8s-dropdown-item"
            :class="{ active: ctx.name === activeContext }"
            @click="selectContext(ctx.name)"
            :title="`Cluster: ${ctx.cluster}\nUser: ${ctx.user}`"
          >
            <i class="fa-solid fa-circle" :style="{ color: ctx.is_active ? 'var(--accent-green)' : 'var(--text-disabled)', fontSize: '8px' }"></i>
            <span class="k8s-dropdown-item-name">{{ ctx.name }}</span>
            <span class="k8s-dropdown-item-meta">{{ ctx.cluster }}</span>
          </button>
          <div v-if="filteredContexts.length === 0" class="k8s-dropdown-empty">No contexts found</div>
        </div>
      </div>
    </div>

    <!-- Namespace Selector -->
    <div class="k8s-selector-wrapper" v-click-outside="() => { namespaceOpen = false }">
      <button class="k8s-selector-btn" @click.stop="namespaceOpen = !namespaceOpen" title="Filter by namespace">
        <i class="fa-solid fa-cubes"></i>
        <span class="k8s-selector-label">{{ activeNamespace ?? 'All Namespaces' }}</span>
        <span v-if="namespaces.length > 0" class="k8s-selector-count">{{ namespaces.length }}</span>
        <i class="fa-solid fa-chevron-down k8s-chevron" :class="{ open: namespaceOpen }"></i>
      </button>
      <div v-if="namespaceOpen" class="k8s-dropdown k8s-ns-dropdown" @click.stop>
        <div class="k8s-dropdown-header">
          <i class="fa-solid fa-cubes"></i>
          <span>Namespaces</span>
        </div>
        <div class="k8s-dropdown-search">
          <i class="fa-solid fa-search"></i>
          <input v-model="namespaceSearch" placeholder="Search namespaces..." autofocus />
        </div>
        <div class="k8s-dropdown-list">
          <button
            class="k8s-dropdown-item"
            :class="{ active: activeNamespace === null }"
            @click="selectNamespace(null)"
          >
            <i class="fa-solid fa-globe"></i>
            <span class="k8s-dropdown-item-name">All Namespaces</span>
          </button>
          <button
            v-for="ns in filteredNamespaces"
            :key="ns"
            class="k8s-dropdown-item"
            :class="{ active: ns === activeNamespace }"
            @click="selectNamespace(ns)"
          >
            <i class="fa-solid fa-cube"></i>
            <span class="k8s-dropdown-item-name">{{ ns }}</span>
          </button>
          <div v-if="filteredNamespaces.length === 0" class="k8s-dropdown-empty">No namespaces found</div>
        </div>
      </div>
    </div>

    <!-- Refresh Button -->
    <button class="k8s-toolbar-action" @click="emit('refresh')" title="Refresh all resources">
      <i class="fa-solid fa-rotate"></i>
    </button>

    <!-- Retry Button (shown on error) -->
    <button
      v-if="connectionStatus === 'offline'"
      class="k8s-toolbar-action k8s-toolbar-action--warn"
      @click="emit('retry')"
      title="Retry connection"
    >
      <i class="fa-solid fa-arrow-rotate-right"></i>
    </button>
  </div>
</template>

<style scoped>
.k8s-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.k8s-connection-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}

.k8s-status-dot {
  font-size: 10px;
  line-height: 1;
}

.k8s-status-label {
  color: var(--text-muted);
}

/* ── Selector ── */
.k8s-selector-wrapper {
  position: relative;
}

.k8s-selector-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
  max-width: 240px;
}

.k8s-selector-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent-cyan);
}

.k8s-selector-btn i:first-child {
  color: var(--accent-cyan);
  font-size: 13px;
}

.k8s-selector-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 140px;
}

.k8s-selector-count {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  background: var(--bg-hover);
  padding: 1px 5px;
  border-radius: 3px;
}

.k8s-chevron {
  font-size: 9px;
  color: var(--text-muted);
  transition: transform var(--transition-fast);
}

.k8s-chevron.open {
  transform: rotate(180deg);
}

/* ── Dropdown ── */
.k8s-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 260px;
  max-height: 360px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  z-index: 50;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: fadeIn 0.15s ease-out;
}

.k8s-context-dropdown {
  min-width: 300px;
}

.k8s-ns-dropdown {
  min-width: 240px;
}

.k8s-dropdown-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.k8s-dropdown-header i {
  color: var(--accent-cyan);
}

.k8s-dropdown-search {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-light);
}

.k8s-dropdown-search i {
  color: var(--text-disabled);
  font-size: 12px;
}

.k8s-dropdown-search input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
}

.k8s-dropdown-search input::placeholder {
  color: var(--text-disabled);
}

.k8s-dropdown-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.k8s-dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: none;
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
  cursor: pointer;
  border-radius: 4px;
  text-align: left;
  transition: background var(--transition-fast);
}

.k8s-dropdown-item:hover {
  background: var(--bg-hover);
}

.k8s-dropdown-item.active {
  background: var(--bg-tertiary);
  color: var(--accent-cyan);
}

.k8s-dropdown-item-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.k8s-dropdown-item-meta {
  font-size: 10px;
  color: var(--text-disabled);
  font-family: var(--font-mono);
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.k8s-dropdown-empty {
  padding: 20px;
  text-align: center;
  color: var(--text-disabled);
  font-size: 12px;
}

/* ── Toolbar Actions ── */
.k8s-toolbar-action {
  width: 30px;
  height: 30px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--bg-tertiary);
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  transition: all var(--transition-fast);
}

.k8s-toolbar-action:hover {
  background: var(--bg-hover);
  color: var(--text-main);
  border-color: var(--accent-cyan);
}

.k8s-toolbar-action--warn {
  color: var(--accent-yellow);
  border-color: rgba(245, 158, 11, 0.3);
}

.k8s-toolbar-action--warn:hover {
  background: rgba(245, 158, 11, 0.1);
  border-color: var(--accent-yellow);
}
</style>
