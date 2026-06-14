import { Page } from '@playwright/test'

export async function mockDockerResponses(page: Page) {
  await page.addInitScript(() => {
    // Provide both __TAURI__ and __TAURI_INTERNALS__ for compatibility
    const mockTauri = {
      invoke: async (cmd: string, args?: any) => {
        console.log(`Mock Tauri Invoke: ${cmd}`, args)
        
        // Check for runtime overrides
        if (window.__tauriMockOverrides && window.__tauriMockOverrides[cmd]) {
          const override = window.__tauriMockOverrides[cmd];
          if (typeof override === 'function') {
            return override(args);
          } else {
            throw override; // can throw error object or return string/number/object
          }
        }

        // Config commands
        if (cmd === 'get_config') {
          if (args?.key === 'theme') return 'dark'
          if (args?.key === 'lang') return 'es'
          if (args?.key === 'onboarding_completed') return 'true'
          return ''
        }
        if (cmd === 'set_config') {
          return null
        }

        // Host / Docker Info commands
        if (cmd === 'get_host_metrics') {
          return {
            cpu_usage_percent: 12.5,
            cpu_cores: 8,
            memory_used_bytes: 4000000000,
            memory_total_bytes: 16000000000,
            uptime_seconds: 3600,
            hostname: 'itzambox-test',
            os_name: 'Linux',
            kernel_version: '5.15.0',
          }
        }
        if (cmd === 'check_docker_installed') {
          return {
            docker_installed: true,
            docker_version: '24.0.7',
            compose_available: true,
            compose_version: '2.21.0',
            service_running: true,
            user_in_docker_group: true,
            socket_exists: true,
          }
        }

        // Container commands
        if (cmd === 'list_containers') {
          return [
            {
              id: '1234567890ab',
              name: 'nginx-test',
              image: 'nginx:alpine',
              status: 'Up 2 hours',
              state: 'running',
              cpu_percentage: 0.5,
              memory_usage_bytes: 15000000,
              memory_limit_bytes: 8000000000,
              network_rx_bytes: 500,
              network_tx_bytes: 300,
              compose_project: null,
              compose_service: null,
            }
          ]
        }
        if (cmd === 'get_container_stats') {
          return {
            container_id: args?.id || '1234567890ab',
            cpu_percentage: 1.2,
            memory_usage_bytes: 18000000,
            memory_limit_bytes: 8000000000,
            memory_percentage: 0.22,
            network_rx_bytes: 1024,
            network_tx_bytes: 2048,
            block_read_bytes: 0,
            block_write_bytes: 4096,
            pids: 3,
            timestamp: Date.now() / 1000,
          }
        }
        if (cmd === 'get_container_logs') {
          return '2026-06-13T20:00:00Z [info] Starting nginx...\n2026-06-13T20:00:01Z [info] Configuration loaded.\n2026-06-13T20:00:01Z [info] Ready for connections.\n'
        }
        if (cmd === 'inspect_container') {
          return {
            Id: '1234567890ab',
            Name: '/nginx-test',
            State: { Status: 'running', Running: true, StartedAt: '2026-06-13T18:00:00Z', FinishedAt: '0001-01-01T00:00:00Z', ExitCode: 0 },
            Config: {
              Image: 'nginx:alpine',
              Cmd: ['nginx', '-g', 'daemon off;'],
              Entrypoint: ['/docker-entrypoint.sh'],
              Env: [
                'PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
                'NGINX_VERSION=1.25.3',
                'NJS_VERSION=0.7.12',
                'PKG_RELEASE=1~bookworm',
              ],
              Labels: { 'com.example.app': 'web', 'com.example.env': 'production', 'maintainer': 'NGINX Docker Maintainers' },
            },
            Platform: 'linux',
            Architecture: 'amd64',
            Driver: 'overlay2',
            RestartCount: 0,
            HostConfig: { RestartPolicy: { Name: 'always' } },
            Mounts: [
              { Type: 'bind', Source: '/host/data', Destination: '/data', Mode: 'rw', RW: true },
              { Type: 'volume', Source: 'nginx-data', Destination: '/usr/share/nginx/html', Mode: 'z', RW: true },
            ],
            LogPath: '/var/lib/docker/containers/1234/log.json',
            Created: '2026-06-13T17:55:00Z',
            NetworkSettings: {
              IPAddress: '172.17.0.2',
              Ports: { '80/tcp': [{ HostIp: '0.0.0.0', HostPort: '8080' }] },
              Networks: {
                'bridge': {
                  IPAddress: '172.17.0.2',
                  IPPrefixLen: 16,
                  Gateway: '172.17.0.1',
                  MacAddress: '02:42:ac:11:00:02',
                  NetworkID: 'abc123def456',
                  EndpointID: 'endpoint789',
                }
              }
            }
          }
        }
        if (cmd === 'list_container_dir') {
          return [
            { name: 'etc', full_path: '/etc', is_dir: true, is_symlink: false, size_bytes: 4096, permissions: 'rwxr-xr-x', owner: 'root', group: 'root', updated_at: 1718320000 },
            { name: 'var', full_path: '/var', is_dir: true, is_symlink: false, size_bytes: 4096, permissions: 'rwxr-xr-x', owner: 'root', group: 'root', updated_at: 1718320000 },
            { name: 'nginx.conf', full_path: '/nginx.conf', is_dir: false, is_symlink: false, size_bytes: 1024, permissions: 'rw-r--r--', owner: 'root', group: 'root', updated_at: 1718320000 }
          ]
        }
        if (cmd === 'read_file_preview') {
          return 'events {}\nhttp {\n  server {\n    listen 80;\n  }\n}\n'
        }

        // Image commands
        if (cmd === 'list_images') {
          return [
            {
              id: 'sha256:abcd1234',
              repository: 'nginx',
              tag: 'alpine',
              size_bytes: 23000000,
              created_at: 1718320000,
            }
          ]
        }
        if (cmd === 'get_image_history') {
          return [
            { digest: 'sha256:1111', size_bytes: 5000000, command: 'ADD file:...', created_at: 1718320000 },
            { digest: 'sha256:2222', size_bytes: 18000000, command: 'CMD ["nginx"]', created_at: 1718320000 }
          ]
        }
        if (cmd === 'search_dockerhub') {
          return [
            { name: 'nginx', description: 'Official build of Nginx.', star_count: 18000, pull_count: 100000000, is_official: true, is_automated: false }
          ]
        }

        // Volume commands
        if (cmd === 'list_volumes') {
          return [
            { name: 'nginx-data', driver: 'local', mountpoint: '/var/lib/docker/volumes/nginx-data/_data' }
          ]
        }

        // Network commands
        if (cmd === 'list_networks') {
          return [
            { id: 'net123', name: 'bridge', driver: 'bridge', scope: 'local', subnet: '172.17.0.0/16', gateway: '172.17.0.1', internal: false, containers: [] }
          ]
        }

        // Compose commands
        if (cmd === 'detect_compose_projects') {
          return [
            {
              name: 'test-project',
              path: '/home/josue/test-project',
              file: 'docker-compose.yml',
              services: ['web', 'db'],
            }
          ]
        }
        if (cmd === 'parse_compose_file') {
          return {
            services: [
              { name: 'web', image: 'nginx:alpine', build: null, ports: ['80:80'], volumes: [], environment: [], depends_on: [] },
              { name: 'db', image: 'postgres:alpine', build: null, ports: ['5432:5432'], volumes: [], environment: [], depends_on: [] }
            ],
            volumes: ['db-data'],
            networks: ['test-net']
          }
        }
        if (cmd === 'compose_ps') {
          return [
            { name: 'web', id: '111', state: 'running', status: 'Up 1 hour', ports: '80->80' },
            { name: 'db', id: '222', state: 'running', status: 'Up 1 hour', ports: '5432->5432' }
          ]
        }
        if (cmd === 'read_compose_file') {
          return 'version: "3.8"\nservices:\n  web:\n    image: nginx:alpine\n    ports:\n      - "80:80"\n  db:\n    image: postgres:alpine\n    ports:\n      - "5432:5432"\nvolumes:\n  db-data:\nnetworks:\n  test-net:\n'
        }
        if (cmd === 'validate_compose_file') {
          return { valid: true, error: null }
        }
        if (cmd === 'format_compose_file') {
          return 'version: "3.8"\nservices:\n  web:\n    image: nginx:alpine\n    ports:\n      - "80:80"\n  db:\n    image: postgres:alpine\n    ports:\n      - "5432:5432"\nvolumes:\n  db-data:\nnetworks:\n  test-net:\n'
        }

        // Notification commands
        if (cmd === 'get_notifications') {
          return [
            { id: '1', type_str: 'success', title: 'Container Started', message: 'nginx-test successfully started', read: false, created_at: Math.floor(Date.now() / 1000) }
          ]
        }
        if (cmd === 'save_notification') return null
        if (cmd === 'mark_notification_read') return null
        if (cmd === 'mark_all_read') return null
        if (cmd === 'clear_notifications') return null

        // Templates commands
        if (cmd === 'list_templates') {
          return [
            {
              id: 1,
              name: 'Nginx Web Server',
              description: 'High-performance HTTP server',
              image: 'nginx:alpine',
              default_ports: '[{"host":"8080","container":"80","protocol":"tcp"}]',
              default_volumes: '[]',
              default_env: '[]',
              default_network: 'bridge',
              default_restart: 'always',
              default_command: null,
              is_builtin: true,
              category: 'web',
              icon: 'globe'
            }
          ]
        }
        if (cmd === 'get_template') {
          return {
            id: 1,
            name: 'Nginx Web Server',
            description: 'High-performance HTTP server',
            image: 'nginx:alpine',
            default_ports: '[{"host":"8080","container":"80","protocol":"tcp"}]',
            default_volumes: '[]',
            default_env: '[]',
            default_network: 'bridge',
            default_restart: 'always',
            default_command: null,
            is_builtin: true,
            category: 'web',
            icon: 'globe'
          }
        }

        // ── Tauri event plugin: used by @tauri-apps/api/event listen() ──
        if (cmd === 'plugin:event|listen') {
          // For backup-progress events, fire a mock progress payload
          if (args?.event === 'backup-progress') {
            const handlerId = args?.handler
            if (handlerId !== undefined && window.__tauri_callbacks__?.[handlerId]) {
              setTimeout(() => {
                window.__tauri_callbacks__[handlerId]({
                  payload: {
                    job_id: 'job-1',
                    snapshot_name: 'postgres_data_2026-06-14T10-30-00.tar.gz',
                    bytes_processed: 600_000_000,
                    bytes_total: 1_200_000_000,
                    elapsed_seconds: 30,
                    percent: 50,
                    status: 'in_progress',
                    message: 'Snapshotting postgres_data',
                  },
                })
              }, 1000)
            }
          }
          return 42
        }
        if (cmd === 'plugin:event|unlisten') {
          return null
        }

        // ── v1.2.0: Backup & Restore commands ──
        if (cmd === 'list_backups') {
          console.log('[MOCK] list_backups called, returning 12 records');
          return [
            { id: 1, job_id: null, name: 'prod-db-2026-06-01', source_volume: 'postgres_data', destination_path: '/backups/postgres', size_bytes: 2_100_000_000, sha256: 'abc123def456', status: 'completed', failure_reason: null, duration_seconds: 145, created_at: 1717200000 },
            { id: 2, job_id: null, name: 'redis-snapshot-2026-06-02', source_volume: 'redis_cache', destination_path: '/backups/redis', size_bytes: 256_000_000, sha256: 'def789abc012', status: 'completed', failure_reason: null, duration_seconds: 23, created_at: 1717286400 },
            { id: 3, job_id: null, name: 'app-configs-backup', source_volume: 'app_configs', destination_path: '/backups/app', size_bytes: 45_000_000, sha256: 'ghi345jkl678', status: 'completed', failure_reason: null, duration_seconds: 5, created_at: 1717372800 },
            { id: 4, job_id: 1, name: 'nightly-grafana-2026-06-03', source_volume: 'grafana_data', destination_path: '/backups/grafana', size_bytes: 1_200_000_000, sha256: 'mno901pqr234', status: 'completed', failure_reason: null, duration_seconds: 89, created_at: 1717459200 },
            { id: 5, job_id: 1, name: 'nightly-postgres-2026-06-04', source_volume: 'postgres_data', destination_path: '/backups/postgres', size_bytes: 2_050_000_000, sha256: 'stu567vwx890', status: 'completed', failure_reason: null, duration_seconds: 138, created_at: 1717545600 },
            { id: 6, job_id: null, name: 'redis-snapshot-2026-06-05', source_volume: 'redis_cache', destination_path: '/backups/redis', size_bytes: 261_000_000, sha256: 'yz123abc456d', status: 'completed', failure_reason: null, duration_seconds: 25, created_at: 1717632000 },
            { id: 7, job_id: null, name: 'pre-upgrade-all-volumes', source_volume: 'postgres_data', destination_path: '/backups/pre-upgrade', size_bytes: 5_600_000_000, sha256: 'efg789hij012', status: 'completed', failure_reason: null, duration_seconds: 312, created_at: 1717718400 },
            { id: 8, job_id: null, name: 'weekly-full-2026-06-07', source_volume: 'app_configs', destination_path: '/backups/weekly', size_bytes: 48_000_000, sha256: 'klm345nop678', status: 'completed', failure_reason: null, duration_seconds: 7, created_at: 1717804800 },
            { id: 9, job_id: 1, name: 'nightly-grafana-2026-06-08', source_volume: 'grafana_data', destination_path: '/backups/grafana', size_bytes: 1_180_000_000, sha256: 'qrs901tuv234', status: 'completed', failure_reason: null, duration_seconds: 92, created_at: 1717891200 },
            { id: 10, job_id: 1, name: 'nightly-postgres-2026-06-09', source_volume: 'postgres_data', destination_path: '/backups/postgres', size_bytes: 2_030_000_000, sha256: 'vwx567yza890', status: 'completed', failure_reason: null, duration_seconds: 141, created_at: 1717977600 },
            { id: 11, job_id: null, name: 'ad-hoc-redis-backup', source_volume: 'redis_cache', destination_path: '/backups/redis', size_bytes: 258_000_000, sha256: 'bcd123efg456', status: 'completed', failure_reason: null, duration_seconds: 22, created_at: 1718064000 },
            { id: 12, job_id: null, name: 'scheduled-db-backup', source_volume: 'postgres_data', destination_path: '/backups/scheduled', size_bytes: 2_080_000_000, sha256: 'hij789klm012', status: 'completed', failure_reason: null, duration_seconds: 144, created_at: 1718150400 },
          ]
        }
        if (cmd === 'get_backup_summary') {
          return { total_snapshots: 12, total_size_bytes: 17_128_000_000, last_backup_at: 1718150400, last_backup_name: 'scheduled-db-backup', oldest_backup_at: 1717200000, active_jobs_count: 1 }
        }
        if (cmd === 'create_backup') {
          return { id: 13, job_id: null, name: 'manual-snapshot', source_volume: args?.volume || 'unknown', destination_path: args?.destPath || '/backups/manual', size_bytes: 0, sha256: null, status: 'in_progress', failure_reason: null, duration_seconds: null, created_at: Math.floor(Date.now() / 1000) }
        }
        if (cmd === 'restore_backup') {
          return null
        }
        if (cmd === 'delete_backup') {
          return null
        }
        if (cmd === 'verify_backup_checksum') {
          return true
        }
        if (cmd === 'cancel_backup') {
          return null
        }
        if (cmd === 'list_backup_jobs') {
          return [
            { id: 1, name: 'Nightly Backup', frequency: 'daily', cron_expression: '0 2 * * *', source_volumes: ['postgres_data', 'grafana_data'], destination_path: '/backups/nightly', retention_count: 30, enabled: true, created_at: 1717000000, updated_at: 1717000000, last_run_at: 1718150400, next_run_at: 1718236800 },
            { id: 2, name: 'Weekly Full', frequency: 'weekly', cron_expression: '0 3 * * 0', source_volumes: ['redis_cache', 'app_configs'], destination_path: '/backups/weekly', retention_count: 12, enabled: true, created_at: 1717000000, updated_at: 1717000000, last_run_at: 1717804800, next_run_at: 1718409600 },
          ]
        }
        if (cmd === 'create_backup_job') { return { id: 3 } }
        if (cmd === 'update_backup_job') { return null }
        if (cmd === 'delete_backup_job') { return null }
        if (cmd === 'toggle_backup_job') { return null }

        // ── v1.2.0: Kubernetes commands ──
        if (cmd === 'detect_kubectl') {
          return { kubectl_installed: true, kubectl_version: '1.28.2', kubeconfig_exists: true, active_context: 'minikube' }
        }
        if (cmd === 'list_k8s_contexts') {
          return [
            { name: 'minikube', cluster: 'minikube', user: 'minikube', is_active: true },
            { name: 'prod-cluster', cluster: 'prod-eks', user: 'admin', is_active: false },
            { name: 'staging', cluster: 'staging-eks', user: 'dev', is_active: false },
          ]
        }
        if (cmd === 'set_k8s_context') { return null }
        if (cmd === 'list_namespaces') {
          return ['default', 'kube-system', 'kube-public', 'production', 'staging', 'development']
        }
        if (cmd === 'list_pods') {
          return [
            { name: 'nginx-7c8d9f-abc12', namespace: 'default', status: 'Running', restarts: 0, age: '12d', node: 'minikube-node-1', ip: '10.1.0.5', containers: [{ name: 'nginx', image: 'nginx:alpine', ports: ['80/tcp', '443/tcp'], ready: true, restart_count: 0, readiness_probe: 'HTTP :80/', liveness_probe: 'HTTP :80/', resources_limits: { memory: '256Mi', cpu: '500m' }, resources_requests: { memory: '128Mi', cpu: '250m' } }], labels: { app: 'nginx', tier: 'web' }, annotations: {} },
            { name: 'api-server-6d4e8f-xyz78', namespace: 'production', status: 'Running', restarts: 1, age: '45d', node: 'minikube-node-2', ip: '10.1.0.12', containers: [{ name: 'api', image: 'api-server:2.1.0', ports: ['3000/tcp'], ready: true, restart_count: 1, readiness_probe: 'TCP :3000', liveness_probe: 'HTTP :3000/health', resources_limits: { memory: '512Mi', cpu: '1' }, resources_requests: { memory: '256Mi', cpu: '500m' } }], labels: { app: 'api', tier: 'backend' }, annotations: { 'monitoring': 'enabled' } },
            { name: 'redis-master-0', namespace: 'staging', status: 'Running', restarts: 0, age: '90d', node: 'minikube-node-1', ip: '10.1.0.20', containers: [{ name: 'redis', image: 'redis:7-alpine', ports: ['6379/tcp'], ready: true, restart_count: 0, readiness_probe: null, liveness_probe: null, resources_limits: null, resources_requests: null }], labels: { app: 'redis', role: 'master' }, annotations: {} },
          ]
        }
        if (cmd === 'list_deployments') {
          return [
            { name: 'nginx-deployment', namespace: 'default', ready: '3/3', up_to_date: 3, available: 3, age: '12d', strategy: 'RollingUpdate', selector: { app: 'nginx' } },
            { name: 'api-deployment', namespace: 'production', ready: '2/2', up_to_date: 2, available: 2, age: '45d', strategy: 'RollingUpdate', selector: { app: 'api' } },
          ]
        }
        if (cmd === 'list_services') {
          return [
            { name: 'nginx-service', namespace: 'default', service_type: 'LoadBalancer', cluster_ip: '10.96.0.1', external_ip: '192.168.49.2', ports: ['80:80/TCP', '443:443/TCP'], age: '12d', selector: { app: 'nginx' } },
            { name: 'api-service', namespace: 'production', service_type: 'ClusterIP', cluster_ip: '10.96.0.10', external_ip: null, ports: ['3000/TCP'], age: '45d', selector: { app: 'api' } },
          ]
        }
        if (cmd === 'list_configmaps') {
          return [
            { name: 'nginx-config', namespace: 'default', keys_count: 3, age: '12d', data_keys: ['nginx.conf', 'mime.types', 'ssl-params.conf'] },
          ]
        }
        if (cmd === 'list_secrets') {
          return [
            { name: 'db-credentials', namespace: 'production', secret_type: 'Opaque', keys_count: 2, age: '30d', data_keys: ['username', 'password'] },
            { name: 'tls-cert', namespace: 'default', secret_type: 'kubernetes.io/tls', keys_count: 2, age: '45d', data_keys: ['tls.crt', 'tls.key'] },
          ]
        }
        if (cmd === 'get_resource_yaml') {
          return 'apiVersion: v1\nkind: Pod\nmetadata:\n  name: ' + (args?.name || 'unknown')
        }
        if (cmd === 'get_pod_events') {
          return [
            { timestamp: '2026-06-10T10:00:00Z', event_type: 'Normal', reason: 'Started', message: 'Container started' },
          ]
        }

        // ── v1.2.0: Swarm commands ──
        if (cmd === 'swarm_status') {
          return { active: false, node_id: null, nodes_count: 0, managers_count: 0, services_count: 0 }
        }
        if (cmd === 'swarm_init') {
          return 'Swarm initialized: current node (abc123def456) is now a manager.\n\nTo add a worker to this swarm, run the following command:\n  docker swarm join --token SWMTKN-1-abcdef 192.168.1.100:2377\n\nTo add a manager, run:\n  docker swarm join --token SWMTKN-1-ghijkl 192.168.1.100:2377'
        }
        if (cmd === 'swarm_join') { return null }
        if (cmd === 'swarm_leave') { return null }
        if (cmd === 'list_swarm_nodes') { return [] }
        if (cmd === 'list_swarm_services') { return [] }
        if (cmd === 'list_swarm_stacks') { return [] }

        // ── v1.2.0: Metrics commands ──
        if (cmd === 'get_host_metrics_range') {
          const now = Date.now() / 1000
          const from = args?.from ?? (now - 3600)
          const to = args?.to ?? now
          const count = Math.min(120, Math.floor((to - from) / 15))
          const points: any[] = []
          for (let i = 0; i < count; i++) {
            const ts = from + ((to - from) * i) / count
            points.push({
              cpu_percent: 25 + Math.sin(i * 0.5) * 15 + Math.random() * 10,
              memory_used_bytes: 4_000_000_000 + Math.sin(i * 0.3) * 500_000_000,
              memory_total_bytes: 16_000_000_000,
              network_rx_bytes: 1_500_000 + Math.sin(i * 0.7) * 500_000,
              network_tx_bytes: 750_000 + Math.sin(i * 0.7) * 250_000,
              disk_read_bytes: 50_000_000 + Math.sin(i * 0.4) * 20_000_000,
              disk_write_bytes: 25_000_000 + Math.sin(i * 0.4) * 10_000_000,
              timestamp: Math.floor(ts),
            })
          }
          return points
        }
        if (cmd === 'get_metrics_history') {
          const now = Date.now()
          const points = []
          for (let i = 60; i >= 0; i--) {
            const ts = Math.floor(now / 1000) - i * 60
            points.push({
              timestamp: ts,
              cpu_percent: 12.5 + Math.random() * 40,
              memory_percent: 55 + Math.random() * 25,
              network_rx_bytes: 1024 * (1 + Math.random() * 10),
              network_tx_bytes: 512 * (1 + Math.random() * 5),
              disk_read_bytes: 4096 * (1 + Math.random() * 8),
              disk_write_bytes: 2048 * (1 + Math.random() * 4),
            })
          }
          return points
        }

        // Fallback or generic commands
        return []
      },
      listen: async (eventName: string, handler: any) => {
        console.log(`Mock Tauri Listen: ${eventName}`)
        return () => {}
      }
    }

    // Set up globals
    window.__tauriMockOverrides = window.__tauriMockOverrides || {}
    window.__TAURI__ = mockTauri as any
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      unregisterListener: () => {},
    }
    window.__TAURI_INTERNALS__ = {
      ...mockTauri,
      transformCallback: (callback: any, once: boolean) => {
        const id = Math.floor(Math.random() * 1000000)
        window.__tauri_callbacks__ = window.__tauri_callbacks__ || {}
        window.__tauri_callbacks__[id] = (res: any) => {
          if (once) {
            delete window.__tauri_callbacks__[id]
          }
          callback(res)
        }
        return id
      },
      metadata: {}
    } as any
  })
}

// Helper to set override during a test — uses addInitScript to survive navigation
export async function setTauriOverride(page: Page, cmd: string, value: any) {
  await page.addInitScript(({ cmd, value }) => {
    window.__tauriMockOverrides = window.__tauriMockOverrides || {}
    if (typeof value === 'object' && value !== null && 'error' in value) {
      window.__tauriMockOverrides[cmd] = new Error(value.error)
    } else {
      window.__tauriMockOverrides[cmd] = () => value
    }
  }, { cmd, value })
}
