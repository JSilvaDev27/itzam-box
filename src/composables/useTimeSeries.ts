// ItzamBox — Rolling Window Time-Series Composable
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref } from 'vue'

export interface TimePoint {
  x: number // timestamp (unix ms)
  y: number // value 0–100
}

export function useTimeSeries(maxPoints = 60) {
  const cpuHistory = ref<TimePoint[]>([])
  const ramHistory = ref<TimePoint[]>([])

  function push(cpuPercent: number, ramPercent: number) {
    const now = Date.now()
    cpuHistory.value.push({ x: now, y: cpuPercent })
    ramHistory.value.push({ x: now, y: ramPercent })
    if (cpuHistory.value.length > maxPoints) cpuHistory.value.shift()
    if (ramHistory.value.length > maxPoints) ramHistory.value.shift()
  }

  function clear() {
    cpuHistory.value = []
    ramHistory.value = []
  }

  return { cpuHistory, ramHistory, push, clear }
}
