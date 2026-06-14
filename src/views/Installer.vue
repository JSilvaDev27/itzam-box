<!-- ItzamBox — Docker Multi-Distro Installer Wizard
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  detectLinuxDistro,
  checkDockerInstalled,
  installDocker,
  validateDockerInstall,
  onInstallProgress,
  type LinuxDistro,
  type DockerInstallStatus,
  type InstallProgress,
} from '../composables/useDocker'

const router = useRouter()

// ─── State ─────────────────────────────────────────────────────────────────

const currentStep = ref(1)
const distro = ref<LinuxDistro | null>(null)
const initialStatus = ref<DockerInstallStatus | null>(null)
const progress = ref<InstallProgress | null>(null)
const validationResult = ref<DockerInstallStatus | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const installError = ref<string | null>(null)
const logLines = ref<string[]>([])
const detecting = ref(true)
const validating = ref(false)
const installComplete = ref(false)
const isLinux = ref(true)
let unlistenProgress: (() => void) | null = null

// ─── Distro icon mapping ───────────────────────────────────────────────────

const distroIcon = computed(() => {
  const id = distro.value?.id ?? ''
  const map: Record<string, string> = {
    ubuntu: 'fa-brands fa-ubuntu',
    debian: 'fa-brands fa-debian',
    linuxmint: 'fa-brands fa-linux-mint',
    fedora: 'fa-brands fa-fedora',
    rhel: 'fa-brands fa-redhat',
    centos: 'fa-brands fa-centos',
    arch: 'fa-brands fa-arch-linux',
    manjaro: 'fa-brands fa-arch-linux',
    endeavouros: 'fa-brands fa-arch-linux',
    opensuse: 'fa-brands fa-suse',
    'opensuse-tumbleweed': 'fa-brands fa-suse',
    'opensuse-leap': 'fa-brands fa-suse',
  }
  return map[id] || 'fa-brands fa-linux'
})

const distroDisplayName = computed(() => distro.value?.name ?? 'Unknown')

const distroColor = computed(() => {
  const id = distro.value?.id ?? ''
  const map: Record<string, string> = {
    ubuntu: '--accent-orange',
    debian: '--accent-red',
    linuxmint: '--accent-green',
    fedora: '--accent-blue',
    rhel: '--accent-red',
    centos: '--accent-yellow',
    arch: '--accent-cyan',
    manjaro: '--accent-green',
    opensuse: '--accent-green',
    'opensuse-tumbleweed': '--accent-green',
  }
  return `var(${map[id] || '--accent-cyan'})`
})

// ─── Step 2: Status checklist ──────────────────────────────────────────────

const statusChecks = computed(() => {
  const s = initialStatus.value
  if (!s) return []
  return [
    { label: 'Docker Engine installed', ok: s.docker_installed, detail: s.docker_version ?? 'Not installed' },
    { label: 'Docker Compose available', ok: s.compose_available, detail: s.compose_version ?? 'Not available' },
    { label: 'Docker service running', ok: s.service_running, detail: s.service_running ? 'Active' : 'Not running' },
    { label: 'User in docker group', ok: s.user_in_docker_group, detail: s.user_in_docker_group ? 'Yes' : 'No' },
    { label: 'Docker socket exists', ok: s.socket_exists, detail: s.socket_exists ? '/var/run/docker.sock' : 'Missing' },
  ]
})

const everythingInstalled = computed(() =>
  initialStatus.value?.docker_installed &&
  initialStatus.value?.service_running &&
  initialStatus.value?.user_in_docker_group
)

// ─── Step 3: Progress bar ─────────────────────────────────────────────────

const progressPercent = computed(() => {
  if (!progress.value) return 0
  const p = progress.value
  if (p.total_steps === 0) return 100
  // Map step fraction: completed step counts, current step in progress
  if (p.status === 'completed') return 100
  if (p.status === 'error') return Math.round((p.step / p.total_steps) * 100)
  // in_progress: current step counts as partial (half a step)
  const partial = ((p.step - 1) + 0.5) / p.total_steps
  return Math.round(partial * 100)
})

const stepLabel = computed(() => {
  if (!progress.value) return 'Preparing...'
  const p = progress.value
  return `Step ${p.step} of ${p.total_steps} — ${p.message}`
})

// ─── Lifecycle ─────────────────────────────────────────────────────────────

onMounted(async () => {
  await detectDistro()
})

onUnmounted(() => {
  unlistenProgress?.()
})

async function detectDistro() {
  detecting.value = true
  error.value = null
  try {
    const result = await detectLinuxDistro()
    distro.value = result
    isLinux.value = result.supported || result.package_manager !== 'unknown'
  } catch (e: any) {
    isLinux.value = false
    error.value = e.toString?.() ?? 'Failed to detect operating system'
  } finally {
    detecting.value = false
  }
}

// ─── Step 2: Check status ──────────────────────────────────────────────────

async function checkStatus() {
  loading.value = true
  error.value = null
  try {
    const status = await checkDockerInstalled()
    initialStatus.value = status
    currentStep.value = 2
  } catch (e: any) {
    error.value = e.toString?.() ?? 'Failed to check Docker status'
  } finally {
    loading.value = false
  }
}

// ─── Step 3: Installation ──────────────────────────────────────────────────

async function startInstallation() {
  installError.value = null
  logLines.value = []
  progress.value = null
  installComplete.value = false
  currentStep.value = 3

  // Subscribe to progress events
  unlistenProgress?.()
  unlistenProgress = await onInstallProgress((p) => {
    progress.value = p
    // Add log line
    const icon = p.status === 'completed' ? '✅' : p.status === 'error' ? '❌' : '⏳'
    logLines.value.push(`${icon} ${p.message}`)
    if (p.status === 'error') {
      installError.value = p.message
    }
    if (p.status === 'completed') {
      installComplete.value = true
    }
  })

  try {
    await installDocker()
  } catch (e: any) {
    const msg = e.toString?.() ?? 'Installation failed'
    installError.value = msg
    logLines.value.push(`❌ ${msg}`)
    const currentProgress = progress.value as InstallProgress | null
    progress.value = {
      step: currentProgress?.step ?? 1,
      total_steps: currentProgress?.total_steps ?? 1,
      message: msg,
      status: 'error',
    }
  }
}

// ─── Step 4: Validation ────────────────────────────────────────────────────

async function runValidation() {
  validating.value = true
  error.value = null
  try {
    const result = await validateDockerInstall()
    validationResult.value = result
    currentStep.value = 4
  } catch (e: any) {
    error.value = e.toString?.() ?? 'Validation failed'
    currentStep.value = 4
  } finally {
    validating.value = false
  }
}

// ─── Retry ─────────────────────────────────────────────────────────────────

function retryInstallation() {
  installError.value = null
  logLines.value = []
  progress.value = null
  installComplete.value = false
  startInstallation()
}

// ─── Navigation ────────────────────────────────────────────────────────────

function goToDashboard() {
  router.push('/')
}
</script>

<template>
  <!-- Breadcrumb -->
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Docker Setup</span>
  </div>
  <h1 class="text-h1">Docker Setup Wizard</h1>

  <!-- ════════════════════════════════════════════════════════════════════ -->
  <!-- Not Linux Error                                                    -->
  <!-- ════════════════════════════════════════════════════════════════════ -->
  <div v-if="!isLinux && !detecting" class="section" style="text-align:center;padding:48px">
    <div style="font-size:48px;margin-bottom:16px;color:var(--accent-red)">
      <i class="fa-brands fa-linux"></i>
    </div>
    <h2 style="font-size:18px;font-weight:600;margin-bottom:8px;color:var(--accent-red)">
      Linux Required
    </h2>
    <p style="color:var(--text-muted);max-width:420px;margin:0 auto 24px;line-height:1.6">
      The Docker installer is only available on Linux.
      ItzamBox requires a Linux host with apt, dnf, pacman, or zypper package manager.
    </p>
    <button class="btn btn-secondary" @click="router.push('/')">
      <i class="fa-solid fa-arrow-left"></i> Back to Dashboard
    </button>
  </div>

  <!-- ════════════════════════════════════════════════════════════════════ -->
  <!-- Main Wizard                                                        -->
  <!-- ════════════════════════════════════════════════════════════════════ -->
  <div v-else style="display:flex;flex-direction:column;gap:20px;max-width:720px">

    <!-- Step Indicators -->
    <div class="wizard-steps">
      <div
        v-for="step in 4"
        :key="step"
        :class="['wizard-step', {
          active: currentStep === step,
          done: currentStep > step,
        }]"
      >
        <div class="wizard-step-circle">
          <i v-if="currentStep > step" class="fa-solid fa-check"></i>
          <span v-else>{{ step }}</span>
        </div>
        <div class="wizard-step-label">
          {{ ['Welcome', 'Pre-flight', 'Install', 'Validate'][step - 1] }}
        </div>
      </div>
    </div>

    <!-- ════════════════════════════════════════════════════════════════ -->
    <!-- Step 1: Welcome                                                -->
    <!-- ════════════════════════════════════════════════════════════════ -->
    <div v-if="currentStep === 1" class="section welcome-card">
      <div style="padding:32px;text-align:center">
        <!-- Docker + ItzamBox logos -->
        <div style="display:flex;align-items:center;justify-content:center;gap:16px;margin-bottom:20px">
          <div class="welcome-logo docker-logo">
            <i class="fa-brands fa-docker" style="font-size:32px"></i>
          </div>
          <div style="font-size:28px;color:var(--text-muted)">+</div>
          <div class="welcome-logo itzam-logo">
            <span style="font-weight:700;font-size:16px">IB</span>
          </div>
        </div>

        <h2 style="font-size:20px;font-weight:700;margin-bottom:8px;letter-spacing:-0.02em">
          Docker Setup Wizard
        </h2>
        <p style="color:var(--text-muted);font-size:14px;margin-bottom:24px;line-height:1.5">
          This wizard will install Docker Engine on your Linux system.
          It will detect your distribution, install the necessary packages,
          and configure Docker to start on boot.
        </p>

        <!-- Detected Distro -->
        <div v-if="detecting" style="display:flex;align-items:center;justify-content:center;gap:8px;color:var(--text-muted);font-size:14px;margin-bottom:24px">
          <i class="fa-solid fa-spinner fa-spin"></i>
          <span>Detecting distribution...</span>
        </div>
        <div v-else-if="distro" class="distro-badge">
          <i :class="distroIcon" :style="{ color: distroColor, fontSize:'22px' }"></i>
          <div style="text-align:left">
            <div style="font-weight:600;font-size:14px">{{ distroDisplayName }}</div>
            <div style="font-size:11px;color:var(--text-muted);font-family:var(--font-mono)">
              {{ distro.id }} · {{ distro.package_manager }}
            </div>
          </div>
        </div>

        <!-- Error -->
        <div v-if="error" style="color:var(--accent-red);font-size:13px;margin-bottom:16px">
          <i class="fa-solid fa-circle-exclamation"></i> {{ error }}
        </div>

        <div style="display:flex;gap:10px;justify-content:center">
          <button class="btn btn-secondary" @click="router.push('/')">
            <i class="fa-solid fa-times"></i> Cancel
          </button>
          <button
            class="btn btn-primary"
            :disabled="!distro || !distro.supported || detecting"
            @click="checkStatus"
          >
            <i class="fa-solid fa-rocket"></i> Start Installation
          </button>
        </div>

        <!-- Unsupported warning -->
        <div
          v-if="distro && !distro.supported"
          style="margin-top:16px;padding:10px 14px;background:rgba(245,158,11,0.08);border:1px solid rgba(245,158,11,0.2);border-radius:var(--radius-md);font-size:12px;color:var(--accent-yellow);text-align:left"
        >
          <i class="fa-solid fa-triangle-exclamation"></i>
          <strong style="margin-left:6px">Unsupported distribution</strong>
          <p style="margin-top:4px;color:var(--text-muted)">
            {{ distro.name }} is not officially supported. The wizard may attempt installation
            using {{ distro.package_manager }} anyway, but success is not guaranteed.
          </p>
        </div>
      </div>
    </div>

    <!-- ════════════════════════════════════════════════════════════════ -->
    <!-- Step 2: Pre-flight Checks                                      -->
    <!-- ════════════════════════════════════════════════════════════════ -->
    <div v-if="currentStep === 2" class="section">
      <div class="section-header">
        <span class="section-title">
          <i class="fa-solid fa-clipboard-check" style="color:var(--accent-cyan);margin-right:8px"></i>
          Pre-flight Checks
        </span>
      </div>
      <div style="padding:20px">
        <p style="font-size:13px;color:var(--text-muted);margin-bottom:16px">
          Checking current Docker installation status on your system:
        </p>

        <!-- Loading -->
        <div v-if="loading" style="display:flex;align-items:center;gap:10px;color:var(--text-muted);padding:16px">
          <i class="fa-solid fa-spinner fa-spin"></i>
          <span>Checking Docker status...</span>
        </div>

        <!-- Checklist -->
        <div v-else class="checklist">
          <div
            v-for="(check, idx) in statusChecks"
            :key="idx"
            class="checklist-item"
          >
            <div :class="['checklist-icon', check.ok ? 'ok' : 'fail']">
              <i :class="check.ok ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-xmark'"></i>
            </div>
            <div style="flex:1;min-width:0">
              <div style="font-size:13px;font-weight:500">{{ check.label }}</div>
              <div style="font-size:11px;color:var(--text-muted);font-family:var(--font-mono);word-break:break-all">
                {{ check.detail }}
              </div>
            </div>
          </div>
        </div>

        <!-- Already installed -->
        <div
          v-if="everythingInstalled"
          style="margin-top:16px;padding:14px;background:rgba(16,185,129,0.08);border:1px solid rgba(16,185,129,0.2);border-radius:var(--radius-md)"
        >
          <div style="display:flex;align-items:center;gap:10px">
            <div style="font-size:24px;color:var(--accent-green)"><i class="fa-solid fa-check-circle"></i></div>
            <div>
              <div style="font-weight:600;font-size:14px;color:var(--accent-green)">Docker is already set up!</div>
              <div style="font-size:12px;color:var(--text-muted);margin-top:2px">
                Docker Engine is installed, the service is running, and you are in the docker group.
              </div>
            </div>
          </div>
        </div>

        <!-- What will be installed -->
        <div
          v-if="!everythingInstalled && !loading && initialStatus"
          style="margin-top:16px;padding:14px;background:rgba(0,229,255,0.05);border:1px solid rgba(0,229,255,0.12);border-radius:var(--radius-md)"
        >
          <div style="font-weight:600;font-size:13px;margin-bottom:8px;display:flex;align-items:center;gap:6px">
            <i class="fa-solid fa-download" style="color:var(--accent-cyan)"></i>
            Will be installed:
          </div>
          <div style="font-size:12px;color:var(--text-muted);display:flex;flex-direction:column;gap:4px">
            <span v-if="!initialStatus.docker_installed">• Docker Engine (docker-ce, containerd.io)</span>
            <span v-if="!initialStatus.compose_available">• Docker Compose plugin (docker-compose-plugin)</span>
            <span v-if="!initialStatus.service_running">• Docker service enabled on boot</span>
            <span v-if="!initialStatus.user_in_docker_group">• User added to docker group</span>
          </div>
        </div>

        <!-- Actions -->
        <div style="display:flex;gap:10px;margin-top:20px;justify-content:flex-end">
          <button class="btn btn-secondary" @click="currentStep = 1">
            <i class="fa-solid fa-arrow-left"></i> Back
          </button>
          <button
            v-if="!everythingInstalled"
            class="btn btn-primary"
            @click="startInstallation"
            :disabled="loading"
          >
            <i class="fa-solid fa-download"></i> Install Docker
          </button>
          <button
            v-else
            class="btn btn-primary"
            @click="goToDashboard"
          >
            <i class="fa-solid fa-check"></i> Finish
          </button>
        </div>
      </div>
    </div>

    <!-- ════════════════════════════════════════════════════════════════ -->
    <!-- Step 3: Installation Progress                                   -->
    <!-- ════════════════════════════════════════════════════════════════ -->
    <div v-if="currentStep === 3" class="section">
      <div class="section-header">
        <span class="section-title">
          <i class="fa-solid fa-download" style="color:var(--accent-cyan);margin-right:8px"></i>
          Installing Docker Engine
        </span>
      </div>
      <div style="padding:20px">
        <!-- Progress Bar -->
        <div style="margin-bottom:16px">
          <div class="progress-bar">
            <div
              class="progress-bar-fill"
              :style="{ width: progressPercent + '%' }"
              :class="{ error: progress?.status === 'error', complete: progress?.status === 'completed' }"
            ></div>
          </div>
          <div style="display:flex;justify-content:space-between;margin-top:6px;font-size:11px">
            <span :style="{ color: progress?.status === 'error' ? 'var(--accent-red)' : 'var(--text-muted)' }">
              <i v-if="progress?.status === 'in_progress'" class="fa-solid fa-spinner fa-spin"></i>
              <i v-else-if="progress?.status === 'error'" class="fa-solid fa-circle-exclamation"></i>
              <i v-else class="fa-solid fa-check-circle" style="color:var(--accent-green)"></i>
              {{ stepLabel }}
            </span>
            <span style="color:var(--text-disabled);font-family:var(--font-mono)">{{ progressPercent }}%</span>
          </div>
        </div>

        <!-- Live Logs -->
        <div class="install-log">
          <div
            v-for="(line, idx) in logLines"
            :key="idx"
            class="install-log-line"
          >
            {{ line }}
          </div>
          <div
            v-if="progress?.status === 'in_progress'"
            class="install-log-line install-log-pulse"
          >
            ⏳ Working...
          </div>
        </div>

        <!-- Error -->
        <div
          v-if="installError"
          style="margin-top:16px;padding:12px;background:rgba(239,68,68,0.08);border:1px solid rgba(239,68,68,0.2);border-radius:var(--radius-md)"
        >
          <div style="display:flex;align-items:center;gap:8px;font-size:13px;color:var(--accent-red);font-weight:500">
            <i class="fa-solid fa-circle-exclamation"></i>
            Installation Error
          </div>
          <div style="font-size:12px;color:var(--text-muted);margin-top:6px;font-family:var(--font-mono);word-break:break-all">
            {{ installError }}
          </div>
        </div>

        <!-- Actions -->
        <div style="display:flex;gap:10px;margin-top:20px;justify-content:flex-end">
          <button
            v-if="installError && !installComplete"
            class="btn btn-danger"
            @click="retryInstallation"
          >
            <i class="fa-solid fa-rotate"></i> Retry
          </button>
          <button
            v-if="installComplete || progress?.status === 'completed'"
            class="btn btn-primary"
            @click="runValidation"
            :disabled="validating"
          >
            <i class="fa-solid fa-flask"></i>
            {{ validating ? 'Validating...' : 'Validate Installation' }}
          </button>
        </div>
      </div>
    </div>

    <!-- ════════════════════════════════════════════════════════════════ -->
    <!-- Step 4: Complete & Validation                                  -->
    <!-- ════════════════════════════════════════════════════════════════ -->
    <div v-if="currentStep === 4" class="section">
      <div class="section-header">
        <span class="section-title">
          <i class="fa-solid fa-check-circle" style="color:var(--accent-green);margin-right:8px"></i>
          Installation Complete
        </span>
      </div>
      <div style="padding:24px;text-align:center">
        <!-- Success Icon -->
        <div style="font-size:64px;color:var(--accent-green);margin-bottom:16px">
          <i class="fa-solid fa-check-circle"></i>
        </div>

        <h2 style="font-size:18px;font-weight:700;margin-bottom:4px">
          Docker is ready!
        </h2>
        <p style="color:var(--text-muted);font-size:13px;margin-bottom:24px;line-height:1.5">
          Docker Engine has been installed successfully.
          Run <code style="background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:var(--font-mono);font-size:12px">docker run hello-world</code> to verify it works.
        </p>

        <!-- Summary -->
        <div v-if="validationResult" class="install-summary">
          <div class="summary-row">
            <span class="summary-label">Docker Engine</span>
            <span class="summary-value" :class="{ ok: validationResult.docker_installed, fail: !validationResult.docker_installed }">
              <i :class="validationResult.docker_installed ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-xmark'"></i>
              {{ validationResult.docker_version ?? 'Not detected' }}
            </span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Docker Compose</span>
            <span class="summary-value" :class="{ ok: validationResult.compose_available, fail: !validationResult.compose_available }">
              <i :class="validationResult.compose_available ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-xmark'"></i>
              {{ validationResult.compose_version ?? 'Not available' }}
            </span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Service Status</span>
            <span class="summary-value" :class="{ ok: validationResult.service_running, fail: !validationResult.service_running }">
              <i :class="validationResult.service_running ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-xmark'"></i>
              {{ validationResult.service_running ? 'Active' : 'Not running' }}
            </span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Docker Group</span>
            <span class="summary-value" :class="{ ok: validationResult.user_in_docker_group, fail: !validationResult.user_in_docker_group }">
              <i :class="validationResult.user_in_docker_group ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-xmark'"></i>
              {{ validationResult.user_in_docker_group ? 'User in group' : 'Not in group' }}
            </span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Docker Socket</span>
            <span class="summary-value" :class="{ ok: validationResult.socket_exists, fail: !validationResult.socket_exists }">
              <i :class="validationResult.socket_exists ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-xmark'"></i>
              {{ validationResult.socket_exists ? 'Accessible' : 'Missing' }}
            </span>
          </div>
        </div>

        <!-- Validation loading -->
        <div v-else-if="validating" style="padding:16px;color:var(--text-muted)">
          <i class="fa-solid fa-spinner fa-spin"></i>
          <span style="margin-left:8px">Running hello-world validation...</span>
        </div>

        <!-- Validation error -->
        <div
          v-if="error && currentStep === 4"
          style="margin-top:16px;padding:12px;background:rgba(239,68,68,0.08);border:1px solid rgba(239,68,68,0.2);border-radius:var(--radius-md);text-align:left"
        >
          <div style="display:flex;align-items:center;gap:8px;font-size:13px;color:var(--accent-red);font-weight:500">
            <i class="fa-solid fa-circle-exclamation"></i>
            Validation Warning
          </div>
          <div style="font-size:12px;color:var(--text-muted);margin-top:6px">
            Docker validation encountered an issue. Try running <code style="background:var(--bg-tertiary);padding:2px 4px;border-radius:4px;font-family:var(--font-mono);font-size:11px">docker run hello-world</code> manually in a terminal.
          </div>
        </div>

        <!-- Warning about group changes -->
        <div
          style="margin-top:20px;padding:12px 16px;background:rgba(245,158,11,0.08);border:1px solid rgba(245,158,11,0.2);border-radius:var(--radius-md);text-align:left"
        >
          <div style="display:flex;align-items:center;gap:8px;font-size:12px;color:var(--accent-yellow);font-weight:500">
            <i class="fa-solid fa-triangle-exclamation"></i>
            Note
          </div>
          <div style="font-size:12px;color:var(--text-muted);margin-top:4px;line-height:1.5">
            You may need to <strong>log out and back in</strong> for docker group changes to take effect.
            Until then, you may need to use <code style="background:var(--bg-tertiary);padding:2px 4px;border-radius:4px;font-family:var(--font-mono);font-size:11px">sudo docker</code> to run Docker commands.
          </div>
        </div>

        <!-- Actions -->
        <div style="display:flex;gap:10px;margin-top:24px;justify-content:center">
          <button class="btn btn-primary" @click="goToDashboard">
            <i class="fa-solid fa-gauge-high"></i> Go to Dashboard
          </button>
          <button class="btn btn-secondary" @click="currentStep = 3">
            <i class="fa-solid fa-scroll"></i> View Install Log
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ─── Wizard Steps Bar ─── */
.wizard-steps {
  display: flex;
  align-items: center;
  gap: 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 16px 20px;
}

.wizard-step {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  position: relative;
}

.wizard-step::after {
  content: '';
  flex: 1;
  height: 2px;
  background: var(--border-color);
  margin: 0 12px;
  transition: background var(--transition-normal);
}

.wizard-step:last-child::after {
  display: none;
}

.wizard-step.done::after {
  background: var(--accent-green);
}

.wizard-step.active::after {
  background: var(--accent-cyan);
}

.wizard-step-circle {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  flex-shrink: 0;
  transition: all var(--transition-normal);
  background: var(--bg-tertiary);
  color: var(--text-disabled);
  border: 2px solid var(--border-color);
}

.wizard-step.done .wizard-step-circle {
  background: var(--accent-green);
  border-color: var(--accent-green);
  color: #fff;
}

.wizard-step.active .wizard-step-circle {
  background: rgba(0, 229, 255, 0.12);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.25);
}

.wizard-step-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-disabled);
  white-space: nowrap;
  transition: color var(--transition-normal);
}

.wizard-step.done .wizard-step-label {
  color: var(--accent-green);
}

.wizard-step.active .wizard-step-label {
  color: var(--accent-cyan);
}

/* ─── Welcome Card ─── */
.welcome-card {
  animation: fadeIn 0.3s ease-out;
}

.welcome-logo {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.docker-logo {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 229, 255, 0.2);
}

.itzam-logo {
  background: linear-gradient(135deg, rgba(0, 229, 255, 0.15), rgba(168, 85, 247, 0.15));
  color: var(--text-main);
  border: 1px solid var(--border-color);
}

/* ─── Distro Badge ─── */
.distro-badge {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  margin-bottom: 24px;
}

/* ─── Checklist ─── */
.checklist {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.checklist-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.checklist-icon {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 14px;
}

.checklist-icon.ok {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
}

.checklist-icon.fail {
  background: rgba(239, 68, 68, 0.1);
  color: var(--accent-red);
}

/* ─── Progress Bar ─── */
.progress-bar {
  width: 100%;
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  border-radius: 3px;
  background: linear-gradient(90deg, var(--accent-cyan), #00b8d4);
  transition: width 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-bar-fill.error {
  background: linear-gradient(90deg, var(--accent-red), #ef4444);
}

.progress-bar-fill.complete {
  background: linear-gradient(90deg, var(--accent-green), #10b981);
}

/* ─── Install Log ─── */
.install-log {
  background: #000000;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  max-height: 240px;
  overflow-y: auto;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
}

.install-log-line {
  color: var(--text-muted);
  white-space: pre-wrap;
  word-break: break-all;
}

.install-log-pulse {
  animation: pulse-opacity 1.5s infinite;
  color: var(--accent-cyan);
}

@keyframes pulse-opacity {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

/* ─── Summary Table ─── */
.install-summary {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
  text-align: left;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 13px;
}

.summary-label {
  color: var(--text-muted);
  font-weight: 500;
}

.summary-value {
  font-family: var(--font-mono);
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.summary-value.ok {
  color: var(--accent-green);
}

.summary-value.fail {
  color: var(--accent-red);
}

/* ─── Responsive ─── */
@media (max-width: 640px) {
  .wizard-steps {
    gap: 4px;
    padding: 12px;
  }
  .wizard-step-label {
    display: none;
  }
  .wizard-step::after {
    margin: 0 6px;
  }
}
</style>
