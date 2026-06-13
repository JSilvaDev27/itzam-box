<!-- ItzamBox — Run Container Wizard (10 Steps)
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { useDocker, type DockerHubImage, type PortMapping } from '../composables/useDocker'
import { useNotifications } from '../composables/useNotifications'

const router = useRouter()
const { searchDockerHub, pullImage, createAndRunContainer, fetchNetworks } = useDocker()
const { success, error: notifyError, info } = useNotifications()

// ─── Wizard State ─────────────────────────────────────────────────────────
const currentStep = ref(1)
const creating = ref(false)
const createdContainerId = ref<string | null>(null)

interface VolumeMount {
  hostPath: string
  containerPath: string
  readOnly: boolean
}

interface EnvVar {
  key: string
  value: string
}

const wizard = reactive({
  // Step 1: Image Selection
  searchQuery: '',
  searchResults: [] as DockerHubImage[],
  searching: false,
  searchError: '',
  selectedImage: '',
  manualImageName: '',
  pulling: false,
  pullError: '',

  // Step 2: Container Name
  containerName: '',
  autoGenerateName: true,

  // Step 3: Port Mapping
  ports: [] as Array<PortMapping>,

  // Step 4: Volume Mounts
  volumes: [] as Array<VolumeMount>,

  // Step 5: Environment Variables
  envVars: [] as Array<EnvVar>,

  // Step 6: Network
  network: 'bridge',
  customNetwork: '',

  // Step 7: Restart Policy
  restartPolicy: 'no',
  restartMaxRetries: 3,

  // Step 8: Command & Entrypoint
  command: '',
  interactive: false,

  // Step 9: Resource Limits & Advanced
  cpuLimit: 0,
  memoryLimit: '',
  memoryUnit: 'MB',
  privileged: false,

  // Step 10: Review
  dockerCommandOutput: '',
})

const networks = ref<Array<{ id: string; name: string; driver: string }>>([])

onMounted(async () => {
  try {
    const nets = await fetchNetworks()
    networks.value = nets.map((n: any) => ({ id: n.id, name: n.name, driver: n.driver }))
  } catch {
    // Networks may not load — that's OK
  }
})

// ─── Step Definitions ─────────────────────────────────────────────────────
const steps = [
  { number: 1, label: 'Image Selection', icon: 'fa-solid fa-image' },
  { number: 2, label: 'Container Name', icon: 'fa-solid fa-tag' },
  { number: 3, label: 'Port Mapping', icon: 'fa-solid fa-plug' },
  { number: 4, label: 'Volume Mounts', icon: 'fa-solid fa-hard-drive' },
  { number: 5, label: 'Environment Variables', icon: 'fa-solid fa-list' },
  { number: 6, label: 'Network', icon: 'fa-solid fa-network-wired' },
  { number: 7, label: 'Restart Policy', icon: 'fa-solid fa-rotate' },
  { number: 8, label: 'Command', icon: 'fa-solid fa-terminal' },
  { number: 9, label: 'Advanced', icon: 'fa-solid fa-sliders' },
  { number: 10, label: 'Review & Create', icon: 'fa-solid fa-check' },
]

const completedSteps = computed(() => {
  const completed: number[] = []
  if (wizard.selectedImage || wizard.manualImageName) completed.push(1)
  if (wizard.autoGenerateName || wizard.containerName.trim()) completed.push(2)
  if (wizard.network) completed.push(6)
  if (wizard.restartPolicy) completed.push(7)
  return completed
})

function isStepCompleted(step: number): boolean {
  if (step === 1) return !!getFinalImage()
  if (step === 2) return wizard.autoGenerateName || wizard.containerName.trim().length > 0
  if (step === 6) return true
  if (step === 7) return true
  if (step === 10) return false
  return false
}

// ─── Image helpers ────────────────────────────────────────────────────────
let searchTimeout: ReturnType<typeof setTimeout> | null = null

watch(() => wizard.searchQuery, (val) => {
  if (searchTimeout) clearTimeout(searchTimeout)
  if (val.trim().length < 2) {
    wizard.searchResults = []
    wizard.searchError = ''
    return
  }
  searchTimeout = setTimeout(() => performSearch(), 300)
})

async function performSearch() {
  wizard.searching = true
  wizard.searchError = ''
  try {
    wizard.searchResults = await searchDockerHub(wizard.searchQuery, 20)
  } catch (e: any) {
    wizard.searchError = 'Search failed. Check your internet connection.'
  }
  wizard.searching = false
}

function selectImage(image: DockerHubImage) {
  wizard.selectedImage = image.name
  wizard.manualImageName = image.name
  wizard.searchResults = []
  wizard.searchQuery = ''
}

function clearSelectedImage() {
  wizard.selectedImage = ''
  wizard.manualImageName = ''
}

async function handlePullImage() {
  const imageName = wizard.manualImageName.trim()
  if (!imageName) return
  wizard.pulling = true
  wizard.pullError = ''
  try {
    await pullImage(imageName)
    success('Image pulled', `${imageName} pulled successfully.`)
    wizard.selectedImage = imageName
  } catch (e: any) {
    wizard.pullError = `Failed to pull ${imageName}: ${e.toString()}`
    notifyError('Pull failed', e.toString())
  }
  wizard.pulling = false
}

function getFinalImage(): string {
  return wizard.manualImageName.trim() || wizard.selectedImage
}

// ─── Port helpers ─────────────────────────────────────────────────────────
function addPort() {
  wizard.ports.push({ host_ip: '0.0.0.0', host_port: 0, container_port: 0, protocol: 'tcp' })
}

function removePort(index: number) {
  wizard.ports.splice(index, 1)
}

// ─── Volume helpers ───────────────────────────────────────────────────────
async function browseHostPath(index: number) {
  try {
    const selected = await open({ directory: true, multiple: false, title: 'Select Host Directory' })
    if (selected && typeof selected === 'string') {
      wizard.volumes[index].hostPath = selected
    }
  } catch {
    // Dialog cancelled
  }
}

function addVolume() {
  wizard.volumes.push({ hostPath: '', containerPath: '', readOnly: false })
}

function removeVolume(index: number) {
  wizard.volumes.splice(index, 1)
}

// ─── Env var helpers ──────────────────────────────────────────────────────
function addEnvVar() {
  wizard.envVars.push({ key: '', value: '' })
}

function removeEnvVar(index: number) {
  wizard.envVars.splice(index, 1)
}

// ─── Network helpers ──────────────────────────────────────────────────────
const networkOptions = [
  { value: 'bridge', label: 'Default bridge', desc: 'Default Docker bridge network. Containers can communicate via IP.' },
  { value: 'host', label: 'Host', desc: 'Use the host network stack. No port mapping needed.' },
  { value: 'none', label: 'None', desc: 'No networking. Only loopback interface available.' },
]

const effectiveNetwork = computed(() => {
  if (wizard.network === 'custom' && wizard.customNetwork) return wizard.customNetwork
  return wizard.network
})

// ─── Restart helpers ──────────────────────────────────────────────────────
const restartOptions = [
  { value: 'no', label: 'No', desc: 'Do not automatically restart the container.' },
  { value: 'always', label: 'Always', desc: 'Always restart the container if it stops.' },
  { value: 'on-failure', label: 'On-failure', desc: 'Restart only if the container exits with a non-zero exit code.' },
  { value: 'unless-stopped', label: 'Unless-stopped', desc: 'Always restart unless explicitly stopped.' },
]

// ─── Validation ───────────────────────────────────────────────────────────
const stepErrors = computed(() => {
  const errors: Record<number, string> = {}

  // Step 1: Image required
  if (currentStep.value === 1 && !getFinalImage()) {
    errors[1] = 'Please select or enter an image name.'
  }

  // Step 2: Name validation
  if (currentStep.value === 2 && !wizard.autoGenerateName) {
    const name = wizard.containerName.trim()
    if (name) {
      if (!/^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/.test(name)) {
        errors[2] = 'Container name may contain only letters, numbers, underscores, periods, and hyphens.'
      }
    }
  }

  // Step 3: Port validation
  if (currentStep.value === 3) {
    for (let i = 0; i < wizard.ports.length; i++) {
      const p = wizard.ports[i]
      if (!p.host_port || !p.container_port) {
        errors[3] = `Port #${i + 1}: Host and container ports are required.`
        break
      }
      if (p.host_port < 1 || p.host_port > 65535 || p.container_port < 1 || p.container_port > 65535) {
        errors[3] = `Port #${i + 1}: Ports must be between 1 and 65535.`
        break
      }
    }
  }

  // Step 4: Volume validation
  if (currentStep.value === 4) {
    for (let i = 0; i < wizard.volumes.length; i++) {
      const v = wizard.volumes[i]
      if (!v.hostPath || !v.containerPath) {
        errors[4] = `Volume #${i + 1}: Both host path and container path are required.`
        break
      }
    }
  }

  return errors
})

const currentError = computed(() => stepErrors.value[currentStep.value] || '')

// ─── Navigation ───────────────────────────────────────────────────────────
function canGoNext(): boolean {
  // Step 1 requires an image
  if (currentStep.value === 1 && !getFinalImage()) return false
  // Step 2 validates name pattern
  if (currentStep.value === 2 && !wizard.autoGenerateName) {
    const name = wizard.containerName.trim()
    if (name && !/^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/.test(name)) return false
  }
  return true
}

function nextStep() {
  if (currentStep.value < 10) {
    currentStep.value++
  }
}

function prevStep() {
  if (currentStep.value > 1) {
    currentStep.value--
  }
}

function cancel() {
  router.push('/')
}

// ─── Docker Run Command Generator (Step 10) ───────────────────────────────
const dockerCommand = computed(() => {
  const parts: string[] = ['docker run']

  // Detach
  parts.push('-d')

  // Name
  if (!wizard.autoGenerateName && wizard.containerName.trim()) {
    parts.push('--name')
    parts.push(wizard.containerName.trim())
  }

  // Privileged
  if (wizard.privileged) {
    parts.push('--privileged')
  }

  // CPU
  if (wizard.cpuLimit > 0) {
    parts.push('--cpus')
    parts.push(wizard.cpuLimit.toString())
  }

  // Memory
  if (wizard.memoryLimit) {
    parts.push('--memory')
    parts.push(wizard.memoryLimit + wizard.memoryUnit.toLowerCase())
  }

  // Ports
  for (const p of wizard.ports) {
    if (p.host_port && p.container_port) {
      parts.push('-p')
      parts.push(`${p.host_ip}:${p.host_port}:${p.container_port}/${p.protocol}`)
    }
  }

  // Volumes
  for (const v of wizard.volumes) {
    if (v.hostPath && v.containerPath) {
      const mode = v.readOnly ? ':ro' : ':rw'
      parts.push('-v')
      parts.push(`${v.hostPath}:${v.containerPath}${mode}`)
    }
  }

  // Env vars
  for (const e of wizard.envVars) {
    if (e.key) {
      parts.push('-e')
      parts.push(`${e.key}=${e.value}`)
    }
  }

  // Network
  if (wizard.network === 'custom' && wizard.customNetwork) {
    parts.push('--network')
    parts.push(wizard.customNetwork)
  } else if (wizard.network !== 'bridge') {
    parts.push('--network')
    parts.push(wizard.network)
  }

  // Restart
  if (wizard.restartPolicy !== 'no') {
    let rp = wizard.restartPolicy
    if (rp === 'on-failure' && wizard.restartMaxRetries > 0) {
      rp = `on-failure:${wizard.restartMaxRetries}`
    }
    parts.push('--restart')
    parts.push(rp)
  }

  // Interactive
  if (wizard.interactive) {
    parts.push('-it')
  }

  // Image
  parts.push(getFinalImage() || '<image>')

  // Command
  if (wizard.command.trim()) {
    parts.push(wizard.command.trim())
  }

  return parts.join(' ')
})

// ─── Create Container ─────────────────────────────────────────────────────
async function handleCreateContainer() {
  creating.value = true
  try {
    const volumes = wizard.volumes
      .filter(v => v.hostPath && v.containerPath)
      .map(v => `${v.hostPath}:${v.containerPath}${v.readOnly ? ':ro' : ':rw'}`)

    const envVars = wizard.envVars
      .filter(e => e.key)
      .map(e => `${e.key}=${e.value}`)

    const commandArr = wizard.command.trim()
      ? wizard.command.trim().split(/\s+/)
      : null

    const id = await createAndRunContainer({
      image: getFinalImage(),
      name: wizard.autoGenerateName ? null : wizard.containerName.trim() || null,
      ports: wizard.ports.filter(p => p.host_port > 0),
      volumes,
      env_vars: envVars,
      network: effectiveNetwork.value !== 'bridge' ? effectiveNetwork.value : null,
      restart_policy: wizard.restartPolicy !== 'no' ? wizard.restartPolicy : null,
      command: commandArr,
      detach: true,
      cpu_limit: wizard.cpuLimit > 0 ? wizard.cpuLimit : null,
      memory_limit: wizard.memoryLimit ? wizard.memoryLimit + wizard.memoryUnit.toLowerCase() : null,
      privileged: wizard.privileged,
    })

    createdContainerId.value = id
    success('Container created', `Container ${id.substring(0, 12)} created successfully.`)

    // Navigate to container detail after a short delay
    setTimeout(() => {
      router.push(`/containers/${id}`)
    }, 1500)
  } catch (e: any) {
    notifyError('Failed to create container', e.toString())
  }
  creating.value = false
}

// ─── Copy to clipboard ────────────────────────────────────────────────────
function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text).then(() => {
    info('Copied!', 'Docker command copied to clipboard.')
  }).catch(() => {
    // Fallback
    const textarea = document.createElement('textarea')
    textarea.value = text
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
    info('Copied!', 'Docker command copied to clipboard.')
  })
}

// ─── Format helpers ───────────────────────────────────────────────────────
function formatPulls(n: number): string {
  if (n >= 1e9) return (n / 1e9).toFixed(1) + 'B'
  if (n >= 1e6) return (n / 1e6).toFixed(1) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return n.toString()
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span>Run Wizard</span>
  </div>

  <div class="wizard-layout">
    <!-- Sidebar with Steps -->
    <aside class="wizard-sidebar">
      <div class="wizard-sidebar-header">
        <i class="fa-solid fa-wand-magic-sparkles"></i>
        <span>Run Wizard</span>
      </div>
      <nav class="wizard-steps">
        <button
          v-for="step in steps"
          :key="step.number"
          :class="[
            'wizard-step',
            { active: currentStep === step.number },
            { completed: isStepCompleted(step.number) && currentStep > step.number }
          ]"
          @click="currentStep = step.number"
        >
          <span class="step-number">
            <i v-if="isStepCompleted(step.number) && currentStep > step.number" class="fa-solid fa-check"></i>
            <span v-else>{{ step.number }}</span>
          </span>
          <span class="step-info">
            <span class="step-label">{{ step.label }}</span>
          </span>
        </button>
      </nav>
    </aside>

    <!-- Main Content Area -->
    <div class="wizard-content">
      <!-- Step 1: Image Selection -->
      <div v-if="currentStep === 1" class="step-panel">
        <h2 class="step-title">Select Image</h2>
        <p class="step-description">Search Docker Hub or enter an image name manually.</p>

        <div class="search-bar-wizard">
          <i class="fa-solid fa-magnifying-glass"></i>
          <input
            v-model="wizard.searchQuery"
            type="text"
            class="form-input"
            placeholder="Search Docker Hub (e.g., nginx, postgres, node)..."
          />
          <span v-if="wizard.searching" class="search-spinner"><i class="fa-solid fa-spinner fa-spin"></i></span>
        </div>

        <div v-if="wizard.searchError" class="error-inline">
          <i class="fa-solid fa-triangle-exclamation"></i> {{ wizard.searchError }}
        </div>

        <!-- Search Results -->
        <div v-if="wizard.searchResults.length > 0" class="search-results">
          <div
            v-for="img in wizard.searchResults"
            :key="img.name"
            class="search-result-item"
            :class="{ selected: wizard.selectedImage === img.name }"
            @click="selectImage(img)"
          >
            <div class="result-info">
              <div class="result-name">
                {{ img.name }}
                <span v-if="img.is_official" class="tag" style="background:rgba(0,229,255,0.1);color:var(--accent-cyan);border:1px solid rgba(0,229,255,0.2);margin-left:8px">Official</span>
              </div>
              <div class="result-desc">{{ img.description || 'No description' }}</div>
            </div>
            <div class="result-meta">
              <span class="result-stat" title="Stars"><i class="fa-solid fa-star" style="color:var(--accent-yellow)"></i> {{ img.star_count.toLocaleString() }}</span>
              <span class="result-stat" title="Pulls"><i class="fa-solid fa-download" style="color:var(--text-muted)"></i> {{ formatPulls(img.pull_count) }}</span>
            </div>
            <button class="btn btn-primary btn-sm" @click.stop="selectImage(img)">Select</button>
          </div>
        </div>

        <div class="form-divider"><span>Or enter manually</span></div>

        <div class="manual-image">
          <div class="form-group" style="flex:1">
            <label class="form-label">Image name (e.g., nginx:latest)</label>
            <input
              v-model="wizard.manualImageName"
              class="form-input mono"
              placeholder="nginx:latest"
              @keyup.enter="handlePullImage"
            />
          </div>
          <button
            class="btn btn-primary"
            @click="handlePullImage"
            :disabled="wizard.pulling || !wizard.manualImageName.trim()"
            style="align-self:flex-end"
          >
            <i class="fa-solid fa-cloud-arrow-down" :class="{ 'fa-spin': wizard.pulling }"></i>
            {{ wizard.pulling ? 'Pulling...' : 'Pull' }}
          </button>
        </div>

        <div v-if="wizard.pullError" class="error-inline">
          <i class="fa-solid fa-circle-xmark"></i> {{ wizard.pullError }}
        </div>

        <div v-if="wizard.selectedImage" class="selected-badge">
          <i class="fa-solid fa-check-circle" style="color:var(--accent-green)"></i>
          Selected: <strong>{{ wizard.selectedImage }}</strong>
          <button class="btn btn-ghost btn-sm" @click="clearSelectedImage">Change</button>
        </div>
      </div>

      <!-- Step 2: Container Name -->
      <div v-if="currentStep === 2" class="step-panel">
        <h2 class="step-title">Container Name</h2>
        <p class="step-description">Give your container a name or auto-generate one.</p>

        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="wizard.autoGenerateName" />
            <span>Auto-generate name</span>
          </label>
        </div>

        <div v-if="!wizard.autoGenerateName" class="form-group">
          <label class="form-label">Container name</label>
          <input
            v-model="wizard.containerName"
            class="form-input mono"
            placeholder="my-nginx-container"
            :class="{ 'input-error': stepErrors[2] }"
            @keyup.enter="nextStep"
          />
          <p class="form-hint">Only letters, numbers, underscores, periods, and hyphens allowed.</p>
          <p v-if="stepErrors[2]" class="error-inline"><i class="fa-solid fa-circle-exclamation"></i> {{ stepErrors[2] }}</p>
        </div>
      </div>

      <!-- Step 3: Port Mapping -->
      <div v-if="currentStep === 3" class="step-panel">
        <h2 class="step-title">Port Mapping</h2>
        <p class="step-description">Map host ports to container ports.</p>

        <div v-if="stepErrors[3]" class="error-inline" style="margin-bottom:12px">
          <i class="fa-solid fa-circle-exclamation"></i> {{ stepErrors[3] }}
        </div>

        <div class="port-list">
          <div v-for="(port, idx) in wizard.ports" :key="idx" class="port-row">
            <div class="port-field">
              <label class="form-label">Host IP</label>
              <input v-model="port.host_ip" class="form-input mono" placeholder="0.0.0.0" />
            </div>
            <div class="port-field">
              <label class="form-label">Host Port</label>
              <input v-model.number="port.host_port" type="number" class="form-input mono" placeholder="8080" min="1" max="65535" />
            </div>
            <div class="port-field">
              <label class="form-label">Container Port</label>
              <input v-model.number="port.container_port" type="number" class="form-input mono" placeholder="80" min="1" max="65535" />
            </div>
            <div class="port-field" style="flex:0.6">
              <label class="form-label">Protocol</label>
              <select v-model="port.protocol" class="form-input">
                <option value="tcp">TCP</option>
                <option value="udp">UDP</option>
              </select>
            </div>
            <button class="btn btn-danger btn-sm" @click="removePort(idx)" style="align-self:flex-end;margin-bottom:2px" title="Remove port">
              <i class="fa-solid fa-trash-can"></i>
            </button>
          </div>
        </div>

        <div class="step-actions">
          <button class="btn btn-secondary" @click="addPort">
            <i class="fa-solid fa-plus"></i> Add Port
          </button>
          <button class="btn btn-ghost btn-sm" disabled title="Not implemented yet">
            <i class="fa-solid fa-wand-magic-sparkles"></i> Detect from Image
          </button>
        </div>
      </div>

      <!-- Step 4: Volume Mounts -->
      <div v-if="currentStep === 4" class="step-panel">
        <h2 class="step-title">Volume Mounts</h2>
        <p class="step-description">Mount host directories into the container.</p>

        <div v-if="stepErrors[4]" class="error-inline" style="margin-bottom:12px">
          <i class="fa-solid fa-circle-exclamation"></i> {{ stepErrors[4] }}
        </div>

        <div class="volume-list">
          <div v-for="(vol, idx) in wizard.volumes" :key="idx" class="volume-row">
            <div class="volume-field">
              <label class="form-label">Host Path</label>
              <div class="input-with-browse">
                <input v-model="vol.hostPath" class="form-input mono" placeholder="/host/data" />
                <button class="btn btn-secondary btn-sm" @click="browseHostPath(idx)">
                  <i class="fa-solid fa-folder-open"></i>
                </button>
              </div>
            </div>
            <div class="volume-field">
              <label class="form-label">Container Path</label>
              <input v-model="vol.containerPath" class="form-input mono" placeholder="/container/data" />
            </div>
            <div class="volume-field" style="flex:0.5">
              <label class="form-label">Mode</label>
              <select v-model="vol.readOnly" class="form-input">
                <option :value="false">Read-Write</option>
                <option :value="true">Read-Only</option>
              </select>
            </div>
            <button class="btn btn-danger btn-sm" @click="removeVolume(idx)" style="align-self:flex-end;margin-bottom:2px" title="Remove volume">
              <i class="fa-solid fa-trash-can"></i>
            </button>
          </div>
        </div>

        <button class="btn btn-secondary" @click="addVolume">
          <i class="fa-solid fa-plus"></i> Add Volume
        </button>
      </div>

      <!-- Step 5: Environment Variables -->
      <div v-if="currentStep === 5" class="step-panel">
        <h2 class="step-title">Environment Variables</h2>
        <p class="step-description">Set environment variables for the container.</p>

        <div class="env-list">
          <div v-for="(env, idx) in wizard.envVars" :key="idx" class="env-row">
            <div class="env-field">
              <label class="form-label">KEY</label>
              <input v-model="env.key" class="form-input mono" placeholder="MY_VARIABLE" style="text-transform:uppercase" />
            </div>
            <div class="env-field">
              <label class="form-label">VALUE</label>
              <input v-model="env.value" class="form-input mono" placeholder="my-value" />
            </div>
            <button class="btn btn-danger btn-sm" @click="removeEnvVar(idx)" style="align-self:flex-end;margin-bottom:2px" title="Remove variable">
              <i class="fa-solid fa-trash-can"></i>
            </button>
          </div>
        </div>

        <div class="step-actions">
          <button class="btn btn-secondary" @click="addEnvVar">
            <i class="fa-solid fa-plus"></i> Add Variable
          </button>
          <button class="btn btn-ghost btn-sm" disabled title="Not implemented yet">
            <i class="fa-solid fa-file-import"></i> Load from .env
          </button>
        </div>
      </div>

      <!-- Step 6: Network -->
      <div v-if="currentStep === 6" class="step-panel">
        <h2 class="step-title">Network</h2>
        <p class="step-description">Configure the container's network mode.</p>

        <div class="network-options">
          <label
            v-for="opt in networkOptions"
            :key="opt.value"
            :class="['network-option', { selected: wizard.network === opt.value }]"
          >
            <input type="radio" :value="opt.value" v-model="wizard.network" />
            <div class="network-option-content">
              <strong>{{ opt.label }}</strong>
              <span class="text-muted" style="font-size:12px">{{ opt.desc }}</span>
            </div>
            <span v-if="wizard.network === opt.value" class="network-check">
              <i class="fa-solid fa-check-circle" style="color:var(--accent-cyan)"></i>
            </span>
          </label>
        </div>

        <div v-if="networks.length > 0" class="form-group" style="margin-top:16px">
          <label class="form-label">Custom network</label>
          <select v-model="wizard.network" class="form-input">
            <option value="bridge">— Select a network —</option>
            <option v-for="n in networks" :key="n.id" :value="n.name">
              {{ n.name }} ({{ n.driver }})
            </option>
          </select>
        </div>

        <div v-if="wizard.network === 'custom'" class="form-group">
          <label class="form-label">Custom network name</label>
          <input v-model="wizard.customNetwork" class="form-input mono" placeholder="my-network" />
        </div>
      </div>

      <!-- Step 7: Restart Policy -->
      <div v-if="currentStep === 7" class="step-panel">
        <h2 class="step-title">Restart Policy</h2>
        <p class="step-description">Configure when to restart the container automatically.</p>

        <div class="restart-options">
          <label
            v-for="opt in restartOptions"
            :key="opt.value"
            :class="['restart-option', { selected: wizard.restartPolicy === opt.value }]"
          >
            <input type="radio" :value="opt.value" v-model="wizard.restartPolicy" />
            <div class="restart-option-content">
              <strong>{{ opt.label }}</strong>
              <span class="text-muted" style="font-size:12px">{{ opt.desc }}</span>
            </div>
            <span v-if="wizard.restartPolicy === opt.value" class="restart-check">
              <i class="fa-solid fa-check-circle" style="color:var(--accent-cyan)"></i>
            </span>
          </label>
        </div>

        <div v-if="wizard.restartPolicy === 'on-failure'" class="form-group" style="margin-top:16px">
          <label class="form-label">Max retries</label>
          <input v-model.number="wizard.restartMaxRetries" type="number" class="form-input" min="0" max="100" style="width:120px" />
        </div>
      </div>

      <!-- Step 8: Command -->
      <div v-if="currentStep === 8" class="step-panel">
        <h2 class="step-title">Command & Entrypoint</h2>
        <p class="step-description">Override the default command for the container.</p>

        <div class="form-group">
          <label class="form-label">Command (optional)</label>
          <input v-model="wizard.command" class="form-input mono" placeholder="nginx -g 'daemon off;'" @keyup.enter="nextStep" />
          <p class="form-hint">Override the default CMD. Separate args with spaces. E.g., <code>nginx -g 'daemon off;'</code></p>
        </div>

        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="wizard.interactive" />
            <span>Interactive mode (<code>-it</code>)</span>
          </label>
          <p class="form-hint" style="margin-left:24px">Keep STDIN open and allocate a pseudo-TTY. Useful for debugging.</p>
        </div>
      </div>

      <!-- Step 9: Resource Limits & Advanced -->
      <div v-if="currentStep === 9" class="step-panel">
        <h2 class="step-title">Resource Limits & Advanced</h2>
        <p class="step-description">Configure CPU and memory constraints for the container.</p>

        <div class="form-group">
          <label class="form-label">CPU limit (cores)</label>
          <div class="cpu-slider-row">
            <input v-model.number="wizard.cpuLimit" type="range" min="0" max="8" step="0.5" class="cpu-slider" />
            <span class="cpu-value">{{ wizard.cpuLimit > 0 ? wizard.cpuLimit.toFixed(1) : 'No limit' }}</span>
          </div>
          <p class="form-hint">Set to 0 for unlimited. Values: 0.5, 1, 2, etc.</p>
        </div>

        <div class="form-group">
          <label class="form-label">Memory limit</label>
          <div class="memory-input-row">
            <input v-model="wizard.memoryLimit" type="number" class="form-input mono" placeholder="512" min="0" style="flex:1" />
            <select v-model="wizard.memoryUnit" class="form-input" style="flex:0.4">
              <option value="MB">MB</option>
              <option value="GB">GB</option>
            </select>
          </div>
          <p class="form-hint">Leave empty for unlimited. Example: 512 MB or 2 GB.</p>
        </div>

        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="wizard.privileged" />
            <span>Privileged mode (<code>--privileged</code>)</span>
          </label>
          <p class="form-hint" style="margin-left:24px">Give extended privileges to the container. Use with caution.</p>
        </div>
      </div>

      <!-- Step 10: Review & Create -->
      <div v-if="currentStep === 10" class="step-panel step-review">
        <h2 class="step-title">Review & Create</h2>
        <p class="step-description">Review your configuration and create the container.</p>

        <div class="review-grid">
          <div class="review-item">
            <span class="review-label">Image</span>
            <span class="review-value mono">{{ getFinalImage() }}</span>
          </div>
          <div class="review-item">
            <span class="review-label">Name</span>
            <span class="review-value mono">{{ wizard.autoGenerateName ? '(auto-generated)' : wizard.containerName || '(none)' }}</span>
          </div>
          <div class="review-item" v-if="wizard.ports.length > 0">
            <span class="review-label">Ports</span>
            <span class="review-value mono">
              <span v-for="(p, i) in wizard.ports" :key="i">
                {{ p.host_ip }}:{{ p.host_port }}→{{ p.container_port }}/{{ p.protocol }}{{ i < wizard.ports.length - 1 ? ', ' : '' }}
              </span>
            </span>
          </div>
          <div class="review-item" v-if="wizard.volumes.length > 0">
            <span class="review-label">Volumes</span>
            <span class="review-value mono">
              <span v-for="(v, i) in wizard.volumes" :key="i">
                {{ v.hostPath }}:{{ v.containerPath }}({{ v.readOnly ? 'ro' : 'rw' }}){{ i < wizard.volumes.length - 1 ? ', ' : '' }}
              </span>
            </span>
          </div>
          <div class="review-item" v-if="wizard.envVars.length > 0">
            <span class="review-label">Env Vars</span>
            <span class="review-value mono">
              <span v-for="(e, i) in wizard.envVars" :key="i">
                {{ e.key }}={{ e.value }}{{ i < wizard.envVars.length - 1 ? ', ' : '' }}
              </span>
            </span>
          </div>
          <div class="review-item">
            <span class="review-label">Network</span>
            <span class="review-value mono">{{ effectiveNetwork }}</span>
          </div>
          <div class="review-item">
            <span class="review-label">Restart</span>
            <span class="review-value mono">{{ wizard.restartPolicy === 'no' ? 'None' : wizard.restartPolicy + (wizard.restartPolicy === 'on-failure' && wizard.restartMaxRetries > 0 ? ':' + wizard.restartMaxRetries : '') }}</span>
          </div>
          <div class="review-item" v-if="wizard.command.trim()">
            <span class="review-label">Command</span>
            <span class="review-value mono">{{ wizard.command }}</span>
          </div>
          <div class="review-item" v-if="wizard.cpuLimit > 0 || wizard.memoryLimit || wizard.privileged">
            <span class="review-label">Advanced</span>
            <span class="review-value mono">
              {{ wizard.cpuLimit > 0 ? 'CPU: ' + wizard.cpuLimit.toFixed(1) + ' ' : '' }}
              {{ wizard.memoryLimit ? 'RAM: ' + wizard.memoryLimit + wizard.memoryUnit + ' ' : '' }}
              {{ wizard.privileged ? 'Privileged' : '' }}
            </span>
          </div>
        </div>

        <!-- Docker command equivalent -->
        <div class="docker-command-box">
          <div class="docker-command-header">
            <span><i class="fa-solid fa-terminal"></i> Equivalent <code>docker run</code> command</span>
            <button class="btn btn-ghost btn-sm" @click="copyToClipboard(dockerCommand)">
              <i class="fa-solid fa-copy"></i> Copy
            </button>
          </div>
          <pre class="docker-command">{{ dockerCommand }}</pre>
        </div>

        <!-- Create result -->
        <div v-if="createdContainerId" class="create-success">
          <i class="fa-solid fa-check-circle"></i>
          Container created! ID: <code>{{ createdContainerId.substring(0, 12) }}...</code>
          <span class="text-muted">Redirecting to container detail...</span>
        </div>
      </div>
    </div>

    <!-- Bottom Navigation Bar -->
    <div class="wizard-footer">
      <div class="wizard-footer-left">
        <button class="btn btn-ghost" @click="cancel">
          <i class="fa-solid fa-xmark"></i> Cancel
        </button>
      </div>
      <div class="wizard-footer-right">
        <button
          v-if="currentStep > 1"
          class="btn btn-secondary"
          @click="prevStep"
        >
          <i class="fa-solid fa-arrow-left"></i> Back
        </button>
        <button
          v-if="currentStep < 10"
          class="btn btn-primary"
          @click="nextStep"
          :disabled="!canGoNext()"
        >
          Next <i class="fa-solid fa-arrow-right"></i>
        </button>
        <button
          v-if="currentStep === 10"
          class="btn btn-primary"
          @click="handleCreateContainer"
          :disabled="creating || !getFinalImage()"
        >
          <i class="fa-solid fa-play" :class="{ 'fa-spin': creating }"></i>
          {{ creating ? 'Creating...' : 'Create Container' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ─── Wizard Layout ─── */
.wizard-layout {
  display: flex;
  flex: 1;
  gap: 0;
  height: calc(100vh - var(--header-height) - 48px - 60px);
  margin: -24px;
  overflow: hidden;
}

/* ─── Sidebar ─── */
.wizard-sidebar {
  width: 220px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow-y: auto;
}

.wizard-sidebar-header {
  padding: 16px;
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 10px;
  border-bottom: 1px solid var(--border-color);
  color: var(--accent-cyan);
}

.wizard-steps {
  display: flex;
  flex-direction: column;
  padding: 8px;
  gap: 2px;
}

.wizard-step {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 13px;
  font-family: var(--font-sans);
  cursor: pointer;
  transition: all var(--transition-fast);
  text-align: left;
  width: 100%;
}

.wizard-step:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

.wizard-step.active {
  background: var(--bg-tertiary);
  color: var(--accent-cyan);
}

.wizard-step.completed {
  color: var(--accent-green);
}

.step-number {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
}

.wizard-step.active .step-number {
  background: rgba(0, 229, 255, 0.1);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.wizard-step.completed .step-number {
  background: rgba(16, 185, 129, 0.1);
  border-color: var(--accent-green);
  color: var(--accent-green);
}

.step-info {
  display: flex;
  flex-direction: column;
}

.step-label {
  font-weight: 500;
}

/* ─── Content Area ─── */
.wizard-content {
  flex: 1;
  overflow-y: auto;
  padding: 32px 40px;
  background: var(--bg-primary);
}

.step-panel {
  animation: fadeIn 0.25s ease-out;
  max-width: 720px;
}

.step-title {
  font-size: 1.5rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin-bottom: 4px;
  color: var(--text-main);
}

.step-description {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 24px;
}

/* ─── Search Bar ─── */
.search-bar-wizard {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 8px 12px;
  margin-bottom: 16px;
  transition: border-color var(--transition-fast);
}

.search-bar-wizard:focus-within {
  border-color: var(--accent-cyan);
}

.search-bar-wizard i {
  color: var(--text-muted);
  font-size: 14px;
}

.search-bar-wizard input {
  border: none;
  background: none;
  outline: none;
  flex: 1;
  color: var(--text-main);
  font-size: 14px;
  font-family: var(--font-sans);
}

.search-bar-wizard input::placeholder {
  color: var(--text-disabled);
}

.search-spinner {
  color: var(--accent-cyan);
}

/* ─── Search Results ─── */
.search-results {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 16px;
  max-height: 320px;
  overflow-y: auto;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.search-result-item:hover {
  border-color: var(--accent-cyan);
  background: var(--bg-tertiary);
}

.search-result-item.selected {
  border-color: var(--accent-cyan);
  background: rgba(0, 229, 255, 0.05);
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-name {
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
}

.result-desc {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-meta {
  display: flex;
  gap: 12px;
  flex-shrink: 0;
}

.result-stat {
  font-size: 11px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 4px;
}

/* ─── Form Divider ─── */
.form-divider {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 16px 0;
  color: var(--text-disabled);
  font-size: 12px;
}

.form-divider::before,
.form-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border-color);
}

/* ─── Manual Image ─── */
.manual-image {
  display: flex;
  gap: 8px;
  align-items: center;
}

.selected-badge {
  margin-top: 12px;
  padding: 10px 14px;
  background: rgba(16, 185, 129, 0.06);
  border: 1px solid rgba(16, 185, 129, 0.15);
  border-radius: var(--radius-md);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

/* ─── Port / Volume / Env Rows ─── */
.port-list,
.volume-list,
.env-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.port-row,
.volume-row,
.env-row {
  display: flex;
  gap: 8px;
  align-items: flex-end;
  padding: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
}

.port-field,
.volume-field,
.env-field {
  flex: 1;
}

.input-with-browse {
  display: flex;
  gap: 4px;
}

.input-with-browse input {
  flex: 1;
}

/* ─── Checkbox ─── */
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
  color: var(--text-main);
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--accent-cyan);
}

/* ─── Network / Restart Options ─── */
.network-options,
.restart-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.network-option,
.restart-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.network-option:hover,
.restart-option:hover {
  border-color: var(--accent-cyan);
  background: var(--bg-tertiary);
}

.network-option.selected,
.restart-option.selected {
  border-color: var(--accent-cyan);
  background: rgba(0, 229, 255, 0.04);
}

.network-option input[type="radio"],
.restart-option input[type="radio"] {
  accent-color: var(--accent-cyan);
  width: 16px;
  height: 16px;
}

.network-option-content,
.restart-option-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  gap: 2px;
  font-size: 13px;
}

.network-check,
.restart-check {
  font-size: 18px;
}

/* ─── CPU Slider ─── */
.cpu-slider-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.cpu-slider {
  flex: 1;
  height: 6px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--bg-tertiary);
  border-radius: 3px;
  outline: none;
  border: 1px solid var(--border-color);
}

.cpu-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--accent-cyan);
  cursor: pointer;
  border: 2px solid var(--bg-primary);
  box-shadow: 0 0 6px rgba(0, 229, 255, 0.3);
}

.cpu-value {
  font-family: var(--font-mono);
  font-size: 14px;
  font-weight: 600;
  color: var(--accent-cyan);
  min-width: 70px;
  text-align: right;
}

.memory-input-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* ─── Review Grid ─── */
.review-grid {
  display: flex;
  flex-direction: column;
  gap: 0;
  margin-bottom: 20px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.review-item {
  display: flex;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-secondary);
}

.review-item:last-child {
  border-bottom: none;
}

.review-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  min-width: 100px;
  flex-shrink: 0;
  padding-top: 2px;
}

.review-value {
  font-size: 13px;
  color: var(--text-main);
  word-break: break-all;
}

.review-value.mono {
  font-family: var(--font-mono);
}

/* ─── Docker Command Box ─── */
.docker-command-box {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  margin-bottom: 20px;
}

.docker-command-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
  font-size: 12px;
  color: var(--text-muted);
}

.docker-command {
  padding: 16px;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
  color: var(--accent-cyan);
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
  overflow-x: auto;
}

/* ─── Create Success ─── */
.create-success {
  padding: 16px;
  background: rgba(16, 185, 129, 0.06);
  border: 1px solid rgba(16, 185, 129, 0.15);
  border-radius: var(--radius-lg);
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--accent-green);
}

.create-success code {
  font-family: var(--font-mono);
  font-size: 13px;
}

/* ─── Steps Actions ─── */
.step-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

/* ─── Inline Error ─── */
.error-inline {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.06);
  border: 1px solid rgba(239, 68, 68, 0.15);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--accent-red);
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
}

.input-error {
  border-color: var(--accent-red) !important;
}

/* ─── Form Hint ─── */
.form-hint {
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-disabled);
}

.form-hint code {
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--bg-tertiary);
  padding: 1px 4px;
  border-radius: 3px;
}

/* ─── Footer Navigation ─── */
.wizard-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.wizard-footer-right {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* ─── Responsive overrides ─── */
@media (max-width: 768px) {
  .wizard-layout {
    flex-direction: column;
  }
  .wizard-sidebar {
    width: 100%;
    max-height: 120px;
    border-right: none;
    border-bottom: 1px solid var(--border-color);
  }
  .wizard-content {
    padding: 20px;
  }
  .port-row,
  .volume-row,
  .env-row {
    flex-wrap: wrap;
  }
}
</style>