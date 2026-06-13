<!-- ItzamBox — Empty State Component
     Copyright (C) 2026 SodigTech — GPL-3.0
     A.6.2 — Empty State: SVG illustration + title + description + CTA -->
<script setup lang="ts">
defineProps<{
  icon?: string
  title: string
  description?: string
  actionLabel?: string
  secondaryLabel?: string
}>()

defineEmits<{
  action: []
  secondary: []
}>()
</script>

<template>
  <div class="empty-state">
    <div class="empty-state-icon">
      <i v-if="icon" :class="icon"></i>
      <svg v-else width="64" height="64" viewBox="0 0 64 64" fill="none">
        <rect x="12" y="20" width="40" height="32" rx="4" stroke="currentColor" stroke-width="1.5" opacity="0.3"/>
        <rect x="18" y="28" width="28" height="3" rx="1.5" fill="currentColor" opacity="0.15"/>
        <rect x="18" y="35" width="20" height="3" rx="1.5" fill="currentColor" opacity="0.1"/>
        <rect x="18" y="42" width="24" height="3" rx="1.5" fill="currentColor" opacity="0.08"/>
        <circle cx="44" cy="36" r="6" stroke="currentColor" stroke-width="1.5" opacity="0.25"/>
        <path d="M44 33v6M41 36h6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" opacity="0.25"/>
      </svg>
    </div>
    <h3 class="empty-state-title">{{ title }}</h3>
    <p v-if="description" class="empty-state-desc">{{ description }}</p>
    <div class="empty-state-actions">
      <button v-if="actionLabel" class="btn btn-primary" @click="$emit('action')">
        <i class="fa-solid fa-plus"></i> {{ actionLabel }}
      </button>
      <button v-if="secondaryLabel" class="btn btn-secondary" @click="$emit('secondary')">
        {{ secondaryLabel }}
      </button>
    </div>
  </div>
</template>
