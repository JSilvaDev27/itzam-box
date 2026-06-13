<!-- ItzamBox — Dashboard View (MVP placeholder) -->
<!-- Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Dashboard</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Dashboard</h1>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary"><i class="fa-solid fa-rotate"></i> Refresh</button>
      <button class="btn btn-primary"><i class="fa-solid fa-plus"></i> New Container</button>
    </div>
  </div>

  <div class="metrics-grid">
    <div class="metric-card">
      <div class="metric-icon green"><i class="fa-solid fa-cubes"></i></div>
      <div class="metric-label">Active Containers</div>
      <div class="metric-value" style="color:var(--accent-green)">--</div>
      <div class="metric-delta up"><i class="fa-solid fa-arrow-up"></i> Loading...</div>
    </div>
    <div class="metric-card">
      <div class="metric-icon cyan"><i class="fa-solid fa-layer-group"></i></div>
      <div class="metric-label">Local Images</div>
      <div class="metric-value">--</div>
      <div class="metric-delta up">Loading...</div>
    </div>
    <div class="metric-card">
      <div class="metric-icon purple"><i class="fa-solid fa-database"></i></div>
      <div class="metric-label">Volumes</div>
      <div class="metric-value">--</div>
      <div class="metric-delta up">Loading...</div>
    </div>
    <div class="metric-card">
      <div class="metric-icon amber"><i class="fa-solid fa-hard-drive"></i></div>
      <div class="metric-label">Disk Used</div>
      <div class="metric-value">-- GB</div>
      <div class="metric-delta">Loading...</div>
    </div>
  </div>

  <div class="section">
    <div class="section-header">
      <span class="section-title">Getting Started</span>
    </div>
    <div style="padding:40px;text-align:center;color:var(--text-muted)">
      <i class="fa-solid fa-rocket" style="font-size:48px;margin-bottom:16px;opacity:0.3"></i>
      <p style="font-size:14px;margin-bottom:8px">ItzamBox v1.0.0 — MVP Phase 1</p>
      <p style="font-size:12px">Docker Engine connection pending. Backend integration in progress.</p>
    </div>
  </div>
</template>
