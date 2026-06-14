<!-- ItzamBox — Kubernetes Error Banner
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import type { ConnectionStatus } from '../../composables/useKubernetes'

const props = defineProps<{
  connectionStatus: ConnectionStatus
  error: string | null
  staleTimestamp: number | null
}>()

const emit = defineEmits<{
  'retry': []
}>()

function formatStaleTime(ts: number): string {
  const diff = Math.floor((Date.now() - ts) / 1000)
  if (diff < 60) return `${diff}s ago`
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  return `${Math.floor(diff / 86400)}d ago`
}
</script>

<template>
  <div v-if="connectionStatus === 'offline' || (error && connectionStatus !== 'no-kubectl' && connectionStatus !== 'no-kubeconfig')" class="k8s-error-banner" :class="{ 'k8s-error-banner--stale': staleTimestamp }">
    <div class="k8s-error-banner-content">
      <i v-if="staleTimestamp" class="fa-solid fa-clock-rotate-left"></i>
      <i v-else class="fa-solid fa-triangle-exclamation"></i>
      <span>
        <template v-if="staleTimestamp">
          Cluster unreachable — showing stale data from {{ formatStaleTime(staleTimestamp) }}
        </template>
        <template v-else>
          {{ error ?? 'Cluster connection lost' }}
        </template>
      </span>
    </div>
    <button class="k8s-error-banner-btn" @click="emit('retry')">
      <i class="fa-solid fa-rotate"></i> Retry
    </button>
  </div>
</template>

<style scoped>
.k8s-error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: rgba(239, 68, 68, 0.08);
  border-bottom: 1px solid rgba(239, 68, 68, 0.2);
  animation: slideDown 0.2s ease-out;
  flex-shrink: 0;
}

.k8s-error-banner--stale {
  background: rgba(245, 158, 11, 0.08);
  border-bottom-color: rgba(245, 158, 11, 0.2);
}

.k8s-error-banner-content {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--accent-red);
}

.k8s-error-banner--stale .k8s-error-banner-content {
  color: var(--accent-yellow);
}

.k8s-error-banner-content i {
  font-size: 14px;
}

.k8s-error-banner-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--radius-sm);
  color: var(--accent-red);
  font-size: 11px;
  font-family: var(--font-sans);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.k8s-error-banner--stale .k8s-error-banner-btn {
  background: rgba(245, 158, 11, 0.1);
  border-color: rgba(245, 158, 11, 0.2);
  color: var(--accent-yellow);
}

.k8s-error-banner-btn:hover {
  background: rgba(239, 68, 68, 0.2);
}

.k8s-error-banner--stale .k8s-error-banner-btn:hover {
  background: rgba(245, 158, 11, 0.2);
}

@keyframes slideDown {
  from { transform: translateY(-100%); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}
</style>
