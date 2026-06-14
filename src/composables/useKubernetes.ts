// ItzamBox — Kubernetes Tauri Commands Wrapper
// Copyright (C) 2026 SodigTech — GPL-3.0

import { invoke } from '@tauri-apps/api/core'
import { ref, computed } from 'vue'

// ─── Types ────────────────────────────────────────────────────────────────

export interface K8sContext {
  name: string
  cluster: string
  user: string
  is_active: boolean
}

export interface K8sPod {
  name: string
  namespace: string
  status: string
  restarts: number
  age: string
  node: string
  ip: string | null
  containers: K8sContainer[]
  labels: Record<string, string>
  annotations: Record<string, string>
}

export interface K8sContainer {
  name: string
  image: string
  ports: string[]
  ready: boolean
  restart_count: number
  readiness_probe: string | null
  liveness_probe: string | null
  resources_limits: Record<string, string> | null
  resources_requests: Record<string, string> | null
}

export interface K8sDeployment {
  name: string
  namespace: string
  ready: string
  up_to_date: number
  available: number
  age: string
  strategy: string
  selector: Record<string, string>
}

export interface K8sService {
  name: string
  namespace: string
  service_type: string
  cluster_ip: string
  external_ip: string | null
  ports: string[]
  age: string
  selector: Record<string, string>
}

export interface K8sConfigMap {
  name: string
  namespace: string
  keys_count: number
  age: string
  data_keys: string[]
}

export interface K8sSecretMeta {
  name: string
  namespace: string
  secret_type: string
  keys_count: number
  age: string
  data_keys: string[]
}

export interface K8sEvent {
  timestamp: string
  event_type: string
  reason: string
  message: string
}

export interface KubectlStatus {
  kubectl_installed: boolean
  kubectl_version: string | null
  kubeconfig_exists: boolean
  active_context: string | null
}

export type ConnectionStatus = 'loading' | 'connected' | 'no-kubectl' | 'no-kubeconfig' | 'offline'

// ─── Resource Kind type ────────────────────────────────────────────────────

export type ResourceKind = 'pod' | 'deployment' | 'service' | 'configmap' | 'secret'

// ─── Composable ────────────────────────────────────────────────────────────

export function useKubernetes() {
  // ── Status & error ──
  const connectionStatus = ref<ConnectionStatus>('loading')
  const kubectlStatus = ref<KubectlStatus | null>(null)
  const error = ref<string | null>(null)
  const loading = ref(false)
  const staleTimestamp = ref<number | null>(null)
  const previousData = ref<{ pods: K8sPod[]; deployments: K8sDeployment[]; services: K8sService[]; configmaps: K8sConfigMap[]; secrets: K8sSecretMeta[] } | null>(null)

  // ── Context & namespace ──
  const contexts = ref<K8sContext[]>([])
  const activeContext = ref<string | null>(null)
  const namespaces = ref<string[]>([])
  const activeNamespace = ref<string | null>(null)

  // ── Resources ──
  const pods = ref<K8sPod[]>([])
  const deployments = ref<K8sDeployment[]>([])
  const services = ref<K8sService[]>([])
  const configmaps = ref<K8sConfigMap[]>([])
  const secrets = ref<K8sSecretMeta[]>([])

  // ── Active tab tracking ──
  const activeTab = ref<string>('pods')
  const activeConfigSubTab = ref<string>('configmaps')

  // ── Inspector state ──
  const inspectorOpen = ref(false)
  const inspectedKind = ref<ResourceKind | null>(null)
  const inspectedName = ref<string | null>(null)
  const inspectedNamespace = ref<string | null>(null)
  const inspectorYaml = ref<string | null>(null)
  const inspectorEvents = ref<K8sEvent[]>([])

  // ── Computed ──
  const isConnected = computed(() => connectionStatus.value === 'connected')

  // ── Core functions ──

  /** Detect kubectl availability and kubeconfig state. */
  async function detectKubectl(): Promise<KubectlStatus> {
    try {
      const status = await invoke<KubectlStatus>('detect_kubectl')
      kubectlStatus.value = status
      if (!status.kubectl_installed) {
        connectionStatus.value = 'no-kubectl'
      } else if (!status.kubeconfig_exists) {
        connectionStatus.value = 'no-kubeconfig'
      } else if (status.active_context) {
        activeContext.value = status.active_context
        connectionStatus.value = 'connected'
      } else {
        connectionStatus.value = 'no-kubeconfig'
      }
      return status
    } catch (e: any) {
      error.value = e.toString?.() ?? 'Unknown error detecting kubectl'
      connectionStatus.value = 'offline'
      throw e
    }
  }

  /** Load all available contexts from kubeconfig. */
  async function loadContexts(): Promise<void> {
    try {
      const ctxs = await invoke<K8sContext[]>('list_k8s_contexts')
      contexts.value = ctxs
      const active = ctxs.find(c => c.is_active)
      if (active) {
        activeContext.value = active.name
      }
    } catch (e: any) {
      console.warn('Failed to load K8s contexts:', e.toString?.() ?? e)
    }
  }

  /** Switch the active kubectl context. */
  async function switchContext(context: string): Promise<void> {
    try {
      loading.value = true
      await invoke('set_k8s_context', { context })
      activeContext.value = context
      // Reload everything
      await Promise.all([
        loadNamespaces(),
        loadPods(activeNamespace.value),
        loadDeployments(activeNamespace.value),
        loadServices(activeNamespace.value),
        loadConfigMaps(activeNamespace.value),
        loadSecrets(activeNamespace.value),
      ])
      error.value = null
    } catch (e: any) {
      error.value = e.toString?.() ?? 'Failed to switch context'
    } finally {
      loading.value = false
    }
  }

  /** Load available namespaces from the active context cluster. */
  async function loadNamespaces(): Promise<void> {
    try {
      const ns = await invoke<string[]>('list_namespaces')
      namespaces.value = ns.sort()
    } catch (e: any) {
      console.warn('Failed to load namespaces:', e.toString?.() ?? e)
      namespaces.value = []
    }
  }

  /** Load pods for the given namespace (null = all namespaces). */
  async function loadPods(namespace: string | null = null): Promise<void> {
    try {
      const result = await invoke<K8sPod[]>('list_pods', { namespace: namespace ?? null })
      previousData.value = { ...(previousData.value ?? { pods: [], deployments: [], services: [], configmaps: [], secrets: [] }), pods: pods.value }
      pods.value = result
      staleTimestamp.value = null
    } catch (e: any) {
      console.warn('Failed to load pods:', e.toString?.() ?? e)
      if (pods.value.length === 0 && previousData.value) {
        pods.value = previousData.value.pods
        staleTimestamp.value = Date.now()
      }
    }
  }

  /** Load deployments for the given namespace (null = all namespaces). */
  async function loadDeployments(namespace: string | null = null): Promise<void> {
    try {
      const result = await invoke<K8sDeployment[]>('list_deployments', { namespace: namespace ?? null })
      previousData.value = { ...(previousData.value ?? { pods: [], deployments: [], services: [], configmaps: [], secrets: [] }), deployments: deployments.value }
      deployments.value = result
    } catch (e: any) {
      console.warn('Failed to load deployments:', e.toString?.() ?? e)
      if (deployments.value.length === 0 && previousData.value) {
        deployments.value = previousData.value.deployments
      }
    }
  }

  /** Load services for the given namespace (null = all namespaces). */
  async function loadServices(namespace: string | null = null): Promise<void> {
    try {
      const result = await invoke<K8sService[]>('list_services', { namespace: namespace ?? null })
      previousData.value = { ...(previousData.value ?? { pods: [], deployments: [], services: [], configmaps: [], secrets: [] }), services: services.value }
      services.value = result
    } catch (e: any) {
      console.warn('Failed to load services:', e.toString?.() ?? e)
      if (services.value.length === 0 && previousData.value) {
        services.value = previousData.value.services
      }
    }
  }

  /** Load configmaps for the given namespace (null = all namespaces). */
  async function loadConfigMaps(namespace: string | null = null): Promise<void> {
    try {
      const result = await invoke<K8sConfigMap[]>('list_configmaps', { namespace: namespace ?? null })
      previousData.value = { ...(previousData.value ?? { pods: [], deployments: [], services: [], configmaps: [], secrets: [] }), configmaps: configmaps.value }
      configmaps.value = result
    } catch (e: any) {
      console.warn('Failed to load configmaps:', e.toString?.() ?? e)
      if (configmaps.value.length === 0 && previousData.value) {
        configmaps.value = previousData.value.configmaps
      }
    }
  }

  /** Load secrets (metadata only) for the given namespace (null = all namespaces). */
  async function loadSecrets(namespace: string | null = null): Promise<void> {
    try {
      const result = await invoke<K8sSecretMeta[]>('list_secrets', { namespace: namespace ?? null })
      previousData.value = { ...(previousData.value ?? { pods: [], deployments: [], services: [], configmaps: [], secrets: [] }), secrets: secrets.value }
      secrets.value = result
    } catch (e: any) {
      console.warn('Failed to load secrets:', e.toString?.() ?? e)
      if (secrets.value.length === 0 && previousData.value) {
        secrets.value = previousData.value.secrets
      }
    }
  }

  /** Reload all resources for the current namespace. */
  async function reloadAll(namespace: string | null = null): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await Promise.all([
        loadPods(namespace),
        loadDeployments(namespace),
        loadServices(namespace),
        loadConfigMaps(namespace),
        loadSecrets(namespace),
      ])
    } catch (e: any) {
      error.value = e.toString?.() ?? 'Failed to reload resources'
    } finally {
      loading.value = false
    }
  }

  /** Switch the active namespace and reload resources. */
  async function switchNamespace(namespace: string | null): Promise<void> {
    activeNamespace.value = namespace
    await reloadAll(namespace)
  }

  /** Get the YAML representation of a specific resource. */
  async function getResourceYaml(kind: ResourceKind, name: string, namespace: string): Promise<string> {
    try {
      const yaml = await invoke<string>('get_resource_yaml', { namespace, kind, name })
      return yaml
    } catch (e: any) {
      throw new Error(e.toString?.() ?? 'Failed to get resource YAML')
    }
  }

  /** Get events for a specific pod. */
  async function getPodEvents(namespace: string, name: string): Promise<K8sEvent[]> {
    try {
      const events = await invoke<K8sEvent[]>('get_pod_events', { namespace, name })
      return events
    } catch (e: any) {
      console.warn('Failed to get pod events:', e.toString?.() ?? e)
      return []
    }
  }

  /** Open the inspector drawer for a specific resource. */
  async function openInspector(kind: ResourceKind, name: string, namespace: string): Promise<void> {
    inspectedKind.value = kind
    inspectedName.value = name
    inspectedNamespace.value = namespace
    inspectorOpen.value = true
    inspectorYaml.value = null
    inspectorEvents.value = []

    // Fetch YAML in the background
    try {
      inspectorYaml.value = await getResourceYaml(kind, name, namespace)
    } catch (e: any) {
      inspectorYaml.value = `error: ${e.message ?? e}`
    }

    // Fetch events for pods
    if (kind === 'pod') {
      try {
        inspectorEvents.value = await getPodEvents(namespace, name)
      } catch { /* events are non-critical */ }
    }
  }

  /** Close the inspector drawer. */
  function closeInspector(): void {
    inspectorOpen.value = false
    inspectedKind.value = null
    inspectedName.value = null
    inspectedNamespace.value = null
    inspectorYaml.value = null
    inspectorEvents.value = []
  }

  /** Initialize: detect kubectl, load contexts, namespaces, and resources. */
  async function initialize(): Promise<void> {
    loading.value = true
    try {
      await detectKubectl()
      if (connectionStatus.value === 'connected') {
        await loadContexts()
        await loadNamespaces()
        await reloadAll(null)
      }
    } catch (e: any) {
      error.value = e.toString?.() ?? 'Initialization failed'
    } finally {
      loading.value = false
    }
  }

  return {
    // State
    connectionStatus, kubectlStatus, error, loading, staleTimestamp,
    contexts, activeContext, namespaces, activeNamespace,
    pods, deployments, services, configmaps, secrets,
    activeTab, activeConfigSubTab,
    inspectorOpen, inspectedKind, inspectedName, inspectedNamespace,
    inspectorYaml, inspectorEvents,
    isConnected,

    // Actions
    detectKubectl, loadContexts, switchContext,
    loadNamespaces, switchNamespace,
    loadPods, loadDeployments, loadServices, loadConfigMaps, loadSecrets,
    reloadAll,
    getResourceYaml, getPodEvents,
    openInspector, closeInspector,
    initialize,
  }
}
