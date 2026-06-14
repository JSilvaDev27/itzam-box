// ItzamBox — Notification System
// Copyright (C) 2026 SodigTech — GPL-3.0

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Notification {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  message: string
  timestamp: number
  read: boolean
}

interface DBNotification {
  id: string
  type_str: string
  title: string
  message: string
  read: boolean
  created_at: number
}

const notifications = ref<Notification[]>([])
const toasts = ref<Notification[]>([])

export function useNotifications() {
  async function loadNotifications() {
    try {
      const dbList = await invoke<DBNotification[]>('get_notifications', { limit: 100 })
      notifications.value = dbList.map(item => ({
        id: item.id,
        type: item.type_str as Notification['type'],
        title: item.title,
        message: item.message,
        timestamp: item.created_at * 1000, // Convert unix seconds to ms
        read: item.read,
      }))
      updateUnread()
    } catch (e) {
      console.error('Failed to load notifications from database:', e)
    }
  }

  async function notify(type: Notification['type'], title: string, message: string) {
    const id = Math.random().toString(36).substring(2, 11)
    const n: Notification = { id, type, title, message, timestamp: Date.now(), read: false }
    
    // Save to DB (fire-and-forget/non-blocking)
    invoke('save_notification', { id, typeStr: type, title, message })
      .then(() => {
        // Load latest list from DB to keep in sync
        loadNotifications()
      })
      .catch(e => {
        console.error('Failed to save notification:', e)
        // Fallback local append
        notifications.value.unshift(n)
        if (notifications.value.length > 100) notifications.value.pop()
        updateUnread()
      })

    // Display Toast
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

  function dismissToast(id: string) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  async function markRead(id: string) {
    try {
      await invoke('mark_notification_read', { id })
      const found = notifications.value.find(n => n.id === id)
      if (found) found.read = true
      updateUnread()
    } catch (e) {
      console.error('Failed to mark notification read:', e)
    }
  }

  async function markAllRead() {
    try {
      await invoke('mark_all_read')
      notifications.value.forEach(n => n.read = true)
      updateUnread()
    } catch (e) {
      console.error('Failed to mark all read:', e)
    }
  }

  async function clearAll() {
    try {
      await invoke('clear_notifications')
      notifications.value = []
      updateUnread()
    } catch (e) {
      console.error('Failed to clear notifications:', e)
    }
  }

  const unreadCount = ref(0)
  const updateUnread = () => {
    unreadCount.value = notifications.value.filter(n => !n.read).length
  }

  return {
    notifications,
    toasts,
    unreadCount,
    updateUnread,
    loadNotifications,
    notify,
    success,
    error,
    warning,
    info,
    dismissToast,
    markRead,
    markAllRead,
    clearAll,
  }
}
