<!-- ItzamBox — Network Topology Visualization
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed, watch, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import type { NetworkInfo, ContainerInfo, NetworkContainer } from '../composables/useDocker'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import ErrorState from '../components/shared/ErrorState.vue'
import EmptyState from '../components/shared/EmptyState.vue'

const router = useRouter()

// ─── State ───
const networks = ref<NetworkInfo[]>([])
const containers = ref<ContainerInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const selectedNetwork = ref<string>('__all__')

// ─── SVG interaction state ───
const scale = ref(1)
const panX = ref(0)
const panY = ref(0)
const isPanning = ref(false)
const panStart = reactive({ x: 0, y: 0 })
const dragNode = ref<string | null>(null)
const hoveredNode = ref<string | null>(null)
const tooltipPos = reactive({ x: 0, y: 0 })

// ─── Graph data ───
interface GraphNode {
  id: string
  type: 'network' | 'container'
  label: string
  x: number
  y: number
  vx: number
  vy: number
  radius: number
  color: string
  borderColor: string
  networkInfo?: NetworkInfo
  containerInfo?: ContainerInfo
  containerNet?: NetworkContainer
}

interface GraphEdge {
  source: string
  target: string
}

const nodes = ref<GraphNode[]>([])
const edges = ref<GraphEdge[]>([])

// ─── SVG dimensions ───
const SVG_W = 1200
const SVG_H = 800
const CENTER_X = SVG_W / 2
const CENTER_Y = SVG_H / 2
const NETWORK_RADIUS = 28
const CONTAINER_RADIUS = 20

// ─── Helper: truncate long labels ───
function truncateLabel(label: string, max: number): string {
  if (label.length <= max) return label
  return label.slice(0, max - 1) + '…'
}

// ─── Helper: find node by id ───
function getNode(id: string): GraphNode | undefined {
  return nodes.value.find(n => n.id === id)
}

// ─── Build graph from API data ───
function buildGraph() {
  const nodeMap = new Map<string, GraphNode>()
  const edgeList: GraphEdge[] = []
  const connectedContainers = new Set<string>()

  // Create network nodes
  for (const net of networks.value) {
    const id = `net:${net.id}`
    nodeMap.set(id, {
      id,
      type: 'network',
      label: net.name,
      x: 0, y: 0, vx: 0, vy: 0,
      radius: NETWORK_RADIUS,
      color: 'var(--accent-cyan)',
      borderColor: 'rgba(0, 229, 255, 0.5)',
      networkInfo: net,
    })

    // Track containers in this network
    for (const c of net.containers) {
      connectedContainers.add(c.container_id)
    }
  }

  // Build container lookup
  const containerMap = new Map<string, ContainerInfo>()
  for (const c of containers.value) {
    containerMap.set(c.id, c)
  }

  // Create container nodes and edges from network membership
  for (const net of networks.value) {
    for (const c of net.containers) {
      const nodeId = `ctn:${c.container_id}`
      const cInfo = containerMap.get(c.container_id)
      const state = cInfo?.state ?? 'unknown'
      const isRunning = state === 'running'

      if (!nodeMap.has(nodeId)) {
        nodeMap.set(nodeId, {
          id: nodeId,
          type: 'container',
          label: c.container_name,
          x: 0, y: 0, vx: 0, vy: 0,
          radius: CONTAINER_RADIUS,
          color: isRunning ? 'var(--status-running)' : 'var(--status-stopped)',
          borderColor: isRunning ? 'rgba(16, 185, 129, 0.5)' : 'rgba(107, 114, 128, 0.3)',
          containerInfo: cInfo,
          containerNet: c,
        })
      }
      edgeList.push({ source: `net:${net.id}`, target: nodeId })
    }
  }

  // Add orphan containers (not connected to any network) under "(none/host)"
  const orphanContainers = containers.value.filter(c => !connectedContainers.has(c.id))
  if (orphanContainers.length > 0) {
    const hostNetId = 'net:__host__'
    if (!nodeMap.has(hostNetId)) {
      nodeMap.set(hostNetId, {
        id: hostNetId,
        type: 'network',
        label: '(none/host)',
        x: 0, y: 0, vx: 0, vy: 0,
        radius: NETWORK_RADIUS,
        color: 'var(--accent-yellow)',
        borderColor: 'rgba(245, 158, 11, 0.5)',
      })
    }

    for (const c of orphanContainers) {
      const nodeId = `ctn:${c.id}`
      const isRunning = c.state === 'running'
      nodeMap.set(nodeId, {
        id: nodeId,
        type: 'container',
        label: c.name,
        x: 0, y: 0, vx: 0, vy: 0,
        radius: CONTAINER_RADIUS,
        color: isRunning ? 'var(--status-running)' : 'var(--status-stopped)',
        borderColor: isRunning ? 'rgba(16, 185, 129, 0.5)' : 'rgba(107, 114, 128, 0.3)',
        containerInfo: c,
      })
      edgeList.push({ source: hostNetId, target: nodeId })
    }
  }

  // Apply filter
  let filteredNodes: GraphNode[]
  let filteredEdges: GraphEdge[]

  if (selectedNetwork.value !== '__all__') {
    const netId = `net:${selectedNetwork.value}`
    const visibleIds = new Set<string>()
    visibleIds.add(netId)
    for (const e of edgeList) {
      if (e.source === netId) visibleIds.add(e.target)
      if (e.target === netId) visibleIds.add(e.source)
    }
    filteredEdges = edgeList.filter(e => visibleIds.has(e.source) && visibleIds.has(e.target))
    filteredNodes = Array.from(nodeMap.values()).filter(n => visibleIds.has(n.id))
  } else {
    filteredNodes = Array.from(nodeMap.values())
    filteredEdges = edgeList
  }

  // Run force-directed layout
  layoutNodes(filteredNodes, filteredEdges)

  nodes.value = filteredNodes
  edges.value = filteredEdges
}

// ─── Force-directed layout ───
function layoutNodes(nodeList: GraphNode[], edgeList: GraphEdge[]) {
  if (nodeList.length === 0) return

  const networkNodes = nodeList.filter(n => n.type === 'network')
  const containerNodes = nodeList.filter(n => n.type === 'container')

  // Build adjacency
  const adj = new Map<string, string[]>()
  for (const n of nodeList) adj.set(n.id, [])
  for (const e of edgeList) {
    adj.get(e.source)?.push(e.target)
    adj.get(e.target)?.push(e.source)
  }

  // Position networks in a circle
  const netCount = networkNodes.length
  const netRadius = Math.min(220, 100 + netCount * 25)
  networkNodes.forEach((n, i) => {
    const angle = (2 * Math.PI * i) / netCount - Math.PI / 2
    n.x = CENTER_X + netRadius * Math.cos(angle)
    n.y = CENTER_Y + netRadius * Math.sin(angle)
  })

  // Position containers near their connected networks
  const containerRadius = 110
  for (const c of containerNodes) {
    const neighbors = adj.get(c.id) || []
    if (neighbors.length > 0) {
      let sumX = 0, sumY = 0
      for (const nid of neighbors) {
        const netNode = nodeList.find(n => n.id === nid)
        if (netNode) { sumX += netNode.x; sumY += netNode.y }
      }
      const avgX = sumX / neighbors.length
      const avgY = sumY / neighbors.length
      const idx = containerNodes.indexOf(c)
      const angle = idx * 2.399 // ~golden angle for even spread
      c.x = avgX + containerRadius * Math.cos(angle)
      c.y = avgY + containerRadius * Math.sin(angle)
    } else {
      c.x = CENTER_X + (Math.random() - 0.5) * 200
      c.y = CENTER_Y + (Math.random() - 0.5) * 200
    }
  }

  // Force-directed iterations
  const iterations = 100
  const repulsion = 10000
  const attraction = 0.004
  const gravity = 0.002
  const damping = 0.8
  const minDist = 50

  for (let iter = 0; iter < iterations; iter++) {
    const forces: Map<string, { fx: number; fy: number }> = new Map()
    for (const n of nodeList) forces.set(n.id, { fx: 0, fy: 0 })

    // Repulsion between all pairs
    for (let i = 0; i < nodeList.length; i++) {
      for (let j = i + 1; j < nodeList.length; j++) {
        const a = nodeList[i]
        const b = nodeList[j]
        let dx = b.x - a.x
        let dy = b.y - a.y
        const dist = Math.max(Math.sqrt(dx * dx + dy * dy), minDist)
        const force = repulsion / (dist * dist)
        const fx = force * (dx / dist)
        const fy = force * (dy / dist)

        const af = forces.get(a.id)!
        af.fx -= fx; af.fy -= fy
        const bf = forces.get(b.id)!
        bf.fx += fx; bf.fy += fy
      }
    }

    // Attraction along edges
    for (const e of edgeList) {
      const a = nodeList.find(n => n.id === e.source)
      const b = nodeList.find(n => n.id === e.target)
      if (!a || !b) continue
      let dx = b.x - a.x
      let dy = b.y - a.y
      const dist = Math.sqrt(dx * dx + dy * dy)
      if (dist < 1) { dx = (Math.random() - 0.5) * 4; dy = (Math.random() - 0.5) * 4 }
      const f = attraction * dist
      const fx = f * (dx / (dist || 1))
      const fy = f * (dy / (dist || 1))

      const af = forces.get(a.id)!
      af.fx += fx; af.fy += fy
      const bf = forces.get(b.id)!
      bf.fx -= fx; bf.fy -= fy
    }

    // Center gravity
    for (const n of nodeList) {
      const dx = CENTER_X - n.x
      const dy = CENTER_Y - n.y
      const dist = Math.sqrt(dx * dx + dy * dy)
      const f = gravity * dist
      const nf = forces.get(n.id)!
      nf.fx += f * (dx / (dist || 1))
      nf.fy += f * (dy / (dist || 1))
    }

    // Apply forces
    const speed = iter < iterations * 0.3 ? 1.0 : iter < iterations * 0.6 ? 0.6 : 0.3
    for (const n of nodeList) {
      const f = forces.get(n.id)!
      n.vx = (n.vx + f.fx * speed) * damping
      n.vy = (n.vy + f.fy * speed) * damping
      n.x += n.vx
      n.y += n.vy
      // Clamp
      n.x = Math.max(NETWORK_RADIUS + 10, Math.min(SVG_W - NETWORK_RADIUS - 10, n.x))
      n.y = Math.max(NETWORK_RADIUS + 10, Math.min(SVG_H - NETWORK_RADIUS - 10, n.y))
    }
  }
}

// ─── Data loading ───
async function loadData() {
  loading.value = true
  error.value = null
  try {
    const [nets, ctrs] = await Promise.all([
      invoke<NetworkInfo[]>('list_networks'),
      invoke<ContainerInfo[]>('list_containers', { showAll: true }),
    ])
    networks.value = nets
    containers.value = ctrs
    buildGraph()
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

onMounted(() => loadData())

// ─── Watch filter changes ───
watch(selectedNetwork, () => { buildGraph() })

// ─── Zoom with mouse wheel ───
function handleWheel(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  const newScale = Math.max(0.2, Math.min(3, scale.value * delta))
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top
  const ratio = newScale / scale.value
  panX.value = mx - ratio * (mx - panX.value)
  panY.value = my - ratio * (my - panY.value)
  scale.value = newScale
}

// ─── Pan by dragging background ───
function handleBgMouseDown(e: MouseEvent) {
  // Only pan if clicking the background (SVG root or rect)
  const target = e.target as HTMLElement
  if (target.closest('.graph-node')) return
  isPanning.value = true
  panStart.x = e.clientX - panX.value
  panStart.y = e.clientY - panY.value
}

function handleMouseMove(e: MouseEvent) {
  if (isPanning.value) {
    panX.value = e.clientX - panStart.x
    panY.value = e.clientY - panStart.y
  }
  if (dragNode.value) {
    const node = nodes.value.find(n => n.id === dragNode.value)
    if (node) {
      const container = (e.currentTarget as HTMLElement).closest('.topology-container') as HTMLElement
      const rect = container.querySelector('svg')?.getBoundingClientRect()
      if (rect) {
        const sx = (e.clientX - rect.left - panX.value) / scale.value
        const sy = (e.clientY - rect.top - panY.value) / scale.value
        node.x = Math.max(NETWORK_RADIUS + 10, Math.min(SVG_W - NETWORK_RADIUS - 10, sx))
        node.y = Math.max(NETWORK_RADIUS + 10, Math.min(SVG_H - NETWORK_RADIUS - 10, sy))
      }
    }
  }
  // Update tooltip position
  if (hoveredNode.value) {
    const container = (e.currentTarget as HTMLElement).closest('.topology-container') as HTMLElement
    if (container) {
      const rect = container.getBoundingClientRect()
      tooltipPos.x = e.clientX - rect.left + 14
      tooltipPos.y = e.clientY - rect.top - 10
    }
  }
}

function handleMouseUp() {
  isPanning.value = false
  dragNode.value = null
}

// ─── Node drag ───
function handleNodeMouseDown(e: MouseEvent, nodeId: string) {
  e.stopPropagation()
  isPanning.value = false
  dragNode.value = nodeId
}

// ─── Node hover ───
function handleNodeMouseEnter(e: MouseEvent, nodeId: string) {
  hoveredNode.value = nodeId
  const container = (e.currentTarget as HTMLElement).closest('.topology-container') as HTMLElement
  if (container) {
    const rect = container.getBoundingClientRect()
    tooltipPos.x = e.clientX - rect.left + 14
    tooltipPos.y = e.clientY - rect.top - 10
  }
}

function handleNodeMouseLeave() {
  hoveredNode.value = null
}

// ─── Node click → navigate ───
function handleNodeClick(nodeId: string) {
  if (nodeId.startsWith('ctn:')) {
    router.push(`/containers/${nodeId.slice(4)}`)
  }
}

// ─── Hovered node data (for tooltip) ───
const hoveredNodeData = computed(() => {
  if (!hoveredNode.value) return null
  return nodes.value.find(n => n.id === hoveredNode.value) ?? null
})

// ─── Network filter options ───
const networkOptions = computed(() => {
  const opts: { value: string; label: string }[] = [{ value: '__all__', label: 'All Networks' }]
  for (const n of networks.value) {
    opts.push({ value: n.id, label: n.name })
  }
  return opts
})

// ─── SVG transform string ───
const svgTransform = computed(() =>
  `translate(${panX.value}, ${panY.value}) scale(${scale.value})`
)
</script>

<template>
  <div>
    <div class="breadcrumb">
      <i class="fa-solid fa-house"></i> <span>Home</span>
      <i class="fa-solid fa-chevron-right"></i>
      <span class="current">Network Topology</span>
    </div>

    <div style="display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:8px;">
      <h1 class="text-h1">
        <i class="fa-solid fa-diagram-project"></i> Network Topology
      </h1>
      <div style="display:flex;gap:8px;align-items:center;">
        <div class="filter-group">
          <i class="fa-solid fa-filter" style="color:var(--text-muted);font-size:12px"></i>
          <select v-model="selectedNetwork" class="filter-select">
            <option v-for="opt in networkOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
        <button class="btn btn-secondary" @click="loadData" :disabled="loading">
          <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> Refresh
        </button>
      </div>
    </div>

    <!-- Loading -->
    <SkeletonLoader v-if="loading && networks.length === 0" variant="card" :rows="6" />

    <!-- Error -->
    <ErrorState
      v-if="error"
      message="Failed to load network topology"
      suggestion="Ensure Docker is running and accessible."
      :detail="error"
      @retry="loadData"
    />

    <!-- Empty -->
    <EmptyState
      v-if="!loading && !error && nodes.length === 0"
      icon="fa-solid fa-diagram-project"
      title="No networks or containers found"
      description="Create a network and run some containers to see the topology."
    />

    <!-- Topology SVG -->
    <div
      v-if="!loading && !error && nodes.length > 0"
      class="topology-container"
      @wheel.prevent="handleWheel"
      @mousedown="handleBgMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseUp"
    >
      <!-- Controls hint -->
      <div class="controls-hint">
        <span><i class="fa-solid fa-arrows"></i> Drag background to pan</span>
        <span><i class="fa-solid fa-up-down-left-right"></i> Drag nodes</span>
        <span><i class="fa-solid fa-plus-minus"></i> Scroll to zoom</span>
      </div>

      <svg
        :width="SVG_W"
        :height="SVG_H"
        :viewBox="`0 0 ${SVG_W} ${SVG_H}`"
        class="topology-svg"
      >
        <defs>
          <!-- Node glow filters -->
          <filter id="glow-cyan" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="4" result="blur" />
            <feFlood flood-color="var(--accent-cyan)" flood-opacity="0.25" />
            <feComposite in2="blur" operator="in" />
            <feMerge><feMergeNode /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
          <filter id="glow-green" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feFlood flood-color="var(--accent-green)" flood-opacity="0.2" />
            <feComposite in2="blur" operator="in" />
            <feMerge><feMergeNode /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
          <filter id="glow-gray" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="2" result="blur" />
            <feFlood flood-color="var(--status-stopped)" flood-opacity="0.15" />
            <feComposite in2="blur" operator="in" />
            <feMerge><feMergeNode /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
        </defs>

        <!-- Background grid -->
        <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse"
          :patternTransform="`translate(${panX}, ${panY}) scale(${scale})`">
          <path d="M 40 0 L 0 0 0 40" fill="none" stroke="var(--border-light)" stroke-width="0.5" />
        </pattern>
        <rect width="100%" height="100%" fill="url(#grid)" />

        <!-- Transformed content group -->
        <g :transform="svgTransform">
          <!-- Edges -->
          <g class="graph-edges">
            <line
              v-for="e in edges"
              :key="`e-${e.source}-${e.target}`"
              :x1="getNode(e.source)?.x ?? 0"
              :y1="getNode(e.source)?.y ?? 0"
              :x2="getNode(e.target)?.x ?? 0"
              :y2="getNode(e.target)?.y ?? 0"
              stroke="var(--border-color)"
              stroke-width="1.5"
              stroke-opacity="0.6"
            />
          </g>

          <!-- Nodes -->
          <g class="graph-nodes">
            <g
              v-for="n in nodes"
              :key="n.id"
              class="graph-node"
              :class="{
                'graph-node--hovered': hoveredNode === n.id,
                'graph-node--dimmed': hoveredNode && hoveredNode !== n.id
              }"
              :transform="`translate(${n.x}, ${n.y})`"
              @mousedown.stop="handleNodeMouseDown($event, n.id)"
              @mouseenter="handleNodeMouseEnter($event, n.id)"
              @mouseleave="handleNodeMouseLeave"
              @click="handleNodeClick(n.id)"
              :style="{ cursor: n.type === 'container' ? 'pointer' : dragNode === n.id ? 'grabbing' : 'grab' }"
            >
              <!-- Glow circle -->
              <circle
                :r="n.radius + 6"
                fill="transparent"
                :stroke="n.type === 'network' ? 'var(--accent-cyan)' : n.color"
                :stroke-opacity="hoveredNode === n.id ? 0.4 : 0"
                stroke-width="2"
                class="node-glow"
              />
              <!-- Main circle -->
              <circle
                :r="n.radius"
                :fill="n.type === 'network' ? 'var(--bg-tertiary)' : n.color"
                :stroke="n.type === 'network' ? n.color : n.borderColor"
                :stroke-width="n.type === 'network' ? 2.5 : 2"
                class="node-circle"
              />
              <!-- Network icon (font-awesome network-wired) -->
              <text
                v-if="n.type === 'network'"
                text-anchor="middle"
                dominant-baseline="central"
                font-size="16"
                fill="var(--accent-cyan)"
                font-family="'Font Awesome 6 Free'"
                font-weight="900"
              >&#xf0e8;</text>
              <!-- Container icon (font-awesome cube) -->
              <text
                v-if="n.type === 'container'"
                text-anchor="middle"
                dominant-baseline="central"
                font-size="12"
                fill="var(--bg-primary)"
                font-family="'Font Awesome 6 Free'"
                font-weight="900"
              >&#xf1b3;</text>
              <!-- Label -->
              <text
                :x="n.radius + 10"
                y="0"
                dominant-baseline="central"
                :font-size="n.type === 'network' ? 12 : 11"
                :fill="n.type === 'network' ? 'var(--accent-cyan)' : 'var(--text-muted)'"
                font-family="var(--font-sans)"
                font-weight="500"
                class="node-label"
              >{{ truncateLabel(n.label, n.type === 'network' ? 20 : 22) }}</text>
            </g>
          </g>
        </g>
      </svg>

      <!-- Floating tooltip -->
      <div
        v-if="hoveredNodeData"
        class="topology-tooltip"
        :style="{ left: tooltipPos.x + 'px', top: tooltipPos.y + 'px' }"
      >
        <div class="tooltip-header">
          <span :class="['tooltip-dot',
            hoveredNodeData.type === 'network' ? 'dot-cyan' :
            hoveredNodeData.color === 'var(--status-running)' ? 'dot-green' : 'dot-gray'
          ]"></span>
          <span class="tooltip-title">{{ hoveredNodeData.label }}</span>
          <span class="tooltip-type">{{ hoveredNodeData.type === 'network' ? 'NETWORK' : 'CONTAINER' }}</span>
        </div>
        <div class="tooltip-body">
          <!-- Network tooltip content -->
          <template v-if="hoveredNodeData.type === 'network' && hoveredNodeData.networkInfo">
            <div class="tooltip-row">
              <span class="tooltip-label">ID</span>
              <span class="tooltip-value mono">{{ hoveredNodeData.networkInfo.id.slice(0, 12) }}…</span>
            </div>
            <div class="tooltip-row">
              <span class="tooltip-label">Driver</span>
              <span class="tooltip-value">{{ hoveredNodeData.networkInfo.driver }}</span>
            </div>
            <div class="tooltip-row" v-if="hoveredNodeData.networkInfo.subnet">
              <span class="tooltip-label">Subnet</span>
              <span class="tooltip-value mono">{{ hoveredNodeData.networkInfo.subnet }}</span>
            </div>
            <div class="tooltip-row" v-if="hoveredNodeData.networkInfo.gateway">
              <span class="tooltip-label">Gateway</span>
              <span class="tooltip-value mono">{{ hoveredNodeData.networkInfo.gateway }}</span>
            </div>
            <div class="tooltip-row">
              <span class="tooltip-label">Scope</span>
              <span class="tooltip-value">{{ hoveredNodeData.networkInfo.scope }}</span>
            </div>
            <div class="tooltip-row" v-if="hoveredNodeData.networkInfo.internal">
              <span class="tooltip-label">Internal</span>
              <span class="tooltip-value" style="color:var(--accent-yellow)">Yes</span>
            </div>
            <div class="tooltip-row">
              <span class="tooltip-label">Containers</span>
              <span class="tooltip-value">{{ hoveredNodeData.networkInfo.containers.length }}</span>
            </div>
          </template>
          <!-- Container tooltip content -->
          <template v-if="hoveredNodeData.type === 'container'">
            <div class="tooltip-row" v-if="hoveredNodeData.containerInfo">
              <span class="tooltip-label">Image</span>
              <span class="tooltip-value mono">{{ hoveredNodeData.containerInfo.image }}</span>
            </div>
            <div class="tooltip-row" v-if="hoveredNodeData.containerInfo">
              <span class="tooltip-label">State</span>
              <span class="tooltip-value" :style="{
                color: hoveredNodeData.containerInfo.state === 'running'
                  ? 'var(--accent-green)' : 'var(--text-muted)'
              }">{{ hoveredNodeData.containerInfo.state }}</span>
            </div>
            <div class="tooltip-row" v-if="hoveredNodeData.containerNet?.ipv4_address">
              <span class="tooltip-label">IP</span>
              <span class="tooltip-value mono">{{ hoveredNodeData.containerNet.ipv4_address }}</span>
            </div>
            <div class="tooltip-row" v-if="hoveredNodeData.containerNet?.mac_address">
              <span class="tooltip-label">MAC</span>
              <span class="tooltip-value mono" style="font-size:10px">{{ hoveredNodeData.containerNet.mac_address }}</span>
            </div>
            <div class="tooltip-row" v-if="hoveredNodeData.containerInfo?.status">
              <span class="tooltip-label">Ports</span>
              <span class="tooltip-value mono" style="font-size:10px">{{ hoveredNodeData.containerInfo.status }}</span>
            </div>
          </template>
        </div>
        <div v-if="hoveredNodeData.type === 'container'" class="tooltip-footer">
          <i class="fa-solid fa-arrow-right"></i> Click to inspect
        </div>
      </div>

      <!-- Legend -->
      <div class="topology-legend">
        <div class="legend-title"><i class="fa-solid fa-info-circle"></i> Legend</div>
        <div class="legend-item">
          <span class="legend-dot" style="background:var(--bg-tertiary);border:2px solid var(--accent-cyan)"></span>
          <span>Network</span>
        </div>
        <div class="legend-item">
          <span class="legend-dot" style="background:var(--accent-green)"></span>
          <span>Running Container</span>
        </div>
        <div class="legend-item">
          <span class="legend-dot" style="background:var(--status-stopped)"></span>
          <span>Stopped Container</span>
        </div>
        <div class="legend-divider"></div>
        <div class="legend-stats">
          <div class="legend-stat">
            <span class="legend-stat-value">{{ networks.length }}</span>
            <span class="legend-stat-label">Networks</span>
          </div>
          <div class="legend-stat">
            <span class="legend-stat-value">{{ containers.length }}</span>
            <span class="legend-stat-label">Containers</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ─── Topology Container ─── */
.topology-container {
  position: relative;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  cursor: grab;
  margin-top: 16px;
  user-select: none;
}
.topology-container:active {
  cursor: grabbing;
}

.topology-svg {
  display: block;
  width: 100%;
  height: auto;
  min-height: 600px;
  max-height: 80vh;
}

/* ─── Controls Hint ─── */
.controls-hint {
  position: absolute;
  top: 10px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 16px;
  align-items: center;
  padding: 6px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  font-size: 10px;
  color: var(--text-disabled);
  pointer-events: none;
  z-index: 5;
  white-space: nowrap;
}
.controls-hint span {
  display: flex;
  align-items: center;
  gap: 4px;
}
.controls-hint i {
  font-size: 9px;
}

/* ─── Node Styles ─── */
.graph-nodes .graph-node--dimmed {
  opacity: 0.35;
  transition: opacity 0.15s ease;
}
.graph-node--hovered {
  opacity: 1 !important;
  z-index: 10;
}

.node-circle {
  transition: stroke-width 0.15s ease;
}
.graph-node--hovered .node-circle {
  stroke-width: 3.5;
}

.node-glow {
  transition: stroke-opacity 0.15s ease;
}

.node-label {
  pointer-events: none;
  dominant-baseline: central;
}

/* ─── Tooltip ─── */
.topology-tooltip {
  position: absolute;
  z-index: 50;
  min-width: 180px;
  max-width: 280px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg), 0 0 20px rgba(0, 0, 0, 0.4);
  pointer-events: none;
  animation: tooltipIn 0.12s ease-out;
}

@keyframes tooltipIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.tooltip-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px 8px;
  border-bottom: 1px solid var(--border-light);
}
.tooltip-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-cyan { background: var(--accent-cyan); }
.dot-green { background: var(--accent-green); }
.dot-gray { background: var(--status-stopped); }
.tooltip-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tooltip-type {
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.06em;
  color: var(--text-disabled);
  text-transform: uppercase;
}
.tooltip-body {
  padding: 8px 12px;
}
.tooltip-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 3px 0;
  font-size: 11px;
}
.tooltip-label {
  color: var(--text-muted);
  flex-shrink: 0;
}
.tooltip-value {
  color: var(--text-main);
  font-weight: 500;
  text-align: right;
  margin-left: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 160px;
}
.tooltip-value.mono {
  font-family: var(--font-mono);
  font-size: 10px;
}
.tooltip-footer {
  padding: 6px 12px 8px;
  border-top: 1px solid var(--border-light);
  font-size: 10px;
  color: var(--accent-cyan);
  display: flex;
  align-items: center;
  gap: 4px;
}

/* ─── Legend ─── */
.topology-legend {
  position: absolute;
  bottom: 16px;
  right: 16px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  min-width: 140px;
  z-index: 5;
}
.legend-title {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: 8px;
}
.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-muted);
  padding: 3px 0;
}
.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
  flex-shrink: 0;
}
.legend-divider {
  height: 1px;
  background: var(--border-light);
  margin: 8px 0;
}
.legend-stats {
  display: flex;
  gap: 16px;
}
.legend-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.legend-stat-value {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-main);
  font-family: var(--font-mono);
}
.legend-stat-label {
  font-size: 9px;
  color: var(--text-disabled);
  text-transform: uppercase;
}

/* ─── Filter ─── */
.filter-group {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 0 10px;
}
.filter-select {
  background: none;
  border: none;
  outline: none;
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
  padding: 6px 4px;
  cursor: pointer;
  min-width: 120px;
}
.filter-select option {
  background: var(--bg-secondary);
  color: var(--text-main);
}

/* ─── Responsive ─── */
@media (max-width: 768px) {
  .controls-hint {
    display: none;
  }
  .topology-legend {
    bottom: 8px;
    right: 8px;
    padding: 8px 10px;
    min-width: 100px;
  }
}
</style>
