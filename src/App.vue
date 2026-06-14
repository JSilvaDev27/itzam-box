<!-- ItzamBox — Root Application Shell
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TerminalPanel from './components/terminal/TerminalPanel.vue'
import ContextMenu from './components/shared/ContextMenu.vue'
import CommandPalette from './components/shared/CommandPalette.vue'
import OnboardingWizard from './components/onboarding/OnboardingWizard.vue'
import PageTransitionWrapper from './components/shared/PageTransitionWrapper.vue'
import { useTheme } from './composables/useTheme'
import { useI18n } from './composables/useI18n'
import { useNotifications } from './composables/useNotifications'
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts'
import { useDocker, relativeTime } from './composables/useDocker'

const router = useRouter()
const { isDark, init: initTheme, toggleTheme } = useTheme()
const { t, init: initI18n } = useI18n()
const {
  toasts,
  unreadCount,
  dismissToast,
  notifications,
  loadNotifications,
  markRead,
  markAllRead,
  clearAll,
} = useNotifications()

const sidebarCollapsed = ref(false)
const showOnboarding = ref(false)
const showNotificationsPanel = ref(false)

const { hostMetrics } = useDocker()
const memPct = computed(() => {
  if (!hostMetrics.value?.memory_total_bytes) return 0
  return (hostMetrics.value.memory_used_bytes / hostMetrics.value.memory_total_bytes) * 100
})
const memPctDisplay = computed(() => {
  if (!hostMetrics.value?.memory_total_bytes) return '--'
  return memPct.value.toFixed(1) + '%'
})
useKeyboardShortcuts()

// Periodically refresh host metrics for sidebar indicators
let metricsInterval: number | undefined

onMounted(async () => {
  await initTheme()
  await initI18n()
  await loadNotifications()
  metricsInterval = window.setInterval(() => {
    invoke('get_host_metrics').then((m: any) => {
      hostMetrics.value = m
    }).catch(() => {})
  }, 5000)

  onUnmounted(() => {
    if (metricsInterval) clearInterval(metricsInterval)
  })

  // Check if onboarding has been completed
  try {
    const completed = await invoke<string>('get_config', { key: 'onboarding_completed' })
    if (completed !== 'true') {
      showOnboarding.value = true
    }
  } catch {
    // First run or config unavailable — show onboarding
    showOnboarding.value = true
  }
})

function toggleNotificationsPanel() {
  showNotificationsPanel.value = !showNotificationsPanel.value
  if (showNotificationsPanel.value) {
    loadNotifications()
  }
}

const sidebarGroups = [
  {
    label: 'Core',
    collapsed: false,
    items: [
      { to: '/', icon: 'fa-chart-line', label: 'Dashboard' },
      { to: '/containers', icon: 'fa-cubes', label: 'Containers' },
      { to: '/images', icon: 'fa-layer-group', label: 'Images' },
      { to: '/volumes', icon: 'fa-database', label: 'Volumes' },
      { to: '/networks', icon: 'fa-network-wired', label: 'Networks' },
    ],
  },
  {
    label: 'Orchestration',
    collapsed: true,
    items: [
      { to: '/compose', icon: 'fa-layer-group', label: 'Compose' },
      { to: '/swarm', icon: 'fa-bees', label: 'Swarm' },
      { to: '/kubernetes', icon: 'fa-ship', label: 'Kubernetes' },
    ],
  },
  {
    label: 'Operations',
    collapsed: false,
    items: [
      { to: '/backup', icon: 'fa-box-archive', label: 'Backup' },
      { to: '/events', icon: 'fa-clock-rotate-left', label: 'Events' },
      { to: '/cleanup', icon: 'fa-broom', label: 'Cleanup' },
    ],
  },
  {
    label: 'System',
    collapsed: false,
    items: [
      { to: '/build', icon: 'fa-hammer', label: 'Build' },
      { to: '/registries', icon: 'fa-server', label: 'Registries' },
      { to: '/run-wizard', icon: 'fa-wand-magic-sparkles', label: 'Run Wizard' },
      { to: '/templates', icon: 'fa-copy', label: 'Templates' },
      { to: '/installer', icon: 'fa-docker', label: 'Docker Setup' },
      { to: '/metrics', icon: 'fa-activity', label: 'Metrics' },
      { to: '/export-import', icon: 'fa-file-export', label: 'Export/Import' },
    ],
  },
]
</script>

<template>
  <div class="app-shell">
    <!-- Header -->
    <header class="app-header">
      <div class="header-brand">
        <div class="header-logo">IB</div>
        <span class="header-title">ItzamBox</span>
      </div>
      <div class="header-right">
        <div class="search-bar">
          <i class="fa-solid fa-magnifying-glass"></i>
          <input type="text" :placeholder="t.common.search || 'Search...'">
        </div>
        <div style="position:relative">
          <button class="header-btn" title="Notifications" @click="toggleNotificationsPanel">
            <i class="fa-solid fa-bell"></i>
            <span v-if="unreadCount > 0" class="badge">{{ unreadCount }}</span>
          </button>

          <!-- Persisted Notifications Center Dropdown -->
          <div v-if="showNotificationsPanel" class="notifications-panel">
            <div class="panel-header">
              <span>Notifications ({{ unreadCount }} unread)</span>
              <div class="panel-actions">
                <button @click="markAllRead" class="action-btn-text">Mark all read</button>
                <button @click="clearAll" class="action-btn-text delete">Clear all</button>
              </div>
            </div>
            <div class="panel-content">
              <div v-if="notifications.length === 0" class="panel-empty">
                <i class="fa-solid fa-bell-slash"></i>
                <p>No notifications yet</p>
              </div>
              <div
                v-for="item in notifications.slice(0, 50)"
                :key="item.id"
                class="notification-item"
                :class="{ 'unread': !item.read }"
                @click="markRead(item.id)"
              >
                <div class="item-icon-wrapper">
                  <i
                    :class="
                      item.type === 'success'
                        ? 'fa-solid fa-circle-check text-success'
                        : item.type === 'error'
                        ? 'fa-solid fa-circle-xmark text-error'
                        : item.type === 'warning'
                        ? 'fa-solid fa-triangle-exclamation text-warning'
                        : 'fa-solid fa-circle-info text-info'
                    "
                  ></i>
                </div>
                <div class="item-body">
                  <div class="item-title">{{ item.title }}</div>
                  <div class="item-message">{{ item.message }}</div>
                  <div class="item-time">{{ relativeTime(Math.floor(item.timestamp / 1000)) }}</div>
                </div>
                <div class="item-indicator" v-if="!item.read"></div>
              </div>
            </div>
          </div>
        </div>
        <button class="header-btn" @click="toggleTheme" title="Toggle theme">
          <i :class="isDark ? 'fa-solid fa-moon' : 'fa-solid fa-sun'"></i>
        </button>
        <button class="header-btn" title="Settings" @click="router.push('/settings')">
          <i class="fa-solid fa-gear"></i>
        </button>
      </div>
    </header>

    <!-- Main Layout -->
    <div class="main-layout">
      <aside :class="['sidebar', { collapsed: sidebarCollapsed }]">
        <nav class="sidebar-nav">
          <template v-for="group in sidebarGroups" :key="group.label">
            <div class="nav-section-label" v-show="!sidebarCollapsed">{{ group.label }}</div>
            <router-link
              v-for="item in group.items"
              :key="item.to"
              :to="item.to"
              class="nav-item"
              :class="{ 'nav-item--collapsed-hint': sidebarCollapsed }"
            >
              <i :class="'fa-solid ' + item.icon"></i>
              <span class="nav-label" v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </template>
          <div class="nav-divider"></div>
          <router-link to="/settings" class="nav-item" :class="{ 'nav-item--collapsed-hint': sidebarCollapsed }">
            <i class="fa-solid fa-gear"></i>
            <span class="nav-label" v-show="!sidebarCollapsed">Settings</span>
          </router-link>
          <router-link to="/help" class="nav-item" :class="{ 'nav-item--collapsed-hint': sidebarCollapsed }">
            <i class="fa-solid fa-circle-question"></i>
            <span class="nav-label" v-show="!sidebarCollapsed">Help</span>
          </router-link>
        </nav>
        <div class="sidebar-widgets">
          <div class="host-widget">
            <div class="host-widget-label">CPU</div>
            <div class="host-widget-value" :style="{ color: (hostMetrics?.cpu_usage_percent ?? 0) > 80 ? 'var(--accent-red)' : 'var(--accent-green)' }">
              {{ hostMetrics?.cpu_usage_percent != null ? (hostMetrics.cpu_usage_percent.toFixed(1) + '%') : '--' }}
            </div>
            <div class="host-widget-bar"><div class="host-widget-bar-fill" :style="{ width: Math.min(hostMetrics?.cpu_usage_percent ?? 0, 100) + '%', background: (hostMetrics?.cpu_usage_percent ?? 0) > 80 ? 'var(--accent-red)' : 'var(--accent-green)' }"></div></div>
          </div>
          <div class="host-widget">
            <div class="host-widget-label">RAM</div>
            <div class="host-widget-value" :style="{ color: (memPct) > 80 ? 'var(--accent-red)' : 'var(--accent-green)' }">
              {{ memPctDisplay }}
            </div>
            <div class="host-widget-bar"><div class="host-widget-bar-fill" :style="{ width: Math.min(memPct, 100) + '%', background: memPct > 80 ? 'var(--accent-red)' : 'var(--accent-green)' }"></div></div>
          </div>
        </div>
      </aside>

      <main class="content">
        <router-view v-slot="{ Component }">
          <PageTransitionWrapper>
            <component :is="Component" />
          </PageTransitionWrapper>
        </router-view>
      </main>
    </div>

    <!-- Terminal Panel -->
    <TerminalPanel />

    <!-- Toast Notifications -->
    <div style="position:fixed;top:60px;right:16px;z-index:200;display:flex;flex-direction:column;gap:8px;max-width:360px">
      <div v-for="toast in toasts" :key="toast.id"
        :style="{
          padding:'12px 16px', borderRadius:'var(--radius-md)',
          background: 'var(--bg-secondary)', border:'1px solid var(--border-color)',
          boxShadow:'var(--shadow-lg)', fontSize:'13px',
          borderLeft: '3px solid ' + (toast.type === 'success' ? 'var(--accent-green)' : toast.type === 'error' ? 'var(--accent-red)' : toast.type === 'warning' ? 'var(--accent-yellow)' : 'var(--accent-blue)'),
          animation:'fadeIn 0.3s ease-out',
        }"
      >
        <div style="display:flex;align-items:center;justify-content:space-between;gap:12px">
          <div>
            <i :class="toast.type === 'success' ? 'fa-solid fa-circle-check' : toast.type === 'error' ? 'fa-solid fa-circle-xmark' : toast.type === 'warning' ? 'fa-solid fa-triangle-exclamation' : 'fa-solid fa-circle-info'"
              :style="{color: toast.type === 'success' ? 'var(--accent-green)' : toast.type === 'error' ? 'var(--accent-red)' : toast.type === 'warning' ? 'var(--accent-yellow)' : 'var(--accent-blue)', marginRight:'8px'}"></i>
            <strong>{{ toast.title }}</strong>
            <span style="color:var(--text-muted);marginLeft:8px">{{ toast.message }}</span>
          </div>
          <button @click="dismissToast(toast.id)" style="background:none;border:none;color:var(--text-muted);cursor:pointer;fontSize:14px">&times;</button>
        </div>
      </div>
    </div>

    <!-- Global Context Menu -->
    <ContextMenu />
    <!-- Command Palette (Ctrl+K) -->
    <CommandPalette />

    <!-- Onboarding Wizard (first run) -->
    <OnboardingWizard v-if="showOnboarding" @done="showOnboarding = false" />
  </div>
</template>

<style>
/* Global style block to define notification center drop-down */
.notifications-panel {
  position: absolute;
  top: 45px;
  right: 0;
  width: 320px;
  max-height: 400px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  z-index: 100;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-main);
}

.panel-actions {
  display: flex;
  gap: 8px;
}

.action-btn-text {
  background: none;
  border: none;
  color: var(--accent-cyan);
  font-size: 11px;
  cursor: pointer;
  padding: 2px 4px;
}

.action-btn-text.delete {
  color: var(--accent-red);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.panel-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-muted);
}

.panel-empty i {
  font-size: 24px;
  margin-bottom: 8px;
}

.panel-empty p {
  margin: 0;
  font-size: 12px;
}

.notification-item {
  display: flex;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-light);
  cursor: pointer;
  transition: background var(--transition-fast);
  position: relative;
}

.notification-item:hover {
  background: var(--bg-hover);
}

.notification-item.unread {
  background: var(--bg-tertiary);
}

.item-icon-wrapper {
  margin-right: 10px;
  margin-top: 2px;
  font-size: 14px;
}

.item-body {
  flex: 1;
  min-width: 0;
}

.item-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 2px;
}

.item-message {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.4;
  margin-bottom: 4px;
  word-break: break-all;
}

.item-time {
  font-size: 9px;
  color: var(--text-disabled);
}

.item-indicator {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 6px;
  height: 6px;
  background: var(--accent-cyan);
  border-radius: 50%;
}

.text-success { color: var(--accent-green); }
.text-error { color: var(--accent-red); }
.text-warning { color: var(--accent-yellow); }
.text-info { color: var(--accent-blue); }
</style>
