/* ═══════════════════════════════════════════
   ItzamBox — Main Entry Point
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import { initSentry } from './lib/sentry'

/* ─── Styles (order matters: tokens → layout → components → pages) ─── */
import './styles/theme.css'
import './styles/layout.css'
import './styles/components.css'
import './styles/animations.css'

/* ─── FontAwesome ─── */
import '@fortawesome/fontawesome-free/css/all.min.css'

/* ─── Routes ─── */
const routes = [
  { path: '/', name: 'Dashboard', component: () => import('./views/Dashboard.vue') },
  { path: '/containers', name: 'Containers', component: () => import('./views/Containers.vue') },
  { path: '/containers/:id', name: 'ContainerDetail', component: () => import('./views/ContainerDetail.vue') },
  { path: '/images', name: 'Images', component: () => import('./views/Images.vue') },
  { path: '/images/:id/layers', name: 'ImageLayers', component: () => import('./views/ImageLayers.vue') },
  { path: '/build', name: 'ImageBuild', component: () => import('./views/ImageBuild.vue') },
  { path: '/volumes', name: 'Volumes', component: () => import('./views/Volumes.vue') },
  { path: '/networks', name: 'Networks', component: () => import('./views/Networks.vue') },
  { path: '/events', name: 'Events', component: () => import('./views/Events.vue') },
  { path: '/cleanup', name: 'Cleanup', component: () => import('./views/Cleanup.vue') },
  { path: '/settings', name: 'Settings', component: () => import('./views/Settings.vue') },
  { path: '/help', name: 'Help', component: () => import('./views/Help.vue') },
  { path: '/installer', name: 'Installer', component: () => import('./views/Installer.vue') },
  { path: '/containers/:id/files', name: 'FileExplorer', component: () => import('./views/FileExplorer.vue'), props: true },
  { path: '/compose', name: 'ComposeList', component: () => import('./views/compose/ComposeList.vue') },
  { path: '/compose/:name', name: 'ComposeDetail', component: () => import('./views/compose/ComposeDetail.vue') },
  { path: '/compose/:name/edit', name: 'ComposeEditor', component: () => import('./views/compose/ComposeEditor.vue') },
  { path: '/network-topology', name: 'NetworkTopology', component: () => import('./views/NetworkTopology.vue') },
  { path: '/registries', name: 'Registries', component: () => import('./views/Registries.vue') },
  { path: '/run-wizard', name: 'RunWizard', component: () => import('./views/RunWizard.vue') },
  { path: '/export-import', name: 'ExportImport', component: () => import('./views/ExportImport.vue') },
  { path: '/templates', name: 'Templates', component: () => import('./views/Templates.vue') },
  { path: '/image-scanner', name: 'ImageScanner', component: () => import('./views/ImageScanner.vue') },

  // ─── v1.2.0: Kubernetes ───
  {
    path: '/kubernetes',
    name: 'kubernetes',
    component: () => import('./views/kubernetes/KubernetesView.vue'),
    meta: { group: 'orchestration', icon: 'fa-ship' },
  },
  {
    path: '/kubernetes/namespace/:ns',
    name: 'kubernetes-namespace',
    component: () => import('./views/kubernetes/KubernetesView.vue'),
    meta: { group: 'orchestration' },
  },
  {
    path: '/kubernetes/:resource/:name',
    name: 'kubernetes-resource',
    component: () => import('./views/kubernetes/KubernetesView.vue'),
    meta: { group: 'orchestration' },
  },

  // ─── v1.2.0: Swarm ───
  {
    path: '/swarm',
    name: 'swarm',
    component: () => import(/* webpackChunkName: "swarm" */ './views/swarm/SwarmView.vue'),
    meta: { group: 'orchestration', icon: 'fa-bees' },
  },

  // ─── v1.2.0: Metrics ───
  {
    path: '/metrics',
    name: 'metrics',
    component: () => import(/* webpackChunkName: "metrics" */ './views/metrics/MetricsView.vue'),
    meta: { group: 'system', icon: 'fa-activity' },
  },
  {
    path: '/metrics/:metric',
    name: 'metrics-type',
    component: () => import(/* webpackChunkName: "metrics" */ './views/metrics/MetricsView.vue'),
    meta: { group: 'system' },
  },

  // ─── v1.2.0: Backup & Restore ───
  {
    path: '/backup',
    name: 'backup',
    component: () => import(/* webpackChunkName: "backup" */ './views/backup/BackupView.vue'),
    meta: { group: 'operations', icon: 'fa-box-archive' },
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

const app = createApp(App)
app.use(router)

// Global error handler to prevent unhandled exceptions from crashing component rendering
app.config.errorHandler = (err) => {
  console.warn('[App] Vue error:', err)
}

// Register v-click-outside directive (used by K8sToolbar)
app.directive('click-outside', {
  mounted(el: HTMLElement, binding: any) {
    (el as any).__clickOutsideHandler = (e: MouseEvent) => {
      if (!el.contains(e.target as Node) && typeof binding.value === 'function') {
        binding.value()
      }
    }
    document.addEventListener('click', (el as any).__clickOutsideHandler)
  },
  unmounted(el: HTMLElement) {
    if ((el as any).__clickOutsideHandler) {
      document.removeEventListener('click', (el as any).__clickOutsideHandler)
    }
  },
})

// Initialize Sentry error tracking (production only, requires VITE_SENTRY_DSN)
initSentry(app, router)

app.mount('#app')
