<!-- ItzamBox — Animated Counter Component
     Copyright (C) 2026 SodigTech — GPL-3.0
     Animates numeric values from 0 → N (or old → new) with requestAnimationFrame.
     Respects prefers-reduced-motion. -->
<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useAnimatedCounter } from '../../composables/useAnimatedCounter'
import { useReducedMotion } from '../../composables/useReducedMotion'

const props = withDefaults(defineProps<{
  value: number
  duration?: number
  decimals?: number
  prefix?: string
  suffix?: string
  /** Only start animating when this element becomes visible */
  lazy?: boolean
}>(), {
  duration: 600,
  decimals: 0,
  prefix: '',
  suffix: '',
  lazy: false,
})

const { reducedMotion } = useReducedMotion()

const visible = ref(!props.lazy)
const targetRef = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

onMounted(() => {
  if (props.lazy && targetRef.value) {
    observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) {
          visible.value = true
          observer?.disconnect()
        }
      },
      { threshold: 0.1 },
    )
    observer.observe(targetRef.value)
  }
})

onUnmounted(() => {
  observer?.disconnect()
})

const displayValue = computed(() => {
  return visible.value ? animatedValue.value : 0
})

const { value: animatedValue } = useAnimatedCounter(
  computed(() => (visible.value ? props.value : 0)),
  props.duration,
  computed(() => reducedMotion.value),
)

const formatted = computed(() => {
  const num = displayValue.value
  return props.prefix + num.toFixed(props.decimals) + props.suffix
})
</script>

<template>
  <span ref="targetRef" class="animated-counter">{{ formatted }}</span>
</template>

<style scoped>
.animated-counter {
  font-variant-numeric: tabular-nums;
  display: inline-block;
}
</style>
