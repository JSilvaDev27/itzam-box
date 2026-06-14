<!-- ItzamBox — Page Transition Wrapper
     Copyright (C) 2026 SodigTech — GPL-3.0
     Wraps <router-view> with Vue <Transition> for fade+slide page transitions.
     Respects prefers-reduced-motion. -->
<script setup lang="ts">
import { useReducedMotion } from '../../composables/useReducedMotion'

const { reducedMotion } = useReducedMotion()

function onBeforeEnter(el: Element) {
  const e = el as HTMLElement
  e.style.willChange = 'opacity, transform'
}

function onAfterEnter(el: Element) {
  const e = el as HTMLElement
  e.style.willChange = 'auto'
}

function onBeforeLeave(el: Element) {
  const e = el as HTMLElement
  e.style.willChange = 'opacity, transform'
}

function onAfterLeave(el: Element) {
  const e = el as HTMLElement
  e.style.willChange = 'auto'
}
</script>

<template>
  <Transition
    mode="out-in"
    :name="reducedMotion ? '' : 'page'"
    :duration="reducedMotion ? 0 : { enter: 200, leave: 150 }"
    @before-enter="onBeforeEnter"
    @after-enter="onAfterEnter"
    @before-leave="onBeforeLeave"
    @after-leave="onAfterLeave"
  >
    <slot />
  </Transition>
</template>

<style scoped>
/* Page transition classes are in animations.css but we also define scoped
   no-motion fallback here for scoped isolation. */
.page-enter-active,
.page-leave-active {
  position: relative;
}

/* Override for reduced motion — instant transitions */
@media (prefers-reduced-motion: reduce) {
  .page-enter-active,
  .page-leave-active {
    transition: none !important;
  }
  .page-enter-from,
  .page-leave-to {
    opacity: 1 !important;
    transform: none !important;
  }
}
</style>
