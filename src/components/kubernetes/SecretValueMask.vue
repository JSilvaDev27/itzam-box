<!-- ItzamBox — Redacted Secret Value Display
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  value?: string
  keyName: string
}>()

const revealed = ref(false)
</script>

<template>
  <span class="secret-mask" @click="revealed = !revealed" :title="revealed ? 'Click to hide' : 'Click to reveal'">
    <template v-if="revealed">
      <span class="secret-mask-value">{{ value ?? keyName }}</span>
    </template>
    <template v-else>
      <span class="secret-mask-dots">●●●●●●●●</span>
      <span class="secret-mask-label">(hidden)</span>
    </template>
    <i :class="revealed ? 'fa-solid fa-eye-slash' : 'fa-solid fa-eye'" class="secret-mask-icon"></i>
  </span>
</template>

<style scoped>
.secret-mask {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.15);
  border: 1px solid transparent;
  transition: all var(--transition-fast);
  font-family: var(--font-mono);
  font-size: 11px;
}

.secret-mask:hover {
  background: rgba(239, 68, 68, 0.05);
  border-color: rgba(239, 68, 68, 0.2);
}

.secret-mask-dots {
  color: var(--accent-red);
  letter-spacing: 2px;
  font-weight: 600;
}

.secret-mask-label {
  color: var(--text-disabled);
  font-size: 10px;
  font-family: var(--font-sans);
}

.secret-mask-value {
  color: var(--accent-yellow);
  word-break: break-all;
}

.secret-mask-icon {
  font-size: 10px;
  color: var(--text-disabled);
  transition: color var(--transition-fast);
}

.secret-mask:hover .secret-mask-icon {
  color: var(--text-muted);
}
</style>
