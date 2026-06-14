<!-- ItzamBox — Chart Export Dropdown Component
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref } from 'vue'
import type { MetricType, TimeRange, MetricsDataPoint } from '../../composables/useMetricsHistory'

const props = defineProps<{
  data: MetricsDataPoint[]
  secondaryData: MetricsDataPoint[] | null
  metricType: MetricType
  timeRange: TimeRange
  chartId: string
}>()

const emit = defineEmits<{
  exportCSV: []
  exportJSON: []
  exportPNG: []
}>()

const menuOpen = ref(false)

function toggleMenu() {
  menuOpen.value = !menuOpen.value
}

function closeMenu() {
  menuOpen.value = false
}

function handleCSV() {
  emit('exportCSV')
  closeMenu()
}

function handleJSON() {
  emit('exportJSON')
  closeMenu()
}

function handlePNG() {
  emit('exportPNG')
  closeMenu()
}

function onBackdropClick(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('.export-wrapper')) return
  closeMenu()
}
</script>

<template>
  <div class="export-wrapper" style="position:relative">
    <button
      class="btn btn--secondary btn--sm"
      :aria-label="'Export metrics'"
      :aria-expanded="menuOpen"
      @click="toggleMenu"
    >
      <i class="fa-solid fa-download"></i> Export
    </button>

    <Teleport to="body">
      <div v-if="menuOpen" class="export-backdrop" @click="onBackdropClick" @contextmenu.prevent></div>
    </Teleport>

    <div v-if="menuOpen" class="export-menu" role="menu" aria-label="Export options">
      <div class="export-menu__item" role="menuitem" tabindex="0" @click="handleCSV" @keydown.enter="handleCSV">
        <i class="fa-solid fa-file-csv" style="color:var(--accent-green)"></i>
        Export as CSV
      </div>
      <div class="export-menu__item" role="menuitem" tabindex="0" @click="handleJSON" @keydown.enter="handleJSON">
        <i class="fa-solid fa-file-code" style="color:var(--accent-cyan)"></i>
        Export as JSON
      </div>
      <div class="export-menu__item" role="menuitem" tabindex="0" @click="handlePNG" @keydown.enter="handlePNG">
        <i class="fa-solid fa-image" style="color:var(--accent-purple)"></i>
        Save as PNG
      </div>
    </div>
  </div>
</template>

<style scoped>
.export-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  min-width: 180px;
  z-index: var(--z-dropdown, 100);
  padding: var(--space-1, 4px);
}

.export-menu__item {
  display: flex;
  align-items: center;
  gap: var(--space-2, 8px);
  padding: var(--space-2, 8px) var(--space-3, 12px);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--text-main);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.export-menu__item:hover {
  background: var(--bg-hover);
}

.export-backdrop {
  position: fixed;
  inset: 0;
  z-index: calc(var(--z-dropdown, 100) - 1);
}
</style>
