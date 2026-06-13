/* ═══════════════════════════════════════════
   ItzamBox — Main Entry Point
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'

/* ─── Styles (order matters: tokens → layout → components → pages) ─── */
import './styles/theme.css'
import './styles/layout.css'
import './styles/components.css'

/* ─── FontAwesome ─── */
import '@fortawesome/fontawesome-free/css/all.min.css'

/* ─── Routes ─── */
const routes = [
  { path: '/', name: 'Dashboard', component: () => import('./views/Dashboard.vue') },
  { path: '/containers', name: 'Containers', component: () => import('./views/Containers.vue') },
  { path: '/images', name: 'Images', component: () => import('./views/Images.vue') },
  { path: '/volumes', name: 'Volumes', component: () => import('./views/Volumes.vue') },
  { path: '/networks', name: 'Networks', component: () => import('./views/Networks.vue') },
  { path: '/events', name: 'Events', component: () => import('./views/Events.vue') },
  { path: '/cleanup', name: 'Cleanup', component: () => import('./views/Cleanup.vue') },
  { path: '/settings', name: 'Settings', component: () => import('./views/Settings.vue') },
  { path: '/help', name: 'Help', component: () => import('./views/Help.vue') },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

const app = createApp(App)
app.use(router)
app.mount('#app')
