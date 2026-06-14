// ItzamBox — Internationalization (i18n) Composable
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref } from 'vue'
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
  swarm: {
    title: 'Swarm', init_title: 'Inicializar Nuevo Swarm', init_desc: 'Crea un nuevo clúster Swarm. Este nodo será el primer administrador.',
    init_btn: 'Inicializar Swarm', join_title: 'Unirse a Swarm Existente', join_desc: 'Únete a un clúster Swarm existente usando un token de un nodo administrador.',
    join_btn: 'Unirse a Swarm', active: 'Swarm Activo', inactive: 'Swarm Inactivo',
    nodes: 'Nodos', managers: 'Administradores', services: 'Servicios', stacks: 'Pilas',
    leave: 'Abandonar Swarm', leave_confirm: '¿Estás seguro de que deseas abandonar el swarm?',
    no_nodes: 'No se encontraron nodos', no_services: 'No hay servicios desplegados', no_stacks: 'No hay pilas desplegadas',
    deploy: 'Desplegar Pila', remove: 'Eliminar', refresh: 'Actualizar',
  },
  kubernetes: {
    title: 'Kubernetes', subtitle: 'Visor de recursos del clúster',
    pods: 'Pods', deployments: 'Deployments', services: 'Servicios', config: 'Config',
    configmaps: 'ConfigMaps', secrets: 'Secrets',
    context: 'Contexto', namespace: 'Namespace', all_namespaces: 'Todos los Namespaces',
    connected: 'Conectado', offline: 'Desconectado', loading_state: 'Cargando',
    status: 'Estado', restarts: 'Reinicios', age: 'Edad', node: 'Nodo',
    type: 'Tipo', cluster_ip: 'IP Clúster', external_ip: 'IP Externa', ports: 'Puertos',
    name: 'Nombre', ready: 'Listo', up_to_date: 'Actualizado', available: 'Disponible',
    keys: 'Claves',
    filter_pods: 'Filtrar pods...', filter_deployments: 'Filtrar deployments...',
    filter_services: 'Filtrar servicios...', filter_configmaps: 'Filtrar configmaps...',
    filter_secrets: 'Filtrar secrets...',
    no_pods: 'Sin pods', no_deployments: 'Sin deployments', no_services: 'Sin servicios',
    no_configmaps: 'Sin configmaps', no_secrets: 'Sin secrets',
    no_kubectl: 'kubectl no encontrado', no_kubeconfig: 'No se encontró kubeconfig',
    cluster_offline: 'Clúster desconectado',
    refresh: 'Actualizar', retry: 'Reintentar',
    overview: 'Resumen', containers: 'Contenedores', conditions: 'Condiciones',
    strategy: 'Estrategia', endpoints: 'Endpoints', data: 'Datos',
    yaml: 'YAML', events: 'Eventos',
    labels: 'Etiquetas', annotations: 'Anotaciones',
    inspector_title: 'Inspector de recursos',
  },
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
  swarm: {
    title: 'Swarm', init_title: 'Initialize New Swarm', init_desc: 'Create a new Swarm cluster. This node will become the first manager node.',
    init_btn: 'Initialize Swarm', join_title: 'Join Existing Swarm', join_desc: 'Join an existing Swarm cluster using a join token from a manager node.',
    join_btn: 'Join Swarm', active: 'Swarm Active', inactive: 'Swarm Inactive',
    nodes: 'Nodes', managers: 'Managers', services: 'Services', stacks: 'Stacks',
    leave: 'Leave Swarm', leave_confirm: 'Are you sure you want to leave the swarm?',
    no_nodes: 'No nodes found', no_services: 'No services deployed', no_stacks: 'No stacks deployed',
    deploy: 'Deploy Stack', remove: 'Remove', refresh: 'Refresh',
  },
  kubernetes: {
    title: 'Kubernetes', subtitle: 'Cluster resource viewer',
    pods: 'Pods', deployments: 'Deployments', services: 'Services', config: 'Config',
    configmaps: 'ConfigMaps', secrets: 'Secrets',
    context: 'Context', namespace: 'Namespace', all_namespaces: 'All Namespaces',
    connected: 'Connected', offline: 'Offline', loading_state: 'Loading',
    status: 'Status', restarts: 'Restarts', age: 'Age', node: 'Node',
    type: 'Type', cluster_ip: 'Cluster IP', external_ip: 'External IP', ports: 'Ports',
    name: 'Name', ready: 'Ready', up_to_date: 'Up-to-Date', available: 'Available',
    keys: 'Keys',
    filter_pods: 'Filter pods...', filter_deployments: 'Filter deployments...',
    filter_services: 'Filter services...', filter_configmaps: 'Filter configmaps...',
    filter_secrets: 'Filter secrets...',
    no_pods: 'No pods', no_deployments: 'No deployments', no_services: 'No services',
    no_configmaps: 'No configmaps', no_secrets: 'No secrets',
    no_kubectl: 'kubectl not found', no_kubeconfig: 'No kubeconfig found',
    cluster_offline: 'Cluster offline',
    refresh: 'Refresh', retry: 'Retry',
    overview: 'Overview', containers: 'Containers', conditions: 'Conditions',
    strategy: 'Strategy', endpoints: 'Endpoints', data: 'Data',
    yaml: 'YAML', events: 'Events',
    labels: 'Labels', annotations: 'Annotations',
    inspector_title: 'Resource Inspector',
  },
}

const locales: Record<Locale, Translations> = { es, en }
const currentLocale = ref<Locale>('es')
const translations = ref<Translations>(locales[currentLocale.value])

export function useI18n() {
  async function init() {
    try {
      const saved = await invoke<string>('get_config', { key: 'lang' })
      if (saved === 'en') {
        currentLocale.value = 'en'
        translations.value = locales[currentLocale.value]
      }
    } catch { /* use default */ }
  }

  function setLocale(locale: Locale) {
    currentLocale.value = locale
    translations.value = locales[locale]
    invoke('set_config', { key: 'lang', value: locale }).catch(() => { /* ok */ })
  }

  return { t: translations, locale: currentLocale, setLocale, init }
}
