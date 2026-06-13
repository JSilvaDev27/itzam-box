// ItzamBox — Context Menu Composable (global right-click menu system)
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref } from 'vue'

export interface ContextMenuItem {
  id: string
  label: string
  icon?: string
  shortcut?: string
  danger?: boolean
  divider?: boolean
  disabled?: boolean
  action?: () => void
}

export interface ContextMenuState {
  visible: boolean
  x: number
  y: number
  items: ContextMenuItem[]
}

const state = ref<ContextMenuState>({
  visible: false,
  x: 0,
  y: 0,
  items: [],
})

// Helper to copy text to clipboard
async function copyToClipboard(text: string, label = 'Copied') {
  try {
    await navigator.clipboard.writeText(text)
    return { id: 'copied', label, icon: 'fa-circle-check' }
  } catch {
    // Fallback
    const ta = document.createElement('textarea')
    ta.value = text; ta.style.position = 'fixed'; ta.style.opacity = '0'
    document.body.appendChild(ta); ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    return { id: 'copied', label, icon: 'fa-circle-check' }
  }
}

// Build menu items for a container row
export function containerContextMenu(container: { id: string; name: string; image: string; state: string }) {
  return [
    { id: 'copy-id', label: 'Copy Container ID', icon: 'fa-copy', action: () => copyToClipboard(container.id, 'Container ID copied!') },
    { id: 'copy-name', label: 'Copy Name', icon: 'fa-copy', action: () => copyToClipboard(container.name, 'Name copied!') },
    { id: 'copy-image', label: 'Copy Image', icon: 'fa-copy', action: () => copyToClipboard(container.image, 'Image copied!') },
    { id: 'div1', label: '', divider: true },
    { id: 'start', label: 'Start', icon: 'fa-play', disabled: container.state === 'running' },
    { id: 'stop', label: 'Stop', icon: 'fa-stop', disabled: container.state !== 'running' },
    { id: 'restart', label: 'Restart', icon: 'fa-rotate-right', disabled: container.state !== 'running' },
    { id: 'pause', label: 'Pause', icon: 'fa-pause', disabled: container.state !== 'running' },
    { id: 'div2', label: '', divider: true },
    { id: 'logs', label: 'View Logs', icon: 'fa-scroll' },
    { id: 'terminal', label: 'Open Terminal', icon: 'fa-terminal' },
    { id: 'inspect', label: 'Inspect', icon: 'fa-magnifying-glass' },
    { id: 'files', label: 'Browse Files', icon: 'fa-folder-tree' },
    { id: 'div3', label: '', divider: true },
    { id: 'remove', label: 'Remove', icon: 'fa-trash-can', danger: true },
  ]
}

// Build menu items for an image row
export function imageContextMenu(
  image: { id: string; repository: string; tag: string },
  callbacks?: { onRun?: () => void; onPull?: () => void; onRemove?: () => void; onInspect?: () => void; onTag?: () => void }
) {
  return [
    { id: 'copy-repo', label: 'Copy Repository', icon: 'fa-copy', action: () => copyToClipboard(image.repository, 'Repository copied!') },
    { id: 'copy-tag', label: 'Copy Tag', icon: 'fa-tag', action: () => copyToClipboard(image.tag, 'Tag copied!') },
    { id: 'copy-id', label: 'Copy Image ID', icon: 'fa-copy', action: () => copyToClipboard(image.id, 'Image ID copied!') },
    { id: 'div1', label: '', divider: true },
    { id: 'run', label: 'Run Container', icon: 'fa-play', action: callbacks?.onRun },
    { id: 'pull', label: 'Pull Latest', icon: 'fa-cloud-arrow-down', action: callbacks?.onPull },
    { id: 'tag', label: 'Tag Image', icon: 'fa-tag', action: callbacks?.onTag },
    { id: 'div2', label: '', divider: true },
    { id: 'inspect', label: 'Inspect', icon: 'fa-magnifying-glass', action: callbacks?.onInspect },
    { id: 'div3', label: '', divider: true },
    { id: 'remove', label: 'Remove', icon: 'fa-trash-can', danger: true, action: callbacks?.onRemove },
  ]
}

// Build menu items for a volume row
export function volumeContextMenu(
  volume: { name: string; driver: string; mountpoint: string },
  callbacks?: { onInspect?: () => void; onRemove?: () => void }
) {
  return [
    { id: 'copy-name', label: 'Copy Name', icon: 'fa-copy', action: () => copyToClipboard(volume.name, 'Volume name copied!') },
    { id: 'div1', label: '', divider: true },
    { id: 'inspect', label: 'Inspect', icon: 'fa-magnifying-glass', action: callbacks?.onInspect },
    { id: 'div2', label: '', divider: true },
    { id: 'remove', label: 'Remove', icon: 'fa-trash-can', danger: true, action: callbacks?.onRemove },
  ]
}

// Build menu items for a network row
export function networkContextMenu(
  network: { id: string; name: string; driver: string; scope: string },
  callbacks?: { onInspect?: () => void; onRemove?: () => void }
) {
  return [
    { id: 'copy-id', label: 'Copy Network ID', icon: 'fa-copy', action: () => copyToClipboard(network.id, 'Network ID copied!') },
    { id: 'copy-name', label: 'Copy Name', icon: 'fa-copy', action: () => copyToClipboard(network.name, 'Network name copied!') },
    { id: 'div1', label: '', divider: true },
    { id: 'inspect', label: 'Inspect', icon: 'fa-magnifying-glass', action: callbacks?.onInspect },
    { id: 'div2', label: '', divider: true },
    { id: 'remove', label: 'Remove', icon: 'fa-trash-can', danger: true, action: callbacks?.onRemove },
  ]
}

// Build menu items for terminal
export function terminalContextMenu(callbacks: { onCopy?: () => void; onPaste?: () => void; onClear?: () => void }) {
  return [
    { id: 'copy', label: 'Copy', icon: 'fa-copy', shortcut: 'Ctrl+Shift+C', action: callbacks.onCopy },
    { id: 'paste', label: 'Paste', icon: 'fa-paste', shortcut: 'Ctrl+Shift+V', action: callbacks.onPaste },
    { id: 'div1', label: '', divider: true },
    { id: 'select-all', label: 'Select All', icon: 'fa-check-double', action: callbacks.onCopy },
    { id: 'clear', label: 'Clear Terminal', icon: 'fa-eraser', action: callbacks.onClear },
  ]
}

// Generic copy menu (for any text)
export function genericCopyMenu(text: string, label: string) {
  return [
    { id: 'copy', label: `Copy ${label}`, icon: 'fa-copy', action: () => copyToClipboard(text, `${label} copied!`) },
  ]
}

export function useContextMenu() {
  function show(e: MouseEvent, items: ContextMenuItem[]) {
    e.preventDefault()
    e.stopPropagation()
    // Ensure menu stays within viewport
    let x = e.clientX
    let y = e.clientY
    if (x + 220 > window.innerWidth) x = window.innerWidth - 220
    if (y + items.length * 36 + 20 > window.innerHeight) y = window.innerHeight - items.length * 36 - 20
    state.value = { visible: true, x, y, items }
  }

  function hide() {
    state.value = { ...state.value, visible: false }
  }

  function handleAction(item: ContextMenuItem) {
    if (item.action) item.action()
    hide()
  }

  return { state, show, hide, handleAction, copyToClipboard }
}
