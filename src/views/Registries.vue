<!-- ItzamBox — Registry Management View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed, onUnmounted } from 'vue'
import {
  listRegistries,
  addRegistry,
  updateRegistry,
  removeRegistry,
  setDefaultRegistry,
  dockerLogin,
  dockerLogout,
  pushImage,
  listenPushLog,
  listenPushComplete,
  type RegistrySafe,
  type PushLogLine,
  type PushComplete,
} from '../composables/useDocker'
import { useNotifications } from '../composables/useNotifications'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

// ─── State ─────────────────────────────────────────────────────────────────

const { info, success, error: notifyError } = useNotifications()

const registries = ref<RegistrySafe[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// Modal state
const showModal = ref(false)
const editingId = ref<number | null>(null)
const formName = ref('')
const formUrl = ref('')
const formUsername = ref('')
const formPassword = ref('')
const formIsDefault = ref(false)
const formSaving = ref(false)

// Login state per registry (tracked by registry id)
const loggedInRegistries = ref<Set<number>>(new Set())
const loggingIn = ref<number | null>(null)
const loggingOut = ref<number | null>(null)

// Push state
const showPushModal = ref(false)
const pushImageName = ref('')
const pushRegistryId = ref<number | null>(null)
const pushing = ref(false)
const pushLogs = ref<PushLogLine[]>([])
const pushSuccess = ref<boolean | null>(null)
let unlistenPushLog: (() => void) | null = null
let unlistenPushComplete: (() => void) | null = null

onMounted(() => {
  loadData()
  subscribePushEvents()
})

onUnmounted(() => {
  unlistenPushLog?.()
  unlistenPushComplete?.()
})

const isEmpty = computed(() => registries.value.length === 0 && !loading.value && !error.value)

// ─── CRUD ──────────────────────────────────────────────────────────────────

async function loadData() {
  loading.value = true
  error.value = null
  try {
    registries.value = await listRegistries()
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

function openAddModal() {
  editingId.value = null
  formName.value = ''
  formUrl.value = ''
  formUsername.value = ''
  formPassword.value = ''
  formIsDefault.value = false
  showModal.value = true
}

function openEditModal(r: RegistrySafe) {
  editingId.value = r.id
  formName.value = r.name
  formUrl.value = r.url
  formUsername.value = r.username ?? ''
  formPassword.value = ''
  formIsDefault.value = r.is_default
  showModal.value = true
}

async function saveRegistry() {
  if (!formName.value || !formUrl.value) {
    notifyError('Validation Error', 'Name and URL are required.')
    return
  }
  formSaving.value = true
  try {
    const authToken = formPassword.value || null
    const username = formUsername.value || null

    if (editingId.value !== null) {
      await updateRegistry(
        editingId.value,
        formName.value,
        formUrl.value,
        username,
        authToken,
        formIsDefault.value,
      )
      success('Registry updated', `${formName.value} updated successfully.`)
    } else {
      await addRegistry(
        formName.value,
        formUrl.value,
        username,
        authToken,
        formIsDefault.value,
      )
      success('Registry added', `${formName.value} added successfully.`)
    }
    showModal.value = false
    await loadData()
  } catch (e: any) {
    notifyError('Failed to save registry', e.toString())
  }
  formSaving.value = false
}

async function handleRemove(id: number, name: string) {
  if (!confirm(`Delete registry "${name}"?`)) return
  try {
    await removeRegistry(id)
    loggedInRegistries.value.delete(id)
    success('Registry removed', `${name} removed successfully.`)
    await loadData()
  } catch (e: any) {
    notifyError('Failed to remove registry', e.toString())
  }
}

async function handleSetDefault(id: number) {
  try {
    await setDefaultRegistry(id)
    success('Default updated', 'Default registry updated successfully.')
    await loadData()
  } catch (e: any) {
    notifyError('Failed to set default', e.toString())
  }
}

// ─── Docker Auth ───────────────────────────────────────────────────────────

async function handleLogin(r: RegistrySafe) {
  if (!r.username) {
    notifyError('Login failed', 'No username configured for this registry. Edit the registry first.')
    return
  }
  loggingIn.value = r.id
  const password = prompt(`Enter password/token for ${r.username} at ${r.url}`)
  if (!password) {
    loggingIn.value = null
    return
  }
  try {
    const result = await dockerLogin(r.url, r.username, password)
    loggedInRegistries.value.add(r.id)
    success('Login Succeeded', result)
  } catch (e: any) {
    notifyError('Login failed', e.toString())
  }
  loggingIn.value = null
}

async function handleLogout(r: RegistrySafe) {
  loggingOut.value = r.id
  try {
    await dockerLogout(r.url)
    loggedInRegistries.value.delete(r.id)
    success('Logged out', `Logged out from ${r.url}`)
  } catch (e: any) {
    notifyError('Logout failed', e.toString())
  }
  loggingOut.value = null
}

// ─── Push Image ────────────────────────────────────────────────────────────

function openPushModal() {
  pushImageName.value = ''
  pushRegistryId.value = registries.value.length > 0 ? registries.value[0].id : null
  pushLogs.value = []
  pushSuccess.value = null
  showPushModal.value = true
}

function subscribePushEvents() {
  listenPushLog((log) => {
    pushLogs.value.push(log)
  }).then((fn) => { unlistenPushLog = fn })

  listenPushComplete((complete) => {
    pushSuccess.value = complete.success
    pushing.value = false
    if (!complete.success) {
      notifyError('Push failed', complete.error ?? 'Unknown error')
    } else {
      success('Push complete', `${complete.image} pushed successfully.`)
    }
  }).then((fn) => { unlistenPushComplete = fn })
}

async function handlePush() {
  if (!pushImageName.value) {
    notifyError('Validation', 'Image name is required.')
    return
  }
  pushing.value = true
  pushLogs.value = []
  pushSuccess.value = null

  try {
    const selectedReg = registries.value.find(r => r.id === pushRegistryId.value)
    await pushImage(pushImageName.value, selectedReg?.url ?? undefined)
  } catch (e: any) {
    pushing.value = false
    pushSuccess.value = false
    notifyError('Push failed', e.toString())
  }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

function registryIcon(url: string): string {
  if (url.includes('docker.io') || url.includes('index.docker.io')) return 'fa-brands fa-docker'
  if (url.includes('ghcr.io')) return 'fa-brands fa-github'
  if (url.includes('gcr.io')) return 'fa-brands fa-google'
  if (url.includes('ecr') || url.includes('amazonaws')) return 'fa-brands fa-aws'
  if (url.includes('azurecr.io')) return 'fa-brands fa-microsoft'
  if (url.includes('gitlab')) return 'fa-brands fa-gitlab'
  return 'fa-solid fa-server'
}

function isLoggedIn(r: RegistrySafe): boolean {
  return loggedInRegistries.value.has(r.id)
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Registries</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">
      <i class="fa-solid fa-server" style="color:var(--accent-cyan);margin-right:10px"></i>
      Registries
    </h1>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary" @click="loadData" :disabled="loading">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> Refresh
      </button>
      <button class="btn btn-primary" @click="openPushModal" :disabled="registries.length === 0">
        <i class="fa-solid fa-upload"></i> Push Image
      </button>
      <button class="btn btn-primary" @click="openAddModal">
        <i class="fa-solid fa-plus"></i> Add Registry
      </button>
    </div>
  </div>

  <!-- Loading state -->
  <SkeletonLoader v-if="loading && registries.length === 0" variant="table-row" :rows="3" />

  <!-- Error state -->
  <ErrorState
    v-if="error"
    message="Error loading registries"
    suggestion="Check that the database is accessible and try again."
    :detail="error"
    @retry="loadData"
  />

  <!-- Content -->
  <template v-if="!loading && !error">
    <!-- Empty state -->
    <EmptyState
      v-if="isEmpty"
      icon="fa-solid fa-server"
      title="No registries configured"
      description="Add a private container registry (Docker Hub, GHCR, etc.) to manage authentication and push images."
      action-label="Add Registry"
      @action="openAddModal"
    />

    <div v-else class="section" style="margin-top:16px;">
      <div class="section-header">
        <span class="section-title">Configured Registries ({{ registries.length }})</span>
      </div>

      <!-- Registry rows -->
      <div v-for="r in registries" :key="r.id" class="data-row">
        <div class="row-info" style="flex:1;display:flex;align-items:center;gap:12px;">
          <i :class="registryIcon(r.url)" style="font-size:20px;width:24px;text-align:center;color:var(--accent-cyan)"></i>
          <div>
            <div class="row-name" style="display:flex;align-items:center;gap:8px;">
              {{ r.name }}
              <span v-if="r.is_default" class="badge badge-primary">Default</span>
            </div>
            <div class="row-meta">
              <code style="font-size:12px;color:var(--text-muted)">{{ r.url }}</code>
              <span v-if="r.username" style="margin-left:12px;color:var(--text-secondary)">
                <i class="fa-solid fa-user"></i> {{ r.username }}
              </span>
            </div>
          </div>
        </div>

        <div style="display:flex;align-items:center;gap:8px;">
          <!-- Login status -->
          <span v-if="isLoggedIn(r)" class="badge badge-success">
            <i class="fa-solid fa-check-circle"></i> Logged in
          </span>
          <span v-else class="badge" style="background:var(--bg-tertiary);color:var(--text-muted);border:1px solid var(--border-color)">
            Not logged in
          </span>

          <!-- Actions -->
          <button
            v-if="!r.is_default"
            class="action-btn"
            title="Set as default"
            @click="handleSetDefault(r.id)"
          >
            <i class="fa-solid fa-star"></i>
          </button>
          <button class="action-btn" title="Edit" @click="openEditModal(r)">
            <i class="fa-solid fa-pen-to-square"></i>
          </button>
          <button
            v-if="!isLoggedIn(r)"
            class="action-btn"
            title="Login"
            :disabled="loggingIn === r.id"
            @click="handleLogin(r)"
          >
            <i class="fa-solid fa-right-to-bracket" :class="{ 'fa-spin': loggingIn === r.id }"></i>
          </button>
          <button
            v-if="isLoggedIn(r)"
            class="action-btn"
            title="Logout"
            :disabled="loggingOut === r.id"
            @click="handleLogout(r)"
          >
            <i class="fa-solid fa-right-from-bracket" :class="{ 'fa-spin': loggingOut === r.id }"></i>
          </button>
          <button class="action-btn" title="Remove" @click="handleRemove(r.id, r.name)">
            <i class="fa-solid fa-trash-can"></i>
          </button>
        </div>
      </div>
    </div>

    <!-- Push section info -->
    <div v-if="registries.length > 0" class="section" style="margin-top:16px;">
      <div class="section-header">
        <span class="section-title"><i class="fa-solid fa-circle-info" style="color:var(--accent-cyan);margin-right:8px"></i>Quick Tips</span>
      </div>
      <div style="padding:16px 20px;font-size:13px;color:var(--text-secondary);line-height:1.7;">
        <p>• Use <strong>Push Image</strong> to upload a local image to a registry. The image must exist locally.</p>
        <p>• If a registry URL is specified, ItzamBox will automatically tag and push (e.g. <code>localhost:5000/my-image</code>).</p>
        <p>• You must be <strong>logged in</strong> to the registry before pushing. Use the login button next to each registry.</p>
        <p>• Tokens are stored securely in the database and masked during retrieval for security.</p>
      </div>
    </div>
  </template>

  <!-- ─── Add/Edit Modal ────────────────────────────────────────────────── -->
  <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
    <div class="modal-content" style="max-width:520px;">
      <div class="modal-header">
        <span class="modal-title">
          <i class="fa-solid fa-server"></i>
          {{ editingId !== null ? 'Edit Registry' : 'Add Registry' }}
        </span>
        <button class="header-btn" @click="showModal = false"><i class="fa-solid fa-xmark"></i></button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label class="form-label">Registry name <span style="color:var(--accent-red)">*</span></label>
          <input class="form-input" v-model="formName" placeholder="Docker Hub, GHCR, Local..." @keyup.enter="saveRegistry">
        </div>
        <div class="form-group">
          <label class="form-label">Registry URL <span style="color:var(--accent-red)">*</span></label>
          <input class="form-input mono" v-model="formUrl" placeholder="https://index.docker.io/v1/" @keyup.enter="saveRegistry">
          <div style="font-size:11px;color:var(--text-muted);margin-top:4px;">
            Examples: <code>https://index.docker.io/v1/</code>, <code>https://ghcr.io</code>
          </div>
        </div>
        <div class="form-group">
          <label class="form-label">Username (optional)</label>
          <input class="form-input" v-model="formUsername" placeholder="Docker Hub or registry username">
        </div>
        <div class="form-group">
          <label class="form-label">{{ editingId !== null ? 'Password/Token (leave blank to keep)' : 'Password/Token' }}</label>
          <input class="form-input" type="password" v-model="formPassword" placeholder="Access token or password">
        </div>
        <div class="form-group" style="display:flex;align-items:center;gap:8px;">
          <input type="checkbox" id="chk-default" v-model="formIsDefault" style="width:16px;height:16px;accent-color:var(--accent-cyan);">
          <label for="chk-default" class="form-label" style="margin:0;cursor:pointer;">Set as default registry</label>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="showModal = false">Cancel</button>
        <button class="btn btn-primary" :disabled="formSaving" @click="saveRegistry">
          <i class="fa-solid fa-spinner fa-spin" v-if="formSaving"></i>
          <i v-else :class="editingId !== null ? 'fa-solid fa-floppy-disk' : 'fa-solid fa-plus'"></i>
          {{ editingId !== null ? ' Update' : ' Add' }}
        </button>
      </div>
    </div>
  </div>

  <!-- ─── Push Image Modal ──────────────────────────────────────────────── -->
  <div v-if="showPushModal" class="modal-backdrop" @click.self="showPushModal = false">
    <div class="modal-content" style="max-width:600px;">
      <div class="modal-header">
        <span class="modal-title"><i class="fa-solid fa-upload"></i> Push Image</span>
        <button class="header-btn" @click="showPushModal = false"><i class="fa-solid fa-xmark"></i></button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label class="form-label">Image name (local)</label>
          <input class="form-input mono" v-model="pushImageName" placeholder="my-image:latest" @keyup.enter="handlePush">
        </div>
        <div class="form-group" v-if="registries.length > 0">
          <label class="form-label">Target registry</label>
          <select class="form-input" v-model="pushRegistryId" style="appearance:auto;">
            <option v-for="r in registries" :key="r.id" :value="r.id">
              {{ r.name }} ({{ r.url }})
            </option>
          </select>
          <div style="font-size:11px;color:var(--text-muted);margin-top:4px;">
            Leave registry empty to push with the image name as-is (already tagged).
          </div>
        </div>

        <!-- Push logs -->
        <div v-if="pushLogs.length > 0" class="section" style="margin-top:12px;">
          <div class="section-header">
            <span class="section-title">Push Output</span>
          </div>
          <div class="terminal-log">
            <div v-for="(log, i) in pushLogs" :key="i"
              :style="{ color: log.stream === 'stderr' ? 'var(--accent-yellow)' : 'var(--text-main)' }">
              <span style="color:var(--text-muted);user-select:none;">{{ String(i + 1).padStart(3, ' ') }}</span>
              {{ log.line }}
            </div>
            <div v-if="pushing" class="push-in-progress">
              <i class="fa-solid fa-spinner fa-spin"></i> Pushing layers...
            </div>
            <div v-else-if="pushSuccess === true" style="color:var(--accent-green);margin-top:8px;">
              <i class="fa-solid fa-check-circle"></i> Push completed successfully!
            </div>
            <div v-else-if="pushSuccess === false" style="color:var(--accent-red);margin-top:8px;">
              <i class="fa-solid fa-circle-xmark"></i> Push failed.
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="showPushModal = false">Close</button>
        <button class="btn btn-primary" :disabled="pushing || !pushImageName" @click="handlePush">
          <i class="fa-solid fa-spinner fa-spin" v-if="pushing"></i>
          <i v-else class="fa-solid fa-upload"></i>
          {{ pushing ? 'Pushing...' : 'Push' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.terminal-log {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  font-family: var(--font-mono, 'JetBrains Mono', monospace);
  font-size: 12px;
  line-height: 1.6;
  max-height: 240px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.push-in-progress {
  color: var(--accent-cyan);
  margin-top: 8px;
  font-style: italic;
}

.badge-success {
  background: rgba(0, 200, 83, 0.12);
  color: var(--accent-green, #00c853);
  border: 1px solid rgba(0, 200, 83, 0.25);
  font-size: 11px;
  padding: 2px 10px;
  border-radius: var(--radius-sm);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.badge-primary {
  background: rgba(0, 229, 255, 0.12);
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 229, 255, 0.25);
  font-size: 10px;
  padding: 1px 8px;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  font-weight: 600;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
