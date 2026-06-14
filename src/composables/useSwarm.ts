// ItzamBox — Docker Swarm Tauri Commands Wrapper (Singleton)
// Copyright (C) 2026 SodigTech — GPL-3.0

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref, readonly } from 'vue'

// ─── Domain Types (mirrors engine/types.rs) ───────────────────────────────

export interface SwarmStatus {
  active: boolean
  node_id: string | null
  nodes_count: number
  managers_count: number
  services_count: number
}

export interface SwarmNode {
  id: string
  hostname: string
  role: string           // "Manager" | "Worker"
  status: string         // "Ready" | "Down" | "Unknown"
  availability: string   // "Active" | "Pause" | "Drain"
  engine_version: string
  ip_address: string
  cpu_cores: number | null
  memory_bytes: number | null
  labels: Record<string, string>
}

export interface SwarmService {
  id: string
  name: string
  mode: string           // "Replicated" | "Global"
  replicas: string       // "3/3"
  image: string
  ports: string[]
}

export interface SwarmStack {
  name: string
  services_count: number
  orchestrator: string   // "Swarm"
}

// ─── Module-level Singleton State ─────────────────────────────────────────

const swarmStatus = ref<SwarmStatus | null>(null)
const nodes = ref<SwarmNode[]>([])
const services = ref<SwarmService[]>([])
const stacks = ref<SwarmStack[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const terminalOutput = ref<string[]>([])
const initTokens = ref<string[]>([])

let unlistenSwarmOutput: (() => void) | null = null

// ─── Composable ──────────────────────────────────────────────────────────

export function useSwarm() {
  // ── Status ──────────────────────────────────────────────────────────

  async function checkStatus() {
    try {
      swarmStatus.value = await invoke<SwarmStatus>('swarm_status')
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      swarmStatus.value = null
    }
  }

  // ── Init / Join / Leave ─────────────────────────────────────────────

  async function initSwarm(advertiseAddr: string): Promise<string> {
    terminalOutput.value = []
    initTokens.value = []
    try {
      const result = await invoke<string>('swarm_init', {
        advertiseAddr,
      })
      const lines = result.split('\n')
      for (const l of lines) {
        terminalOutput.value.push(l)
        if (l.includes('--token')) {
          const trimmed = l.trim()
          if (trimmed && !initTokens.value.includes(trimmed)) {
            initTokens.value.push(trimmed)
          }
        }
      }
      await checkStatus()
      error.value = null
      return result
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function joinSwarm(token: string, managerAddr: string): Promise<string> {
    terminalOutput.value = []
    try {
      const result = await invoke<string>('swarm_join', {
        token,
        managerAddr,
      })
      terminalOutput.value.push(result)
      await checkStatus()
      error.value = null
      return result
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function leaveSwarm(force = false): Promise<void> {
    try {
      await invoke<void>('swarm_leave', { force })
      swarmStatus.value = null
      nodes.value = []
      services.value = []
      stacks.value = []
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  // ── Nodes ───────────────────────────────────────────────────────────

  async function loadNodes() {
    try {
      nodes.value = await invoke<SwarmNode[]>('list_swarm_nodes')
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    }
  }

  async function inspectNode(nodeId: string): Promise<Record<string, unknown>> {
    return await invoke<Record<string, unknown>>('inspect_swarm_node', { nodeId })
  }

  // ── Services ────────────────────────────────────────────────────────

  async function loadServices() {
    try {
      services.value = await invoke<SwarmService[]>('list_swarm_services')
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    }
  }

  async function inspectService(serviceId: string): Promise<Record<string, unknown>> {
    return await invoke<Record<string, unknown>>('inspect_swarm_service', { serviceId })
  }

  // ── Stacks ──────────────────────────────────────────────────────────

  async function loadStacks() {
    try {
      stacks.value = await invoke<SwarmStack[]>('list_stacks')
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    }
  }

  async function deployStack(name: string, composePath: string): Promise<void> {
    try {
      await invoke<void>('deploy_stack', { name, composePath })
      await loadStacks()
      await loadServices()
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function removeStack(name: string): Promise<void> {
    try {
      await invoke<void>('remove_stack', { name })
      await loadStacks()
      await loadServices()
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  // ── Terminal Output Listeners ───────────────────────────────────────

  async function listenSwarmOutput(): Promise<void> {
    if (unlistenSwarmOutput) return
    unlistenSwarmOutput = await listen<string>('swarm-output', (event) => {
      terminalOutput.value.push(event.payload)
    })
  }

  function clearTerminalOutput() {
    terminalOutput.value = []
  }

  function stopListeningSwarmOutput() {
    if (unlistenSwarmOutput) {
      unlistenSwarmOutput()
      unlistenSwarmOutput = null
    }
  }

  // ── Bulk Refresh ────────────────────────────────────────────────────

  async function refreshAll() {
    loading.value = true
    error.value = null
    try {
      await checkStatus()
      if (swarmStatus.value?.active) {
        await Promise.allSettled([
          loadNodes(),
          loadServices(),
          loadStacks(),
        ])
      }
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    } finally {
      loading.value = false
    }
  }

  return {
    // State
    swarmStatus: readonly(swarmStatus),
    nodes: readonly(nodes),
    services: readonly(services),
    stacks: readonly(stacks),
    loading: readonly(loading),
    error: readonly(error),
    terminalOutput: readonly(terminalOutput),
    initTokens: readonly(initTokens),

    // Status
    checkStatus,

    // Init / Join / Leave
    initSwarm,
    joinSwarm,
    leaveSwarm,

    // Nodes
    loadNodes,
    inspectNode,

    // Services
    loadServices,
    inspectService,

    // Stacks
    loadStacks,
    deployStack,
    removeStack,

    // Terminal
    listenSwarmOutput,
    clearTerminalOutput,
    stopListeningSwarmOutput,

    // Bulk
    refreshAll,
  }
}
