/* ItzamBox — Animated Counter Composable
   Copyright (C) 2026 SodigTech — GPL-3.0 */

import { ref, watch, type Ref } from 'vue'

type EasingFn = (t: number) => number

/**
 * Easing functions for animated counters.
 */
export const easings = {
  linear: ((t: number) => t) as EasingFn,
  easeOutCubic: ((t: number) => 1 - Math.pow(1 - t, 3)) as EasingFn,
  easeOutExpo: ((t: number) => (t === 1 ? 1 : 1 - Math.pow(2, -10 * t))) as EasingFn,
}

function chooseEasing(delta: number): EasingFn {
  const absDelta = Math.abs(delta)
  if (absDelta > 100) return easings.easeOutExpo
  if (absDelta > 10) return easings.easeOutCubic
  return easings.linear
}

/**
 * Composable that watches a source value and animates numeric changes.
 *
 * @param source - A reactive ref or computed containing the target number.
 * @param duration - Animation duration in ms. Default 600.
 * @param disabledRef - Optional reactive boolean ref to disable animation.
 * @returns An object with the animated `value` ref.
 */
export function useAnimatedCounter(
  source: Ref<number>,
  duration = 600,
  disabledRef?: Ref<boolean>,
) {
  const animatedValue = ref(source.value)
  let rafId: number | null = null

  function cancelPending() {
    if (rafId !== null) {
      cancelAnimationFrame(rafId)
      rafId = null
    }
  }

  watch(
    source,
    (newVal, oldVal) => {
      cancelPending()

      if (disabledRef?.value) {
        animatedValue.value = newVal
        return
      }

      if (oldVal === newVal) {
        animatedValue.value = newVal
        return
      }

      const from = oldVal
      const to = newVal
      const easing = chooseEasing(to - from)
      const startTime = performance.now()

      function animate(currentTime: number) {
        const elapsed = currentTime - startTime
        const progress = Math.min(elapsed / duration, 1)
        const easedProgress = easing(progress)

        animatedValue.value = from + (to - from) * easedProgress

        if (progress < 1) {
          rafId = requestAnimationFrame(animate)
        } else {
          animatedValue.value = to
          rafId = null
        }
      }

      rafId = requestAnimationFrame(animate)
    },
    { immediate: false },
  )

  return { value: animatedValue }
}
