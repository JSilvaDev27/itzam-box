<!-- ItzamBox — Events View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import EmptyState from '../components/shared/EmptyState.vue'
</script>
<template>
  <div class="breadcrumb"><i class="fa-solid fa-house"></i> <span>Home</span> <i class="fa-solid fa-chevron-right"></i> <span class="current">Events</span></div>
  <h1 class="text-h1">Docker Events</h1>
  <div class="section">
    <EmptyState
      icon="fa-solid fa-clock-rotate-left"
      title="Docker Engine event stream"
      description="Real-time container event monitoring will be available in Phase 2. Track create, start, stop, die, and destroy events as they happen."
    />
  </div>
</template>
