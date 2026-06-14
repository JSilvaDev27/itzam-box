// ItzamBox — Metrics Historical Data Composable
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ── Types ────────────────────────────────────────────────────────────

export interface MetricsDataPoint {
  x: number  // unix ms
  y: number  // value
}

export interface HostMetricsSnapshot {
  cpu_percent: number
  memory_used_bytes: number
  memory_total_bytes: number
  network_rx_bytes: number
  network_tx_bytes: number
  disk_read_bytes: number
  disk_write_bytes: number
  timestamp: number
}

export type MetricType = 'cpu' | 'memory' | 'network' | 'disk'
export type TimeRange = '1h' | '6h' | '24h' | '7d' | '30d'

export interface MetricSummary {
  current: number
  average: number
  max: number
  min: number
  maxAt: number   // timestamp of max
  minAt: number   // timestamp of min
}

export const TIME_RANGE_MS: Record<TimeRange, number> = {
  '1h': 3_600_000,
  '6h': 21_600_000,
  '24h': 86_400_000,
  '7d': 604_800_000,
  '30d': 2_592_000_000,
}

// ── Metric color CSS variable map ──

export const METRIC_COLORS: Record<MetricType, string> = {
  cpu: 'var(--accent-cyan)',
  memory: 'var(--accent-purple)',
  network: 'var(--accent-green)',
  disk: 'var(--accent-yellow)',
}

export const METRIC_ICONS: Record<MetricType, string> = {
  cpu: 'fa-microchip',
  memory: 'fa-memory',
  network: 'fa-network-wired',
  disk: 'fa-hard-drive',
}

export const METRIC_LABELS: Record<MetricType, string> = {
  cpu: 'CPU',
  memory: 'Memory',
  network: 'Network',
  disk: 'Disk',
}

// ── Composable ───────────────────────────────────────────────────────

export function useMetricsHistory() {
  const loading = ref(false)
  const error = ref<string | null>(null)
  const activeMetric = ref<MetricType>('cpu')
  const activeRange = ref<TimeRange>('1h')

  // Per-metric data
  const cpuData = ref<MetricsDataPoint[]>([])
  const memoryData = ref<MetricsDataPoint[]>([])
  const networkRxData = ref<MetricsDataPoint[]>([])
  const networkTxData = ref<MetricsDataPoint[]>([])
  const diskReadData = ref<MetricsDataPoint[]>([])
  const diskWriteData = ref<MetricsDataPoint[]>([])

  // Summary for active metric
  const summary = ref<MetricSummary>({
    current: 0,
    average: 0,
    max: 0,
    min: 0,
    maxAt: Date.now(),
    minAt: Date.now(),
  })

  // ── Helpers ──

  function computeSummary(data: MetricsDataPoint[], _metricType: MetricType): MetricSummary {
    if (data.length === 0) {
      return { current: 0, average: 0, max: 0, min: 0, maxAt: Date.now(), minAt: Date.now() }
    }
    const values = data.map(p => p.y)
    const current = values[values.length - 1]
    const average = values.reduce((a, b) => a + b, 0) / values.length
    const max = Math.max(...values)
    const min = Math.min(...values)
    const maxAt = data.find(p => p.y === max)?.x ?? Date.now()
    const minAt = data.find(p => p.y === min)?.x ?? Date.now()
    return { current, average, max, min, maxAt, minAt }
  }

  // ── Data generation (mock for frontend — connects to backend when available) ──

  function generateMockData(from: number, to: number, metricType: MetricType): MetricsDataPoint[] {
    const count = Math.min(120, Math.floor((to - from) / 15_000))
    const points: MetricsDataPoint[] = []
    const interval = (to - from) / count

    for (let i = 0; i < count; i++) {
      const t = from + i * interval
      let y = 0
      // Base sine wave + noise for realistic-looking data
      const phase = (i / count) * Math.PI * 4
      switch (metricType) {
        case 'cpu':
          y = 30 + Math.sin(phase) * 25 + Math.random() * 15
          y = Math.max(0, Math.min(100, y))
          break
        case 'memory':
          y = 55 + Math.sin(phase * 0.5) * 10 + Math.random() * 5
          y = Math.max(0, Math.min(100, y))
          break
        case 'network':
          y = 20 + Math.sin(phase * 0.7) * 15 + Math.random() * 20
          y = Math.max(0, y)
          break
        case 'disk':
          y = 10 + Math.sin(phase * 1.2) * 8 + Math.random() * 12
          y = Math.max(0, y)
          break
      }
      points.push({ x: t, y: Math.round(y * 10) / 10 })
    }
    return points
  }

  async function loadData(_forceGenerate = false) {
    const from = Date.now() - TIME_RANGE_MS[activeRange.value]
    const to = Date.now()

    loading.value = true
    error.value = null

    try {
      // Try to load from backend
      const result = await invoke<HostMetricsSnapshot[]>('get_host_metrics_range', {
        from: Math.floor(from / 1000),
        to: Math.floor(to / 1000),
      })

      if (result && result.length > 0) {
        cpuData.value = result.map(r => ({ x: r.timestamp * 1000, y: r.cpu_percent }))
        memoryData.value = result.map(r => ({
          x: r.timestamp * 1000,
          y: (r.memory_used_bytes / r.memory_total_bytes) * 100,
        }))
        networkRxData.value = result.map(r => ({ x: r.timestamp * 1000, y: r.network_rx_bytes / 1_000_000 }))
        networkTxData.value = result.map(r => ({ x: r.timestamp * 1000, y: r.network_tx_bytes / 1_000_000 }))
        diskReadData.value = result.map(r => ({ x: r.timestamp * 1000, y: r.disk_read_bytes / 1_000_000 }))
        diskWriteData.value = result.map(r => ({ x: r.timestamp * 1000, y: r.disk_write_bytes / 1_000_000 }))
      } else {
        // Fallback: generate mock data
        cpuData.value = generateMockData(from, to, 'cpu')
        memoryData.value = generateMockData(from, to, 'memory')
        networkRxData.value = generateMockData(from, to, 'network')
        networkTxData.value = generateMockData(from, to, 'network')
        diskReadData.value = generateMockData(from, to, 'disk')
        diskWriteData.value = generateMockData(from, to, 'disk')
      }

      // Update summary based on active metric
      const activePoints = getActiveData()
      summary.value = computeSummary(activePoints, activeMetric.value)
    } catch (e: any) {
      // Backend not available — use mock data
      cpuData.value = generateMockData(from, to, 'cpu')
      memoryData.value = generateMockData(from, to, 'memory')
      networkRxData.value = generateMockData(from, to, 'network')
      networkTxData.value = generateMockData(from, to, 'network')
      diskReadData.value = generateMockData(from, to, 'disk')
      diskWriteData.value = generateMockData(from, to, 'disk')

      const activePoints = getActiveData()
      summary.value = computeSummary(activePoints, activeMetric.value)
    } finally {
      loading.value = false
    }
  }

  function getActiveData(): MetricsDataPoint[] {
    switch (activeMetric.value) {
      case 'cpu':
        return cpuData.value
      case 'memory':
        return memoryData.value
      case 'network':
        // For network, return RX as primary, TX as secondary
        return networkRxData.value
      case 'disk':
        return diskReadData.value
    }
  }

  function getActiveSecondaryData(): MetricsDataPoint[] | null {
    switch (activeMetric.value) {
      case 'network':
        return networkTxData.value
      case 'disk':
        return diskWriteData.value
      default:
        return null
    }
  }

  function setTimeRange(range: TimeRange) {
    activeRange.value = range
    loadData(true)
  }

  function setMetric(metric: MetricType) {
    activeMetric.value = metric
    const activePoints = getActiveData()
    summary.value = computeSummary(activePoints, metric)
  }

  function resetZoom() {
    loadData(true)
  }

  function zoomRange(start: number, end: number) {
    const allData = getActiveData()
    if (!allData || allData.length === 0) return

    const filtered = allData.filter(p => p.x >= start && p.x <= end)
    // Assign filtered data back based on active metric
    switch (activeMetric.value) {
      case 'cpu':
        cpuData.value = filtered
        break
      case 'memory':
        memoryData.value = filtered
        break
      case 'network':
        networkRxData.value = filtered
        break
      case 'disk':
        diskReadData.value = filtered
        break
    }
    summary.value = computeSummary(filtered, activeMetric.value)
  }

  return {
    // State
    loading,
    error,
    activeMetric,
    activeRange,
    cpuData,
    memoryData,
    networkRxData,
    networkTxData,
    diskReadData,
    diskWriteData,
    summary,

    // Methods
    loadData,
    setTimeRange,
    setMetric,
    resetZoom,
    zoomRange,
    getActiveData,
    getActiveSecondaryData,
  }
}
