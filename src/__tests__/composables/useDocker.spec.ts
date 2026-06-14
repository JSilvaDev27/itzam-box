import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  useDocker,
  detectLinuxDistro,
  checkDockerInstalled,
  installDocker,
  validateDockerInstall,
  onInstallProgress,
  listContainerDir,
  downloadFile,
  uploadFile,
  readFilePreview,
  startEventStream,
  stopEventStream,
  detectComposeProjects,
  parseComposeFile,
  composeUp,
  composeDown,
  composeRestart,
  composeLogs,
  composePs,
  readComposeFile,
  writeComposeFile,
  validateComposeFile,
  formatComposeFile,
  listTemplates,
  getTemplate,
  saveTemplate,
  deleteTemplate,
  seedBuiltinTemplates,
  buildImage,
  listRegistries,
  addRegistry,
  updateRegistry,
  removeRegistry,
  setDefaultRegistry,
  dockerLogin,
  dockerLogout,
  pushImage,
  detectScanner,
  scanImage,
  getScanHistory
} from '../../composables/useDocker'
import { invoke } from '@tauri-apps/api/core'

describe('useDocker composable', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should fetch containers and set reactive value', async () => {
    const { containers, fetchContainers, error } = useDocker()
    const mockContainers = [{ id: '1', name: 'c1', image: 'nginx', status: 'Up', state: 'running' }]
    vi.mocked(invoke).mockResolvedValueOnce(mockContainers)

    await fetchContainers(true)
    expect(invoke).toHaveBeenCalledWith('list_containers', { showAll: true })
    expect(containers.value).toEqual(mockContainers)
    expect(error.value).toBeNull()
  })

  it('should fetch images and set reactive value', async () => {
    const { images, fetchImages } = useDocker()
    const mockImages = [{ id: 'i1', repository: 'nginx', tag: 'latest', size_bytes: 100, created_at: 12345 }]
    vi.mocked(invoke).mockResolvedValueOnce(mockImages)

    await fetchImages()
    expect(invoke).toHaveBeenCalledWith('list_images')
    expect(images.value).toEqual(mockImages)
  })

  it('should fetch volumes and set reactive value', async () => {
    const { volumes, fetchVolumes } = useDocker()
    const mockVolumes = [{ name: 'v1', driver: 'local', mountpoint: '/var' }]
    vi.mocked(invoke).mockResolvedValueOnce(mockVolumes)

    await fetchVolumes()
    expect(invoke).toHaveBeenCalledWith('list_volumes')
    expect(volumes.value).toEqual(mockVolumes)
  })

  it('should fetch networks and set reactive value', async () => {
    const { networks, fetchNetworks } = useDocker()
    const mockNetworks = [{ id: 'n1', name: 'net1', driver: 'bridge', scope: 'local', subnet: null, gateway: null, internal: false, containers: [] }]
    vi.mocked(invoke).mockResolvedValueOnce(mockNetworks)

    await fetchNetworks()
    expect(invoke).toHaveBeenCalledWith('list_networks')
    expect(networks.value).toEqual(mockNetworks)
  })

  it('should fetch host metrics and set reactive value', async () => {
    const { hostMetrics, fetchHostMetrics } = useDocker()
    const mockMetrics = { cpu_usage_percent: 12, cpu_cores: 4, memory_used_bytes: 100, memory_total_bytes: 1000, uptime_seconds: 3600, hostname: 'host', os_name: 'linux', kernel_version: '1.2' }
    vi.mocked(invoke).mockResolvedValueOnce(mockMetrics)

    await fetchHostMetrics()
    expect(invoke).toHaveBeenCalledWith('get_host_metrics')
    expect(hostMetrics.value).toEqual(mockMetrics)
  })

  it('should start and stop containers', async () => {
    const { startContainer, stopContainer } = useDocker()
    vi.mocked(invoke).mockResolvedValue(true)

    await startContainer('c1')
    expect(invoke).toHaveBeenCalledWith('start_container', { id: 'c1' })

    await stopContainer('c2')
    expect(invoke).toHaveBeenCalledWith('stop_container', { id: 'c2' })
  })

  it('covers installer, file explorer, events, compose, templates, build, registries and scanner helpers', async () => {
    vi.mocked(invoke).mockResolvedValue(true)

    await detectLinuxDistro()
    await checkDockerInstalled()
    await installDocker()
    await validateDockerInstall()
    
    // onInstallProgress
    const unlisten = await onInstallProgress(() => {})
    expect(unlisten).toBeTypeOf('function')

    await listContainerDir('c1', '/')
    await downloadFile('c1', '/remote', '/local')
    await uploadFile('c1', '/local', '/remote')
    await readFilePreview('c1', '/remote', 100)

    await startEventStream()
    await stopEventStream()

    await detectComposeProjects()
    await parseComposeFile('/path')
    await composeUp('/path', true)
    await composeDown('/path', true, true)
    await composeRestart('/path')
    await composeLogs('/path', 10)
    await composePs('/path')
    await readComposeFile('/path')
    await writeComposeFile('/path', 'content')
    await validateComposeFile('/path')
    await formatComposeFile('/path')

    await listTemplates()
    await getTemplate(1)
    await saveTemplate({ id: 1, name: 't', description: 'd', image: 'i', is_builtin: false } as any)
    await deleteTemplate(1)
    await seedBuiltinTemplates()

    await buildImage({ context_path: '/ctx', tag: 't1', dockerfile_path: 'df' } as any)

    await listRegistries()
    await addRegistry('r', 'u', 'u', 'p', false)
    await updateRegistry(1, 'r', 'u', 'u', 'p', false)
    await removeRegistry(1)
    await setDefaultRegistry(1)
    await dockerLogin('u', 'u', 'p')
    await dockerLogout('u')
    await pushImage('img')

    await detectScanner()
    await scanImage('img')
    await getScanHistory('img')
  })

  it('covers error handling branches of helper functions', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Mock failure'))

    await detectLinuxDistro().catch(() => {})
    await checkDockerInstalled().catch(() => {})
    await installDocker().catch(() => {})
    await validateDockerInstall().catch(() => {})
    await listContainerDir('c1', '/').catch(() => {})
    await downloadFile('c1', '/remote', '/local').catch(() => {})
    await uploadFile('c1', '/local', '/remote').catch(() => {})
    await readFilePreview('c1', '/remote', 100).catch(() => {})
    await startEventStream().catch(() => {})
    await stopEventStream().catch(() => {})
    await detectComposeProjects().catch(() => {})
    await parseComposeFile('/path').catch(() => {})
    await composeUp('/path', true).catch(() => {})
    await composeDown('/path', true, true).catch(() => {})
    await composeRestart('/path').catch(() => {})
    await composeLogs('/path', 10).catch(() => {})
    await composePs('/path').catch(() => {})
    await readComposeFile('/path').catch(() => {})
    await writeComposeFile('/path', 'content').catch(() => {})
    await validateComposeFile('/path').catch(() => {})
    await formatComposeFile('/path').catch(() => {})
    await listTemplates().catch(() => {})
    await getTemplate(1).catch(() => {})
    await saveTemplate({} as any).catch(() => {})
    await deleteTemplate(1).catch(() => {})
    await seedBuiltinTemplates().catch(() => {})
    await buildImage({} as any).catch(() => {})
    await listRegistries().catch(() => {})
    await addRegistry('', '', '', '', false).catch(() => {})
    await updateRegistry(1, '', '', '', '', false).catch(() => {})
    await removeRegistry(1).catch(() => {})
    await setDefaultRegistry(1).catch(() => {})
    await dockerLogin('u', 'u', 'p').catch(() => {})
    await dockerLogout('u').catch(() => {})
    await pushImage('img').catch(() => {})
    await detectScanner().catch(() => {})
    await scanImage('img').catch(() => {})
    await getScanHistory('img').catch(() => {})
  })

  it('covers useDocker hook error paths', async () => {
    const { fetchContainers, fetchImages, fetchVolumes, fetchNetworks, fetchHostMetrics, startContainer, stopContainer, restartContainer, pauseContainer } = useDocker()
    vi.mocked(invoke).mockRejectedValue(new Error('hook error'))
    await fetchContainers()
    await fetchImages()
    await fetchVolumes()
    await fetchNetworks()
    await fetchHostMetrics()
    await startContainer('c1')
    await stopContainer('c1')
    await restartContainer('c1')
    await pauseContainer('c1')
  })

  it('covers useDocker hook remaining action methods', async () => {
    const docker = useDocker()
    vi.mocked(invoke).mockResolvedValue(true)

    await docker.restartContainer('c1')
    await docker.pauseContainer('c1')
    await docker.unpauseContainer('c1')
    await docker.removeContainer('c1')

    await docker.pullImage('img')
    await docker.removeImage('img')
    await docker.getImageHistory('img')
    await docker.searchDockerHub('query')
    await docker.createAndRunContainer({ image: 'nginx', ports: [], volumes: [], env_vars: [], privileged: false, detach: true })
    await docker.refreshAll()

    await docker.inspectContainer('c1')
    await docker.getContainerLogs('c1', 10, false)
    await docker.getContainerStats('c1')

    await docker.exportContainer('c1', '/path')
    await docker.commitContainer('c1', 'repo', 'tag', 'msg', 'author')
    await docker.saveImage('img', '/path')
    await docker.loadImage('/path')
  })

  it('covers remaining useDocker hook error paths', async () => {
    const docker = useDocker()
    vi.mocked(invoke).mockRejectedValue(new Error('hook error'))

    await docker.restartContainer('c1')
    await docker.pauseContainer('c1')
    await docker.unpauseContainer('c1')
    await docker.removeContainer('c1')
    await docker.pullImage('img')
    await docker.removeImage('img')
    await docker.getImageHistory('img').catch(() => {})
    await docker.searchDockerHub('query').catch(() => {})
    await docker.createAndRunContainer({ image: 'nginx', ports: [], volumes: [], env_vars: [], privileged: false, detach: true }).catch(() => {})
    await docker.inspectContainer('c1').catch(() => {})
    await docker.getContainerLogs('c1', 10, false).catch(() => {})
    await docker.getContainerStats('c1').catch(() => {})
    await docker.exportContainer('c1', '/path').catch(() => {})
    await docker.commitContainer('c1', 'repo', 'tag', 'msg', 'author').catch(() => {})
    await docker.saveImage('img', '/path').catch(() => {})
    await docker.loadImage('/path').catch(() => {})
  })
})
