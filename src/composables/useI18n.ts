// ItzamBox — Internationalization (i18n) Composable
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

type Locale = 'es' | 'en'
type Translations = Record<string, any>

const es: Translations = {
  dashboard: { title: 'Panel General', cpu: 'Uso de CPU', ram: 'Uso de Memoria', active_containers: 'Contenedores Activos', disk_usage: 'Uso de Disco', host_info: 'Información del Host' },
  containers: { title: 'Contenedores', start: 'Iniciar', stop: 'Detener', restart: 'Reiniciar', pause: 'Pausar', unpause: 'Reanudar', remove: 'Eliminar', kill: 'Matar', rename: 'Renombrar', logs: 'Ver Logs', terminal: 'Terminal', inspect: 'Inspeccionar', no_containers: 'Sin contenedores' },
  images: { title: 'Imágenes', pull: 'Descargar Imagen', build: 'Construir Imagen', remove: 'Eliminar', scan: 'Escanear', tag: 'Etiquetar' },
  volumes: { title: 'Volúmenes', create: 'Crear Volumen', remove: 'Eliminar', no_volumes: 'Sin volúmenes' },
  networks: { title: 'Redes', create: 'Crear Red', remove: 'Eliminar' },
  terminal: { host: 'Terminal Host', container: 'Terminal Contenedor', no_running: 'Sin contenedores activos', close_panel: 'Cerrar panel', new_terminal: 'Nuevo terminal…', close_tab: 'Cerrar terminal', host_tab: 'Host' },
  settings: { title: 'Configuración', theme: 'Tema', language: 'Idioma', dark: 'Oscuro', light: 'Claro', spanish: 'Español', english: 'Inglés', version: 'Versión' },
  common: { search: 'Buscar...', loading: 'Cargando...', cancel: 'Cancelar', confirm: 'Confirmar', save: 'Guardar', close: 'Cerrar', refresh: 'Actualizar', error: 'Error', success: 'Éxito', warning: 'Advertencia' },
  installer: { title: 'Docker Engine Requerido', description: 'ItzamBox requiere Docker Engine. ¿Instalar automáticamente?', install_btn: 'Instalar', success: '¡Instalación completada!' },
  help: { title: 'Ayuda', about: 'Acerca de', shortcuts: 'Atajos', dockerInfo: 'Información de Docker', quickStart: 'Guía Rápida', license: 'Licencia' },
}

const en: Translations = {
  dashboard: { title: 'Dashboard', cpu: 'CPU Usage', ram: 'Memory Usage', active_containers: 'Active Containers', disk_usage: 'Disk Usage', host_info: 'Host Info' },
  containers: { title: 'Containers', start: 'Start', stop: 'Stop', restart: 'Restart', pause: 'Pause', unpause: 'Unpause', remove: 'Remove', kill: 'Kill', rename: 'Rename', logs: 'View Logs', terminal: 'Terminal', inspect: 'Inspect', no_containers: 'No containers' },
  images: { title: 'Images', pull: 'Pull Image', build: 'Build Image', remove: 'Remove', scan: 'Scan', tag: 'Tag' },
  volumes: { title: 'Volumes', create: 'Create Volume', remove: 'Remove', no_volumes: 'No volumes' },
  networks: { title: 'Networks', create: 'Create Network', remove: 'Remove' },
  terminal: { host: 'Host Terminal', container: 'Container Terminal', no_running: 'No running containers', close_panel: 'Close terminal panel', new_terminal: 'Open new terminal…', close_tab: 'Close terminal', host_tab: 'Host' },
  settings: { title: 'Settings', theme: 'Theme', language: 'Language', dark: 'Dark', light: 'Light', spanish: 'Spanish', english: 'English', version: 'Version' },
  common: { search: 'Search...', loading: 'Loading...', cancel: 'Cancel', confirm: 'Confirm', save: 'Save', close: 'Close', refresh: 'Refresh', error: 'Error', success: 'Success', warning: 'Warning' },
  installer: { title: 'Docker Engine Required', description: 'ItzamBox requires Docker Engine. Install automatically?', install_btn: 'Install', success: 'Installation complete!' },
  help: { title: 'Help', about: 'About', shortcuts: 'Shortcuts', dockerInfo: 'Docker Info', quickStart: 'Quick Start', license: 'License' },
}

const locales: Record<Locale, Translations> = { es, en }
const currentLocale = ref<Locale>('es')
const t = computed(() => locales[currentLocale.value])

export function useI18n() {
  async function init() {
    try {
      const saved = await invoke<string>('get_config', { key: 'lang' })
      if (saved === 'en') currentLocale.value = 'en'
    } catch { currentLocale.value = 'es' }
  }

  async function setLocale(locale: Locale) {
    currentLocale.value = locale
    try { await invoke('set_config', { key: 'lang', value: locale }) } catch { /* ok */ }
  }

  return { t, locale: currentLocale, setLocale, init }
}
