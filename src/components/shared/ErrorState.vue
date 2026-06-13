<!-- ItzamBox — Error State Component
     Copyright (C) 2026 SodigTech — GPL-3.0
     A.6.3 — Error State: icon + message + suggestion + collapsible detail + retry/copy -->
<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  message: string
  suggestion?: string
  detail?: string
  detailLabel?: string
  icon?: string
}>()

defineEmits<{
  retry: []
}>()

const showDetail = ref(false)
const copied = ref(false)

async function copyDetail() {
  if (!props.detail) return
  try {
    await navigator.clipboard.writeText(props.detail)
    copied.value = true
    setTimeout(() => copied.value = false, 2000)
  } catch {
    // Fallback: select text
    const ta = document.createElement('textarea')
    ta.value = props.detail
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    copied.value = true
    setTimeout(() => copied.value = false, 2000)
  }
}
</script>

<template>
  <div class="error-state">
    <div class="error-state-icon">
      <i v-if="icon" :class="icon" style="font-size:40px;opacity:0.6"></i>
      <svg v-else width="48" height="48" viewBox="0 0 48 48" fill="none">
        <circle cx="24" cy="24" r="20" stroke="currentColor" stroke-width="1.5" opacity="0.4"/>
        <path d="M24 16v10M24 30h.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    </div>
    <h3 class="error-state-title">{{ message }}</h3>
    <p v-if="suggestion" class="error-state-suggestion">{{ suggestion }}</p>
    <div class="error-state-actions">
      <button class="btn btn-primary" @click="$emit('retry')">
        <i class="fa-solid fa-rotate"></i> Retry
      </button>
      <button v-if="detail" class="btn btn-ghost" @click="copyDetail">
        <i :class="copied ? 'fa-solid fa-check' : 'fa-solid fa-copy'"></i>
        {{ copied ? 'Copied!' : 'Copy Error' }}
      </button>
    </div>
    <div v-if="detail" class="error-detail">
      <button class="error-detail-toggle" @click="showDetail = !showDetail">
        <i :class="showDetail ? 'fa-solid fa-chevron-up' : 'fa-solid fa-chevron-down'"></i>
        {{ detailLabel || 'Technical details' }}
      </button>
      <pre v-if="showDetail" class="error-detail-text"><code>{{ detail }}</code></pre>
    </div>
  </div>
</template>
