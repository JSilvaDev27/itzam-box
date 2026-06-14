<!-- ItzamBox — Swarm Stacks Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref } from 'vue'
import type { SwarmStack, SwarmService } from '../../composables/useSwarm'

const props = defineProps<{
  stacks: SwarmStack[]
  services: SwarmService[]
  loading: boolean
}>()

defineEmits<{
  (e: 'inspect-stack', stackName: string): void
  (e: 'deploy'): void
  (e: 'remove', stackName: string): void
  (e: 'refresh'): void
}>()

const filter = ref('')

function filteredStacks(): SwarmStack[] {
  if (!filter.value.trim()) return props.stacks
  const q = filter.value.toLowerCase()
  return props.stacks.filter(s => s.name.toLowerCase().includes(q))
}

function getStackServices(stackName: string): SwarmService[] {
  return props.services.filter(s => {
    // Services in a stack are prefixed with stack name
    return s.name.startsWith(stackName + '_') || s.name === stackName
  })
}
</script>

<template>
  <div class="swarm-table-section">
    <div class="table-toolbar">
      <span class="section-title" style="font-size: 14px">
        <i class="fa-solid fa-layer-group" style="margin-right: 6px; color: var(--accent-cyan)"></i>
        Deployed Stacks
      </span>
      <div class="table-toolbar__actions">
        <div class="table-filter">
          <i class="fa-solid fa-magnifying-glass"></i>
          <input
            v-model="filter"
            type="text"
            placeholder="Filter stacks..."
            aria-label="Filter stacks by name"
          />
        </div>
        <button class="btn btn-primary btn-sm" aria-label="Deploy new stack" @click="$emit('deploy')">
          <i class="fa-solid fa-plus"></i> Deploy Stack
        </button>
        <button class="btn btn-ghost btn-sm" aria-label="Refresh stacks" @click="$emit('refresh')">
          <i class="fa-solid fa-arrows-rotate" :class="{ 'fa-spin': loading }"></i>
        </button>
      </div>
    </div>

    <!-- Skeleton loading -->
    <template v-if="loading && stacks.length === 0">
      <div v-for="i in 3" :key="i" class="data-row skeleton-row">
        <div class="row-info">
          <div class="skeleton skeleton-text-md" style="width: 140px"></div>
        </div>
        <div class="skeleton skeleton-tag" style="width: 30px"></div>
        <div class="skeleton skeleton-tag" style="width: 60px"></div>
      </div>
    </template>

    <!-- Empty state -->
    <div v-else-if="stacks.length === 0" class="empty-state" style="padding: 32px">
      <div class="empty-state-icon" style="width: 56px; height: 56px; font-size: 24px">
        <i class="fa-solid fa-layer-group"></i>
      </div>
      <p class="empty-state-title">No stacks deployed</p>
      <p class="empty-state-desc">Deploy a Docker Compose stack to get started.</p>
      <button class="btn btn-primary btn-sm" @click="$emit('deploy')">
        <i class="fa-solid fa-plus"></i> Deploy Stack
      </button>
    </div>

    <!-- Table -->
    <div v-else class="swarm-table">
      <div class="col-headers">
        <span class="col-header" style="flex: 3">Stack Name</span>
        <span class="col-header" style="flex: 1; text-align: center">Services</span>
        <span class="col-header" style="flex: 1.5">Orchestrator</span>
        <span class="col-header" style="flex: 1; text-align: right">Actions</span>
      </div>

      <div
        v-for="stack in filteredStacks()"
        :key="stack.name"
        class="data-row"
      >
        <div
          class="row-info"
          style="flex: 3; cursor: pointer"
          @click="$emit('inspect-stack', stack.name)"
          role="button"
          :tabindex="0"
          :aria-label="'Inspect stack ' + stack.name"
          @keydown.enter="$emit('inspect-stack', stack.name)"
        >
          <div class="row-name">
            <i class="fa-solid fa-layer-group" style="color: var(--accent-cyan); margin-right: 6px; font-size: 12px"></i>
            {{ stack.name }}
          </div>
          <div class="row-meta">
            {{ getStackServices(stack.name).length }} service(s)
          </div>
        </div>

        <div style="flex: 1; text-align: center; font-family: var(--font-mono); font-weight: 600">
          {{ stack.services_count }}
        </div>

        <div style="flex: 1.5">
          <span class="badge badge--info">{{ stack.orchestrator }}</span>
        </div>

        <div style="flex: 1; text-align: right">
          <button
            class="action-btn"
            aria-label="Remove stack"
            title="Remove stack"
            @click.stop="$emit('remove', stack.name)"
          >
            <i class="fa-solid fa-trash-can" style="color: var(--accent-red)"></i>
          </button>
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

.badge--info {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  background: rgba(59, 130, 246, 0.1);
  color: var(--accent-blue);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.action-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-tertiary);
}
</style>
