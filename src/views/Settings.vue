<!-- ItzamBox — Settings View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { useTheme } from '../composables/useTheme'
import { useI18n } from '../composables/useI18n'

const { isDark, toggleTheme } = useTheme()
const { locale, setLocale, t } = useI18n()

function setDarkTheme() { toggleTheme(); }
function setLightTheme() { toggleTheme(); }
</script>

<template>
  <div class="view-root">
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">{{ t.settings.title }}</span>
  </div>
  <h1 class="text-h1" data-testid="settings-title">{{ t.settings.title }}</h1>

  <div style="display:flex;flex-direction:column;gap:16px;max-width:600px">
    <!-- Theme -->
    <section class="section" style="padding:20px">
      <div class="section-header" style="margin:-20px -20px 0 -20px;padding:12px 20px"><span class="section-title"><i class="fa-solid fa-palette" style="color:var(--accent-cyan);margin-right:8px"></i>{{ t.settings.theme }}</span></div>
      <div data-testid="theme-card-dark" @click="setDarkTheme"
        :style="{marginTop:'16px',padding:'20px',borderRadius:'var(--radius-lg)',cursor:'pointer',border:'2px solid ' + (isDark ? 'var(--accent-cyan)' : 'var(--border-color)'),background:'#0a0c10',textAlign:'center'}">
        <i class="fa-solid fa-moon" style="font-size:32px;color:var(--accent-cyan);margin-bottom:8px;display:block"></i>
        <div style="fontWeight:600;fontSize:14px;color:var(--text-main)">Dark</div>
        <div v-if="isDark" style="color:var(--accent-cyan);fontSize:11px;marginTop:4px">✓ Active</div>
      </div>
      <div data-testid="theme-card-light" @click="setLightTheme"
        :style="{marginTop:'8px',padding:'20px',borderRadius:'var(--radius-lg)',cursor:'pointer',border:'2px solid ' + (!isDark ? 'var(--accent-cyan)' : 'var(--border-color)'),background:'#f8fafc',textAlign:'center'}">
        <i class="fa-solid fa-sun" style="font-size:32px;color:'#0284c7';margin-bottom:8px;display:block"></i>
        <div style="fontWeight:600;fontSize:14px;color:'#0f172a'">Light</div>
        <div v-if="!isDark" style="color:'#0284c7';fontSize:11px;marginTop:4px">✓ Active</div>
      </div>
    </section>

    <!-- Language -->
    <section class="section" style="padding:20px">
      <div class="section-header" style="margin:-20px -20px 0 -20px;padding:12px 20px"><span class="section-title"><i class="fa-solid fa-language" style="color:var(--accent-purple);margin-right:8px"></i>{{ t.settings.language }}</span></div>
      <div @click="setLocale('es')" data-testid="locale-es"
        :style="{marginTop:'16px',padding:'16px 24px',borderRadius:'var(--radius-md)',cursor:'pointer',border:'2px solid ' + (locale === 'es' ? 'var(--accent-cyan)' : 'var(--border-color)'),background:locale === 'es' ? 'rgba(0,229,255,0.05)' : 'var(--bg-tertiary)',display:'flex',alignItems:'center',gap:'10px'}">
        <span style="fontSize:20px">🇪🇸</span>
        <span style="fontWeight:600;fontSize:14px">{{ t.settings.spanish }}</span>
      </div>
      <div @click="setLocale('en')" data-testid="locale-en"
        :style="{marginTop:'8px',padding:'16px 24px',borderRadius:'var(--radius-md)',cursor:'pointer',border:'2px solid ' + (locale === 'en' ? 'var(--accent-cyan)' : 'var(--border-color)'),background:locale === 'en' ? 'rgba(0,229,255,0.05)' : 'var(--bg-tertiary)',display:'flex',alignItems:'center',gap:'10px'}">
        <span style="fontSize:20px">🇺🇸</span>
        <span style="fontWeight:600;fontSize:14px">{{ t.settings.english }}</span>
      </div>
    </section>

    <!-- About -->
    <section class="section">
      <div class="section-header"><span class="section-title"><i class="fa-solid fa-circle-info" style="color:var(--accent-cyan);margin-right:8px"></i>{{ t.settings.version }}</span></div>
      <div style="padding:16px 20px;fontSize:13px;display:flex;flexDirection:column;gap:6px">
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">ItzamBox</span><span style="fontFamily:'var(--font-mono)'">v1.0.0</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Tauri</span><span style="fontFamily:'var(--font-mono)'">v2.11</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">License</span><span>GNU GPL v3.0</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">© 2026</span><span>SodigTech</span></div>
      </div>
    </section>
  </div>
  </div>
</template>
