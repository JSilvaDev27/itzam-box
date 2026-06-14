<!-- ItzamBox — Ripple Surface Component
     Copyright (C) 2026 SodigTech — GPL-3.0
     Wraps content in a relative container and adds a material-style
     click ripple effect at the pointer position. -->
<script setup lang="ts">
import { ref } from 'vue'
import { useReducedMotion } from '../../composables/useReducedMotion'

const { reducedMotion } = useReducedMotion()

interface Ripple {
  id: number
  x: number
  y: number
}

const ripples = ref<Ripple[]>([])
let nextId = 0

function handleClick(event: MouseEvent) {
  if (reducedMotion.value) return

  const target = event.currentTarget as HTMLElement | null
  if (!target) return

  const rect = target.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top

  const id = nextId++
  ripples.value = [...ripples.value, { id, x, y }]

  setTimeout(() => {
    ripples.value = ripples.value.filter((r) => r.id !== id)
  }, 600)
}
</script>

<template>
  <span
    class="ripple-surface"
    @click="handleClick"
    style="position:relative;overflow:hidden;display:inline-block"
  >
    <slot />
    <span
      v-for="ripple in ripples"
      :key="ripple.id"
      class="ripple-effect"
      :style="{
        left: ripple.x + 'px',
        top: ripple.y + 'px',
      }"
    />
  </span>
</template>

<style scoped>
.ripple-surface {
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}

.ripple-effect {
  position: absolute;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--accent-cyan);
  opacity: 0.2;
  transform: translate(-50%, -50%) scale(0);
  animation: ripple-expand 0.5s ease-out forwards;
  pointer-events: none;
}

@keyframes ripple-expand {
  0% {
    transform: translate(-50%, -50%) scale(0);
    opacity: 0.25;
  }
  100% {
    transform: translate(-50%, -50%) scale(12);
    opacity: 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .ripple-effect {
    display: none !important;
  }
}
</style>
