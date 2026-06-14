<!-- ItzamBox — Skeleton Loader Component
     Copyright (C) 2026 SodigTech — GPL-3.0 
     A.6.1 — Loading State: Skeleton screens with shimmer animation
     Variants: metric-grid, table-row, card, text, header, chart, drawer, form, list -->
<script setup lang="ts">
defineProps<{
  variant?: 'card' | 'table-row' | 'text' | 'header' | 'metric-grid' | 'chart' | 'drawer' | 'form' | 'list'
  count?: number
  rows?: number
  lines?: number
}>()

function staggerStyle(i: number) {
  return { animationDelay: `${(i - 1) * 100}ms` }
}
</script>

<template>
  <!-- ═══ Metric Grid (Dashboard top cards) ═══ -->
  <div v-if="variant === 'metric-grid'" class="metrics-grid skeleton-staggered">
    <div v-for="i in (count || 4)" :key="i" class="metric-card skeleton-card" :style="staggerStyle(i)">
      <div class="skeleton skeleton-icon"></div>
      <div class="skeleton skeleton-text-sm"></div>
      <div class="skeleton skeleton-text-lg"></div>
      <div class="skeleton skeleton-text-xs"></div>
    </div>
  </div>

  <!-- ═══ Table Row (Data tables) ═══ -->
  <div v-else-if="variant === 'table-row'" class="section skeleton-staggered">
    <div class="col-headers">
      <div class="skeleton skeleton-text-xs" style="width:80px"></div>
      <div class="skeleton skeleton-text-xs" style="width:120px"></div>
      <div class="skeleton skeleton-text-xs" style="width:60px"></div>
    </div>
    <div v-for="i in (rows || 5)" :key="i" class="data-row skeleton-row" :style="staggerStyle(i)">
      <div class="skeleton skeleton-dot"></div>
      <div class="row-info" style="flex:1">
        <div class="skeleton skeleton-text-md" style="width:40%;margin-bottom:4px"></div>
        <div class="skeleton skeleton-text-sm" style="width:60%"></div>
      </div>
      <div class="skeleton skeleton-tag"></div>
      <div class="skeleton skeleton-text-sm" style="width:80px"></div>
    </div>
  </div>

  <!-- ═══ Card (Generic card list) ═══ -->
  <div v-else-if="variant === 'card'" class="section skeleton-staggered">
    <div class="section-header">
      <div class="skeleton skeleton-text-md" style="width:140px"></div>
    </div>
    <div v-for="i in (rows || 4)" :key="i" class="data-row skeleton-row" :style="staggerStyle(i)">
      <div class="skeleton skeleton-dot"></div>
      <div style="flex:1">
        <div class="skeleton skeleton-text-md" style="width:35%;margin-bottom:4px"></div>
        <div class="skeleton skeleton-text-sm" style="width:55%"></div>
      </div>
      <div class="skeleton skeleton-tag"></div>
    </div>
  </div>

  <!-- ═══ Text (Paragraph blocks) ═══ -->
  <div v-else-if="variant === 'text'" class="skeleton-staggered" style="display:flex;flex-direction:column;gap:12px;padding:20px">
    <div v-for="i in (lines || 3)" :key="i" class="skeleton skeleton-text-md" :style="{ width: i === lines ? '50%' : '100%', animationDelay: `${(i - 1) * 100}ms` }"></div>
  </div>

  <!-- ═══ Header (Page title + subtitle) ═══ -->
  <div v-else-if="variant === 'header'" style="padding:4px 0 20px">
    <div class="skeleton skeleton-text-lg" style="width:200px;margin-bottom:8px"></div>
    <div class="skeleton skeleton-text-sm" style="width:320px"></div>
  </div>

  <!-- ═══ Chart (CPU/RAM line chart placeholder) ═══ -->
  <div v-else-if="variant === 'chart'" class="skeleton-pulse" style="background:var(--bg-secondary);border:1px solid var(--border-color);border-radius:var(--radius-lg);overflow:hidden;height:196px">
    <div style="padding:12px 16px 0;display:flex;justify-content:space-between">
      <div class="skeleton skeleton-text-sm" style="width:80px"></div>
      <div class="skeleton skeleton-text-xs" style="width:60px"></div>
    </div>
    <div style="padding:20px 16px;display:flex;flex-direction:column;gap:8px">
      <div class="skeleton" style="height:60px;border-radius:6px;width:100%"></div>
      <div style="display:flex;justify-content:space-between;padding:0 4px">
        <div class="skeleton skeleton-text-xs" style="width:30px"></div>
        <div class="skeleton skeleton-text-xs" style="width:30px"></div>
        <div class="skeleton skeleton-text-xs" style="width:30px"></div>
        <div class="skeleton skeleton-text-xs" style="width:30px"></div>
      </div>
    </div>
  </div>

  <!-- ═══ Drawer (Right panel / Inspector form skeleton) ═══ -->
  <div v-else-if="variant === 'drawer'" class="skeleton-staggered" style="padding:24px 20px;display:flex;flex-direction:column;gap:20px">
    <div style="display:flex;justify-content:space-between;align-items:center">
      <div class="skeleton skeleton-text-md" style="width:120px"></div>
      <div class="skeleton" style="width:24px;height:24px;border-radius:6px"></div>
    </div>
    <div v-for="i in (count || 5)" :key="i" style="display:flex;flex-direction:column;gap:6px" :style="staggerStyle(i)">
      <div class="skeleton skeleton-text-xs" style="width:80px"></div>
      <div class="skeleton skeleton-text-md" style="width:100%;height:32px;border-radius:6px"></div>
    </div>
  </div>

  <!-- ═══ Form (Input fields skeleton) ═══ -->
  <div v-else-if="variant === 'form'" class="skeleton-staggered" style="display:flex;flex-direction:column;gap:16px;padding:4px 0">
    <div v-for="i in (count || 4)" :key="i" style="display:flex;flex-direction:column;gap:6px" :style="staggerStyle(i)">
      <div class="skeleton skeleton-text-xs" style="width:100px"></div>
      <div class="skeleton" style="width:100%;height:36px;border-radius:var(--radius-md)"></div>
    </div>
    <div class="skeleton" style="width:120px;height:36px;border-radius:var(--radius-md);margin-top:4px"></div>
  </div>

  <!-- ═══ List (Simple vertical list of items) ═══ -->
  <div v-else-if="variant === 'list'" class="skeleton-staggered" style="display:flex;flex-direction:column;gap:8px">
    <div v-for="i in (count || 5)" :key="i" style="display:flex;align-items:center;gap:12px;padding:10px 0" :style="staggerStyle(i)">
      <div class="skeleton" style="width:32px;height:32px;border-radius:8px;flex-shrink:0"></div>
      <div style="flex:1;display:flex;flex-direction:column;gap:4px">
        <div class="skeleton skeleton-text-md" style="width:55%"></div>
        <div class="skeleton skeleton-text-sm" style="width:35%"></div>
      </div>
      <div class="skeleton skeleton-tag"></div>
    </div>
  </div>

  <!-- ═══ Default: Full page skeleton ═══ -->
  <div v-else class="skeleton-staggered" style="display:flex;flex-direction:column;gap:20px">
    <div class="skeleton skeleton-text-lg" style="width:200px"></div>
    <div class="metrics-grid">
      <div v-for="i in 4" :key="i" class="metric-card skeleton-card" :style="staggerStyle(i)">
        <div class="skeleton skeleton-icon"></div>
        <div class="skeleton skeleton-text-sm"></div>
        <div class="skeleton skeleton-text-lg"></div>
      </div>
    </div>
    <div class="section" style="padding:20px">
      <div v-for="i in 3" :key="i" class="skeleton skeleton-text-md" style="margin-bottom:8px" :style="{width: (100 - i * 15) + '%'}"></div>
    </div>
  </div>
</template>
