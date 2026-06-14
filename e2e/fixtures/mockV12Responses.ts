import { Page } from '@playwright/test'

/**
 * Mocks for the ItzamBox v1.2.0 Tauri commands.
 * (Sprints 18-22: Kubernetes, Swarm, Backup, Metrics, UI Polish.)
 *
 * This is designed to be added ON TOP of `mockDockerResponses(page)`.
 * It registers a separate `__tauriV12MockOverrides` window global that
 * is checked before the default `mockDockerResponses` fallbacks, so it
 * will not interfere with the existing tests.
 *
 * Usage:
 *   await mockDockerResponses(page)
 *   await mockV12Responses(page)   // extend with v1.2.0 mocks
 */
export async function mockV12Responses(page: Page) {
  await page.addInitScript(() => {
    window.__tauriV12MockOverrides = window.__tauriV12MockOverrides || {}

    // Build a sample of 1 hour of CPU data points (one per minute, 60 points)
    const now = Math.floor(Date.now() / 1000)
    const hostMetricsRange: any[] = []
    for (let i = 59; i >= 0; i--) {
      hostMetricsRange.push({
        timestamp: now - i * 60,
        cpu_percent: 15 + Math.sin(i / 5) * 8 + Math.random() * 5,
        memory_used_bytes: 4_000_000_000 + Math.random() * 1_000_000_000,
        memory_total_bytes: 16_000_000_000,
        network_rx_bytes: Math.floor(Math.random() * 1_000_000),
        network_tx_bytes: Math.floor(Math.random() * 500_000),
        disk_read_bytes: Math.floor(Math.random() * 2_000_000),
        disk_write_bytes: Math.floor(Math.random() * 1_000_000),
      })
    }

    // ── Kubernetes ──
    const kubectlDetected = {
      kubectl_installed: true,
      kubectl_version: 'v1.28.4',
      kubeconfig_exists: true,
      active_context: 'minikube',
    }
    const kubectlMissing = {
      kubectl_installed: false,
      kubectl_version: null,
      kubeconfig_exists: false,
      active_context: null,
    }

    const k8sContexts = [
      { name: 'minikube', cluster: 'minikube', user: 'minikube', is_active: true },
      { name: 'prod-cluster', cluster: 'prod-us-east-1', user: 'admin', is_active: false },
      { name: 'staging', cluster: 'staging-eu-west-1', user: 'developer', is_active: false },
    ]
    const k8sNamespaces = ['default', 'kube-public', 'kube-system', 'monitoring', 'observability', 'staging']
    const k8sPods = [
      {
        name: 'nginx-7c8d9f-abc12',
        namespace: 'default',
        status: 'Running',
        restarts: 0,
        age: '2d',
        node: 'node-1',
        ip: '10.244.0.5',
        containers: [{ name: 'nginx', image: 'nginx:1.25', ports: ['80/TCP'], ready: true, restart_count: 0, readiness_probe: 'httpGet /healthz', liveness_probe: 'httpGet /healthz', resources_limits: null, resources_requests: null }],
        labels: { app: 'nginx', tier: 'frontend' },
        annotations: {},
      },
      {
        name: 'redis-0',
        namespace: 'default',
        status: 'Running',
        restarts: 0,
        age: '5d',
        node: 'node-2',
        ip: '10.244.1.3',
        containers: [{ name: 'redis', image: 'redis:7-alpine', ports: ['6379/TCP'], ready: true, restart_count: 0, readiness_probe: null, liveness_probe: 'tcpSocket 6379', resources_limits: null, resources_requests: null }],
        labels: { app: 'redis', role: 'cache' },
        annotations: {},
      },
    ]
    const k8sDeployments = [
      { name: 'nginx', namespace: 'default', ready: '2/2', up_to_date: 2, available: 2, age: '2d', strategy: 'RollingUpdate', selector: { app: 'nginx' } },
    ]
    const k8sServices = [
      { name: 'nginx-svc', namespace: 'default', service_type: 'ClusterIP', cluster_ip: '10.96.0.10', external_ip: null, ports: ['80:30080/TCP'], age: '2d', selector: { app: 'nginx' } },
    ]
    const k8sConfigmaps = [
      { name: 'nginx-config', namespace: 'default', keys_count: 3, age: '2d', data_keys: ['server.conf', 'mime.types', 'nginx.conf'] },
    ]
    const k8sSecrets = [
      { name: 'db-credentials', namespace: 'default', secret_type: 'Opaque', keys_count: 2, age: '5d', data_keys: ['username', 'password'] },
    ]
    const k8sEvents = [
      { timestamp: '2026-06-14T10:15:00Z', event_type: 'Normal', reason: 'Scheduled', message: 'Successfully assigned default/nginx-7c8d9f-abc12 to node-1' },
      { timestamp: '2026-06-14T10:15:02Z', event_type: 'Normal', reason: 'Pulled', message: 'Container image "nginx:1.25" already present on machine' },
    ]
    const k8sResourceYaml = `apiVersion: v1
kind: Pod
metadata:
  name: nginx-7c8d9f-abc12
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:1.25
status:
  phase: Running
`

    // ── Swarm ──
    const swarmInactive = {
      active: false,
      node_id: null,
      nodes_count: 0,
      managers_count: 0,
      services_count: 0,
    }
    const swarmActive = {
      active: true,
      node_id: 'swarm-node-1-abc123def',
      nodes_count: 3,
      managers_count: 1,
      services_count: 2,
    }
    const swarmNodes = [
      { id: 'swarm-node-1-abc123def', hostname: 'manager-01', role: 'Manager', status: 'Ready', availability: 'Active', engine_version: '24.0.7', ip_address: '192.168.1.10', cpu_cores: 8, memory_bytes: 16_000_000_000, labels: { 'node.role': 'manager' } },
      { id: 'swarm-node-2-xyz789ghi', hostname: 'worker-01', role: 'Worker', status: 'Ready', availability: 'Active', engine_version: '24.0.7', ip_address: '192.168.1.11', cpu_cores: 4, memory_bytes: 8_000_000_000, labels: {} },
      { id: 'swarm-node-3-jkl456mno', hostname: 'worker-02', role: 'Worker', status: 'Ready', availability: 'Active', engine_version: '24.0.7', ip_address: '192.168.1.12', cpu_cores: 4, memory_bytes: 8_000_000_000, labels: {} },
    ]
    const swarmServices = [
      { id: 'svc-nginx-aaa111', name: 'nginx_web', mode: 'Replicated', replicas: '3/3', image: 'nginx:alpine', ports: ['80:8080'] },
      { id: 'svc-redis-bbb222', name: 'redis_cache', mode: 'Replicated', replicas: '1/1', image: 'redis:7-alpine', ports: ['6379'] },
    ]
    const swarmStacks = [
      { name: 'monitoring', services_count: 3, orchestrator: 'Swarm' },
      { name: 'webapp', services_count: 5, orchestrator: 'Swarm' },
    ]
    const swarmInspectNode = {
      ID: 'swarm-node-1-abc123def',
      Spec: { Role: 'manager', Availability: 'active' },
      Status: { State: 'ready' },
      Description: { Hostname: 'manager-01', Platform: { Architecture: 'x86_64' } },
    }

    // ── Backup ──
    const backupSummary = {
      total_snapshots: 12,
      total_size_bytes: 4_500_000_000,
      last_backup_at: now - 3600,
      last_backup_name: 'postgres_data_2026-06-14.tar.gz',
      oldest_backup_at: now - 7 * 24 * 3600,
      active_jobs_count: 2,
    }
    const backupSnapshots = (() => {
      const base = [
        { id: 1, job_id: 1, name: 'postgres_data_2026-06-14T09-00-00.tar.gz', source_volume: 'postgres_data', destination_path: '/var/backups/itzambox/', size_bytes: 1_200_000_000, sha256: 'a1b2c3d4e5f6', status: 'completed', failure_reason: null, duration_seconds: 45, created_at: now - 3600 },
        { id: 2, job_id: null, name: 'redis_cache_2026-06-14T08-30-00.tar.gz', source_volume: 'redis_cache', destination_path: '/var/backups/itzambox/', size_bytes: 256_000_000, sha256: 'b2c3d4e5f678', status: 'completed', failure_reason: null, duration_seconds: 12, created_at: now - 7200 },
        { id: 3, job_id: null, name: 'app_configs_2026-06-14.tar.gz', source_volume: 'app_configs', destination_path: '/var/backups/itzambox/', size_bytes: 45_000_000, sha256: 'c3d4e5f67890', status: 'completed', failure_reason: null, duration_seconds: 5, created_at: now - 10800 },
        { id: 4, job_id: 1, name: 'nightly_grafana_2026-06-13.tar.gz', source_volume: 'grafana_data', destination_path: '/var/backups/itzambox/', size_bytes: 1_180_000_000, sha256: 'd4e5f6789012', status: 'completed', failure_reason: null, duration_seconds: 85, created_at: now - 86400 },
        { id: 5, job_id: 1, name: 'nightly_postgres_2026-06-13.tar.gz', source_volume: 'postgres_data', destination_path: '/var/backups/itzambox/', size_bytes: 2_050_000_000, sha256: 'e5f678901234', status: 'completed', failure_reason: null, duration_seconds: 140, created_at: now - 86400 },
        { id: 6, job_id: null, name: 'redis_snapshot_2026-06-12.tar.gz', source_volume: 'redis_cache', destination_path: '/var/backups/itzambox/', size_bytes: 261_000_000, sha256: 'f67890123456', status: 'completed', failure_reason: null, duration_seconds: 22, created_at: now - 172800 },
        { id: 7, job_id: null, name: 'pre_upgrade_all_volumes.tar.gz', source_volume: 'postgres_data', destination_path: '/var/backups/pre-upgrade/', size_bytes: 5_600_000_000, sha256: '789012345678', status: 'completed', failure_reason: null, duration_seconds: 300, created_at: now - 259200 },
        { id: 8, job_id: null, name: 'weekly_full_2026-06-11.tar.gz', source_volume: 'app_configs', destination_path: '/var/backups/weekly/', size_bytes: 48_000_000, sha256: '890123456789', status: 'completed', failure_reason: null, duration_seconds: 7, created_at: now - 259200 },
        { id: 9, job_id: 1, name: 'nightly_grafana_2026-06-11.tar.gz', source_volume: 'grafana_data', destination_path: '/var/backups/itzambox/', size_bytes: 1_150_000_000, sha256: '901234567890', status: 'completed', failure_reason: null, duration_seconds: 88, created_at: now - 345600 },
        { id: 10, job_id: 1, name: 'nightly_postgres_2026-06-10.tar.gz', source_volume: 'postgres_data', destination_path: '/var/backups/itzambox/', size_bytes: 2_030_000_000, sha256: '012345678901', status: 'completed', failure_reason: null, duration_seconds: 138, created_at: now - 432000 },
        { id: 11, job_id: null, name: 'adhoc_redis_backup.tar.gz', source_volume: 'redis_cache', destination_path: '/var/backups/itzambox/', size_bytes: 258_000_000, sha256: '123456789012', status: 'completed', failure_reason: null, duration_seconds: 20, created_at: now - 518400 },
        { id: 12, job_id: null, name: 'scheduled_db_backup.tar.gz', source_volume: 'postgres_data', destination_path: '/var/backups/scheduled/', size_bytes: 2_080_000_000, sha256: '234567890123', status: 'completed', failure_reason: null, duration_seconds: 142, created_at: now - 604800 },
      ]
      return base
    })()
    const backupJobs = [
      {
        id: 1,
        name: 'Nightly DB Backup',
        frequency: 'daily',
        cron_expression: '0 2 * * *',
        source_volumes: ['postgres_data', 'redis_cache'],
        destination_path: '/var/backups/itzambox/',
        retention_count: 7,
        enabled: true,
        created_at: now - 30 * 86400,
        updated_at: now - 86400,
        last_run_at: now - 86400,
        next_run_at: now + 3600,
      },
      {
        id: 2,
        name: 'Hourly Config Snapshot',
        frequency: 'hourly',
        cron_expression: '0 * * * *',
        source_volumes: ['app_configs'],
        destination_path: '/var/backups/itzambox/',
        retention_count: 24,
        enabled: true,
        created_at: now - 14 * 86400,
        updated_at: now - 3600,
        last_run_at: now - 3600,
        next_run_at: now + 1800,
      },
    ]

    // ── Metrics ──
    const metricsSummary = {
      avg: 18.5,
      max: 45.2,
      min: 5.1,
      current: 12.3,
      totalPoints: hostMetricsRange.length,
    }

    // Default map of v1.2.0 command → mock response
    const defaults: Record<string, any> = {
      // Kubernetes
      detect_kubectl: kubectlDetected,
      list_k8s_contexts: k8sContexts,
      set_k8s_context: null,
      list_namespaces: k8sNamespaces,
      list_pods: k8sPods,
      list_deployments: k8sDeployments,
      list_services: k8sServices,
      list_configmaps: k8sConfigmaps,
      list_secrets: k8sSecrets,
      get_resource_yaml: k8sResourceYaml,
      get_pod_events: k8sEvents,
      detect_linux_distro: { id: 'ubuntu', name: 'Ubuntu', version: '22.04', id_like: 'debian', package_manager: 'apt', supported: true },

      // Swarm
      swarm_status: swarmInactive,
      swarm_init: 'Swarm initialized: node-id swarm-node-1-abc123def',
      swarm_join: 'This node has joined the swarm as a worker.',
      swarm_leave: null,
      list_swarm_nodes: swarmNodes,
      inspect_swarm_node: swarmInspectNode,
      list_swarm_services: swarmServices,
      inspect_swarm_service: { ID: 'svc-nginx-aaa111', Spec: { Name: 'nginx_web' } },
      list_stacks: swarmStacks,
      deploy_stack: null,
      remove_stack: null,

      // Backup
      get_backup_summary: backupSummary,
      list_backups: backupSnapshots,
      create_backup: backupSnapshots[0],
      restore_backup: null,
      delete_backup: null,
      verify_checksum: true,
      cancel_backup: null,
      list_backup_jobs: backupJobs,
      schedule_backup: backupJobs[0],
      toggle_backup_job: null,
      delete_backup_job: null,

      // Metrics
      get_host_metrics_range: hostMetricsRange,
      get_container_metrics_range: [],
      export_metrics_csv: null,
      export_metrics_json: null,
      get_metrics_db_size: 12_400_000,
    }

    Object.entries(defaults).forEach(([cmd, value]) => {
      window.__tauriV12MockOverrides[cmd] = () => value
    })

    // Override the global Tauri invoke to consult __tauriV12MockOverrides first,
    // then fall through to __tauriMockOverrides (set by mockDockerResponses).
    const tauri = (window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__
    if (tauri && tauri.invoke) {
      const originalInvoke = tauri.invoke
      tauri.invoke = async (cmd: string, args?: any) => {
        if (window.__tauriV12MockOverrides && window.__tauriV12MockOverrides[cmd] !== undefined) {
          const fn = window.__tauriV12MockOverrides[cmd]
          if (typeof fn === 'function') return fn(args)
          return fn
        }
        return originalInvoke(cmd, args)
      }
    }
  })
}

/**
 * Helper to override a v1.2.0 command during a test.
 * Uses addInitScript so the override survives navigation.
 */
export async function setV12Override(page: Page, cmd: string, value: any) {
  await page.addInitScript(({ cmd, value }) => {
    window.__tauriV12MockOverrides = window.__tauriV12MockOverrides || {}
    if (typeof value === 'object' && value !== null && 'error' in value) {
      window.__tauriV12MockOverrides[cmd] = () => { throw new Error(value.error) }
    } else {
      window.__tauriV12MockOverrides[cmd] = () => value
    }
  }, { cmd, value })
}

// Type augmentation for the window globals
declare global {
  interface Window {
    __tauriV12MockOverrides: Record<string, any>
  }
}
