<!-- ItzamBox — Swarm Services Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SwarmService } from '../../composables/useSwarm'

const props = defineProps<{
  services: SwarmService[]
  loading: boolean
}>()

defineEmits<{
  (e: 'inspect', serviceId: string): void
  (e: 'refresh'): void
}>()

const filter = ref('')

const filteredServices = computed(() => {
  if (!filter.value.trim()) return props.services
  const q = filter.value.toLowerCase()
  return props.services.filter(
    s =>
      s.name.toLowerCase().includes(q) ||
      s.image.toLowerCase().includes(q) ||
      s.mode.toLowerCase().includes(q),
  )
})

function replicasClass(replicas: string): string {
  const [ready, total] = replicas.split('/').map(Number)
  if (isNaN(ready) || isNaN(total)) return 'replicas--zero'
  if (ready === 0) return 'replicas--zero'
  if (ready < total) return 'replicas--partial'
  return 'replicas--full'
}

function safePorts(ports: string[]): string {
  if (!ports || ports.length === 0) return 'internal'
  return ports.join(', ')
}
</script>

<template>
  <div class="swarm-table-section">
    <div class="table-toolbar">
      <span class="section-title" style="font-size: 14px">
        <i class="fa-solid fa-diagram-project" style="margin-right: 6px; color: var(--accent-cyan)"></i>
        Swarm Services
      </span>
      <div class="table-toolbar__actions">
        <div class="table-filter">
          <i class="fa-solid fa-magnifying-glass"></i>
          <input
            v-model="filter"
            type="text"
            placeholder="Filter services..."
            aria-label="Filter services by name, image, or mode"
          />
        </div>
        <button class="btn btn-ghost btn-sm" aria-label="Refresh services" @click="$emit('refresh')">
          <i class="fa-solid fa-arrows-rotate" :class="{ 'fa-spin': loading }"></i>
        </button>
      </div>
    </div>

    <!-- Skeleton loading -->
    <template v-if="loading && services.length === 0">
      <div v-for="i in 4" :key="i" class="data-row skeleton-row">
        <div class="row-info">
          <div class="skeleton skeleton-text-md" style="width: 160px"></div>
        </div>
        <div class="skeleton skeleton-tag"></div>
        <div class="skeleton skeleton-tag" style="width: 40px"></div>
        <div class="skeleton skeleton-text-sm"></div>
        <div class="skeleton skeleton-text-xs" style="width: 120px"></div>
      </div>
    </template>

    <!-- Empty state -->
    <div v-else-if="services.length === 0" class="empty-state" style="padding: 32px">
      <div class="empty-state-icon" style="width: 56px; height: 56px; font-size: 24px">
        <i class="fa-solid fa-diagram-project"></i>
      </div>
      <p class="empty-state-title">No services deployed</p>
      <p class="empty-state-desc">Deploy a stack or create services using <code>docker service create</code>.</p>
    </div>

    <!-- Table -->
    <div v-else class="swarm-table">
      <div class="col-headers">
        <span class="col-header" style="flex: 2.5">Name</span>
        <span class="col-header" style="flex: 1">Mode</span>
        <span class="col-header" style="flex: 1; text-align: center">Replicas</span>
        <span class="col-header" style="flex: 2">Image</span>
        <span class="col-header" style="flex: 2">Ports</span>
      </div>

      <div
        v-for="svc in filteredServices"
        :key="svc.id"
        class="data-row"
        @click="$emit('inspect', svc.id)"
        role="button"
        :tabindex="0"
        :aria-label="'Inspect service ' + svc.name"
        @keydown.enter="$emit('inspect', svc.id)"
      >
        <div class="row-info" style="flex: 2.5">
          <div class="row-name">{{ svc.name }}</div>
          <div class="row-meta">{{ svc.id.slice(0, 12) }}...</div>
        </div>

        <div style="flex: 1">
          <span :class="['badge', svc.mode === 'Global' ? 'badge--global' : 'badge--replicated']">
            {{ svc.mode }}
          </span>
        </div>

        <div style="flex: 1; text-align: center">
          <span :class="['replicas', replicasClass(svc.replicas)]">
            {{ svc.replicas }}
          </span>
        </div>

        <div style="flex: 2; font-size: 12px" class="text-mono">
          {{ svc.image }}
        </div>

        <div style="flex: 2; font-size: 12px" :class="svc.ports.length > 0 ? 'text-mono' : 'text-muted'">
          {{ safePorts(svc.ports) }}
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

.replicas--full { color: var(--accent-green); }
.replicas--partial { color: var(--accent-yellow); }
.replicas--zero { color: var(--accent-red); }

.empty-state-desc code {
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--bg-tertiary);
  padding: 1px 4px;
  border-radius: 3px;
}
</style>
