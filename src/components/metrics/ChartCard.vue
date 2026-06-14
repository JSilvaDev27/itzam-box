<!-- ItzamBox — SVG-based Chart Card with Tooltip, Zoom & Pan
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { MetricsDataPoint, MetricType } from '../../composables/useMetricsHistory'

const props = defineProps<{
  data: MetricsDataPoint[]
  secondaryData: MetricsDataPoint[] | null
  metricType: MetricType
  chartId: string
  isZoomed: boolean
}>()

const emit = defineEmits<{
  zoom: [start: number, end: number]
  resetZoom: []
}>()

// ── Chart dimensions ──
const MARGIN = { top: 20, right: 20, bottom: 30, left: 50 }
const SVG_WIDTH = 900
const SVG_HEIGHT = 280
const plotW = computed(() => SVG_WIDTH - MARGIN.left - MARGIN.right)
const plotH = computed(() => SVG_HEIGHT - MARGIN.top - MARGIN.bottom)

// ── Tooltip state ──
const tooltip = ref<{ visible: boolean; x: number; y: number; time: string; value: string; secondaryValue?: string }>({
  visible: false,
  x: 0,
  y: 0,
  time: '',
  value: '',
})

// ── Zoom / pan state ──
const isDragging = ref(false)
const dragStart = ref<{ x: number; dataIndex: number } | null>(null)
const dragEnd = ref<{ x: number } | null>(null)
const hoveredIndex = ref<number | null>(null)

// ── Colors ──
const primaryColor = computed(() => {
  const map: Record<MetricType, string> = {
    cpu: 'var(--accent-cyan)',
    memory: 'var(--accent-purple)',
    network: 'var(--accent-green)',
    disk: 'var(--accent-yellow)',
  }
  return map[props.metricType]
})

const secondaryColor = computed(() => {
  const map: Record<string, string> = {
    network: 'var(--accent-blue)',
    disk: 'var(--accent-orange, #f97316)',
  }
  return map[props.metricType] ?? 'var(--accent-green)'
})

// ── Scale computation ──
const yMin = computed(() => 0)
const yMax = computed(() => {
  if (props.data.length === 0) return 100
  const allValues = [...props.data.map(p => p.y)]
  if (props.secondaryData) allValues.push(...props.secondaryData.map(p => p.y))
  const max = Math.max(...allValues, 1)
  // Round up to nice number
  const magnitude = Math.pow(10, Math.floor(Math.log10(max)))
  return Math.ceil(max / magnitude) * magnitude || 100
})

const xMin = computed(() => props.data.length > 0 ? props.data[0].x : 0)
const xMax = computed(() => props.data.length > 0 ? props.data[props.data.length - 1].x : 1)
const xRange = computed(() => Math.max(xMax.value - xMin.value, 1))

// ── SVG coordinate mapping ──
function xToSvg(x: number): number {
  return MARGIN.left + ((x - xMin.value) / xRange.value) * plotW.value
}

function yToSvg(y: number): number {
  return MARGIN.top + (1 - (y - yMin.value) / (yMax.value - yMin.value)) * plotH.value
}

// ── Grid lines ──
const gridLines = computed(() => {
  const lines: { y: number; label: string }[] = []
  const steps = 5
  for (let i = 0; i <= steps; i++) {
    const val = yMin.value + (yMax.value - yMin.value) * (i / steps)
    const y = yToSvg(val)
    let label = ''
    if (props.metricType === 'cpu' || props.metricType === 'memory') {
      label = `${Math.round(val)}%`
    } else if (props.metricType === 'network') {
      label = val >= 1000 ? `${(val / 1000).toFixed(0)} GB/s` : `${val.toFixed(0)} MB/s`
    } else {
      label = val >= 1000 ? `${(val / 1000).toFixed(1)} GB/s` : `${val.toFixed(0)} MB/s`
    }
    lines.push({ y, label })
  }
  return lines
})

// ── X-axis labels ──
const xLabels = computed(() => {
  const count = 6
  const labels: { x: number; label: string }[] = []
  for (let i = 0; i < count; i++) {
    const t = xMin.value + (xRange.value / (count - 1)) * i
    const d = new Date(t)
    const h = String(d.getHours()).padStart(2, '0')
    const m = String(d.getMinutes()).padStart(2, '0')
    labels.push({ x: xToSvg(t), label: `${h}:${m}` })
  }
  return labels
})

// ── SVG path generation ──
function buildPath(data: MetricsDataPoint[]): string {
  if (data.length < 2) return ''
  return data.map((p, i) => {
    const x = xToSvg(p.x)
    const y = yToSvg(p.y)
    return `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`
  }).join(' ')
}

function buildAreaPath(data: MetricsDataPoint[]): string {
  if (data.length < 2) return ''
  const bottom = yToSvg(0)
  const linePath = buildPath(data)
  if (!linePath) return ''
  const lastX = xToSvg(data[data.length - 1].x)
  const firstX = xToSvg(data[0].x)
  return `${linePath} L${lastX.toFixed(1)},${bottom.toFixed(1)} L${firstX.toFixed(1)},${bottom.toFixed(1)} Z`
}

const primaryPath = computed(() => buildPath(props.data))
const primaryAreaPath = computed(() => buildAreaPath(props.data))
const secondaryPath = computed(() => props.secondaryData ? buildPath(props.secondaryData) : '')
const secondaryAreaPath = computed(() => props.secondaryData ? buildAreaPath(props.secondaryData) : '')

// ── Mouse interaction (tooltip + drag-zoom) ──
function findClosestIndex(clientX: number, svgEl: SVGSVGElement): number {
  const rect = svgEl.getBoundingClientRect()
  const relativeX = clientX - rect.left
  const svgXRatio = relativeX / rect.width * SVG_WIDTH
  // Map svgXRatio back to data x
  const dataX = xMin.value + (svgXRatio - MARGIN.left) / plotW.value * xRange.value
  let closest = 0
  let minDist = Infinity
  props.data.forEach((p, i) => {
    const dist = Math.abs(p.x - dataX)
    if (dist < minDist) {
      minDist = dist
      closest = i
    }
  })
  return closest
}

function onMouseMove(e: MouseEvent) {
  const svgEl = (e.currentTarget as SVGSVGElement)
  const idx = findClosestIndex(e.clientX, svgEl)
  if (idx < 0 || idx >= props.data.length) return

  hoveredIndex.value = idx
  const point = props.data[idx]
  const secondaryPoint = props.secondaryData?.[idx]

  const rect = svgEl.getBoundingClientRect()
  const svgX = xToSvg(point.x)
  const svgY = yToSvg(point.y)
  const ratioX = svgX / SVG_WIDTH * rect.width
  const ratioY = svgY / SVG_HEIGHT * rect.height

  const d = new Date(point.x)
  const time = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')} — ${d.toLocaleDateString()}`

  let value = ''
  if (props.metricType === 'cpu' || props.metricType === 'memory') {
    value = `${point.y.toFixed(1)}%`
  } else {
    value = `${point.y.toFixed(1)} MB/s`
  }

  let secondaryValue: string | undefined
  if (secondaryPoint !== undefined) {
    secondaryValue = `${secondaryPoint.y.toFixed(1)} MB/s`
  }

  tooltip.value = {
    visible: true,
    x: ratioX + rect.left,
    y: Math.max(0, ratioY + rect.top - 60),
    time,
    value,
    secondaryValue,
  }
}

function onMouseLeave() {
  tooltip.value.visible = false
  hoveredIndex.value = null
  if (!isDragging.value) {
    dragStart.value = null
    dragEnd.value = null
  }
}

function onMouseDown(e: MouseEvent) {
  const svgEl = (e.currentTarget as SVGSVGElement)
  const idx = findClosestIndex(e.clientX, svgEl)
  isDragging.value = true
  dragStart.value = { x: e.clientX, dataIndex: idx }
  dragEnd.value = null
}

function onMouseUp(e: MouseEvent) {
  if (!isDragging.value || !dragStart.value) return
  isDragging.value = false

  const delta = e.clientX - dragStart.value.x
  if (Math.abs(delta) > 20) {
    // Zoom to selection
    const svgEl = (e.currentTarget as SVGSVGElement)
    const rect = svgEl.getBoundingClientRect()
    const startRatio = (dragStart.value.x - rect.left) / rect.width
    const endRatio = (e.clientX - rect.left) / rect.width
    const dataStart = xMin.value + startRatio * xRange.value
    const dataEnd = xMin.value + endRatio * xRange.value

    if (dataEnd > dataStart) {
      emit('zoom', dataStart, dataEnd)
    }
  }
  dragStart.value = null
  dragEnd.value = null
}

// ── Total / capacity line (memory) ──
const showTotalLine = computed(() => props.metricType === 'memory')

// ── Legend items ──
const legendItems = computed(() => {
  const items: { color: string; label: string; dashed?: boolean; dotted?: boolean }[] = [
    { color: primaryColor.value, label: props.metricType === 'network' ? 'RX' : props.metricType === 'disk' ? 'Read' : props.metricType === 'cpu' ? 'Usage' : 'Used' },
  ]
  if (props.secondaryData) {
    items.push({
      color: secondaryColor.value,
      label: props.metricType === 'network' ? 'TX' : 'Write',
    })
  }
  if (props.metricType === 'memory') {
    items.push({ color: primaryColor.value, label: 'Total', dashed: true })
  }
  return items
})
</script>

<template>
  <div
    :id="chartId"
    class="chart-container"
    :class="{ 'chart-container--zoomed': isZoomed }"
  >
    <div class="chart-area" style="position:relative">
      <svg
        class="chart-svg"
        :viewBox="`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`"
        preserveAspectRatio="none"
        @mousemove="onMouseMove"
        @mouseleave="onMouseLeave"
        @mousedown="onMouseDown"
        @mouseup="onMouseUp"
        style="cursor:crosshair"
      >
        <!-- Grid lines -->
        <line
          v-for="(gl, i) in gridLines"
          :key="i"
          :x1="MARGIN.left" :y1="gl.y"
          :x2="MARGIN.left + plotW" :y2="gl.y"
          class="chart-grid-line"
        />

        <!-- Y-axis labels -->
        <text
          v-for="(gl, i) in gridLines"
          :key="'y' + i"
          :x="MARGIN.left - 8" :y="gl.y + 4"
          class="chart-axis-label"
          text-anchor="end"
        >{{ gl.label }}</text>

        <!-- X-axis labels -->
        <text
          v-for="(xl, i) in xLabels"
          :key="'x' + i"
          :x="xl.x" :y="SVG_HEIGHT - 6"
          class="chart-axis-label"
          text-anchor="middle"
        >{{ xl.label }}</text>

        <!-- Secondary area fill -->
        <path
          v-if="secondaryAreaPath"
          :d="secondaryAreaPath"
          :fill="secondaryColor"
          class="chart-area-fill"
        />

        <!-- Primary area fill -->
        <path
          v-if="primaryAreaPath"
          :d="primaryAreaPath"
          :fill="primaryColor"
          class="chart-area-fill"
        />

        <!-- Total line (memory) -->
        <line
          v-if="showTotalLine"
          :x1="MARGIN.left" :y1="MARGIN.top"
          :x2="MARGIN.left + plotW" :y2="MARGIN.top"
          :stroke="primaryColor"
          stroke-width="1"
          stroke-dasharray="6,4"
          opacity="0.5"
        />
        <text
          v-if="showTotalLine"
          :x="MARGIN.left + plotW + 4" :y="MARGIN.top + 4"
          :fill="primaryColor"
          font-size="9"
          font-family="var(--font-mono)"
        >Total</text>

        <!-- Secondary line -->
        <path
          v-if="secondaryPath"
          :d="secondaryPath"
          :stroke="secondaryColor"
          class="chart-line"
        />

        <!-- Primary line -->
        <path
          v-if="primaryPath"
          :d="primaryPath"
          :stroke="primaryColor"
          class="chart-line"
        />

        <!-- Hovered data point dot -->
        <circle
          v-if="hoveredIndex !== null && hoveredIndex < data.length"
          :cx="xToSvg(data[hoveredIndex].x)"
          :cy="yToSvg(data[hoveredIndex].y)"
          r="4"
          :fill="primaryColor"
          stroke="var(--bg-primary)"
          stroke-width="2"
        />

        <!-- Dragged selection rectangle -->
        <rect
          v-if="isDragging && dragStart"
          :x="xToSvg(xMin + ((Math.min(dragStart?.x ?? 0, 0) / SVG_WIDTH) * xRange))"
          y="0"
          :width="0"
          :height="SVG_HEIGHT"
          fill="var(--accent-cyan)"
          opacity="0.1"
        />
      </svg>

      <!-- Tooltip -->
      <div
        v-if="tooltip.visible"
        class="chart-tooltip"
        :class="{ visible: tooltip.visible }"
        :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
      >
        <div class="chart-tooltip__time">{{ tooltip.time }}</div>
        <div class="chart-tooltip__value" :style="{ color: primaryColor }">
          {{ tooltip.value }}
        </div>
        <div
          v-if="tooltip.secondaryValue"
          class="chart-tooltip__value"
          :style="{ color: secondaryColor, fontSize: '12px' }"
        >
          {{ tooltip.secondaryValue }}
        </div>
      </div>
    </div>

    <!-- Legend -->
    <div class="chart-legend">
      <span
        v-for="(item, i) in legendItems"
        :key="i"
        class="chart-legend__item"
      >
        <span
          class="chart-legend__line"
          :style="{
            background: item.color,
            ...(item.dashed ? { height: '0', borderTop: '2px dashed ' + item.color, background: 'transparent' } : {}),
            ...(item.dotted ? { height: '0', borderTop: '2px dotted ' + item.color, background: 'transparent' } : {}),
          }"
        ></span>
        {{ item.label }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.chart-container {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  position: relative;
  transition: border-color var(--transition-fast);
}

.chart-container:hover {
  border-color: rgba(0, 229, 255, 0.15);
}

.chart-container--zoomed {
  border-color: var(--accent-cyan);
  box-shadow: var(--shadow-glow);
}

.chart-area {
  padding: var(--space-5, 20px);
  height: 340px;
  position: relative;
}

.chart-svg {
  width: 100%;
  height: 100%;
}

.chart-grid-line {
  stroke: var(--border-light);
  stroke-width: 1;
}

.chart-axis-label {
  fill: var(--text-disabled);
  font-size: 10px;
  font-family: var(--font-mono);
}

.chart-line {
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.chart-area-fill {
  opacity: 0.12;
}

.chart-tooltip {
  position: fixed;
  background: var(--chart-tooltip-bg, rgba(18, 22, 31, 0.95));
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: var(--space-2, 8px) var(--space-3, 12px);
  font-size: 12px;
  pointer-events: none;
  z-index: var(--z-toast, 500);
  box-shadow: var(--shadow-lg);
  transform: translate(-50%, -100%);
  white-space: nowrap;
}

.chart-tooltip__time {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.chart-tooltip__value {
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 14px;
}

.chart-legend {
  display: flex;
  gap: 16px;
  padding: 0 var(--space-5, 20px) var(--space-3, 12px);
  font-size: 11px;
  color: var(--text-muted);
}

.chart-legend__item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.chart-legend__line {
  width: 14px;
  height: 3px;
  border-radius: 2px;
  flex-shrink: 0;
}
</style>
