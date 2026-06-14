<!-- ItzamBox — Kubernetes Inspector Drawer
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ResourceKind, K8sPod, K8sDeployment, K8sService, K8sConfigMap, K8sSecretMeta, K8sContainer, K8sEvent } from '../../composables/useKubernetes'
import YamlViewer from './YamlViewer.vue'
import SecretValueMask from './SecretValueMask.vue'

const props = defineProps<{
  open: boolean
  kind: ResourceKind | null
  name: string | null
  namespace: string | null
  yaml: string | null
  events: K8sEvent[]
  pods?: K8sPod[]
  deployments?: K8sDeployment[]
  services?: K8sService[]
  configmaps?: K8sConfigMap[]
  secrets?: K8sSecretMeta[]
}>()

const emit = defineEmits<{
  'close': []
}>()

const activeInspectorTab = ref<string>('overview')

// Find resource from the lists
const selectedResource = computed(() => {
  if (!props.kind || !props.name) return null
  switch (props.kind) {
    case 'pod': return props.pods?.find(p => p.name === props.name && p.namespace === props.namespace) ?? null
    case 'deployment': return props.deployments?.find(d => d.name === props.name && d.namespace === props.namespace) ?? null
    case 'service': return props.services?.find(s => s.name === props.name && s.namespace === props.namespace) ?? null
    case 'configmap': return props.configmaps?.find(c => c.name === props.name && c.namespace === props.namespace) ?? null
    case 'secret': return props.secrets?.find(s => s.name === props.name && s.namespace === props.namespace) ?? null
    default: return null
  }
})

const inspectorTabs = computed(() => {
  switch (props.kind) {
    case 'pod': return ['Overview', 'Containers', 'Conditions', 'Events', 'YAML']
    case 'deployment': return ['Overview', 'Strategy', 'YAML']
    case 'service': return ['Overview', 'Endpoints', 'YAML']
    case 'configmap': return ['Overview', 'Data', 'YAML']
    case 'secret': return ['Overview', 'Keys', 'YAML']
    default: return ['Overview']
  }
})

const pod = computed(() => selectedResource.value as K8sPod | null)
const deployment = computed(() => selectedResource.value as K8sDeployment | null)
const service = computed(() => selectedResource.value as K8sService | null)
const configmap = computed(() => selectedResource.value as K8sConfigMap | null)
const secret = computed(() => selectedResource.value as K8sSecretMeta | null)

function tabKey(label: string): string {
  return label.toLowerCase().replace(/\s+/g, '-')
}

function setActiveTab(tab: string) {
  activeInspectorTab.value = tabKey(tab)
}

function statusClass(status: string): string {
  const s = status.toLowerCase()
  if (s === 'running') return 'k8s-pill--running'
  if (s.includes('crash') || s === 'error' || s === 'failed') return 'k8s-pill--error'
  if (s === 'pending') return 'k8s-pill--pending'
  return 'k8s-pill--default'
}
</script>

<template>
  <Transition name="drawer-slide">
    <div v-if="open" class="k8s-inspector-overlay" @click.self="emit('close')">
      <div class="k8s-inspector-drawer">
        <!-- Header -->
        <div class="k8s-inspector-header">
          <div class="k8s-inspector-title">
            <i class="fa-solid fa-circle-info"></i>
            <div>
              <div class="k8s-inspector-name">{{ name }}</div>
              <div class="k8s-inspector-meta">
                <span class="k8s-inspector-kind">{{ kind }}</span>
                <span class="k8s-inspector-sep">·</span>
                <span class="k8s-inspector-ns">{{ namespace }}</span>
              </div>
            </div>
          </div>
          <button class="k8s-inspector-close" @click="emit('close')">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>

        <!-- Tabs -->
        <div class="k8s-inspector-tabs">
          <button
            v-for="tab in inspectorTabs"
            :key="tab"
            class="k8s-inspector-tab"
            :class="{ active: activeInspectorTab === tabKey(tab) }"
            @click="setActiveTab(tab)"
          >
            {{ tab }}
          </button>
        </div>

        <!-- Content -->
        <div class="k8s-inspector-body">
          <!-- ── OVERVIEW (Universal) ── -->
          <template v-if="activeInspectorTab === 'overview'">
            <div class="k8s-inspector-section">
              <template v-if="pod">
                <div class="k8s-info-row"><span class="k8s-info-label">Status</span><span :class="['k8s-pill', statusClass(pod.status)]">{{ pod.status }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Node</span><span class="k8s-info-value k8s-info-mono">{{ pod.node }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">IP</span><span class="k8s-info-value k8s-info-mono">{{ pod.ip ?? '—' }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Restarts</span><span class="k8s-info-value">{{ pod.restarts }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Age</span><span class="k8s-info-value">{{ pod.age }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Containers</span><span class="k8s-info-value">{{ pod.containers.length }}</span></div>
              </template>
              <template v-else-if="deployment">
                <div class="k8s-info-row"><span class="k8s-info-label">Ready</span><span class="k8s-info-value">{{ deployment.ready }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Up-to-Date</span><span class="k8s-info-value">{{ deployment.up_to_date }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Available</span><span class="k8s-info-value">{{ deployment.available }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Strategy</span><span class="k8s-info-value">{{ deployment.strategy }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Age</span><span class="k8s-info-value">{{ deployment.age }}</span></div>
              </template>
              <template v-else-if="service">
                <div class="k8s-info-row"><span class="k8s-info-label">Type</span><span class="k8s-info-value">{{ service.service_type }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Cluster IP</span><span class="k8s-info-value k8s-info-mono">{{ service.cluster_ip }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">External IP</span><span class="k8s-info-value k8s-info-mono">{{ service.external_ip ?? '—' }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Ports</span><span class="k8s-info-value"><span v-for="p in service.ports" :key="p" class="k8s-port-chip">{{ p }}</span></span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Age</span><span class="k8s-info-value">{{ service.age }}</span></div>
              </template>
              <template v-else-if="configmap">
                <div class="k8s-info-row"><span class="k8s-info-label">Data Keys</span><span class="k8s-info-value">{{ configmap.keys_count }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Age</span><span class="k8s-info-value">{{ configmap.age }}</span></div>
              </template>
              <template v-else-if="secret">
                <div class="k8s-info-row"><span class="k8s-info-label">Type</span><span class="k8s-info-value">{{ secret.secret_type }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Data Keys</span><span class="k8s-info-value">{{ secret.keys_count }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Age</span><span class="k8s-info-value">{{ secret.age }}</span></div>
              </template>
              <div v-else class="k8s-inspector-empty">No resource selected</div>
            </div>

            <!-- Labels -->
            <div v-if="pod" class="k8s-inspector-section">
              <h4 class="k8s-section-title">Labels</h4>
              <div class="k8s-labels-grid">
                <div v-for="(val, key) in pod.labels" :key="key" class="k8s-label-chip">
                  <span class="k8s-label-key">{{ key }}:</span>
                  <span class="k8s-label-val">{{ val }}</span>
                </div>
                <div v-if="Object.keys(pod.labels).length === 0" class="k8s-info-empty">No labels</div>
              </div>
            </div>
          </template>

          <!-- ── CONTAINERS (Pod only) ── -->
          <template v-if="activeInspectorTab === 'containers'">
            <div v-if="pod && pod.containers.length > 0">
              <div v-for="ctr in pod.containers" :key="ctr.name" class="k8s-inspector-section">
                <h4 class="k8s-section-title">
                  {{ ctr.name }}
                  <span :class="['k8s-container-status', ctr.ready ? 'k8s-container--ready' : 'k8s-container--not-ready']">
                    {{ ctr.ready ? 'Ready' : 'Not Ready' }}
                  </span>
                </h4>
                <div class="k8s-info-row"><span class="k8s-info-label">Image</span><span class="k8s-info-value k8s-info-mono">{{ ctr.image }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Restarts</span><span class="k8s-info-value">{{ ctr.restart_count }}</span></div>
                <div class="k8s-info-row" v-if="ctr.ports.length"><span class="k8s-info-label">Ports</span><span class="k8s-info-value"><span v-for="p in ctr.ports" :key="p" class="k8s-port-chip">{{ p }}</span></span></div>
                <div class="k8s-info-row" v-if="ctr.readiness_probe"><span class="k8s-info-label">Readiness</span><span class="k8s-info-value k8s-info-mono">{{ ctr.readiness_probe }}</span></div>
                <div class="k8s-info-row" v-if="ctr.liveness_probe"><span class="k8s-info-label">Liveness</span><span class="k8s-info-value k8s-info-mono">{{ ctr.liveness_probe }}</span></div>
              </div>
            </div>
            <div v-else class="k8s-inspector-empty">No container information</div>
          </template>

          <!-- ── CONDITIONS (Pod only) ── -->
          <template v-if="activeInspectorTab === 'conditions'">
            <div class="k8s-inspector-section">
              <h4 class="k8s-section-title">Pod Conditions</h4>
              <div v-if="pod" class="k8s-conditions-list">
                <div class="k8s-condition-row" v-for="(val, key) in pod.labels" :key="key">
                  <span class="k8s-condition-name">{{ key }}</span>
                  <span class="k8s-condition-status">—</span>
                </div>
                <div class="k8s-info-empty">Conditions parsed from pod status</div>
              </div>
              <div v-else class="k8s-inspector-empty">No conditions available</div>
            </div>
          </template>

          <!-- ── STRATEGY (Deployment only) ── -->
          <template v-if="activeInspectorTab === 'strategy'">
            <div class="k8s-inspector-section">
              <h4 class="k8s-section-title">Update Strategy</h4>
              <div v-if="deployment">
                <div class="k8s-info-row"><span class="k8s-info-label">Type</span><span class="k8s-info-value">{{ deployment.strategy }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Selector</span><span class="k8s-info-value">{{ deployment.selector }}</span></div>
              </div>
              <div v-else class="k8s-inspector-empty">No strategy information</div>
            </div>
          </template>

          <!-- ── ENDPOINTS (Service only) ── -->
          <template v-if="activeInspectorTab === 'endpoints'">
            <div class="k8s-inspector-section">
              <h4 class="k8s-section-title">Service Endpoints</h4>
              <div v-if="service">
                <div class="k8s-info-row"><span class="k8s-info-label">Cluster IP</span><span class="k8s-info-value k8s-info-mono">{{ service.cluster_ip }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">External IP</span><span class="k8s-info-value k8s-info-mono">{{ service.external_ip ?? '—' }}</span></div>
                <div class="k8s-info-row"><span class="k8s-info-label">Ports</span><span class="k8s-info-value"><span v-for="p in service.ports" :key="p" class="k8s-port-chip">{{ p }}</span></span></div>
              </div>
              <div v-else class="k8s-inspector-empty">No endpoint information</div>
            </div>
          </template>

          <!-- ── DATA (ConfigMap only) ── -->
          <template v-if="activeInspectorTab === 'data'">
            <div v-if="configmap && configmap.data_keys.length > 0">
              <div class="k8s-inspector-section">
                <h4 class="k8s-section-title">Data Keys ({{ configmap.keys_count }})</h4>
                <div v-for="key in configmap.data_keys" :key="key" class="k8s-data-key-row">
                  <span class="k8s-data-key-name">{{ key }}</span>
                  <span class="k8s-data-key-value">—</span>
                </div>
              </div>
            </div>
            <div v-else class="k8s-inspector-empty">No data keys</div>
          </template>

          <!-- ── KEYS (Secret only - redacted) ── -->
          <template v-if="activeInspectorTab === 'keys'">
            <div v-if="secret && secret.data_keys.length > 0">
              <div class="k8s-inspector-section">
                <h4 class="k8s-section-title">Data Keys ({{ secret.keys_count }})</h4>
                <div v-for="key in secret.data_keys" :key="key" class="k8s-data-key-row">
                  <span class="k8s-data-key-name">{{ key }}</span>
                  <SecretValueMask :key-name="key" />
                </div>
              </div>
            </div>
            <div v-else class="k8s-inspector-empty">No secret keys</div>
          </template>

          <!-- ── EVENTS ── -->
          <template v-if="activeInspectorTab === 'events'">
            <div class="k8s-inspector-section">
              <h4 class="k8s-section-title">Events</h4>
              <div v-if="events.length > 0" class="k8s-events-list">
                <div v-for="(evt, i) in events" :key="i" class="k8s-event-row" :class="{ 'k8s-event--warning': evt.event_type === 'Warning' }">
                  <span class="k8s-event-time">{{ evt.timestamp }}</span>
                  <span :class="['k8s-event-type', evt.event_type === 'Warning' ? 'k8s-event-type--warn' : 'k8s-event-type--normal']">{{ evt.event_type }}</span>
                  <span class="k8s-event-reason">{{ evt.reason }}</span>
                  <span class="k8s-event-message">{{ evt.message }}</span>
                </div>
              </div>
              <div v-else class="k8s-inspector-empty">No events recorded</div>
            </div>
          </template>

          <!-- ── YAML ── -->
          <template v-if="activeInspectorTab === 'yaml'">
            <div class="k8s-inspector-section k8s-inspector-section--grow">
              <YamlViewer :yaml="yaml ?? ''" :loading="!yaml" />
            </div>
          </template>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.k8s-inspector-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 80;
  display: flex;
  justify-content: flex-end;
}

.k8s-inspector-drawer {
  width: 480px;
  max-width: 100vw;
  height: 100%;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  box-shadow: -8px 0 24px rgba(0, 0, 0, 0.3);
  animation: drawerIn 0.2s ease-out;
}

@keyframes drawerIn {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

/* Header */
.k8s-inspector-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.k8s-inspector-title {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.k8s-inspector-title > i {
  font-size: 18px;
  color: var(--accent-cyan);
  margin-top: 2px;
}

.k8s-inspector-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-main);
  word-break: break-all;
}

.k8s-inspector-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}

.k8s-inspector-kind {
  text-transform: uppercase;
  font-family: var(--font-mono);
  color: var(--accent-cyan);
}

.k8s-inspector-sep {
  color: var(--text-disabled);
}

.k8s-inspector-ns {
  font-family: var(--font-mono);
}

.k8s-inspector-close {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: none;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  transition: all var(--transition-fast);
}

.k8s-inspector-close:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

/* Tabs */
.k8s-inspector-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  padding: 0 12px;
  flex-shrink: 0;
  overflow-x: auto;
}

.k8s-inspector-tab {
  padding: 8px 14px;
  border: none;
  background: none;
  color: var(--text-muted);
  font-size: 12px;
  font-family: var(--font-sans);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.k8s-inspector-tab:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.k8s-inspector-tab.active {
  color: var(--accent-cyan);
  border-bottom-color: var(--accent-cyan);
}

/* Body */
.k8s-inspector-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
}

.k8s-inspector-section {
  margin-bottom: 16px;
}

.k8s-inspector-section--grow {
  flex: 1;
  min-height: 200px;
}

.k8s-section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Info rows */
.k8s-info-row {
  display: flex;
  align-items: center;
  padding: 6px 0;
  border-bottom: 1px solid var(--border-light);
  font-size: 12px;
}

.k8s-info-row:last-child {
  border-bottom: none;
}

.k8s-info-label {
  flex: 0 0 110px;
  color: var(--text-muted);
  font-weight: 500;
}

.k8s-info-value {
  color: var(--text-main);
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.k8s-info-mono {
  font-family: var(--font-mono);
  font-size: 11px;
}

.k8s-info-empty {
  font-size: 12px;
  color: var(--text-disabled);
  padding: 8px 0;
}

.k8s-inspector-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px 20px;
  color: var(--text-disabled);
  font-size: 13px;
}

/* Labels grid */
.k8s-labels-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.k8s-label-chip {
  display: inline-flex;
  padding: 2px 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 10px;
  font-family: var(--font-mono);
}

.k8s-label-key {
  color: var(--accent-cyan);
}

.k8s-label-val {
  color: var(--text-muted);
  margin-left: 4px;
}

/* Container status */
.k8s-container-status {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 3px;
}

.k8s-container--ready {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
}

.k8s-container--not-ready {
  background: rgba(239, 68, 68, 0.08);
  color: var(--accent-red);
}

/* Events */
.k8s-events-list {
  display: flex;
  flex-direction: column;
}

.k8s-event-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 6px 0;
  border-bottom: 1px solid var(--border-light);
  font-size: 11px;
  align-items: center;
}

.k8s-event--warning {
  background: rgba(245, 158, 11, 0.03);
}

.k8s-event-time {
  font-family: var(--font-mono);
  color: var(--text-disabled);
  font-size: 10px;
  flex: 0 0 100px;
}

.k8s-event-type {
  font-size: 9px;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: 3px;
  text-transform: uppercase;
}

.k8s-event-type--normal {
  background: rgba(16, 185, 129, 0.08);
  color: var(--accent-green);
}

.k8s-event-type--warn {
  background: rgba(245, 158, 11, 0.08);
  color: var(--accent-yellow);
}

.k8s-event-reason {
  color: var(--text-main);
  font-weight: 600;
  font-size: 11px;
  min-width: 80px;
}

.k8s-event-message {
  color: var(--text-muted);
  flex: 1;
  min-width: 100px;
  word-break: break-all;
}

/* Data keys */
.k8s-data-key-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 0;
  border-bottom: 1px solid var(--border-light);
  font-size: 11px;
}

.k8s-data-key-name {
  font-family: var(--font-mono);
  color: var(--text-main);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.k8s-data-key-value {
  font-family: var(--font-mono);
  color: var(--text-muted);
  flex-shrink: 0;
}

/* Port chip */
.k8s-port-chip {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 1px 5px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-muted);
  white-space: nowrap;
}

.k8s-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 600;
}

.k8s-pill--running { background: rgba(16, 185, 129, 0.1); color: var(--accent-green); border: 1px solid rgba(16, 185, 129, 0.2); }
.k8s-pill--pending { background: rgba(245, 158, 11, 0.08); color: var(--accent-yellow); border: 1px solid rgba(245, 158, 11, 0.15); }
.k8s-pill--error { background: rgba(239, 68, 68, 0.1); color: var(--accent-red); border: 1px solid rgba(239, 68, 68, 0.3); }
.k8s-pill--default { background: var(--bg-tertiary); color: var(--text-muted); border: 1px solid var(--border-color); }

/* Transition */
.drawer-slide-enter-active,
.drawer-slide-leave-active {
  transition: all 0.2s ease-out;
}

.drawer-slide-enter-from .k8s-inspector-drawer,
.drawer-slide-leave-to .k8s-inspector-drawer {
  transform: translateX(100%);
}

.drawer-slide-enter-from,
.drawer-slide-leave-to {
  background: rgba(0, 0, 0, 0);
}
</style>
