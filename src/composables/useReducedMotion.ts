/* ItzamBox — Reduced Motion Composable
   Copyright (C) 2026 SodigTech — GPL-3.0 */

import { ref, onMounted, onUnmounted } from 'vue'

/**
 * Reactive wrapper around `prefers-reduced-motion: reduce` media query.
 * Returns `true` when the user prefers reduced motion.
 * Listens for live changes in the OS setting.
 */
export function useReducedMotion() {
  const reducedMotion = ref(false)
  let mediaQuery: MediaQueryList | null = null
  let listener: (() => void) | null = null

  onMounted(() => {
    mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    reducedMotion.value = mediaQuery.matches

    const handler = (e: MediaQueryListEvent) => {
      reducedMotion.value = e.matches
    }
    mediaQuery.addEventListener('change', handler)
    listener = () => mediaQuery?.removeEventListener('change', handler)
  })

  onUnmounted(() => {
    if (listener) listener()
  })

  return { reducedMotion }
}
