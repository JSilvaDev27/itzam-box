// ItzamBox — Keyboard Shortcuts
// Copyright (C) 2026 SodigTech — GPL-3.0

import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'

export function useKeyboardShortcuts() {
  const router = useRouter()

  function handler(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey

    if (ctrl && e.key === 'k') { e.preventDefault(); /* TODO: open command palette */ }
    else if (ctrl && e.key === '1') { e.preventDefault(); router.push('/') }
    else if (ctrl && e.key === '2') { e.preventDefault(); router.push('/containers') }
    else if (ctrl && e.key === '3') { e.preventDefault(); router.push('/images') }
    else if (ctrl && e.key === '4') { e.preventDefault(); router.push('/volumes') }
    else if (ctrl && e.key === '5') { e.preventDefault(); router.push('/networks') }
    else if (ctrl && e.key === 'r') { e.preventDefault(); window.location.reload() }
    else if (ctrl && e.key === ',') { e.preventDefault(); router.push('/settings') }
    else if (e.key === 'Escape') { /* close modals handled by components */ }
  }

  onMounted(() => window.addEventListener('keydown', handler))
  onUnmounted(() => window.removeEventListener('keydown', handler))
}
