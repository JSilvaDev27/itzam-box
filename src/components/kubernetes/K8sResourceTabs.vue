<!-- ItzamBox — Kubernetes Resource Tabs
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import type { K8sPod, K8sDeployment, K8sService, K8sConfigMap, K8sSecretMeta } from '../../composables/useKubernetes'
import PodTable from './PodTable.vue'
import DeploymentTable from './DeploymentTable.vue'
import ServiceTable from './ServiceTable.vue'
import ConfigResourceTable from './ConfigResourceTable.vue'

defineProps<{
  activeTab: string
  activeConfigSubTab: string
  pods: K8sPod[]
  deployments: K8sDeployment[]
  services: K8sService[]
  configmaps: K8sConfigMap[]
  secrets: K8sSecretMeta[]
  loading: boolean
}>()

const emit = defineEmits<{
  'switch-tab': [tab: string]
  'switch-subtab': [tab: string]
  'inspect-pod': [pod: K8sPod]
  'inspect-deployment': [deployment: K8sDeployment]
  'inspect-service': [service: K8sService]
  'inspect-cm': [cm: K8sConfigMap]
  'inspect-secret': [secret: K8sSecretMeta]
}>()

</script>

<template>
  <div class="k8s-resource-tabs">
    <!-- Main Tabs -->
    <div class="k8s-tab-bar">
      <button
        class="k8s-tab"
        :class="{ active: activeTab === 'pods' }"
        @click="emit('switch-tab', 'pods')"
      >
        <i class="fa-solid fa-cubes"></i>
        Pods
        <span v-if="pods.length > 0" class="k8s-tab-badge">{{ pods.length }}</span>
      </button>
      <button
        class="k8s-tab"
        :class="{ active: activeTab === 'deployments' }"
        @click="emit('switch-tab', 'deployments')"
      >
        <i class="fa-solid fa-layer-group"></i>
        Deployments
        <span v-if="deployments.length > 0" class="k8s-tab-badge">{{ deployments.length }}</span>
      </button>
      <button
        class="k8s-tab"
        :class="{ active: activeTab === 'services' }"
        @click="emit('switch-tab', 'services')"
      >
        <i class="fa-solid fa-share-nodes"></i>
        Services
        <span v-if="services.length > 0" class="k8s-tab-badge">{{ services.length }}</span>
      </button>
      <button
        class="k8s-tab"
        :class="{ active: activeTab === 'config' }"
        @click="emit('switch-tab', 'config')"
      >
        <i class="fa-solid fa-file-lines"></i>
        Config
        <span v-if="configmaps.length + secrets.length > 0" class="k8s-tab-badge">{{ configmaps.length + secrets.length }}</span>
      </button>
    </div>

    <!-- Tab Content -->
    <div class="k8s-tab-content">
      <PodTable
        v-if="activeTab === 'pods'"
        :pods="pods"
        :loading="loading"
        @inspect="emit('inspect-pod', $event)"
      />
      <DeploymentTable
        v-else-if="activeTab === 'deployments'"
        :deployments="deployments"
        :loading="loading"
        @inspect="emit('inspect-deployment', $event)"
      />
      <ServiceTable
        v-else-if="activeTab === 'services'"
        :services="services"
        :loading="loading"
        @inspect="emit('inspect-service', $event)"
      />
      <ConfigResourceTable
        v-else-if="activeTab === 'config'"
        :configmaps="configmaps"
        :secrets="secrets"
        :loading="loading"
        :active-sub-tab="activeConfigSubTab"
        @switch-subtab="emit('switch-subtab', $event)"
        @inspect-cm="emit('inspect-cm', $event)"
        @inspect-secret="emit('inspect-secret', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.k8s-resource-tabs {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.k8s-tab-bar {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  padding: 0 16px;
  gap: 0;
  flex-shrink: 0;
  background: var(--bg-secondary);
}

.k8s-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border: none;
  background: none;
  color: var(--text-muted);
  font-size: 12px;
  font-family: var(--font-sans);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
  margin-bottom: -1px;
  position: relative;
}

.k8s-tab:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.k8s-tab.active {
  color: var(--accent-cyan);
  border-bottom-color: var(--accent-cyan);
}

.k8s-tab i {
  font-size: 13px;
}

.k8s-tab-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 1px 6px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  color: var(--text-muted);
  min-width: 18px;
  text-align: center;
}

.k8s-tab.active .k8s-tab-badge {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
}

.k8s-tab-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
