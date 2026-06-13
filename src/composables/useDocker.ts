// ItzamBox — Docker Tauri Commands Wrapper
// Copyright (C) 2026 SodigTech — GPL-3.0

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref } from 'vue'

export interface ContainerInfo {
  id: string; name: string; image: string; status: string; state: string
  cpu_percentage: number; memory_usage_bytes: number; memory_limit_bytes: number
  network_rx_bytes: number; network_tx_bytes: number
  compose_project: string | null; compose_service: string | null
}

export interface ImageInfo {
  id: string; repository: string; tag: string
  size_bytes: number; created_at: number
}

export interface ImageLayerInfo {
  digest: string
  size_bytes: number
  command: string
  created_at: number
}

export interface VolumeInfo {
  name: string; driver: string; mountpoint: string
}

export interface NetworkContainer {
  container_id: string
  container_name: string
  ipv4_address: string
  mac_address: string
}

export interface NetworkInfo {
  id: string; name: string; driver: string; scope: string
  subnet: string | null; gateway: string | null; internal: boolean
  containers: NetworkContainer[]
}

export interface DockerHubImage {
  name: string
  description: string
  star_count: number
  pull_count: number
  is_official: boolean
  is_automated: boolean
}

export interface PortMapping {
  host_ip: string
  host_port: number
  container_port: number
  protocol: string
}

export interface CreateContainerParams {
  image: string
  name?: string | null
  ports: PortMapping[]
  volumes: string[]
  env_vars: string[]
  network?: string | null
  restart_policy?: string | null
  command?: string[] | null
  detach: boolean
  cpu_limit?: number | null
  memory_limit?: string | null
  privileged: boolean
}

export interface ContainerStats {
  container_id: string
  cpu_percentage: number
  memory_usage_bytes: number
  memory_limit_bytes: number
  memory_percentage: number
  network_rx_bytes: number
  network_tx_bytes: number
  block_read_bytes: number
  block_write_bytes: number
  pids: number
  timestamp: number
}

export interface HostMetrics {
  cpu_usage_percent: number; cpu_cores: number
  memory_used_bytes: number; memory_total_bytes: number
  uptime_seconds: number; hostname: string; os_name: string; kernel_version: string
}

export function useDocker() {
  const containers = ref<ContainerInfo[]>([])
  const images = ref<ImageInfo[]>([])
  const volumes = ref<VolumeInfo[]>([])
  const networks = ref<NetworkInfo[]>([])
  const hostMetrics = ref<HostMetrics | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchContainers(showAll = true) {
    try {
      containers.value = await invoke<ContainerInfo[]>('list_containers', { showAll })
      error.value = null
    } catch (e: any) {
      error.value = e.toString()
    }
  }

  async function fetchImages() {
    try {
      images.value = await invoke<ImageInfo[]>('list_images')
      error.value = null
    } catch (e: any) {
      // Don't overwrite error from containers — images are non-critical
      console.warn('Failed to fetch images:', e.toString())
    }
  }

  async function fetchVolumes() {
    try {
      volumes.value = await invoke<VolumeInfo[]>('list_volumes')
      error.value = null
    } catch (e: any) {
      console.warn('Failed to fetch volumes:', e.toString())
    }
  }

  async function fetchNetworks() {
    try {
      networks.value = await invoke<NetworkInfo[]>('list_networks')
      error.value = null
    } catch (e: any) {
      console.warn('Failed to fetch networks:', e.toString())
    }
    return networks.value
  }

  async function fetchHostMetrics() {
    try {
      hostMetrics.value = await invoke<HostMetrics>('get_host_metrics')
    } catch (e: any) {
      console.warn('Failed to fetch host metrics:', e.toString())
    }
  }

  async function startContainer(id: string) {
    try {
      await invoke('start_container', { id })
      await fetchContainers()
    } catch (e: any) { error.value = e.toString() }
  }

  async function stopContainer(id: string) {
    try {
      await invoke('stop_container', { id })
      await fetchContainers()
    } catch (e: any) { error.value = e.toString() }
  }

  async function restartContainer(id: string) {
    try {
      await invoke('restart_container', { id })
      await fetchContainers()
    } catch (e: any) { error.value = e.toString() }
  }

  async function pauseContainer(id: string) {
    try {
      await invoke('pause_container', { id })
      await fetchContainers()
    } catch (e: any) { error.value = e.toString() }
  }

  async function unpauseContainer(id: string) {
    try {
      await invoke('unpause_container', { id })
      await fetchContainers()
    } catch (e: any) { error.value = e.toString() }
  }

  async function removeContainer(id: string, force = false, removeVolumes = false) {
    try {
      await invoke('remove_container', { id, force, removeVolumes })
      await fetchContainers()
    } catch (e: any) { error.value = e.toString() }
  }

  async function searchDockerHub(query: string, limit?: number): Promise<DockerHubImage[]> {
    try {
      return await invoke<DockerHubImage[]>('search_dockerhub', { query, limit: limit ?? null })
    } catch (e: any) {
      error.value = e.toString()
      return []
    }
  }

  async function createAndRunContainer(params: CreateContainerParams): Promise<string> {
    try {
      const result = await invoke<string>('create_and_run_container', {
        image: params.image,
        name: params.name ?? null,
        ports: params.ports,
        volumes: params.volumes,
        envVars: params.env_vars,
        network: params.network ?? null,
        restartPolicy: params.restart_policy ?? null,
        command: params.command ?? null,
        detach: params.detach,
        cpuLimit: params.cpu_limit ?? null,
        memoryLimit: params.memory_limit ?? null,
        privileged: params.privileged,
      })
      error.value = null
      return result
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  async function getImageHistory(id: string): Promise<ImageLayerInfo[]> {
    try {
      const result = await invoke<ImageLayerInfo[]>('get_image_history', { id })
      error.value = null
      return result
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  async function pullImage(imageName: string) {
    try {
      await invoke('pull_image', { imageName })
      await fetchImages()
    } catch (e: any) { error.value = e.toString() }
  }

  async function removeImage(id: string, force = false) {
    try {
      await invoke('remove_image', { id, force })
      await fetchImages()
    } catch (e: any) { error.value = e.toString() }
  }

  async function inspectContainer(id: string): Promise<Record<string, any>> {
    return await invoke<Record<string, any>>('inspect_container', { id })
  }

  async function getContainerLogs(id: string, tail: number, timestamps: boolean): Promise<string> {
    return await invoke<string>('get_container_logs', { id, tail, timestamps })
  }

  async function getContainerStats(id: string): Promise<ContainerStats> {
    return await invoke<ContainerStats>('get_container_stats', { id })
  }

  async function refreshAll() {
    loading.value = true
    error.value = null
    const results = await Promise.allSettled([fetchContainers(), fetchImages(), fetchVolumes(), fetchHostMetrics()])
    // Only set error if containers failed (critical path)
    const containersResult = results[0]
    if (containersResult.status === 'rejected') {
      error.value = containersResult.reason?.toString() || 'Failed to fetch containers'
    }
    loading.value = false
  }

  async function exportContainer(containerId: string, outputPath: string): Promise<void> {
    try {
      await invoke('export_container', { containerId, outputPath })
      error.value = null
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  async function commitContainer(
    containerId: string,
    repository: string,
    tag: string,
    message?: string | null,
    author?: string | null,
  ): Promise<string> {
    try {
      const result = await invoke<string>('commit_container', {
        containerId,
        repository,
        tag,
        message: message ?? null,
        author: author ?? null,
      })
      error.value = null
      return result
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  async function saveImage(imageName: string, outputPath: string): Promise<void> {
    try {
      await invoke('save_image', { imageName, outputPath })
      error.value = null
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  async function loadImage(inputPath: string): Promise<string> {
    try {
      const result = await invoke<string>('load_image', { inputPath })
      error.value = null
      return result
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  return {
    containers, images, volumes, networks, hostMetrics, loading, error,
    fetchContainers, fetchImages, fetchVolumes, fetchNetworks, fetchHostMetrics,
    startContainer, stopContainer, restartContainer, pauseContainer, unpauseContainer, removeContainer,
    pullImage, removeImage, getImageHistory, searchDockerHub, createAndRunContainer, refreshAll,
    inspectContainer, getContainerLogs, getContainerStats,
    exportContainer, commitContainer, saveImage, loadImage,
  }
}

// ─── Docker Installer Types & Functions ───────────────────────────────────

export interface LinuxDistro {
  id: string
  name: string
  version: string
  id_like: string | null
  package_manager: string
  supported: boolean
}

export interface DockerInstallStatus {
  docker_installed: boolean
  docker_version: string | null
  compose_available: boolean
  compose_version: string | null
  service_running: boolean
  user_in_docker_group: boolean
  socket_exists: boolean
}

export interface InstallProgress {
  step: number
  total_steps: number
  message: string
  status: 'in_progress' | 'completed' | 'error'
}

/** Detect the Linux distribution for Docker installation. */
export async function detectLinuxDistro(): Promise<LinuxDistro> {
  return invoke<LinuxDistro>('detect_linux_distro')
}

/** Check whether Docker is currently installed and its state. */
export async function checkDockerInstalled(): Promise<DockerInstallStatus> {
  return invoke<DockerInstallStatus>('check_docker_installed')
}

/**
 * Run the full Docker Engine installation for the detected distro.
 * Subscribe to 'installer-progress' events via `listen` for real-time updates.
 */
export async function installDocker(): Promise<void> {
  return invoke<void>('install_docker')
}

/** Validate that Docker runs correctly (hello-world container). */
export async function validateDockerInstall(): Promise<DockerInstallStatus> {
  return invoke<DockerInstallStatus>('validate_docker_install')
}

/** Listen to installer progress events. Returns an unlisten function. */
export function onInstallProgress(
  callback: (progress: InstallProgress) => void,
): Promise<() => void> {
  return listen<InstallProgress>('installer-progress', (event) => {
    callback(event.payload)
  })
}

// ─── File Explorer Types & Functions ──────────────────────────────────────

export interface FileMetadata {
  name: string
  full_path: string
  is_dir: boolean
  is_symlink: boolean
  size_bytes: number
  permissions: string
  owner: string
  group: string
  updated_at: number
}

export async function listContainerDir(containerId: string, path: string): Promise<FileMetadata[]> {
  return invoke<FileMetadata[]>('list_container_dir', { containerId, path })
}

export async function downloadFile(containerId: string, remotePath: string, localPath: string): Promise<void> {
  return invoke<void>('download_file_from_container', { containerId, remotePath, localDest: localPath })
}

export async function uploadFile(containerId: string, localPath: string, remotePath: string): Promise<void> {
  return invoke<void>('upload_file_to_container', { containerId, localSrc: localPath, remoteDest: remotePath })
}

export async function readFilePreview(containerId: string, remotePath: string, maxBytes: number): Promise<string> {
  return invoke<string>('read_file_preview', { containerId, remotePath, maxBytes })
}

// ─── Docker Event Stream Types & Functions ──────────────────────────────────

export interface DockerEvent {
  event_type: string   // "container", "image", "volume", "network"
  action: string        // "start", "stop", "create", "destroy", "die", etc.
  actor_id: string      // Container / Image / Volume ID
  actor_name: string    // Container / Image name
  timestamp: number     // Unix epoch seconds
  attributes: Record<string, string>
}

/**
 * Subscribe to real-time Docker events emitted by the backend.
 *
 * The returned `unlisten` function MUST be called when the consumer is done
 * (e.g. in `onUnmounted`) to avoid stale listeners.
 */
export function listenDockerEvents(
  callback: (event: DockerEvent) => void,
): Promise<() => void> {
  return listen<DockerEvent>('docker-event', (event) => {
    callback(event.payload)
  })
}

/** Start the Docker event stream in the backend. */
export async function startEventStream(): Promise<void> {
  return invoke<void>('start_event_stream')
}

/** Stop the Docker event stream in the backend. */
export async function stopEventStream(): Promise<void> {
  return invoke<void>('stop_event_stream')
}

// ─── Relative Timestamp Helper ──────────────────────────────────────────────

/**
 * Format a Unix-epoch-second timestamp as a human-friendly relative string
 * (e.g. "just now", "2s ago", "5m ago", "1h ago", "3d ago").
 */
export function relativeTime(unixSeconds: number): string {
  const now = Math.floor(Date.now() / 1000)
  const diff = now - unixSeconds

  if (diff < 0) return 'just now'
  if (diff < 5) return 'just now'
  if (diff < 60) return `${diff}s ago`
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  if (diff < 2592000) return `${Math.floor(diff / 86400)}d ago`
  return `${Math.floor(diff / 2592000)}mo ago`
}

// ─── Safe Formatting Utilities ──────────────────────────────────────────────

/** Safely format a numeric value, returning a fallback on corrupt data. */
export function safeCurrency(val: unknown): string {
  if (typeof val !== 'number' || isNaN(val)) return '$0.00'
  return '$' + val.toFixed(2)
}

/** Safely format a percentage value, returning a fallback on corrupt data. */
export function safePct(val: unknown): string {
  if (typeof val !== 'number' || isNaN(val)) return '0%'
  return val.toFixed(1) + '%'
}

/** Safely format a number with commas. */
export function safeNumber(val: unknown): string {
  if (typeof val !== 'number' || isNaN(val)) return '0'
  return val.toLocaleString()
}

// Sanitize paths to prevent traversal attacks
export function sanitizePath(userPath: string): string {
  // Remove null bytes
  let clean = userPath.replace(/\0/g, '')
  // Resolve and normalize to prevent path traversal
  // We don't allow ".." segments
  const segments = clean.split('/').filter(s => s !== '' && s !== '.')
  const resolved: string[] = []
  for (const seg of segments) {
    if (seg === '..') continue // Skip parent directory references
    resolved.push(seg)
  }
  return '/' + resolved.join('/')
}

// ─── Docker Compose Types & Functions ──────────────────────────────────────

export interface ComposeProject {
  name: string
  path: string
  file: string
  services: string[]
}

export interface ComposeServiceStatus {
  name: string
  id: string | null
  state: string
  status: string
  ports: string
}

export interface ComposeFileInfo {
  services: ComposeServiceDefinition[]
  volumes: string[]
  networks: string[]
}

export interface ComposeServiceDefinition {
  name: string
  image: string | null
  build: string | null
  ports: string[]
  volumes: string[]
  environment: string[]
  depends_on: string[]
}

export async function detectComposeProjects(dir?: string): Promise<ComposeProject[]> {
  return invoke<ComposeProject[]>('detect_compose_projects', { dir: dir || null })
}

export async function parseComposeFile(projectPath: string): Promise<ComposeFileInfo> {
  return invoke<ComposeFileInfo>('parse_compose_file', { projectPath })
}

export async function composeUp(projectPath: string, detached: boolean, services?: string[]): Promise<void> {
  return invoke<void>('compose_up', { projectPath, detached, services: services || null })
}

export async function composeDown(projectPath: string, removeVolumes: boolean, removeImages: boolean): Promise<void> {
  return invoke<void>('compose_down', { projectPath, removeVolumes, removeImages })
}

export async function composeRestart(projectPath: string, services?: string[]): Promise<void> {
  return invoke<void>('compose_restart', { projectPath, services: services || null })
}

export async function composeLogs(projectPath: string, tail: number, services?: string[]): Promise<string> {
  return invoke<string>('compose_logs', { projectPath, tail, services: services || null })
}

export async function composePs(projectPath: string): Promise<ComposeServiceStatus[]> {
  return invoke<ComposeServiceStatus[]>('compose_ps', { projectPath })
}

// ─── Container Templates Types & Functions ────────────────────────────────

export interface ContainerTemplate {
  id?: number | null
  name: string
  description: string
  image: string
  default_ports: string
  default_volumes: string
  default_env: string
  default_network: string
  default_restart: string
  default_command: string | null
  is_builtin: boolean
  category: string
  icon: string
}

export interface PortConfig {
  host: string
  container: string
  protocol: string
}

export async function listTemplates(): Promise<ContainerTemplate[]> {
  return invoke<ContainerTemplate[]>('list_templates')
}

export async function getTemplate(id: number): Promise<ContainerTemplate> {
  return invoke<ContainerTemplate>('get_template', { id })
}

export async function saveTemplate(template: ContainerTemplate): Promise<number> {
  return invoke<number>('save_template', { template })
}

export async function deleteTemplate(id: number): Promise<void> {
  return invoke<void>('delete_template', { id })
}

export async function seedBuiltinTemplates(): Promise<void> {
  return invoke<void>('seed_builtin_templates')
}

/** Parse a JSON string array of port configs into a usable array. */
export function parseTemplatePorts(portsJson: string): PortConfig[] {
  try {
    return JSON.parse(portsJson) as PortConfig[]
  } catch {
    return []
  }
}

/** Parse a JSON string array of volume mounts. */
export function parseJsonArray(json: string): string[] {
  try {
    return JSON.parse(json) as string[]
  } catch {
    return []
  }
}

/** Format category for display. */
export function formatCategory(cat: string): string {
  return cat
    .split('-')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

/** Get a display color for a category. */
export function categoryColor(cat: string): string {
  const colors: Record<string, string> = {
    web: '#0ea5e9',
    database: '#22c55e',
    cache: '#eab308',
    runtime: '#a855f7',
    'message-queue': '#f97316',
    custom: '#64748b',
  }
  return colors[cat] || colors.custom
}

// ─── Image Build Types & Functions ──────────────────────────────────────────

export interface BuildLogLine {
  line: string
  stream: 'stdout' | 'stderr'
}

export interface BuildCompletePayload {
  success: boolean
  image_id: string | null
  tags: string[]
  error: string | null
}

export interface BuildImageParams {
  dockerfilePath: string
  contextDir: string
  tags: string[]
  buildArgs: string[]
  noCache: boolean
  pullBase: boolean
}

/** Invoke the `docker build` command on the backend. Returns the image ID on success. */
export async function buildImage(params: BuildImageParams): Promise<string> {
  return invoke<string>('build_image', {
    dockerfilePath: params.dockerfilePath,
    contextDir: params.contextDir,
    tags: params.tags,
    buildArgs: params.buildArgs,
    noCache: params.noCache,
    pullBase: params.pullBase,
  })
}

/** Subscribe to real-time build log lines. Returns an `unlisten` function. */
export function listenBuildLog(
  callback: (log: BuildLogLine) => void,
): Promise<() => void> {
  return listen<BuildLogLine>('build-log', (event) => {
    callback(event.payload)
  })
}

/** Subscribe to the build completion event. Returns an `unlisten` function. */
export function listenBuildComplete(
  callback: (payload: BuildCompletePayload) => void,
): Promise<() => void> {
  return listen<BuildCompletePayload>('build-complete', (event) => {
    callback(event.payload)
  })
}

// ─── Registry Types & Functions ────────────────────────────────────────────

export interface Registry {
  id: number | null
  name: string
  url: string
  username: string | null
  auth_token?: string | null
  is_default: boolean
}

export interface RegistrySafe {
  id: number
  name: string
  url: string
  username: string | null
  is_default: boolean
}

export interface PushLogLine {
  line: string
  stream: 'stdout' | 'stderr'
}

export interface PushComplete {
  success: boolean
  image: string
  error: string | null
}

/** List all registries (without auth tokens). */
export async function listRegistries(): Promise<RegistrySafe[]> {
  return invoke<RegistrySafe[]>('list_registries')
}

/** Add a new registry. */
export async function addRegistry(
  name: string,
  url: string,
  username: string | null,
  authToken: string | null,
  isDefault: boolean,
): Promise<number> {
  return invoke<number>('add_registry', { name, url, username, authToken, isDefault })
}

/** Update an existing registry. */
export async function updateRegistry(
  id: number,
  name: string,
  url: string,
  username: string | null,
  authToken: string | null,
  isDefault: boolean,
): Promise<void> {
  return invoke<void>('update_registry', { id, name, url, username, authToken, isDefault })
}

/** Remove a registry by id. */
export async function removeRegistry(id: number): Promise<void> {
  return invoke<void>('remove_registry', { id })
}

/** Set a registry as default. */
export async function setDefaultRegistry(id: number): Promise<void> {
  return invoke<void>('set_default_registry', { id })
}

/** Authenticate with a Docker registry via CLI login. */
export async function dockerLogin(url: string, username: string, password: string): Promise<string> {
  return invoke<string>('docker_login', { url, username, password })
}

/** Log out from a Docker registry. */
export async function dockerLogout(url: string): Promise<void> {
  return invoke<void>('docker_logout', { url })
}

/** Push an image to a registry. Optionally tags with registry prefix first. */
export async function pushImage(imageName: string, registryUrl?: string): Promise<void> {
  return invoke<void>('push_image', { imageName, registryUrl: registryUrl ?? null })
}

/** Subscribe to real-time push log lines. Returns an `unlisten` function. */
export function listenPushLog(
  callback: (log: PushLogLine) => void,
): Promise<() => void> {
  return listen<PushLogLine>('push-log', (event) => {
    callback(event.payload)
  })
}

/** Subscribe to push completion event. Returns an `unlisten` function. */
export function listenPushComplete(
  callback: (payload: PushComplete) => void,
): Promise<() => void> {
  return listen<PushComplete>('push-complete', (event) => {
    callback(event.payload)
  })
}

// ─── Vulnerability Scanner Types & Functions ──────────────────────────────

export interface Vulnerability {
  id: string
  package: string
  installed_version: string
  fixed_version: string | null
  severity: string
  title: string
  description: string
}

export interface VulnerabilityReport {
  image_name: string
  scanned_at: number
  total: number
  critical: Vulnerability[]
  high: Vulnerability[]
  medium: Vulnerability[]
  low: Vulnerability[]
}

export interface ScanProgress {
  step: string
  message: string
}

/** Detect which vulnerability scanner is available (Trivy or Grype). */
export async function detectScanner(): Promise<string | null> {
  try {
    return await invoke<string | null>('detect_scanner')
  } catch {
    return null
  }
}

/** Scan an image for vulnerabilities. Emits 'scan-progress' events. */
export async function scanImage(imageName: string): Promise<VulnerabilityReport> {
  return invoke<VulnerabilityReport>('scan_image', { imageName })
}

/** Retrieve scan history for an image. */
export async function getScanHistory(imageName: string): Promise<VulnerabilityReport[]> {
  return invoke<VulnerabilityReport[]>('get_scan_history', { imageName })
}

/** Listen to scan progress events. Returns an unlisten function. */
export function listenScanProgress(
  callback: (progress: ScanProgress) => void,
): Promise<() => void> {
  return listen<ScanProgress>('scan-progress', (event) => {
    callback(event.payload)
  })
}

/** Get severity color for vulnerability badges. */
export function severityColor(severity: string): string {
  const map: Record<string, string> = {
    critical: 'var(--accent-red)',
    high: 'var(--accent-yellow)',
    medium: 'var(--accent-blue)',
    low: 'var(--text-muted)',
  }
  return map[severity.toLowerCase()] || map.low
}

/** Get severity background for vulnerability badges. */
export function severityBg(severity: string): string {
  const map: Record<string, string> = {
    critical: 'rgba(239, 68, 68, 0.1)',
    high: 'rgba(245, 158, 11, 0.1)',
    medium: 'rgba(59, 130, 246, 0.1)',
    low: 'rgba(156, 163, 175, 0.08)',
  }
  return map[severity.toLowerCase()] || map.low
}

/** Safely format vulnerability count. */
export function safeCount(val: unknown): string {
  if (typeof val !== 'number' || isNaN(val)) return '0'
  return val.toLocaleString()
}

// Format bytes to human-readable string
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const idx = Math.min(i, units.length - 1)
  const val = bytes / Math.pow(1024, idx)
  return val.toFixed(idx === 0 ? 0 : 1) + ' ' + units[idx]
}

// Format timestamp to readable date
export function formatFileTime(timestamp: number): string {
  if (!timestamp) return '—'
  const d = new Date(timestamp * 1000)
  return d.toLocaleDateString(undefined, {
    year: 'numeric', month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

// Get file icon class based on extension
export function getFileIconClass(name: string, isDir: boolean, isSymlink: boolean): string {
  if (isSymlink) return 'fa-solid fa-link'
  if (isDir) return 'fa-solid fa-folder'
  const ext = name.split('.').pop()?.toLowerCase() || ''
  const iconMap: Record<string, string> = {
    html: 'fa-brands fa-html5',
    htm: 'fa-brands fa-html5',
    js: 'fa-brands fa-js',
    jsx: 'fa-brands fa-react',
    ts: 'fa-brands fa-js',
    tsx: 'fa-brands fa-react',
    css: 'fa-brands fa-css3-alt',
    scss: 'fa-brands fa-sass',
    json: 'fa-solid fa-brackets-curly',
    xml: 'fa-solid fa-code',
    yaml: 'fa-solid fa-file-lines',
    yml: 'fa-solid fa-file-lines',
    md: 'fa-solid fa-file-lines',
    txt: 'fa-solid fa-file-lines',
    py: 'fa-brands fa-python',
    go: 'fa-solid fa-file-code',
    rs: 'fa-solid fa-file-code',
    java: 'fa-brands fa-java',
    sh: 'fa-solid fa-terminal',
    bash: 'fa-solid fa-terminal',
    log: 'fa-solid fa-scroll',
    conf: 'fa-solid fa-gear',
    cfg: 'fa-solid fa-gear',
    ini: 'fa-solid fa-gear',
    env: 'fa-solid fa-gear',
    dockerfile: 'fa-brands fa-docker',
    dockerignore: 'fa-brands fa-docker',
    sql: 'fa-solid fa-database',
    db: 'fa-solid fa-database',
    png: 'fa-solid fa-image',
    jpg: 'fa-solid fa-image',
    jpeg: 'fa-solid fa-image',
    gif: 'fa-solid fa-image',
    svg: 'fa-solid fa-image',
    ico: 'fa-solid fa-image',
    pdf: 'fa-solid fa-file-pdf',
    zip: 'fa-solid fa-file-zipper',
    tar: 'fa-solid fa-file-zipper',
    gz: 'fa-solid fa-file-zipper',
    gzip: 'fa-solid fa-file-zipper',
  }
  return iconMap[ext] || 'fa-solid fa-file'
}
