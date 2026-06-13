<!-- ItzamBox — Context Menu Component
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { watch, onMounted, onUnmounted } from 'vue'
import { useContextMenu } from '../../composables/useContextMenu'
import type { ContextMenuItem } from '../../composables/useContextMenu'

const { state, hide, handleAction } = useContextMenu()

// Close on click outside
function onClick(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('.ctx-menu')) return
  hide()
}

// Close on scroll or resize
function onScroll() { hide() }

onMounted(() => {
  document.addEventListener('click', onClick)
  document.addEventListener('scroll', onScroll, true)
  window.addEventListener('resize', hide)
})
onUnmounted(() => {
  document.removeEventListener('click', onClick)
  document.removeEventListener('scroll', onScroll, true)
  window.removeEventListener('resize', hide)
})

// Close on Escape
watch(() => state.value.visible, (v) => {
  if (v) {
    const esc = (e: KeyboardEvent) => { if (e.key === 'Escape') { hide(); document.removeEventListener('keydown', esc) } }
    document.addEventListener('keydown', esc)
  }
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="state.visible"
      class="ctx-menu"
      :style="{ position: 'fixed', left: state.x + 'px', top: state.y + 'px', zIndex: 300 }"
    >
      <template v-for="item in state.items" :key="item.id">
        <div v-if="item.divider" class="ctx-divider"></div>
        <div
          v-else
          :class="['ctx-item', { 'ctx-danger': item.danger, 'ctx-disabled': item.disabled }]"
          @click.stop="!item.disabled && handleAction(item)"
        >
          <i v-if="item.icon" :class="'fa-solid ' + item.icon"></i>
          <span class="ctx-label">{{ item.label }}</span>
          <span v-if="item.shortcut" class="ctx-shortcut">{{ item.shortcut }}</span>
        </div>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.ctx-menu {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.6), 0 0 40px rgba(0, 229, 255, 0.04);
  padding: 4px;
  min-width: 210px;
  animation: ctxEnter 0.12s ease-out;
  backdrop-filter: blur(12px);
}

@keyframes ctxEnter {
  from { opacity: 0; transform: scale(0.96) translateY(-4px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

.ctx-item {
  display: flex; align-items: center; gap: 10px;
  padding: 7px 12px; border-radius: 4px;
  font-size: 12px; color: var(--text-main);
  cursor: pointer; transition: all 0.1s ease;
}

.ctx-item:hover { background: var(--bg-hover); }
.ctx-item i { width: 16px; text-align: center; font-size: 13px; color: var(--text-muted); flex-shrink: 0; }

.ctx-label { flex: 1; }
.ctx-shortcut { font-size: 10px; color: var(--text-disabled); font-family: var(--font-mono); margin-left: 16px; }

.ctx-item.ctx-danger { color: var(--accent-red); }
.ctx-item.ctx-danger i { color: var(--accent-red); }
.ctx-item.ctx-danger:hover { background: rgba(239, 68, 68, 0.1); }

.ctx-item.ctx-disabled { opacity: 0.4; cursor: not-allowed; pointer-events: none; }

.ctx-divider { height: 1px; background: var(--border-light); margin: 3px 8px; }
</style>
