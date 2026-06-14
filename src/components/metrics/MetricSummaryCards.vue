<!-- ItzamBox — Metric Summary Cards Component
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'
import type { MetricType, MetricSummary } from '../../composables/useMetricsHistory'

const props = defineProps<{
  summary: MetricSummary
  metricType: MetricType
  unit: string
  secondaryLabel?: string
}>()

const colorVar = computed(() => {
  const map: Record<MetricType, string> = {
    cpu: 'var(--accent-cyan)',
    memory: 'var(--accent-purple)',
    network: 'var(--accent-green)',
    disk: 'var(--accent-yellow)',
  }
  return map[props.metricType]
})

function formatTime(ts: number): string {
  const d = new Date(ts)
  const h = String(d.getHours()).padStart(2, '0')
  const m = String(d.getMinutes()).padStart(2, '0')
  return `${h}:${m}`
}

function formatValue(val: number): string {
  if (props.metricType === 'network' || props.metricType === 'disk') {
    return val.toFixed(1)
  }
  return val.toFixed(1)
}
</script>

<template>
  <div class="metrics-summary-grid">
    <div class="metric-card">
      <div class="metric-card__label">Current</div>
      <div class="metric-card__value" :style="{ color: colorVar }">
        {{ formatValue(summary.current) }}<span class="metric-card__unit">{{ unit }}</span>
      </div>
      <div class="metric-card__sub">{{ secondaryLabel ?? 'Latest value' }}</div>
    </div>
    <div class="metric-card">
      <div class="metric-card__label">Average</div>
      <div class="metric-card__value" style="color:var(--text-main)">
        {{ formatValue(summary.average) }}<span class="metric-card__unit">{{ unit }}</span>
      </div>
      <div class="metric-card__sub">Over selected range</div>
    </div>
    <div class="metric-card">
      <div class="metric-card__label">Peak</div>
      <div class="metric-card__value" style="color:var(--accent-yellow)">
        {{ formatValue(summary.max) }}<span class="metric-card__unit">{{ unit }}</span>
      </div>
      <div class="metric-card__sub">at {{ formatTime(summary.maxAt) }}</div>
    </div>
    <div class="metric-card">
      <div class="metric-card__label">Minimum</div>
      <div class="metric-card__value" style="color:var(--accent-green)">
        {{ formatValue(summary.min) }}<span class="metric-card__unit">{{ unit }}</span>
      </div>
      <div class="metric-card__sub">at {{ formatTime(summary.minAt) }}</div>
    </div>
  </div>
</template>

<style scoped>
.metrics-summary-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-4, 16px);
}

.metric-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: var(--space-4, 16px) var(--space-5, 20px);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metric-card__label {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.metric-card__value {
  font-size: 1.75rem;
  font-weight: 700;
  font-family: var(--font-mono);
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.metric-card__unit {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-muted);
  margin-left: 2px;
}

.metric-card__sub {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
