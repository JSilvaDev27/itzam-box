<!-- ItzamBox — Vulnerability Scanner View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useDocker } from '../composables/useDocker'
import {
  detectScanner,
  scanImage,
  getScanHistory,
  listenScanProgress,
  severityColor,
  severityBg,
  safeCount,
  type VulnerabilityReport,
  type Vulnerability,
  type ScanProgress,
} from '../composables/useDocker'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

// ─── Shared Docker state ──────────────────────────────────────────────────

const { images, fetchImages } = useDocker()

// ─── Scanner State ────────────────────────────────────────────────────────

const scannerDetected = ref<string | null>(null)
const scannerDetecting = ref(true)
const scannerError = ref<string | null>(null)

// ─── Scan State ───────────────────────────────────────────────────────────

const selectedImage = ref('')
const scanning = ref(false)
const scanError = ref<string | null>(null)
const currentReport = ref<VulnerabilityReport | null>(null)
const progressStep = ref('')
const progressMessage = ref('')

let unlistenProgress: (() => void) | null = null

// ─── Scan History ─────────────────────────────────────────────────────────

const scanHistory = ref<VulnerabilityReport[]>([])
const showHistory = ref(false)
const loadingHistory = ref(false)

// ─── Table Filters ────────────────────────────────────────────────────────

const severityFilter = ref<string>('all')
const packageSearch = ref('')
const expandedVuln = ref<string | null>(null)

// ─── Image options derived from local images ──────────────────────────────

const imageOptions = computed(() => {
  return images.value
    .filter(img => img.repository && img.repository !== '<none>')
    .map(img => ({
      label: `${img.repository}:${img.tag}`,
      value: `${img.repository}:${img.tag}`,
    }))
    .filter((opt, i, arr) => arr.findIndex(o => o.value === opt.value) === i) // deduplicate
    .sort((a, b) => a.label.localeCompare(b.label))
})

const canScan = computed(() => {
  return selectedImage.value.trim().length > 0 && !scanning.value && scannerDetected.value !== null
})

// ─── Filtered vulnerabilities ─────────────────────────────────────────────

const allVulns = computed<Vulnerability[]>(() => {
  if (!currentReport.value) return []
  return [
    ...currentReport.value.critical,
    ...currentReport.value.high,
    ...currentReport.value.medium,
    ...currentReport.value.low,
  ]
})

const filteredVulns = computed(() => {
  let list = allVulns.value

  if (severityFilter.value !== 'all') {
    list = list.filter(v => v.severity.toLowerCase() === severityFilter.value)
  }

  if (packageSearch.value.trim()) {
    const q = packageSearch.value.trim().toLowerCase()
    list = list.filter(v =>
      v.package.toLowerCase().includes(q) ||
      v.id.toLowerCase().includes(q) ||
      v.title.toLowerCase().includes(q)
    )
  }

  return list
})

// ─── Severity breakdown for chart ─────────────────────────────────────────

const severityBreakdown = computed(() => {
  if (!currentReport.value) return []
  return [
    { label: 'Critical', count: currentReport.value.critical.length, color: 'var(--accent-red)', key: 'critical' as const },
    { label: 'High', count: currentReport.value.high.length, color: 'var(--accent-yellow)', key: 'high' as const },
    { label: 'Medium', count: currentReport.value.medium.length, color: 'var(--accent-blue)', key: 'medium' as const },
    { label: 'Low', count: currentReport.value.low.length, color: 'var(--text-muted)', key: 'low' as const },
  ]
})

const maxSeverityCount = computed(() => {
  return Math.max(...severityBreakdown.value.map(s => s.count), 1)
})

// ─── Lifecycle ────────────────────────────────────────────────────────────

onMounted(async () => {
  await Promise.all([detectAvailableScanner(), fetchImages()])
})

onUnmounted(() => {
  unlistenProgress?.()
})

// ─── Scanner detection ─────────────────────────────────────────────────────

async function detectAvailableScanner() {
  scannerDetecting.value = true
  scannerError.value = null
  try {
    scannerDetected.value = await detectScanner()
  } catch (e: any) {
    scannerError.value = e?.toString() ?? 'Failed to detect scanner'
    scannerDetected.value = null
  }
  scannerDetecting.value = false
}

// ─── Scan execution ────────────────────────────────────────────────────────

async function handleScan() {
  if (!canScan.value || !selectedImage.value) return

  scanning.value = true
  scanError.value = null
  currentReport.value = null
  showHistory.value = false
  progressStep.value = ''
  progressMessage.value = ''
  expandedVuln.value = null

  // Subscribe to progress events
  try {
    unlistenProgress = await listenScanProgress((progress: ScanProgress) => {
      progressStep.value = progress.step
      progressMessage.value = progress.message
    })
  } catch (e: any) {
    console.warn('Failed to listen to scan progress:', e)
  }

  try {
    const report = await scanImage(selectedImage.value)
    currentReport.value = report
    // Refresh history after scan
    await loadScanHistory()
  } catch (e: any) {
    scanError.value = e?.toString() ?? 'Scan failed'
  } finally {
    scanning.value = false
    unlistenProgress?.()
    unlistenProgress = null
  }
}

// ─── Scan History ─────────────────────────────────────────────────────────

async function loadScanHistory() {
  if (!selectedImage.value) return
  loadingHistory.value = true
  try {
    scanHistory.value = await getScanHistory(selectedImage.value)
  } catch (e: any) {
    console.warn('Failed to load scan history:', e)
  }
  loadingHistory.value = false
}

async function toggleHistory() {
  showHistory.value = !showHistory.value
  if (showHistory.value && scanHistory.value.length === 0 && selectedImage.value) {
    await loadScanHistory()
  }
}

// ─── Row expansion ────────────────────────────────────────────────────────

function toggleExpand(vulnId: string) {
  expandedVuln.value = expandedVuln.value === vulnId ? null : vulnId
}

function nvdLink(vulnId: string): string {
  return `https://nvd.nist.gov/vuln/detail/${vulnId}`
}

// ─── Summary helpers ──────────────────────────────────────────────────────

function severitySummaryClass(severity: string): string {
  const map: Record<string, string> = {
    critical: 'severity-critical',
    high: 'severity-high',
    medium: 'severity-medium',
    low: 'severity-low',
  }
  return map[severity.toLowerCase()] || 'severity-low'
}

function formatTimestamp(ts: number): string {
  const d = new Date(ts * 1000)
  return d.toLocaleDateString(undefined, {
    year: 'numeric', month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

// Handle reselecting image to suggest loading history
watch(selectedImage, () => {
  // Clear previous results when image changes
  currentReport.value = null
  scanError.value = null
  showHistory.value = false
  expandedVuln.value = null
})

// ─── Severity badge in table ──────────────────────────────────────────────

function severityBadgeClass(severity: string): string {
  const s = severity.toLowerCase()
  if (s === 'critical') return 'badge-critical'
  if (s === 'high') return 'badge-high'
  if (s === 'medium') return 'badge-medium'
  return 'badge-low'
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Vulnerability Scanner</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px;">
    <h1 class="text-h1">Vulnerability Scanner</h1>
  </div>

  <!-- ════ Split Layout ════ -->
  <div class="scanner-split">
    <!-- ─── Left Panel: Configuration ─── -->
    <div class="scanner-config">
      <div class="section">
        <div class="section-header">
          <span class="section-title"><i class="fa-solid fa-shield-halved"></i> Scan Configuration</span>
        </div>
        <div class="scanner-config-body">
          <!-- Scanner Status -->
          <div class="form-group">
            <label class="form-label">Scanner Engine</label>
            <div v-if="scannerDetecting" class="scanner-status detecting">
              <i class="fa-solid fa-spinner fa-spin"></i> Detecting scanner…
            </div>
            <div v-else-if="scannerDetected" class="scanner-status detected">
              <i class="fa-solid fa-circle-check"></i> {{ scannerDetected }}
            </div>
            <div v-else class="scanner-status not-found">
              <i class="fa-solid fa-triangle-exclamation"></i> No scanner found — install Trivy or Grype
            </div>
          </div>

          <!-- Install Instructions (when no scanner) -->
          <div v-if="!scannerDetected && !scannerDetecting" class="install-instructions">
            <div class="form-group">
              <label class="form-label">Install Trivy (recommended)</label>
              <div class="code-block">curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh</div>
            </div>
            <div class="form-group">
              <label class="form-label">Install Grype</label>
              <div class="code-block">curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin</div>
            </div>
          </div>

          <!-- Image Selector -->
          <div class="form-group">
            <label class="form-label">Target Image</label>
            <div class="image-select-wrapper">
              <select v-model="selectedImage" class="form-input image-select" :disabled="scanning">
                <option value="" disabled>Select a local image…</option>
                <option v-for="opt in imageOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <i class="fa-solid fa-chevron-down select-chevron"></i>
            </div>
          </div>

          <!-- Custom image input -->
          <div class="form-group">
            <label class="form-label">Or type a custom image</label>
            <input
              class="form-input mono"
              v-model="selectedImage"
              placeholder="nginx:latest"
              :disabled="scanning"
            
            />
          </div>

          <!-- Scan Button -->
          <div class="scan-actions">
            <button
              class="btn btn-primary scan-btn"
              @click="handleScan"
              :disabled="!canScan"
            >
              <i class="fa-solid fa-shield"></i>
              {{ scanning ? 'Scanning…' : 'Scan Image' }}
            </button>
            <button
              v-if="selectedImage"
              class="btn btn-secondary"
              @click="toggleHistory"
              :disabled="scanning"
            >
              <i class="fa-solid fa-clock-rotate-left"></i> History
            </button>
          </div>
        </div>
      </div>

      <!-- Scan History panel (inline, shown on toggle) -->
      <div v-if="showHistory" class="section scan-history-section" style="margin-top:12px;">
        <div class="section-header">
          <span class="section-title"><i class="fa-solid fa-clock-rotate-left"></i> Scan History</span>
          <button class="header-btn" @click="showHistory = false"><i class="fa-solid fa-xmark"></i></button>
        </div>
        <div v-if="loadingHistory" class="history-loading">
          <i class="fa-solid fa-spinner fa-spin"></i> Loading…
        </div>
        <div v-else-if="scanHistory.length === 0" class="history-empty">
          No previous scans for this image.
        </div>
        <div v-else class="history-list">
          <div v-for="(entry, i) in scanHistory" :key="i" class="history-entry">
            <div class="history-date">{{ formatTimestamp(entry.scanned_at) }}</div>
            <div class="history-summary">
              <span class="total-badge">{{ entry.total }} CVEs</span>
              <span v-if="entry.critical.length" class="sev-badge sev-critical">{{ entry.critical.length }} critical</span>
              <span v-if="entry.high.length" class="sev-badge sev-high">{{ entry.high.length }} high</span>
              <span v-if="entry.medium.length" class="sev-badge sev-medium">{{ entry.medium.length }} medium</span>
              <span v-if="entry.low.length" class="sev-badge sev-low">{{ entry.low.length }} low</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ─── Right Panel: Results ─── -->
    <div class="scanner-results">
      <!-- Loading state (A.6.1) -->
      <div v-if="scanning" class="section scanning-panel">
        <div class="section-header">
          <span class="section-title"><i class="fa-solid fa-shield fa-beat-fade"></i> Scanning</span>
        </div>
        <div class="scan-progress-body">
          <!-- Progress bar -->
          <div class="scan-progress-bar-container">
            <div class="scan-progress-bar">
              <div
                class="scan-progress-fill"
                :class="{ indeterminate: progressStep === 'scan' }"
              ></div>
            </div>
          </div>
          <!-- Status message -->
          <div class="scan-status-message">
            <i class="fa-solid fa-spinner fa-spin"></i>
            <span>{{ progressMessage || 'Initializing scanner…' }}</span>
          </div>
          <!-- Animated placeholder -->
          <div class="scanning-placeholder">
            <i class="fa-solid fa-shield-halved"></i>
            <p>Scanning <strong>{{ selectedImage }}</strong> for known vulnerabilities.</p>
            <p class="placeholder-hint">This may take a few moments depending on image size.</p>
          </div>
        </div>
      </div>

      <!-- Error state (A.6.3) -->
      <ErrorState
        v-if="scanError && !scanning"
        :message="'Scan Failed'"
        :suggestion="'Check that the image exists and the scanner is working properly.'"
        :detail="scanError"
        @retry="handleScan"
      />

      <!-- Empty / No vulnerabilities found (A.6.2) -->
      <EmptyState
        v-if="currentReport && currentReport.total === 0 && !scanning && !scanError"
        icon="fa-solid fa-circle-check"
        title="No vulnerabilities detected! 🎉"
        :description="`${currentReport.image_name} appears to have no known CVEs at this time.`"
      />

      <!-- Results Dashboard -->
      <template v-if="currentReport && currentReport.total > 0 && !scanning && !scanError">
        <!-- Summary Cards -->
        <div class="metrics-grid vuln-metrics">
          <div class="metric-card vuln-metric total-metric">
            <div class="metric-icon cyan"><i class="fa-solid fa-shield"></i></div>
            <div class="metric-label">Total CVEs</div>
            <div class="metric-value">{{ safeCount(currentReport.total) }}</div>
          </div>
          <div class="metric-card vuln-metric">
            <div class="metric-icon" style="background:rgba(239,68,68,0.1);color:var(--accent-red);">
              <i class="fa-solid fa-bug"></i>
            </div>
            <div class="metric-label">Critical</div>
            <div class="metric-value" style="color:var(--accent-red)">{{ safeCount(currentReport.critical.length) }}</div>
          </div>
          <div class="metric-card vuln-metric">
            <div class="metric-icon" style="background:rgba(245,158,11,0.1);color:var(--accent-yellow);">
              <i class="fa-solid fa-exclamation"></i>
            </div>
            <div class="metric-label">High</div>
            <div class="metric-value" style="color:var(--accent-yellow)">{{ safeCount(currentReport.high.length) }}</div>
          </div>
          <div class="metric-card vuln-metric">
            <div class="metric-icon" style="background:rgba(59,130,246,0.1);color:var(--accent-blue);">
              <i class="fa-solid fa-circle-info"></i>
            </div>
            <div class="metric-label">Medium</div>
            <div class="metric-value" style="color:var(--accent-blue)">{{ safeCount(currentReport.medium.length) }}</div>
          </div>
          <div class="metric-card vuln-metric">
            <div class="metric-icon" style="background:rgba(156,163,175,0.08);color:var(--text-muted);">
              <i class="fa-solid fa-flag"></i>
            </div>
            <div class="metric-label">Low</div>
            <div class="metric-value" style="color:var(--text-muted)">{{ safeCount(currentReport.low.length) }}</div>
          </div>
        </div>

        <!-- Severity Bar Chart -->
        <div class="section">
          <div class="section-header">
            <span class="section-title">Severity Distribution</span>
          </div>
          <div class="severity-chart-body">
            <div
              v-for="item in severityBreakdown"
              :key="item.key"
              class="severity-bar-row"
            >
              <span class="severity-bar-label">{{ item.label }}</span>
              <div class="severity-bar-track">
                <div
                  class="severity-bar-fill"
                  :style="{
                    width: (item.count / maxSeverityCount * 100) + '%',
                    background: item.color,
                  }"
                ></div>
              </div>
              <span class="severity-bar-count">{{ safeCount(item.count) }}</span>
            </div>
          </div>
        </div>

        <!-- Vulnerability Table -->
        <div class="section vuln-table-section">
          <div class="section-header">
            <span class="section-title">Vulnerabilities ({{ filteredVulns.length }})</span>
            <div class="table-toolbar">
              <!-- Severity Filter -->
              <div class="table-filter">
                <i class="fa-solid fa-filter"></i>
                <select v-model="severityFilter" class="filter-select">
                  <option value="all">All Severities</option>
                  <option value="critical">Critical</option>
                  <option value="high">High</option>
                  <option value="medium">Medium</option>
                  <option value="low">Low</option>
                </select>
              </div>
              <!-- Package Search -->
              <div class="table-filter">
                <i class="fa-solid fa-search"></i>
                <input
                  v-model="packageSearch"
                  placeholder="Search CVE, package, title…"
                />
              </div>
            </div>
          </div>

          <!-- Table Header -->
          <div class="col-headers">
            <span class="col-cve">CVE ID</span>
            <span class="col-package">Package</span>
            <span class="col-version">Installed</span>
            <span class="col-fix">Fixed</span>
            <span class="col-severity">Severity</span>
            <span class="col-title">Title</span>
          </div>

          <!-- Table Rows -->
          <div
            v-for="vuln in filteredVulns"
            :key="vuln.id + vuln.package"
            class="vuln-row-wrapper"
          >
            <div
              class="vuln-row"
              :class="{ expanded: expandedVuln === vuln.id + vuln.package }"
              @click="toggleExpand(vuln.id + vuln.package)"
            >
              <span class="col-cve mono">{{ vuln.id }}</span>
              <span class="col-package">{{ vuln.package }}</span>
              <span class="col-version mono">{{ vuln.installed_version }}</span>
              <span class="col-fix">
                <span v-if="vuln.fixed_version" class="fix-badge" title="Fix available">
                  <i class="fa-solid fa-arrow-up"></i> {{ vuln.fixed_version }}
                </span>
                <span v-else class="no-fix">—</span>
              </span>
              <span class="col-severity">
                <span :class="['severity-badge', severityBadgeClass(vuln.severity)]">
                  {{ vuln.severity }}
                </span>
              </span>
              <span class="col-title">{{ vuln.title || vuln.id }}</span>
              <span class="col-expand">
                <i
                  :class="expandedVuln === vuln.id + vuln.package ? 'fa-solid fa-chevron-up' : 'fa-solid fa-chevron-down'"
                ></i>
              </span>
            </div>

            <!-- Expanded Detail -->
            <div v-if="expandedVuln === vuln.id + vuln.package" class="vuln-detail">
              <div class="detail-section">
                <span class="detail-label">Description</span>
                <p class="detail-text">{{ vuln.description || 'No description available.' }}</p>
              </div>
              <div v-if="vuln.fixed_version" class="detail-section">
                <span class="detail-label fix-label">
                  <i class="fa-solid fa-wrench"></i> Suggested Fix
                </span>
                <p class="detail-text fix-text">Upgrade <strong>{{ vuln.package }}</strong> to version <code class="fix-version">{{ vuln.fixed_version }}</code></p>
              </div>
              <div class="detail-actions">
                <a
                  :href="nvdLink(vuln.id)"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="btn btn-secondary btn-sm"
                >
                  <i class="fa-solid fa-up-right-from-square"></i> View on NVD
                </a>
              </div>
            </div>
          </div>

          <!-- No results matching filters -->
          <div v-if="filteredVulns.length === 0 && allVulns.length > 0" class="no-filter-results">
            <i class="fa-solid fa-filter-circle-xmark"></i>
            <span>No vulnerabilities match the current filters.</span>
          </div>
        </div>

        <!-- Scan info footer -->
        <div class="scan-footer">
          <i class="fa-solid fa-circle-info"></i>
          Scanned <strong>{{ currentReport.image_name }}</strong> at {{ formatTimestamp(currentReport.scanned_at) }}
          &middot; {{ currentReport.total }} total vulnerabilities
        </div>
      </template>

      <!-- Idle state (no scan performed yet) -->
      <div v-if="!currentReport && !scanning && !scanError" class="section idle-panel">
        <div class="idle-body">
          <i class="fa-solid fa-shield-halved idle-icon"></i>
          <h3>Ready to Scan</h3>
          <p>Select an image from the left panel and click <strong>Scan Image</strong> to check for known vulnerabilities (CVEs).</p>
          <p class="idle-hint">Scans use Trivy or Grype to detect security issues in your container images.</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ─── Split Layout ─── */
.scanner-split {
  display: flex;
  gap: 16px;
  flex: 1;
  min-height: 0;
  height: calc(100vh - var(--header-height) - 100px);
}

.scanner-config {
  flex: 0 0 340px;
  max-width: 340px;
  overflow-y: auto;
}

.scanner-results {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
}

/* ─── Config Body ─── */
.scanner-config-body {
  padding: 20px;
}

.scanner-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
}

.scanner-status.detecting {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.scanner-status.detected {
  background: rgba(16, 185, 129, 0.08);
  color: var(--accent-green);
  border: 1px solid rgba(16, 185, 129, 0.15);
}

.scanner-status.not-found {
  background: rgba(239, 68, 68, 0.06);
  color: var(--accent-red);
  border: 1px solid rgba(239, 68, 68, 0.15);
}

/* ─── Install Instructions ─── */
.install-instructions {
  margin-top: 8px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.code-block {
  background: #000;
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.5;
}

/* ─── Image Select ─── */
.image-select-wrapper {
  position: relative;
}

.image-select {
  appearance: none;
  -webkit-appearance: none;
  padding-right: 32px;
  cursor: pointer;
}

.select-chevron {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  font-size: 11px;
  pointer-events: none;
}

/* ─── Scan Actions ─── */
.scan-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.scan-btn {
  flex: 1;
  justify-content: center;
}

/* ─── Scan History ─── */
.scan-history-section {
  max-height: 300px;
  overflow-y: auto;
}

.history-loading,
.history-empty {
  padding: 20px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}

.history-list {
  padding: 8px 0;
}

.history-entry {
  padding: 10px 20px;
  border-bottom: 1px solid var(--border-light);
  transition: background var(--transition-fast);
}

.history-entry:last-child {
  border-bottom: none;
}

.history-entry:hover {
  background: var(--bg-hover);
}

.history-date {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.history-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.total-badge {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-main);
  font-family: var(--font-mono);
}

.sev-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 3px;
}

.sev-badge.sev-critical {
  background: rgba(239, 68, 68, 0.1);
  color: var(--accent-red);
}

.sev-badge.sev-high {
  background: rgba(245, 158, 11, 0.1);
  color: var(--accent-yellow);
}

.sev-badge.sev-medium {
  background: rgba(59, 130, 246, 0.1);
  color: var(--accent-blue);
}

.sev-badge.sev-low {
  background: rgba(156, 163, 175, 0.08);
  color: var(--text-muted);
}

/* ─── Progress Panel ─── */
.scanning-panel {
  display: flex;
  flex-direction: column;
}

.scan-progress-body {
  padding: 24px 20px;
}

.scan-progress-bar-container {
  margin-bottom: 16px;
}

.scan-progress-bar {
  height: 4px;
  background: var(--bg-hover);
  border-radius: 2px;
  overflow: hidden;
}

.scan-progress-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

.scan-progress-fill.indeterminate {
  width: 30%;
  background: linear-gradient(90deg, var(--accent-cyan), var(--accent-purple));
  animation: progressIndeterminate 1.5s ease-in-out infinite;
}

@keyframes progressIndeterminate {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(400%); }
}

.scan-status-message {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--accent-cyan);
  margin-bottom: 20px;
}

.scanning-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 20px;
  color: var(--text-muted);
}

.scanning-placeholder i {
  font-size: 40px;
  opacity: 0.2;
  margin-bottom: 12px;
}

.scanning-placeholder p {
  font-size: 13px;
  color: var(--text-disabled);
}

.placeholder-hint {
  font-size: 11px !important;
  margin-top: 4px;
  opacity: 0.6;
}

/* ─── Idle State ─── */
.idle-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
}

.idle-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
}

.idle-icon {
  font-size: 48px;
  opacity: 0.15;
  margin-bottom: 16px;
}

.idle-body h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 8px;
}

.idle-body p {
  font-size: 13px;
  max-width: 400px;
  line-height: 1.5;
  color: var(--text-disabled);
}

.idle-hint {
  font-size: 11px !important;
  margin-top: 8px;
  opacity: 0.6;
}

/* ─── Metric Cards Grid ─── */
.vuln-metrics {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
}

.vuln-metric {
  padding: 16px;
}

.vuln-metric .metric-value {
  font-size: 1.5rem;
}

/* ─── Severity Bar Chart ─── */
.severity-chart-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.severity-bar-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.severity-bar-label {
  flex: 0 0 70px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  text-align: right;
}

.severity-bar-track {
  flex: 1;
  height: 20px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  overflow: hidden;
}

.severity-bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
  min-width: 2px;
}

.severity-bar-count {
  flex: 0 0 40px;
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
  text-align: right;
}

/* ─── Vulnerability Table ─── */
.vuln-table-section {
  display: flex;
  flex-direction: column;
}

/* Table column sizes */
.col-headers .col-cve,
.vuln-row .col-cve {
  flex: 0 0 140px;
}

.col-headers .col-package,
.vuln-row .col-package {
  flex: 0 0 120px;
}

.col-headers .col-version,
.vuln-row .col-version {
  flex: 0 0 90px;
}

.col-headers .col-fix,
.vuln-row .col-fix {
  flex: 0 0 110px;
}

.col-headers .col-severity,
.vuln-row .col-severity {
  flex: 0 0 80px;
}

.col-headers .col-title,
.vuln-row .col-title {
  flex: 1;
  min-width: 0;
}

.col-headers .col-expand,
.vuln-row .col-expand {
  flex: 0 0 24px;
  text-align: center;
}

/* Vuln row */
.vuln-row-wrapper {
  border-bottom: 1px solid var(--border-light);
}

.vuln-row-wrapper:last-child {
  border-bottom: none;
}

.vuln-row {
  display: flex;
  align-items: center;
  padding: 10px 20px;
  gap: 12px;
  cursor: pointer;
  transition: background var(--transition-fast);
  font-size: 13px;
}

.vuln-row:hover {
  background: var(--bg-hover);
}

.vuln-row.expanded {
  background: var(--bg-tertiary);
}

/* Filter select */
.filter-select {
  background: transparent;
  border: none;
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
  outline: none;
  cursor: pointer;
}

.filter-select option {
  background: var(--bg-secondary);
  color: var(--text-main);
}

/* Mono text */
.mono {
  font-family: var(--font-mono);
  font-size: 11px;
}

/* Fix badge */
.fix-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.no-fix {
  color: var(--text-disabled);
  font-size: 12px;
}

/* Severity badges */
.severity-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.badge-critical {
  background: rgba(239, 68, 68, 0.15);
  color: var(--accent-red);
  border: 1px solid rgba(239, 68, 68, 0.25);
}

.badge-high {
  background: rgba(245, 158, 11, 0.15);
  color: var(--accent-yellow);
  border: 1px solid rgba(245, 158, 11, 0.25);
}

.badge-medium {
  background: rgba(59, 130, 246, 0.15);
  color: var(--accent-blue);
  border: 1px solid rgba(59, 130, 246, 0.25);
}

.badge-low {
  background: rgba(156, 163, 175, 0.1);
  color: var(--text-muted);
  border: 1px solid rgba(156, 163, 175, 0.15);
}

/* Expanded detail row */
.vuln-detail {
  padding: 16px 20px 16px 172px; /* offset to align with content after CVE+Package */
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border-light);
  animation: fadeIn 0.2s ease-out;
}

.detail-section {
  margin-bottom: 12px;
}

.detail-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: var(--text-disabled);
  margin-bottom: 4px;
  display: block;
}

.detail-text {
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-muted);
  max-width: 600px;
}

.fix-label {
  color: var(--accent-green);
}

.fix-text {
  color: var(--text-main);
}

.fix-version {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
  padding: 1px 6px;
  border-radius: 3px;
  font-family: var(--font-mono);
  font-size: 12px;
}

.detail-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

/* No filter results */
.no-filter-results {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 24px 20px;
  color: var(--text-disabled);
  font-size: 13px;
}

/* ─── Scan Footer ─── */
.scan-footer {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-disabled);
  padding: 8px 4px;
}

.scan-footer i {
  font-size: 12px;
  opacity: 0.5;
}

/* ─── Custom animation ─── */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
