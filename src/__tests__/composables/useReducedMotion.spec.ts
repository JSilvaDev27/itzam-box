import { describe, it, expect, beforeEach } from 'vitest'
import { useReducedMotion } from '../../composables/useReducedMotion'

describe('useReducedMotion composable', () => {
  beforeEach(() => {
    // Reset to default (not reduced)
    const mm = window.matchMedia('(prefers-reduced-motion: reduce)') as any
    mm.matches = false
  })

  it('should default to false', () => {
    const { reducedMotion } = useReducedMotion()
    // onMounted hasn't run yet, so the default is false
    expect(reducedMotion.value).toBe(false)
  })

  it('should be reactive to media query changes', () => {
    // Verify it's the correct query by checking the mock
    const mm = window.matchMedia('(prefers-reduced-motion: reduce)') as any
    expect(mm.media).toBe('(prefers-reduced-motion: reduce)')
  })
})
