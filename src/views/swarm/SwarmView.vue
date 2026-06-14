<!-- ItzamBox — Swarm View (Parent Route)
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import { useSwarm } from '../../composables/useSwarm'
import SwarmInactiveCTA from '../../components/swarm/SwarmInactiveCTA.vue'
import SwarmInitModal from '../../components/swarm/SwarmInitModal.vue'
import SwarmJoinModal from '../../components/swarm/SwarmJoinModal.vue'
import SwarmNodesTab from '../../components/swarm/SwarmNodesTab.vue'
import SwarmServicesTab from '../../components/swarm/SwarmServicesTab.vue'
import SwarmStacksTab from '../../components/swarm/SwarmStacksTab.vue'
import SwarmTopologyView from '../../components/swarm/SwarmTopologyView.vue'
import SwarmInspectorDrawer from '../../components/swarm/SwarmInspectorDrawer.vue'
import SkeletonLoader from '../../components/shared/SkeletonLoader.vue'
import type { SwarmNode, SwarmService, SwarmStack } from '../../composables/useSwarm'

const {
  swarmStatus,
  nodes,
  services,
  stacks,
  loading,
  error,
  leaveSwarm,
  refreshAll,
  inspectNode,
  inspectService,
  listenSwarmOutput,
  stopListeningSwarmOutput,
  loadNodes,
  loadServices,
  loadStacks,
} = useSwarm()

// ─── Modal State ─────────────────────────────────────────────────────────

const showInitModal = ref(false)
const showJoinModal = ref(false)
const showLeaveConfirm = ref(false)
const leaving = ref(false)

// ─── Tabs ────────────────────────────────────────────────────────────────

type SwarmTab = 'nodes' | 'services' | 'stacks' | 'topology'
const activeTab = ref<SwarmTab>('nodes')

function setTab(tab: SwarmTab) {
  activeTab.value = tab
}

// ─── Inspector Drawer ────────────────────────────────────────────────────

const selectedNode = ref<SwarmNode | null>(null)
const selectedService = ref<SwarmService | null>(null)
const nodeInspectData = ref<Record<string, unknown> | null>(null)
const serviceInspectData = ref<Record<string, unknown> | null>(null)

async function openNodeInspector(nodeId: string) {
  const found = nodes.value.find((n: SwarmNode) => n.id === nodeId)
  if (!found) return
  selectedNode.value = found
  selectedService.value = null
  try {
    nodeInspectData.value = await inspectNode(nodeId)
  } catch {
    nodeInspectData.value = null
  }
}

async function openServiceInspector(serviceId: string) {
  const found = (services.value as SwarmService[]).find((s: SwarmService) => s.id === serviceId)
  if (!found) return
  selectedService.value = found as SwarmService
  selectedNode.value = null
  try {
    serviceInspectData.value = await inspectService(serviceId)
  } catch {
    serviceInspectData.value = null
  }
}

function closeInspector() {
  selectedNode.value = null
  selectedService.value = null
  nodeInspectData.value = null
  serviceInspectData.value = null
}

// ─── Leave Swarm ─────────────────────────────────────────────────────────

async function handleLeaveSwarm() {
  leaving.value = true
  try {
    await leaveSwarm(true)
    showLeaveConfirm.value = false
  } catch {
    // error state handled by composable
  } finally {
    leaving.value = false
  }
}

// ─── Lifecycle ───────────────────────────────────────────────────────────

onMounted(async () => {
  await listenSwarmOutput()
  await refreshAll()
})

onBeforeUnmount(() => {
  stopListeningSwarmOutput()
})

// ─── Summary Counts ──────────────────────────────────────────────────────

const summaryNodes = computed(() => swarmStatus.value?.nodes_count ?? 0)
const summaryManagers = computed(() => swarmStatus.value?.managers_count ?? 0)
const summaryServices = computed(() => swarmStatus.value?.services_count ?? 0)
const summaryStacks = computed(() => stacks.value.length)
</script>

<template>
  <div class="swarm-view">
    <!-- Breadcrumb -->
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <span>ItzamBox</span>
      <i class="fa-solid fa-chevron-right" aria-hidden="true"></i>
      <span>Orchestration</span>
      <i class="fa-solid fa-chevron-right" aria-hidden="true"></i>
      <span class="current">Swarm</span>
    </nav>

    <!-- Page Header -->
    <div class="content__page-header">
      <h1 class="text-h1" style="display: flex; align-items: center; gap: 12px">
        <i class="fa-solid fa-bees" style="color: var(--accent-cyan); font-size: 1.5rem"></i>
        Docker Swarm
      </h1>
    </div>

    <!-- Global Error Banner -->
    <div v-if="error" class="error-state" role="alert">
      <div class="error-state-icon">
        <i class="fa-solid fa-circle-exclamation"></i>
      </div>
      <p class="error-state-title">Swarm Error</p>
      <p class="error-state-suggestion">{{ error }}</p>
      <div class="error-state-actions">
        <button class="btn btn-primary btn-sm" @click="refreshAll">
          <i class="fa-solid fa-arrows-rotate"></i> Retry
        </button>
      </div>
    </div>

    <!-- ═══ STATE: Loading (initial) ─── -->
    <div v-if="loading && swarmStatus === null" class="swarm-loading">
      <SkeletonLoader variant="metric-grid" :count="4" />
      <SkeletonLoader variant="table-row" :rows="4" style="margin-top: 16px" />
    </div>

    <!-- ═══ STATE: Loading (active but data pending) ─── -->
    <template v-else-if="loading && swarmStatus?.active">
      <div class="swarm-status-header skeleton-card">
        <div class="skeleton skeleton-text-md" style="width:120px"></div>
        <div class="skeleton skeleton-text-sm" style="width:200px"></div>
      </div>
      <SkeletonLoader variant="table-row" :rows="4" />
    </template>

    <!-- ═══ STATE: Inactive ─── -->
    <div v-else-if="swarmStatus && !swarmStatus.active" class="swarm-inactive">
      <div class="info-banner" role="status">
        <i class="fa-solid fa-circle-info" style="color: var(--accent-cyan); margin-right: 8px"></i>
        <span>Swarm mode is not active on this node. Initialize a new swarm or join an existing one to get started.</span>
      </div>
      <SwarmInactiveCTA
        @init="showInitModal = true"
        @join="showJoinModal = true"
      />
    </div>

    <!-- ═══ STATE: Active ─── -->
    <div v-else-if="swarmStatus && swarmStatus.active" class="swarm-active">
      <!-- Status Header -->
      <div class="swarm-status-header">
        <div class="swarm-status-badge swarm-status-badge--active">
          <span class="status-dot status-dot--running"></span>
          Swarm Active
        </div>
        <div class="swarm-status-body">
          <div class="swarm-summary">
            <div class="swarm-summary__item">
              <div class="swarm-summary__value" style="color: var(--accent-cyan)">
                {{ summaryNodes }}
              </div>
              <div class="swarm-summary__label">Nodes</div>
            </div>
            <div class="swarm-summary__item">
              <div class="swarm-summary__value" style="color: var(--accent-green)">
                {{ summaryManagers }}
              </div>
              <div class="swarm-summary__label">Managers</div>
            </div>
            <div class="swarm-summary__item">
              <div class="swarm-summary__value" style="color: var(--accent-purple)">
                {{ summaryServices }}
              </div>
              <div class="swarm-summary__label">Services</div>
            </div>
            <div class="swarm-summary__item">
              <div class="swarm-summary__value" style="color: var(--accent-yellow)">
                {{ summaryStacks }}
              </div>
              <div class="swarm-summary__label">Stacks</div>
            </div>
          </div>
        </div>
        <div class="swarm-actions">
          <button
            class="btn btn-ghost btn-sm"
            aria-label="Leave swarm"
            title="Leave Swarm (Advanced)"
            @click="showLeaveConfirm = true"
          >
            <i class="fa-solid fa-right-from-bracket" style="color: var(--accent-red)"></i>
          </button>
          <button class="btn btn-ghost btn-sm" aria-label="Refresh" @click="refreshAll">
            <i class="fa-solid fa-arrows-rotate" :class="{ 'fa-spin': loading }"></i>
          </button>
        </div>
      </div>

      <!-- Tabs -->
      <div class="swarm-tabs" role="tablist" aria-label="Swarm sections">
        <button
          class="swarm-tab"
          :class="{ active: activeTab === 'nodes' }"
          role="tab"
          :aria-selected="activeTab === 'nodes'"
          @click="setTab('nodes')"
        >
          <i class="fa-solid fa-server" style="margin-right: 4px"></i>
          Nodes
        </button>
        <button
          class="swarm-tab"
          :class="{ active: activeTab === 'services' }"
          role="tab"
          :aria-selected="activeTab === 'services'"
          @click="setTab('services')"
        >
          <i class="fa-solid fa-diagram-project" style="margin-right: 4px"></i>
          Services
        </button>
        <button
          class="swarm-tab"
          :class="{ active: activeTab === 'stacks' }"
          role="tab"
          :aria-selected="activeTab === 'stacks'"
          @click="setTab('stacks')"
        >
          <i class="fa-solid fa-layer-group" style="margin-right: 4px"></i>
          Stacks
        </button>
        <button
          class="swarm-tab"
          :class="{ active: activeTab === 'topology' }"
          role="tab"
          :aria-selected="activeTab === 'topology'"
          @click="setTab('topology')"
        >
          <i class="fa-solid fa-diagram-nodes" style="margin-right: 4px"></i>
          Topology
        </button>
      </div>

      <!-- Tab Panels -->
      <div class="swarm-panels">
        <div v-show="activeTab === 'nodes'" role="tabpanel">
          <SwarmNodesTab
            :nodes="(nodes as unknown as SwarmNode[])"
            :loading="loading"
            @inspect="openNodeInspector"
            @refresh="loadNodes"
          />
        </div>

        <div v-show="activeTab === 'services'" role="tabpanel">
          <SwarmServicesTab
            :services="(services as unknown as SwarmService[])"
            :loading="loading"
            @inspect="openServiceInspector"
            @refresh="loadServices"
          />
        </div>

        <div v-show="activeTab === 'stacks'" role="tabpanel">
          <SwarmStacksTab
            :stacks="(stacks as unknown as SwarmStack[])"
            :services="(services as unknown as SwarmService[])"
            :loading="loading"
            @refresh="loadStacks"
          />
        </div>

        <div v-show="activeTab === 'topology'" role="tabpanel">
          <SwarmTopologyView
            :nodes="(nodes as unknown as SwarmNode[])"
            :services="(services as unknown as SwarmService[])"
          />
        </div>
      </div>
    </div>

    <!-- ═══ Modals ─── -->
    <SwarmInitModal
      v-if="showInitModal"
      @close="showInitModal = false"
      @done="showInitModal = false; refreshAll()"
    />
    <SwarmJoinModal
      v-if="showJoinModal"
      @close="showJoinModal = false"
      @done="showJoinModal = false; refreshAll()"
    />

    <!-- Leave Confirmation Dialog -->
    <Teleport to="body">
      <div v-if="showLeaveConfirm" class="modal-backdrop" @click.self="showLeaveConfirm = false">
        <div class="modal-content" style="max-width: 420px" role="alertdialog" aria-label="Leave swarm confirmation">
          <div class="modal-header">
            <h2 class="modal-title">
              <i class="fa-solid fa-right-from-bracket" style="color: var(--accent-red); margin-right: 8px"></i>
              Leave Swarm
            </h2>
            <button class="action-btn" aria-label="Close" @click="showLeaveConfirm = false">
              <i class="fa-solid fa-xmark"></i>
            </button>
          </div>
          <div class="modal-body">
            <p style="font-size: 13px; line-height: 1.6">
              Are you sure you want to leave the swarm? This node will stop participating in cluster orchestration.
            </p>
            <p style="font-size: 12px; color: var(--text-muted); margin-top: 8px">
              Running containers will not be affected but the node will no longer accept tasks.
            </p>
          </div>
          <div class="modal-footer">
            <button class="btn btn-ghost" @click="showLeaveConfirm = false">Cancel</button>
            <button
              class="btn btn-danger"
              :disabled="leaving"
              @click="handleLeaveSwarm"
            >
              <i v-if="leaving" class="fa-solid fa-spinner fa-spin"></i>
              <i v-else class="fa-solid fa-right-from-bracket"></i>
              {{ leaving ? 'Leaving...' : 'Leave Swarm' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Inspector Drawer -->
    <SwarmInspectorDrawer
      :node="selectedNode"
      :service="selectedService"
      :node-inspect="nodeInspectData"
      :service-inspect="serviceInspectData"
      @close="closeInspector"
    />
  </div>
</template>

<style>
/* ─── Swarm View Root Styles ─── */
.swarm-view {
  display: flex;
  flex-direction: column;
  gap: 20px;
  width: 100%;
}

/* Info Banner */
.info-banner {
  display: flex;
  align-items: center;
  padding: 12px 18px;
  background: rgba(0, 229, 255, 0.06);
  border: 1px solid rgba(0, 229, 255, 0.15);
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-main);
}

/* Loading */
.swarm-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 64px 20px;
}

/* Status Header */
.swarm-status-header {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 16px 24px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
}

.swarm-status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  flex-shrink: 0;
}

.swarm-status-badge--active {
  color: var(--accent-green);
}

.swarm-status-body {
  flex: 1;
}

.swarm-summary {
  display: flex;
  gap: 32px;
}

.swarm-summary__item {
  text-align: center;
}

.swarm-summary__value {
  font-size: 1.5rem;
  font-weight: 700;
  font-family: var(--font-mono);
}

.swarm-summary__label {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-top: 2px;
}

.swarm-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

/* Tabs */
.swarm-tabs {
  display: flex;
  gap: 0;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 4px;
  align-items: center;
}

.swarm-tab {
  padding: 8px 20px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-sans);
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.swarm-tab:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.swarm-tab.active {
  background: var(--bg-secondary);
  color: var(--accent-cyan);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

/* Panels */
.swarm-panels {
  min-height: 200px;
}

/* Page header */
.content__page-header {
  display: flex;
  align-items: center;
}

/* Modal backdrop reuse */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-content {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg), 0 0 40px rgba(0, 229, 255, 0.05);
  animation: modalEnter 0.2s cubic-bezier(0, 0, 0.2, 1);
  max-width: 560px;
  width: 90%;
}

.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-light);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-title {
  font-size: 15px;
  font-weight: 600;
}

.modal-body {
  padding: 20px;
}

.modal-footer {
  padding: 12px 20px;
  border-top: 1px solid var(--border-light);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn-danger {
  background: rgba(239, 68, 68, 0.1);
  color: var(--accent-red);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.btn-danger:hover {
  background: rgba(239, 68, 68, 0.2);
}

@keyframes modalEnter {
  from { opacity: 0; transform: scale(0.95) translateY(10px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}
</style>
