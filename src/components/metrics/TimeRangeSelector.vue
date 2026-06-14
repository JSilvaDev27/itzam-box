<!-- ItzamBox — Time Range Selector Pill Component
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import type { TimeRange } from '../../composables/useMetricsHistory'

defineProps<{
  modelValue: TimeRange
}>()

const emit = defineEmits<{
  'update:modelValue': [value: TimeRange]
}>()

const ranges: { value: TimeRange; label: string; ariaLabel: string }[] = [
  { value: '1h', label: '1h', ariaLabel: '1 hour' },
  { value: '6h', label: '6h', ariaLabel: '6 hours' },
  { value: '24h', label: '24h', ariaLabel: '24 hours' },
  { value: '7d', label: '7d', ariaLabel: '7 days' },
  { value: '30d', label: '30d', ariaLabel: '30 days' },
]

function select(range: TimeRange) {
  emit('update:modelValue', range)
}
</script>

<template>
  <div class="time-range" role="group" :aria-label="'Time range selector'">
    <button
      v-for="r in ranges"
      :key="r.value"
      :class="['time-range__btn', { active: modelValue === r.value }]"
      :aria-label="r.ariaLabel"
      :aria-pressed="modelValue === r.value"
      @click="select(r.value)"
    >
      {{ r.label }}
    </button>
  </div>
</template>

<style scoped>
.time-range {
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-full);
  padding: 2px;
}

.time-range__btn {
  padding: 4px 12px;
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-mono);
  transition: all var(--transition-fast);
}

.time-range__btn:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.time-range__btn.active {
  background: var(--accent-cyan);
  color: var(--text-inverse);
}
</style>
