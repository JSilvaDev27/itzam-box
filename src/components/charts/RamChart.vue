<!-- ItzamBox — RAM Time-Series Chart Component
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import {
  Chart,
  LineController,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale,
  Filler,
  Tooltip,
  Legend,
  type ChartConfiguration,
} from 'chart.js'

Chart.register(
  LineController, LineElement, PointElement,
  LinearScale, CategoryScale,
  Filler, Tooltip, Legend,
)

interface TimePoint {
  x: number
  y: number
}

const props = defineProps<{
  data: TimePoint[]
  totalGb: number
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
let chartInstance: Chart<'line', number[], string> | null = null

function formatTime(ts: number): string {
  const d = new Date(ts)
  const h = String(d.getHours()).padStart(2, '0')
  const m = String(d.getMinutes()).padStart(2, '0')
  const s = String(d.getSeconds()).padStart(2, '0')
  return `${h}:${m}:${s}`
}

const labels = computed(() => props.data.map(p => formatTime(p.x)))
const values = computed(() => props.data.map(p => p.y))

function getCssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

const lineColor = computed(() => {
  return getCssVar('--accent-yellow') || '#f59e0b'
})

function safeGb(bytes: number): string {
  return (bytes / 1e9).toFixed(1)
}

function buildConfig(): ChartConfiguration<'line', number[], string> {
  const textMuted = getCssVar('--text-muted') || '#9ca3af'
  const gridColor = getCssVar('--border-light') || 'rgba(255,255,255,0.06)'
  const tooltipBg = getCssVar('--bg-tertiary') || '#1b202e'
  const tooltipBorder = getCssVar('--border-color') || '#262f45'
  const tooltipText = getCssVar('--text-main') || '#f3f4f6'
  const color = lineColor.value

  return {
    type: 'line',
    data: {
      labels: labels.value,
      datasets: [
        {
          label: 'RAM %',
          data: values.value,
          borderColor: color,
          backgroundColor: (ctx) => {
            if (!ctx.chart.chartArea) return `${color}20`
            const { top, bottom } = ctx.chart.chartArea
            const gradient = ctx.chart.ctx.createLinearGradient(0, top, 0, bottom)
            gradient.addColorStop(0, `${color}30`)
            gradient.addColorStop(1, `${color}02`)
            return gradient
          },
          fill: true,
          tension: 0.3,
          pointRadius: 0,
          pointHitRadius: 6,
          borderWidth: 2,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: { duration: 300 },
      interaction: {
        intersect: false,
        mode: 'index',
      },
      scales: {
        x: {
          type: 'category',
          display: true,
          ticks: {
            color: textMuted,
            font: { size: 9 },
            maxTicksLimit: 8,
          },
          grid: { display: false },
        },
        y: {
          min: 0,
          max: 100,
          display: true,
          ticks: {
            color: textMuted,
            font: { size: 9 },
            callback: (v) => `${v}%`,
          },
          grid: { color: gridColor },
        },
      },
      plugins: {
        legend: { display: false },
        tooltip: {
          enabled: true,
          backgroundColor: tooltipBg,
          titleColor: tooltipText,
          bodyColor: tooltipText,
          borderColor: tooltipBorder,
          borderWidth: 1,
          padding: 8,
          callbacks: {
            title(items) {
              if (items.length > 0) return items[0].label
              return ''
            },
            label(item) {
              const usedGb = safeGb((props.totalGb * item.parsed.y) / 100)
              return `RAM: ${usedGb} GB / ${props.totalGb.toFixed(1)} GB (${item.parsed.y.toFixed(1)}%)`
            },
          },
        },
      },
    },
  } as ChartConfiguration<'line', number[], string>
}

function createOrUpdateChart() {
  if (!canvasRef.value) return

  if (!chartInstance) {
    chartInstance = new Chart(canvasRef.value, buildConfig())
  } else {
    chartInstance.data.labels = labels.value
    chartInstance.data.datasets[0].data = values.value
    chartInstance.update()
  }
}

onMounted(() => {
  createOrUpdateChart()
})

watch([labels, values], () => {
  createOrUpdateChart()
}, { deep: false })

onUnmounted(() => {
  if (chartInstance) {
    chartInstance.destroy()
    chartInstance = null
  }
})
</script>

<template>
  <div class="chart-card">
    <div class="chart-header">
      <span class="chart-label">Memory Usage</span>
      <span class="chart-meta">{{ totalGb.toFixed(1) }} GB total</span>
    </div>
    <div class="chart-canvas-wrapper">
      <canvas ref="canvasRef" />
    </div>
  </div>
</template>

<style scoped>
.chart-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition: border-color 0.2s;
}
.chart-card:hover {
  border-color: rgba(245, 158, 11, 0.15);
}
.chart-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 0;
}
.chart-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: var(--text-muted);
}
.chart-meta {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-disabled);
}
.chart-canvas-wrapper {
  padding: 8px 12px 4px;
  height: 160px;
  position: relative;
}
</style>
