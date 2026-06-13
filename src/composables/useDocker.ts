// ItzamBox — Docker Tauri Commands Wrapper
// Copyright (C) 2026 SodigTech — GPL-3.0

import { invoke } from '@tauri-apps/api/core'
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

export interface VolumeInfo {
  name: string; driver: string; mountpoint: string
}

export interface NetworkInfo {
  id: string; name: string; driver: string; scope: string
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

  return {
    containers, images, volumes, hostMetrics, loading, error,
    fetchContainers, fetchImages, fetchVolumes, fetchHostMetrics,
    startContainer, stopContainer, restartContainer, pauseContainer, unpauseContainer, removeContainer,
    pullImage, removeImage, refreshAll,
  }
}
