<!-- ItzamBox — Root Application Shell
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TerminalPanel from './components/terminal/TerminalPanel.vue'
import ContextMenu from './components/shared/ContextMenu.vue'
import CommandPalette from './components/shared/CommandPalette.vue'
import OnboardingWizard from './components/onboarding/OnboardingWizard.vue'
import { useTheme } from './composables/useTheme'
import { useI18n } from './composables/useI18n'
import { useNotifications } from './composables/useNotifications'
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts'

const router = useRouter()
const { isDark, init: initTheme, toggleTheme } = useTheme()
const { t, init: initI18n } = useI18n()
const { toasts, unreadCount, dismissToast } = useNotifications()
const sidebarCollapsed = ref(false)
const showOnboarding = ref(false)

useKeyboardShortcuts()

onMounted(async () => {
  await initTheme()
  await initI18n()

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

const navItems = [
  { to: '/', icon: 'fa-chart-line', label: 'Dashboard' },
  { to: '/containers', icon: 'fa-cubes', label: 'Containers' },
  { to: '/images', icon: 'fa-layer-group', label: 'Images' },
  { to: '/build', icon: 'fa-hammer', label: 'Build' },
  { to: '/volumes', icon: 'fa-database', label: 'Volumes' },
  { to: '/networks', icon: 'fa-network-wired', label: 'Networks' },
  { to: '/events', icon: 'fa-clock-rotate-left', label: 'Events' },
  { to: '/cleanup', icon: 'fa-broom', label: 'Cleanup' },
  { to: '/compose', icon: 'fa-layer-group', label: 'Compose' },
  { to: '/registries', icon: 'fa-server', label: 'Registries' },
  { to: '/run-wizard', icon: 'fa-wand-magic-sparkles', label: 'Run Wizard' },
  { to: '/templates', icon: 'fa-copy', label: 'Templates' },
  { to: '/installer', icon: 'fa-docker', label: 'Docker Setup' },
  { to: '/export-import', icon: 'fa-file-export', label: 'Export/Import' },
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
        <button class="header-btn" title="Notifications" style="position:relative">
          <i class="fa-solid fa-bell"></i>
          <span v-if="unreadCount > 0" class="badge">{{ unreadCount }}</span>
        </button>
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
          <router-link v-for="item in navItems" :key="item.to" :to="item.to" class="nav-item">
            <i :class="'fa-solid ' + item.icon"></i>
            <span class="nav-label" v-show="!sidebarCollapsed">{{ item.label }}</span>
          </router-link>
          <div class="nav-divider"></div>
          <router-link to="/settings" class="nav-item">
            <i class="fa-solid fa-gear"></i>
            <span class="nav-label" v-show="!sidebarCollapsed">Settings</span>
          </router-link>
          <router-link to="/help" class="nav-item">
            <i class="fa-solid fa-circle-question"></i>
            <span class="nav-label" v-show="!sidebarCollapsed">Help</span>
          </router-link>
        </nav>
        <div class="sidebar-widgets">
          <div class="host-widget">
            <div class="host-widget-label">CPU</div>
            <div class="host-widget-value" style="color:var(--accent-green)">--</div>
            <div class="host-widget-bar"><div class="host-widget-bar-fill" style="width:0%"></div></div>
          </div>
          <div class="host-widget">
            <div class="host-widget-label">RAM</div>
            <div class="host-widget-value" style="color:var(--accent-green)">--</div>
            <div class="host-widget-bar"><div class="host-widget-bar-fill" style="width:0%"></div></div>
          </div>
        </div>
      </aside>

      <main class="content">
        <router-view />
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
