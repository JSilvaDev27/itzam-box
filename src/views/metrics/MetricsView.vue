<!-- ItzamBox — Metrics & Historical Charts View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  useMetricsHistory,
  type MetricType,
  type TimeRange,
} from '../../composables/useMetricsHistory'
import { useMetricsExport } from '../../composables/useMetricsExport'
import SkeletonLoader from '../../components/shared/SkeletonLoader.vue'
import TimeRangeSelector from '../../components/metrics/TimeRangeSelector.vue'
import MetricSummaryCards from '../../components/metrics/MetricSummaryCards.vue'
import ChartCard from '../../components/metrics/ChartCard.vue'
import ChartExport from '../../components/metrics/ChartExport.vue'

const route = useRoute()
const router = useRouter()

const {
  loading,
  activeMetric,
  activeRange,
  cpuData,
  memoryData,
  networkRxData,
  networkTxData,
  diskReadData,
  diskWriteData,
  summary,
  loadData,
  setTimeRange,
  setMetric,
  resetZoom,
  zoomRange,
} = useMetricsHistory()

const { exportCSV, exportJSON, exportPNG } = useMetricsExport()

const isZoomed = ref(false)
let autoRefreshInterval: number | undefined

// ── Tabs configuration ──
const tabs: { key: MetricType; icon: string; label: string }[] = [
  { key: 'cpu', icon: 'fa-microchip', label: 'CPU' },
  { key: 'memory', icon: 'fa-memory', label: 'Memory' },
  { key: 'network', icon: 'fa-network-wired', label: 'Network' },
  { key: 'disk', icon: 'fa-hard-drive', label: 'Disk' },
]

// ── Current metric data ──
const activeData = computed(() => {
  switch (activeMetric.value) {
    case 'cpu': return cpuData.value
    case 'memory': return memoryData.value
    case 'network': return networkRxData.value
    case 'disk': return diskReadData.value
  }
})

const activeSecondaryData = computed(() => {
  switch (activeMetric.value) {
    case 'network': return networkTxData.value
    case 'disk': return diskWriteData.value
    default: return null
  }
})

const unit = computed(() => {
  if (activeMetric.value === 'cpu' || activeMetric.value === 'memory') return '%'
  return ' MB/s'
})

const secondaryLabel = computed(() => {
  switch (activeMetric.value) {
    case 'cpu': return `${cpuData.value.length} data points`
    case 'memory': return `${memoryData.value.length} data points`
    case 'network': return 'RX throughput'
    case 'disk': return 'Read throughput'
  }
})

const hasData = computed(() => activeData.value.length > 0)

// ── Methods ──
function onTabClick(metric: MetricType) {
  setMetric(metric)
  isZoomed.value = false
  // Deep-link via query param
  router.replace({ query: { metric } })
}

function onRangeChanged(range: TimeRange) {
  setTimeRange(range)
  isZoomed.value = false
}

function onZoom(start: number, end: number) {
  zoomRange(start, end)
  isZoomed.value = true
}

function onResetZoom() {
  resetZoom()
  isZoomed.value = false
}

function handleExportCSV() {
  exportCSV(activeData.value, activeSecondaryData.value, activeMetric.value)
}

function handleExportJSON() {
  exportJSON(activeData.value, activeSecondaryData.value, activeMetric.value, activeRange.value)
}

function handleExportPNG() {
  exportPNG(`chart-${activeMetric.value}`, activeMetric.value)
}

// ── Lifecycle ──
onMounted(async () => {
  // Restore metric from query param
  const queryMetric = route.query.metric as MetricType | undefined
  if (queryMetric && ['cpu', 'memory', 'network', 'disk'].includes(queryMetric)) {
    activeMetric.value = queryMetric
  }

  await loadData()

  // Auto-refresh for short ranges
  autoRefreshInterval = window.setInterval(async () => {
    if (activeRange.value === '1h' || activeRange.value === '6h') {
      await loadData()
    }
  }, 10_000)
})

onUnmounted(() => {
  if (autoRefreshInterval) clearInterval(autoRefreshInterval)
})
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span>System</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Metrics</span>
  </div>

  <div class="content__page-header">
    <h1 class="text-h1">
      <i class="fa-solid fa-activity" style="color:var(--accent-cyan);margin-right:8px;font-size:1.5rem"></i>
      Metrics
    </h1>
  </div>

  <!-- Sub-tabs: CPU | Memory | Network | Disk -->
  <div class="tabs" role="tablist">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      :class="['tabs__item', { active: activeMetric === tab.key }]"
      role="tab"
      :aria-selected="activeMetric === tab.key"
      @click="onTabClick(tab.key)"
    >
      <i :class="'fa-solid ' + tab.icon" style="margin-right:4px;font-size:11px"></i>
      {{ tab.label }}
    </button>
  </div>

  <!-- Loading skeleton -->
  <SkeletonLoader v-if="loading && !hasData" variant="chart" :count="1" />

  <!-- Empty / collecting data state -->
  <div v-else-if="!hasData && !loading" class="chart-empty">
    <div class="chart-empty__icon">
      <i class="fa-solid fa-chart-line"></i>
    </div>
    <div class="chart-empty__title">No data available</div>
    <div class="chart-empty__desc">
      Metrics data collection is in progress. Data will appear once the first samples are recorded.
    </div>
    <div class="chart-empty__progress">
      <i class="fa-solid fa-circle-notch fa-spin"></i>
      Collecting data...
    </div>
  </div>

  <!-- Metrics content -->
  <template v-if="hasData">
    <!-- Summary cards -->
    <MetricSummaryCards
      :summary="summary"
      :metric-type="activeMetric"
      :unit="unit"
      :secondary-label="secondaryLabel"
    />

    <!-- Chart toolbar -->
    <div class="chart-toolbar">
      <div class="chart-toolbar__left">
        <TimeRangeSelector
          :model-value="activeRange"
          @update:model-value="onRangeChanged"
        />
        <div class="chart-options">
          <!-- Zoom reset button -->
          <button
            v-if="isZoomed"
            class="btn btn--ghost btn--sm zoom-reset visible"
            @click="onResetZoom"
            aria-label="Reset zoom"
          >
            <i class="fa-solid fa-compress"></i> Reset Zoom
          </button>
        </div>
      </div>
      <div class="chart-toolbar__right">
        <ChartExport
          :data="activeData"
          :secondary-data="activeSecondaryData"
          :metric-type="activeMetric"
          :time-range="activeRange"
          :chart-id="'chart-' + activeMetric"
          @export-csv="handleExportCSV"
          @export-json="handleExportJSON"
          @export-png="handleExportPNG"
        />
      </div>
    </div>

    <!-- Chart -->
    <ChartCard
      :data="activeData"
      :secondary-data="activeSecondaryData"
      :metric-type="activeMetric"
      :chart-id="'chart-' + activeMetric"
      :is-zoomed="isZoomed"
      @zoom="onZoom"
      @reset-zoom="onResetZoom"
    />

    <!-- Zoom hint -->
    <div class="zoom-hint">
      <i class="fa-solid fa-hand-pointer"></i>
      Click and drag on the chart to zoom into a region
    </div>
  </template>
</template>

<style scoped>
.content__page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.chart-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3, 12px);
}

.chart-toolbar__left {
  display: flex;
  align-items: center;
  gap: var(--space-3, 12px);
}

.chart-toolbar__right {
  display: flex;
  align-items: center;
  gap: var(--space-2, 8px);
}

.chart-options {
  display: flex;
  align-items: center;
  gap: var(--space-3, 12px);
}

.zoom-reset {
  display: none;
}

.zoom-reset.visible {
  display: flex;
}

.zoom-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-disabled);
}

.zoom-hint i {
  font-size: 10px;
}

/* Empty chart state */
.chart-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  gap: var(--space-3, 12px);
  text-align: center;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: var(--space-10, 40px);
}

.chart-empty__icon {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-xl, 16px);
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  color: var(--text-disabled);
}

.chart-empty__title {
  font-size: 14px;
  font-weight: 600;
}

.chart-empty__desc {
  font-size: 12px;
  color: var(--text-muted);
  max-width: 360px;
  line-height: 1.5;
}

.chart-empty__progress {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--accent-cyan);
  font-family: var(--font-mono);
}
</style>
