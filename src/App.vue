<!-- ItzamBox — Local and open-source alternative to Docker Desktop -->
<!-- Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import TerminalPanel from './components/terminal/TerminalPanel.vue'

const router = useRouter()
const sidebarCollapsed = ref(false)
const isDark = ref(true)

function toggleSidebar() { sidebarCollapsed.value = !sidebarCollapsed.value }
function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}

const navItems = [
  { to: '/', icon: 'fa-chart-line', label: 'Dashboard' },
  { to: '/containers', icon: 'fa-cubes', label: 'Containers' },
  { to: '/images', icon: 'fa-layer-group', label: 'Images' },
  { to: '/volumes', icon: 'fa-database', label: 'Volumes' },
  { to: '/networks', icon: 'fa-network-wired', label: 'Networks' },
  { to: '/events', icon: 'fa-clock-rotate-left', label: 'Events' },
  { to: '/cleanup', icon: 'fa-broom', label: 'Cleanup' },
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
          <input type="text" placeholder="Search... (Ctrl+K)">
        </div>
        <button class="header-btn" title="Notifications">
          <i class="fa-solid fa-bell"></i>
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
      <!-- Sidebar -->
      <aside :class="['sidebar', { collapsed: sidebarCollapsed }]">
        <nav class="sidebar-nav">
          <router-link
            v-for="item in navItems" :key="item.to"
            :to="item.to" class="nav-item"
          >
            <i :class="'fa-solid ' + item.icon"></i>
            <span class="nav-label">{{ sidebarCollapsed ? '' : item.label }}</span>
          </router-link>
          <div class="nav-divider"></div>
          <router-link to="/settings" class="nav-item">
            <i class="fa-solid fa-gear"></i>
            <span class="nav-label">{{ sidebarCollapsed ? '' : 'Settings' }}</span>
          </router-link>
          <router-link to="/help" class="nav-item">
            <i class="fa-solid fa-circle-question"></i>
            <span class="nav-label">{{ sidebarCollapsed ? '' : 'Help' }}</span>
          </router-link>
        </nav>
        <!-- Host widgets -->
        <div class="sidebar-widgets">
          <div class="host-widget">
            <div class="host-widget-label">CPU</div>
            <div class="host-widget-value" style="color: var(--accent-green)">--</div>
            <div class="host-widget-bar"><div class="host-widget-bar-fill" style="width:0%"></div></div>
          </div>
          <div class="host-widget">
            <div class="host-widget-label">RAM</div>
            <div class="host-widget-value" style="color: var(--accent-green)">--</div>
            <div class="host-widget-bar"><div class="host-widget-bar-fill" style="width:0%"></div></div>
          </div>
        </div>
      </aside>

      <!-- Content -->
      <main class="content">
        <router-view />
      </main>
    </div>

    <!-- Terminal Panel -->
    <TerminalPanel />
  </div>
</template>
