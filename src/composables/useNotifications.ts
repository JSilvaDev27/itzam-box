// ItzamBox — Notification System
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref } from 'vue'

export interface Notification {
  id: number
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  message: string
  timestamp: number
  read: boolean
}

const notifications = ref<Notification[]>([])
const toasts = ref<Notification[]>([])
let nextId = 1

export function useNotifications() {
  function notify(type: Notification['type'], title: string, message: string) {
    const n: Notification = { id: nextId++, type, title, message, timestamp: Date.now(), read: false }
    notifications.value.unshift(n)
    if (notifications.value.length > 100) notifications.value.pop()
    // Toast
    toasts.value.push(n)
    setTimeout(() => {
      const idx = toasts.value.indexOf(n)
      if (idx >= 0) toasts.value.splice(idx, 1)
    }, 4000)
  }

  function success(title: string, message = '') { notify('success', title, message) }
  function error(title: string, message = '') { notify('error', title, message) }
  function warning(title: string, message = '') { notify('warning', title, message) }
  function info(title: string, message = '') { notify('info', title, message) }

  function dismissToast(id: number) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  function markAllRead() {
    notifications.value.forEach(n => n.read = true)
  }

  const unreadCount = ref(0)
  // Simple derivation
  const updateUnread = () => {
    unreadCount.value = notifications.value.filter(n => !n.read).length
  }

  return { notifications, toasts, unreadCount, updateUnread, notify, success, error, warning, info, dismissToast, markAllRead }
}
