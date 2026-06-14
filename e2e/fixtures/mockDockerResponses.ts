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
