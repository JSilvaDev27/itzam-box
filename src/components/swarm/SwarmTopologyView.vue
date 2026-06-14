<!-- ItzamBox — Swarm Topology SVG Graph
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { computed, ref } from 'vue'
import type { SwarmNode, SwarmService } from '../../composables/useSwarm'

const props = defineProps<{
  nodes: SwarmNode[]
  services: SwarmService[]
}>()

const hoveredNode = ref<string | null>(null)
const hoveredService = ref<string | null>(null)

const NODE_WIDTH = 180
const NODE_HEIGHT = 38
const NODE_GAP = 12
const SVG_PADDING = 20

const activeNodes = computed(() => {
  return props.nodes.filter(n => n.status === 'Ready')
})

const nodePositions = computed(() => {
  return activeNodes.value.map((n, i) => ({
    id: n.id,
    hostname: n.hostname,
    role: n.role,
    status: n.status,
    y: SVG_PADDING + i * (NODE_HEIGHT + NODE_GAP),
  }))
})

const servicePositions = computed(() => {
  return props.services.map((s, i) => ({
    id: s.id,
    name: s.name,
    mode: s.mode,
    replicas: s.replicas,
    y: SVG_PADDING + i * (NODE_HEIGHT + NODE_GAP),
  }))
})

const svgHeight = computed(() => {
  const nh = nodePositions.value.length
  const sh = servicePositions.value.length
  const maxRows = Math.max(nh, sh, 1)
  return SVG_PADDING * 2 + maxRows * (NODE_HEIGHT + NODE_GAP) - NODE_GAP + 20
})

const svgWidth = 800

const nodeX = 20
const serviceX = svgWidth - NODE_WIDTH - 20

const connections = computed(() => {
  const lines: Array<{
    x1: number; y1: number; x2: number; y2: number
    color: string; dashed: boolean; opacity: number
    nodeId: string; serviceId: string
  }> = []

  for (const svc of servicePositions.value) {
    // Distribute connections across nodes
    for (let ni = 0; ni < nodePositions.value.length; ni++) {
      const node = nodePositions.value[ni]
      // Simulate placement: replicated services run on some nodes, global on all
      const shouldConnect =
        svc.mode === 'Global' ||
        (svc.mode === 'Replicated' && (ni % Math.max(1, Math.floor(activeNodes.value.length / 2)) === 0))

      if (!shouldConnect) continue

      lines.push({
        x1: nodeX + NODE_WIDTH,
        y1: node.y + NODE_HEIGHT / 2,
        x2: serviceX,
        y2: svc.y + NODE_HEIGHT / 2,
        color: svc.mode === 'Global' ? '#10b981' : '#00e5ff',
        dashed: svc.mode === 'Global',
        opacity: 0.3,
        nodeId: node.id,
        serviceId: svc.id,
      })
    }
  }

  return lines
})

function isLineHighlighted(line: { nodeId: string; serviceId: string }): boolean {
  if (hoveredNode.value) return line.nodeId === hoveredNode.value
  if (hoveredService.value) return line.serviceId === hoveredService.value
  return false
}

function nodeFillColor(role: string): string {
  return role === 'Manager' ? '#1b202e' : '#1b202e'
}

function nodeStrokeColor(node: typeof nodePositions.value[0]): string {
  if (hoveredNode.value === node.id) return '#00e5ff'
  return node.role === 'Manager' ? '#262f45' : '#262f45'
}

function serviceStrokeColor(svc: typeof servicePositions.value[0]): string {
  if (hoveredService.value === svc.id) return '#f3f4f6'
  return svc.mode === 'Global' ? '#10b981' : '#262f45'
}
</script>

<template>
  <div class="topology-container" role="region" aria-label="Swarm service-node topology graph">
    <div class="topology-header">
      <span class="topology-title">Service-Node Topology</span>
      <div class="topology-legend">
        <div class="topology-legend__item">
          <span class="topology-legend__swatch" style="background: var(--accent-cyan)"></span>
          <span>Replicated</span>
        </div>
        <div class="topology-legend__item">
          <span class="topology-legend__swatch" style="background: var(--accent-green); height: 0; border-top: 2px dashed var(--accent-green)"></span>
          <span>Global</span>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="activeNodes.length === 0 && servicePositions.length === 0" class="topology-empty">
      <i class="fa-solid fa-diagram-nodes" style="font-size: 36px; color: var(--text-disabled)"></i>
      <p style="margin-top: 12px; color: var(--text-muted)">No active nodes or services to display</p>
    </div>

    <!-- SVG Graph -->
    <svg
      v-else
      :viewBox="`0 0 ${svgWidth} ${svgHeight}`"
      style="width: 100%; height: auto; min-height: 320px"
      role="img"
      aria-label="Topology graph showing connections between nodes and services"
    >
      <!-- Connection lines -->
      <g>
        <line
          v-for="(conn, i) in connections"
          :key="'conn-' + i"
          :x1="conn.x1"
          :y1="conn.y1"
          :x2="conn.x2"
          :y2="conn.y2"
          :stroke="isLineHighlighted(conn) ? conn.color : conn.color"
          :stroke-width="isLineHighlighted(conn) ? 2.5 : 1.5"
          :stroke-dasharray="conn.dashed ? '6,4' : 'none'"
          :opacity="isLineHighlighted(conn) ? 0.8 : conn.opacity"
          style="transition: all 0.15s ease-out"
        />
      </g>

      <!-- Node rectangles (left column) -->
      <g v-for="node in nodePositions" :key="'node-' + node.id">
        <rect
          :x="nodeX"
          :y="node.y"
          :width="NODE_WIDTH"
          :height="NODE_HEIGHT"
          rx="8"
          :fill="nodeFillColor(node.role)"
          :stroke="nodeStrokeColor(node)"
          :stroke-width="hoveredNode === node.id ? 2 : 1"
          :stroke-dasharray="node.status === 'Down' ? '4,4' : 'none'"
          style="cursor: pointer; transition: all 0.15s ease-out"
          @mouseenter="hoveredNode = node.id"
          @mouseleave="hoveredNode = null"
        />
        <text
          :x="nodeX + NODE_WIDTH / 2"
          :y="node.y + NODE_HEIGHT / 2 + 4"
          text-anchor="middle"
          :fill="node.role === 'Manager' ? '#00e5ff' : '#9ca3af'"
          font-size="12"
          font-family="Inter, system-ui, sans-serif"
          font-weight="600"
          style="pointer-events: none"
        >
          {{ node.role === 'Manager' ? '★ ' : '' }}{{ node.hostname }}
        </text>
      </g>

      <!-- Service rectangles (right column) -->
      <g v-for="svc in servicePositions" :key="'svc-' + svc.id">
        <rect
          :x="serviceX"
          :y="svc.y"
          :width="NODE_WIDTH"
          :height="NODE_HEIGHT"
          rx="8"
          fill="#1b202e"
          :stroke="serviceStrokeColor(svc)"
          :stroke-width="hoveredService === svc.id ? 2 : 1"
          style="cursor: pointer; transition: all 0.15s ease-out"
          @mouseenter="hoveredService = svc.id"
          @mouseleave="hoveredService = null"
        />
        <text
          :x="serviceX + NODE_WIDTH / 2"
          :y="svc.y + NODE_HEIGHT / 2 + 4"
          text-anchor="middle"
          fill="#f3f4f6"
          font-size="12"
          font-family="Inter, system-ui, sans-serif"
          font-weight="500"
          style="pointer-events: none"
        >
          {{ svc.name }} ({{ svc.replicas }})
        </text>
      </g>
    </svg>
  </div>
</template>

<style scoped>
.topology-container {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 20px;
  min-height: 350px;
}

.topology-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.topology-title {
  font-size: 14px;
  font-weight: 600;
}

.topology-legend {
  display: flex;
  gap: 20px;
  font-size: 12px;
  color: var(--text-muted);
}

.topology-legend__item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.topology-legend__swatch {
  width: 24px;
  height: 2px;
  border-radius: 1px;
}

.topology-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 20px;
  text-align: center;
}
</style>
