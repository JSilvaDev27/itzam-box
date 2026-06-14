<!-- ItzamBox — Kubernetes Cluster Viewer
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useKubernetes } from '../../composables/useKubernetes'
import K8sToolbar from '../../components/kubernetes/K8sToolbar.vue'
import K8sResourceTabs from '../../components/kubernetes/K8sResourceTabs.vue'
import K8sInspectorDrawer from '../../components/kubernetes/K8sInspectorDrawer.vue'
import K8sErrorBanner from '../../components/kubernetes/K8sErrorBanner.vue'
import KubectlInstallGuide from '../../components/kubernetes/KubectlInstallGuide.vue'
import SkeletonLoader from '../../components/shared/SkeletonLoader.vue'

const route = useRoute()

const {
  connectionStatus, error, loading, staleTimestamp,
  contexts, activeContext, namespaces, activeNamespace,
  pods, deployments, services, configmaps, secrets,
  activeTab, activeConfigSubTab,
  inspectorOpen, inspectedKind, inspectedName, inspectedNamespace,
  inspectorYaml, inspectorEvents,
  switchContext,
  switchNamespace,
  reloadAll, openInspector, closeInspector, initialize,
} = useKubernetes()

// ── Deep-linking: read route params on mount ──
onMounted(async () => {
  await initialize()
  
  // Deep-link: namespace pre-select
  const nsParam = route.params.ns as string | undefined
  if (nsParam && connectionStatus.value === 'connected') {
    await switchNamespace(nsParam)
  }

})

// ── Watch route changes for deep-linking (immediate to handle initial mount) ──
watch(() => ({ ...route.params }), async (params) => {
  const nsParam = params.ns as string | undefined
  if (nsParam && nsParam !== activeNamespace.value) {
    await switchNamespace(nsParam)
  }

  const resourceParam = params.resource as string | undefined
  const nameParam = params.name as string | undefined
  if (resourceParam && nameParam) {
    const kind = resourceParam as 'pod' | 'deployment' | 'service' | 'configmap' | 'secret'
    if (['pod', 'deployment', 'service', 'configmap', 'secret'].includes(kind)) {
      const targetNs = nsParam || activeNamespace.value || 'default'
      console.warn('[K8sView] watch opening inspector', kind, nameParam, 'ns:', targetNs)
      await openInspector(kind, nameParam, targetNs)
    }
  }
}, { immediate: true, deep: true })

// ── Tab switching (preserved on namespace switch per US-21) ──
function onSwitchTab(tab: string) {
  activeTab.value = tab
}

function onSwitchSubTab(tab: string) {
  activeConfigSubTab.value = tab
}

// ── Refresh ──
async function handleRefresh() {
  await reloadAll(activeNamespace.value)
}

// ── Retry after error ──
async function handleRetry() {
  await initialize()
}

// ── Context switch ──
async function handleSwitchContext(ctx: string) {
  await switchContext(ctx)
  if (activeNamespace.value) {
    await switchNamespace(activeNamespace.value)
  }
}

// ── Namespace switch ──
async function handleSwitchNamespace(ns: string | null) {
  await switchNamespace(ns)
}

// ── Inspect from pods table ──
async function handleInspectPod(pod: any) {
  await openInspector('pod', pod.name, pod.namespace)
}

async function handleInspectDeployment(dep: any) {
  await openInspector('deployment', dep.name, dep.namespace)
}

async function handleInspectService(svc: any) {
  await openInspector('service', svc.name, svc.namespace)
}

async function handleInspectConfigMap(cm: any) {
  await openInspector('configmap', cm.name, cm.namespace)
}

async function handleInspectSecret(sec: any) {
  await openInspector('secret', sec.name, sec.namespace)
}
</script>

<template>
  <div class="kubernetes-view">
    <!-- Page Header -->
    <div class="page-header">
      <div class="page-header-left">
        <h1 class="page-title">
          <i class="fa-solid fa-ship"></i>
          Kubernetes
        </h1>
        <p class="page-subtitle">Cluster resource viewer</p>
      </div>
    </div>

    <!-- ── Empty State: kubectl not found ── -->
    <template v-if="connectionStatus === 'no-kubectl'">
      <KubectlInstallGuide @retry="handleRetry" />
    </template>

    <!-- ── Empty State: no kubeconfig ── -->
    <template v-else-if="connectionStatus === 'no-kubeconfig'">
      <div class="empty-state">
        <div class="empty-state-icon">
          <i class="fa-solid fa-file-circle-exclamation"></i>
        </div>
        <h3 class="empty-state-title">No kubeconfig Found</h3>
        <p class="empty-state-desc">
          kubectl is installed but no kubeconfig file was found at <code>~/.kube/config</code>.
          Configure cluster access and try again.
        </p>
        <div class="empty-state-actions">
          <button class="btn btn-primary" @click="handleRetry">
            <i class="fa-solid fa-rotate"></i> Check Again
          </button>
        </div>
      </div>
    </template>

    <!-- ── Loading State (connected but data pending) ── -->
    <template v-else-if="connectionStatus === 'connected' && loading && pods.length === 0">
      <SkeletonLoader variant="header" />
      <div style="display:flex;gap:8px;margin-bottom:16px">
        <div class="skeleton skeleton-text-sm" style="width:120px;height:32px;border-radius:6px"></div>
        <div class="skeleton skeleton-text-sm" style="width:160px;height:32px;border-radius:6px"></div>
      </div>
      <SkeletonLoader variant="table-row" :rows="5" />
    </template>

    <!-- ── Main Panel ── -->
    <template v-else>
      <!-- Error Banner -->
      <K8sErrorBanner
        :connection-status="connectionStatus"
        :error="error"
        :stale-timestamp="staleTimestamp"
        @retry="handleRetry"
      />

      <!-- Toolbar -->
      <K8sToolbar
        :contexts="contexts"
        :active-context="activeContext"
        :namespaces="namespaces"
        :active-namespace="activeNamespace"
        :connection-status="connectionStatus"
        @switch-context="handleSwitchContext"
        @switch-namespace="handleSwitchNamespace"
        @refresh="handleRefresh"
        @retry="handleRetry"
      />

      <!-- Resource Tabs -->
      <div class="k8s-resources-panel">
        <K8sResourceTabs
          :active-tab="activeTab"
          :active-config-sub-tab="activeConfigSubTab"
          :pods="pods"
          :deployments="deployments"
          :services="services"
          :configmaps="configmaps"
          :secrets="secrets"
          :loading="loading"
          @switch-tab="onSwitchTab"
          @switch-subtab="onSwitchSubTab"
          @inspect-pod="handleInspectPod"
          @inspect-deployment="handleInspectDeployment"
          @inspect-service="handleInspectService"
          @inspect-cm="handleInspectConfigMap"
          @inspect-secret="handleInspectSecret"
        />
      </div>
    </template>

    <!-- Inspector Drawer -->
    <K8sInspectorDrawer
      :open="inspectorOpen"
      :kind="inspectedKind"
      :name="inspectedName"
      :namespace="inspectedNamespace"
      :yaml="inspectorYaml"
      :events="inspectorEvents"
      :pods="pods"
      :deployments="deployments"
      :services="services"
      :configmaps="configmaps"
      :secrets="secrets"
      @close="closeInspector"
    />
  </div>
</template>

<style scoped>
.kubernetes-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  position: relative;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0;
  flex-shrink: 0;
}

.page-header-left {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.page-title {
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--text-main);
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-title i {
  color: var(--accent-cyan);
}

.page-subtitle {
  font-size: 12px;
  color: var(--text-muted);
}

.k8s-resources-panel {
  flex: 1;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Override empty state for no-kubeconfig */
.empty-state-icon i {
  font-size: 36px;
}

.empty-state-desc code {
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--bg-hover);
  padding: 1px 5px;
  border-radius: 3px;
  color: var(--accent-cyan);
}
</style>
