<!-- ItzamBox — Container Templates Gallery
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  listTemplates,
  deleteTemplate,
  seedBuiltinTemplates,
  saveTemplate,
  parseTemplatePorts,
  parseJsonArray,
  formatCategory,
  categoryColor,
  type ContainerTemplate,
  type PortConfig,
} from '../composables/useDocker'
import { useDocker } from '../composables/useDocker'
import { useNotifications } from '../composables/useNotifications'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const router = useRouter()
const { pullImage, createAndRunContainer } = useDocker()
const { success, error: notifyError, info } = useNotifications()

const templates = ref<ContainerTemplate[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')
const categoryFilter = ref('all')
const deploying = ref<Set<number>>(new Set())
const deleting = ref<Set<number>>(new Set())
const showSaveModal = ref(false)
const seeding = ref(false)

// Save template form
const saveForm = ref({
  name: '',
  description: '',
  image: '',
  default_ports: '[]',
  default_volumes: '[]',
  default_env: '[]',
  default_network: 'bridge',
  default_restart: 'unless-stopped',
  default_command: null as string | null,
  category: 'custom',
  icon: 'fa-cube',
})

onMounted(() => loadTemplates())

async function loadTemplates() {
  loading.value = true
  error.value = null
  try {
    templates.value = await listTemplates()
    // Auto-seed if no templates exist
    if (templates.value.length === 0) {
      await seedBuiltinTemplates()
      templates.value = await listTemplates()
    }
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

const isEmpty = computed(() => templates.value.length === 0 && !loading.value && !error.value)
const categories = computed(() => {
  const cats = new Set(templates.value.map(t => t.category))
  return ['all', ...Array.from(cats).sort()]
})
const categoryLabels: Record<string, string> = {
  all: 'All',
  web: 'Web',
  database: 'Database',
  cache: 'Cache',
  runtime: 'Runtime',
  'message-queue': 'Message Queue',
  custom: 'Custom',
}

const filteredTemplates = computed(() => {
  let result = templates.value
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(t =>
      t.name.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q) ||
      t.image.toLowerCase().includes(q)
    )
  }
  if (categoryFilter.value !== 'all') {
    result = result.filter(t => t.category === categoryFilter.value)
  }
  return result
})

function getPorts(template: ContainerTemplate): PortConfig[] {
  return parseTemplatePorts(template.default_ports)
}

function getVolumes(template: ContainerTemplate): string[] {
  return parseJsonArray(template.default_volumes)
}

function getEnv(template: ContainerTemplate): string[] {
  return parseJsonArray(template.default_env)
}

async function handleDeploy(template: ContainerTemplate) {
  if (!template.id) return
  deploying.value.add(template.id)
  try {
    // 1. Pull the image
    info('Pulling image', `Pulling ${template.image}...`)
    await pullImage(template.image)

    // 2. Parse ports
    const ports = getPorts(template).map(p => ({
      host_ip: '0.0.0.0',
      host_port: parseInt(p.host, 10) || 0,
      container_port: parseInt(p.container, 10) || 0,
      protocol: p.protocol || 'tcp',
    }))

    // 3. Parse volumes
    const volumes = getVolumes(template)

    // 4. Parse env vars
    const envVars = getEnv(template)

    // 5. Parse command
    const commandArr = template.default_command
      ? template.default_command.split(/\s+/)
      : null

    // 6. Create and run container
    const containerId = await createAndRunContainer({
      image: template.image,
      name: null,
      ports,
      volumes,
      env_vars: envVars,
      network: template.default_network || null,
      restart_policy: template.default_restart !== 'no' ? template.default_restart : null,
      command: commandArr,
      detach: true,
      cpu_limit: null,
      memory_limit: null,
      privileged: false,
    })

    success('Container deployed', `Container ${containerId.substring(0, 12)} from template "${template.name}" created successfully.`)

    // 7. Redirect to Containers view
    setTimeout(() => {
      router.push('/containers')
    }, 1500)
  } catch (e: any) {
    notifyError('Deploy failed', `Failed to deploy ${template.name}: ${e.toString()}`)
  } finally {
    if (template.id) deploying.value.delete(template.id)
  }
}

async function handleDelete(template: ContainerTemplate) {
  if (!template.id || template.is_builtin) return
  deleting.value.add(template.id)
  try {
    await deleteTemplate(template.id)
    templates.value = templates.value.filter(t => t.id !== template.id)
    success('Template deleted', `"${template.name}" has been removed.`)
  } catch (e: any) {
    notifyError('Delete failed', e.toString())
  } finally {
    if (template.id) deleting.value.delete(template.id)
  }
}

function openSaveModal() {
  saveForm.value = {
    name: '',
    description: '',
    image: '',
    default_ports: '[]',
    default_volumes: '[]',
    default_env: '[]',
    default_network: 'bridge',
    default_restart: 'unless-stopped',
    default_command: null,
    category: 'custom',
    icon: 'fa-cube',
  }
  showSaveModal.value = true
}

async function handleSaveTemplate() {
  if (!saveForm.value.name.trim() || !saveForm.value.image.trim()) return
  try {
    await saveTemplate({
      id: null,
      name: saveForm.value.name.trim(),
      description: saveForm.value.description.trim(),
      image: saveForm.value.image.trim(),
      default_ports: saveForm.value.default_ports,
      default_volumes: saveForm.value.default_volumes,
      default_env: saveForm.value.default_env,
      default_network: saveForm.value.default_network,
      default_restart: saveForm.value.default_restart,
      default_command: saveForm.value.default_command?.trim() || null,
      is_builtin: false,
      category: saveForm.value.category,
      icon: saveForm.value.icon,
    })
    success('Template saved', `"${saveForm.value.name}" saved successfully.`)
    showSaveModal.value = false
    await loadTemplates()
  } catch (e: any) {
    notifyError('Save failed', e.toString())
  }
}

async function handleSeed() {
  seeding.value = true
  try {
    await seedBuiltinTemplates()
    await loadTemplates()
    success('Templates seeded', 'Built-in templates have been restored.')
  } catch (e: any) {
    notifyError('Seed failed', e.toString())
  }
  seeding.value = false
}
</script>

<template>
  <div class="view-root">
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Templates</span>
  </div>

  <div class="page-header">
    <h1 class="text-h1">Container Templates</h1>
    <div class="page-header-actions">
      <button class="btn btn-secondary" @click="loadTemplates" :disabled="loading">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> Refresh
      </button>
      <button class="btn btn-secondary" @click="handleSeed" :disabled="seeding">
        <i class="fa-solid fa-seedling" :class="{ 'fa-spin': seeding }"></i> Restore Built-ins
      </button>
      <button class="btn btn-primary" @click="openSaveModal">
        <i class="fa-solid fa-floppy-disk"></i> Save Current as Template
      </button>
    </div>
  </div>

  <!-- Loading state -->
  <SkeletonLoader v-if="loading && templates.length === 0" variant="card" :rows="6" />

  <!-- Error state -->
  <ErrorState
    v-if="error && !loading"
    :message="'Error loading templates'"
    :suggestion="'Check that the database is accessible.'"
    :detail="error"
    @retry="loadTemplates"
  />

  <!-- Empty state -->
  <EmptyState
    v-if="isEmpty"
    icon="fa-solid fa-cubes"
    title="No templates found"
    description="Templates let you quickly deploy common container stacks. Click the button below to restore built-in templates."
    action-label="Restore Built-in Templates"
    @action="handleSeed"
  />

  <template v-if="!loading && !error && !isEmpty">
    <!-- Search & Filter Bar -->
    <div class="templates-toolbar">
      <div class="search-bar">
        <i class="fa-solid fa-magnifying-glass"></i>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search templates by name, description, or image..."
        />
        <button v-if="searchQuery" class="search-clear" @click="searchQuery = ''">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>
      <div class="filter-chips">
        <button
          v-for="cat in categories"
          :key="cat"
          :class="['chip', { active: categoryFilter === cat }]"
          @click="categoryFilter = cat"
        >
          <span v-if="cat !== 'all'" class="chip-dot" :style="{ background: categoryColor(cat) }"></span>
          {{ categoryLabels[cat] || formatCategory(cat) }}
        </button>
      </div>
    </div>

    <!-- Template Grid -->
    <div v-if="filteredTemplates.length === 0" class="no-results">
      <i class="fa-solid fa-search"></i>
      <p>No templates match your search</p>
      <button class="btn btn-ghost btn-sm" @click="searchQuery = ''; categoryFilter = 'all'">Clear filters</button>
    </div>

    <div v-else class="template-grid">
      <div
        v-for="template in filteredTemplates"
        :key="template.id ?? template.name"
        class="template-card"
      >
        <!-- Card Header -->
        <div class="template-card-header">
          <div class="template-icon-wrapper" :style="{ background: categoryColor(template.category) + '18', color: categoryColor(template.category) }">
            <i :class="'fa-solid ' + template.icon" v-if="!template.icon.includes('fa-brands')"></i>
            <i :class="'fa-brands ' + template.icon.replace('fa-brands ', '')" v-else></i>
          </div>
          <div class="template-card-title-group">
            <div class="template-card-title-row">
              <h3 class="template-name">{{ template.name }}</h3>
              <span v-if="template.is_builtin" class="tag builtin-tag">Built-in</span>
            </div>
            <span
              class="category-badge"
              :style="{ background: categoryColor(template.category) + '18', color: categoryColor(template.category) }"
            >
              {{ formatCategory(template.category) }}
            </span>
          </div>
        </div>

        <!-- Card Body -->
        <div class="template-card-body">
          <p class="template-desc">{{ template.description }}</p>

          <div class="template-details">
            <div class="detail-item">
              <span class="detail-label">Image</span>
              <span class="detail-value mono">{{ template.image }}</span>
            </div>

            <div v-if="getPorts(template).length > 0" class="detail-item">
              <span class="detail-label">Ports</span>
              <div class="port-chips">
                <span
                  v-for="(port, idx) in getPorts(template)"
                  :key="idx"
                  class="port-chip"
                  :title="port.protocol.toUpperCase()"
                >
                  <i class="fa-solid fa-plug"></i>
                  {{ port.host }}:{{ port.container }}
                  <span class="port-protocol">{{ port.protocol }}</span>
                </span>
              </div>
            </div>

            <div v-if="getVolumes(template).length > 0" class="detail-item">
              <span class="detail-label">Volumes</span>
              <div class="volume-chips">
                <span v-for="(vol, idx) in getVolumes(template)" :key="idx" class="volume-chip">
                  <i class="fa-solid fa-hard-drive"></i>
                  {{ vol }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Card Footer -->
        <div class="template-card-footer">
          <button
            class="btn btn-primary btn-sm"
            @click="handleDeploy(template)"
            :disabled="!template.id || deploying.has(template.id)"
          >
            <i class="fa-solid fa-play" :class="{ 'fa-spin': template.id && deploying.has(template.id) }"></i>
            {{ template.id && deploying.has(template.id) ? 'Deploying...' : 'Deploy' }}
          </button>
          <button
            v-if="!template.is_builtin"
            class="btn btn-danger btn-sm"
            @click="handleDelete(template)"
            :disabled="!template.id || deleting.has(template.id)"
          >
            <i class="fa-solid fa-trash-can" :class="{ 'fa-spin': template.id && deleting.has(template.id) }"></i>
            {{ template.id && deleting.has(template.id) ? 'Deleting...' : 'Delete' }}
          </button>
        </div>
      </div>
    </div>
  </template>

  <!-- Save Template Modal -->
  <div v-if="showSaveModal" class="modal-overlay" @click.self="showSaveModal = false">
    <div class="modal-panel">
      <div class="modal-header">
        <h2><i class="fa-solid fa-floppy-disk"></i> Save Current as Template</h2>
        <button class="btn btn-ghost btn-sm" @click="showSaveModal = false"><i class="fa-solid fa-xmark"></i></button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label class="form-label">Template Name *</label>
          <input v-model="saveForm.name" class="form-input" placeholder="My Custom Stack" />
        </div>
        <div class="form-group">
          <label class="form-label">Description</label>
          <input v-model="saveForm.description" class="form-input" placeholder="Brief description of this template" />
        </div>
        <div class="form-group">
          <label class="form-label">Docker Image *</label>
          <input v-model="saveForm.image" class="form-input mono" placeholder="nginx:latest" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Category</label>
            <select v-model="saveForm.category" class="form-input">
              <option value="web">Web</option>
              <option value="database">Database</option>
              <option value="cache">Cache</option>
              <option value="runtime">Runtime</option>
              <option value="message-queue">Message Queue</option>
              <option value="custom">Custom</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label">Icon</label>
            <select v-model="saveForm.icon" class="form-input">
              <option value="fa-cube">Cube</option>
              <option value="fa-globe">Globe</option>
              <option value="fa-database">Database</option>
              <option value="fa-registered">Registered</option>
              <option value="fa-leaf">Leaf</option>
              <option value="fa-server">Server</option>
              <option value="fa-cloud">Cloud</option>
              <option value="fa-gear">Gear</option>
              <option value="fa-bolt">Bolt</option>
              <option value="fa-brands fa-node-js">Node.js</option>
              <option value="fa-brands fa-python">Python</option>
              <option value="fa-brands fa-docker">Docker</option>
            </select>
          </div>
        </div>
        <div class="form-group">
          <label class="form-label">Network</label>
          <select v-model="saveForm.default_network" class="form-input">
            <option value="bridge">Bridge</option>
            <option value="host">Host</option>
            <option value="none">None</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">Restart Policy</label>
          <select v-model="saveForm.default_restart" class="form-input">
            <option value="no">No</option>
            <option value="always">Always</option>
            <option value="on-failure">On Failure</option>
            <option value="unless-stopped">Unless Stopped</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">JSON Ports</label>
          <textarea v-model="saveForm.default_ports" class="form-input mono" rows="2" placeholder='[{"host":"80","container":"80","protocol":"tcp"}]'></textarea>
        </div>
        <div class="form-group">
          <label class="form-label">JSON Volumes</label>
          <textarea v-model="saveForm.default_volumes" class="form-input mono" rows="2" placeholder='["/host/data:/data"]'></textarea>
        </div>
        <div class="form-group">
          <label class="form-label">JSON Environment Variables</label>
          <textarea v-model="saveForm.default_env" class="form-input mono" rows="2" placeholder='["KEY=VALUE"]'></textarea>
        </div>
        <div class="form-group">
          <label class="form-label">Default Command (optional)</label>
          <input v-model="saveForm.default_command" class="form-input mono" placeholder="node server.js" />
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-ghost" @click="showSaveModal = false">Cancel</button>
        <button
          class="btn btn-primary"
          @click="handleSaveTemplate"
          :disabled="!saveForm.name.trim() || !saveForm.image.trim()"
        >
          <i class="fa-solid fa-floppy-disk"></i> Save Template
        </button>
      </div>
    </div>
  </div>
  </div>
</template>

<style scoped>
/* ─── Page Layout ─── */
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 12px;
}

.page-header-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

/* ─── Toolbar ─── */
.templates-toolbar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 8px 12px;
  transition: border-color var(--transition-fast);
}

.search-bar:focus-within {
  border-color: var(--accent-cyan);
}

.search-bar i {
  color: var(--text-muted);
  font-size: 14px;
}

.search-bar input {
  border: none;
  background: none;
  outline: none;
  flex: 1;
  color: var(--text-main);
  font-size: 14px;
  font-family: var(--font-sans);
}

.search-bar input::placeholder {
  color: var(--text-disabled);
}

.search-clear {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  font-size: 14px;
}

.search-clear:hover {
  color: var(--text-main);
}

.filter-chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 20px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-size: 12px;
  font-family: var(--font-sans);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.chip:hover {
  border-color: var(--accent-cyan);
  color: var(--text-main);
}

.chip.active {
  border-color: var(--accent-cyan);
  background: rgba(0, 229, 255, 0.08);
  color: var(--accent-cyan);
}

.chip-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* ─── Template Grid ─── */
.template-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

@media (max-width: 1100px) {
  .template-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 700px) {
  .template-grid {
    grid-template-columns: 1fr;
  }
}

.template-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition: all var(--transition-fast);
  display: flex;
  flex-direction: column;
}

.template-card:hover {
  border-color: var(--accent-cyan);
  box-shadow: 0 4px 20px rgba(0, 229, 255, 0.06);
}

/* ─── Card Header ─── */
.template-card-header {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 18px 18px 12px;
}

.template-icon-wrapper {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  flex-shrink: 0;
}

.template-card-title-group {
  flex: 1;
  min-width: 0;
}

.template-card-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.template-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-main);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.builtin-tag {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 229, 255, 0.2);
  white-space: nowrap;
  flex-shrink: 0;
  font-weight: 500;
}

.category-badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
}

/* ─── Card Body ─── */
.template-card-body {
  padding: 0 18px 12px;
  flex: 1;
}

.template-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0 0 12px;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.template-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.detail-value {
  font-size: 12px;
  color: var(--text-main);
  word-break: break-all;
}

.detail-value.mono {
  font-family: var(--font-mono);
}

/* ─── Port / Volume Chips ─── */
.port-chips,
.volume-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.port-chip,
.volume-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-family: var(--font-mono);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  color: var(--text-muted);
}

.port-chip i,
.volume-chip i {
  font-size: 10px;
}

.port-protocol {
  font-size: 9px;
  text-transform: uppercase;
  color: var(--text-disabled);
}

/* ─── Card Footer ─── */
.template-card-footer {
  display: flex;
  gap: 8px;
  padding: 12px 18px;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
}

.template-card-footer .btn {
  flex: 1;
}

/* ─── No Results ─── */
.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  gap: 8px;
}

.no-results i {
  font-size: 36px;
  opacity: 0.3;
  margin-bottom: 8px;
}

.no-results p {
  font-size: 14px;
  margin: 0;
}

/* ─── Modal ─── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-panel {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  width: 100%;
  max-width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h2 {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-body {
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid var(--border-color);
}

/* ─── Form Elements ─── */
.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.form-input {
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
  background: var(--bg-tertiary);
  color: var(--text-main);
  font-size: 13px;
  font-family: var(--font-sans);
  outline: none;
  transition: border-color var(--transition-fast);
}

.form-input:focus {
  border-color: var(--accent-cyan);
}

.form-input.mono {
  font-family: var(--font-mono);
  font-size: 12px;
}

.form-input::placeholder {
  color: var(--text-disabled);
}

.form-input textarea {
  resize: vertical;
}

select.form-input {
  cursor: pointer;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-row .form-group {
  flex: 1;
}

/* ─── Responsive ─── */
@media (max-width: 900px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .page-header-actions {
    width: 100%;
  }

  .form-row {
    flex-direction: column;
  }
}
</style>
