import { describe, it, expect } from 'vitest'
import { ref, nextTick } from 'vue'
import { useAnimatedCounter, easings } from '../../composables/useAnimatedCounter'

describe('useAnimatedCounter composable', () => {
  it('should ease-out-expo for large deltas', () => {
    const easing = easings.easeOutExpo
    expect(easing(0)).toBe(0)
    expect(easing(1)).toBe(1)
    expect(easing(0.5)).toBeGreaterThan(0.9)
  })

  it('should ease-out-cubic for medium deltas', () => {
    const easing = easings.easeOutCubic
    expect(easing(0)).toBe(0)
    expect(easing(1)).toBe(1)
    expect(easing(0.5)).toBeCloseTo(0.875)
  })

  it('should be linear for small deltas', () => {
    const easing = easings.linear
    expect(easing(0)).toBe(0)
    expect(easing(0.5)).toBe(0.5)
    expect(easing(1)).toBe(1)
  })

  it('should immediately set value when disabled', async () => {
    const source = ref(5)
    const disabled = ref(true)
    const { value } = useAnimatedCounter(source, 600, disabled)
    source.value = 10
    await nextTick()
    // When disabled, the value should match the source immediately
    expect(value.value).toBe(10)
  })

  it('should track initial source value', () => {
    const source = ref(42)
    const { value } = useAnimatedCounter(source, 100)
    // Initial value matches source
    expect(value.value).toBe(42)
  })
})
