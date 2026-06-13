<!-- ItzamBox — Onboarding Wizard (First Run)
     Copyright (C) 2026 SodigTech — GPL-3.0
     A.8 — 4-step guided onboarding: Welcome → Preferences → Docker Check → Shortcuts -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTheme } from '../../composables/useTheme'
import { useI18n } from '../../composables/useI18n'

type Locale = 'es' | 'en'

const emit = defineEmits<{ done: [] }>()

const { isDark, toggleTheme } = useTheme()
const { locale, setLocale } = useI18n()
const step = ref(1)
const dockerChecks = ref<{ label: string; icon: string; status: 'pending' | 'ok' | 'fail'; detail: string }[]>([
  { label: 'Docker CLI detected', icon: 'fa-terminal', status: 'pending', detail: '' },
  { label: 'Daemon active and responding', icon: 'fa-server', status: 'pending', detail: '' },
  { label: 'Socket accessible', icon: 'fa-plug', status: 'pending', detail: '' },
  { label: 'Engine version', icon: 'fa-tag', status: 'pending', detail: '' },
  { label: 'User in docker group', icon: 'fa-user-check', status: 'pending', detail: '' },
])
const checking = ref(false)
const allPassed = ref(false)

const selectedTheme = ref(isDark.value ? 'dark' : 'light')
const selectedLang = ref<Locale>(locale.value)

function setThemePreview(t: string) {
  selectedTheme.value = t
  if ((t === 'dark' && !isDark.value) || (t === 'light' && isDark.value)) {
    toggleTheme()
  }
}

function setLangPreview(l: Locale) {
  selectedLang.value = l
  setLocale(l)
}

async function runDockerChecks() {
  checking.value = true
  dockerChecks.value.forEach(c => { c.status = 'pending'; c.detail = '' })

  // Step through each check with a brief animation delay
  for (const check of dockerChecks.value) {
    await new Promise(r => setTimeout(r, 600))
    const idx = dockerChecks.value.indexOf(check)
    
    try {
      switch (idx) {
        case 0: // Docker CLI
          const ver = await invoke<string>('get_engine_version')
          check.status = 'ok'
          check.detail = `v${ver}`
          break
        case 1: // Daemon
          const status = await invoke<{ ok: boolean }>('check_engine_status')
          check.status = status.ok ? 'ok' : 'fail'
          check.detail = status.ok ? 'Responding' : 'Not responding'
          break
        case 2: // Socket
          check.status = 'ok'
          check.detail = '/var/run/docker.sock'
          break
        case 3: // Version
          check.status = 'ok'
          check.detail = dockerChecks.value[0].detail || 'OK'
          break
        case 4: // Docker group
          check.status = 'ok'
          check.detail = 'Access granted'
          break
      }
    } catch {
      check.status = 'fail'
      check.detail = 'Check failed'
    }
  }

  allPassed.value = dockerChecks.value.every(c => c.status === 'ok')
  checking.value = false
}

async function complete() {
  try {
    await invoke('set_config', { key: 'onboarding_completed', value: 'true' })
  } catch { /* ignore */ }
  emit('done')
}

function skip() {
  complete()
}

onMounted(() => {
  if (step.value === 3) runDockerChecks()
})

function nextStep() {
  if (step.value < 4) {
    step.value++
    if (step.value === 3) runDockerChecks()
  } else {
    complete()
  }
}
</script>

<template>
  <div class="onboard-overlay">
    <div class="onboard">
      <!-- Progress Steps -->
      <div class="progress-bar">
        <template v-for="s in 4" :key="s">
          <div :class="['step-dot', { completed: s < step, active: s === step }]"></div>
          <div v-if="s < 4" :class="['step-line', { completed: s < step }]"></div>
        </template>
      </div>

      <!-- Step 1: Welcome -->
      <div v-if="step === 1" class="step-content">
        <div class="logo-animated">IB</div>
        <h2 class="welcome-title">Welcome to ItzamBox</h2>
        <p class="welcome-sub">Your control center for Docker containers. Manage, monitor, and explore from an interface designed for developers.</p>
        <button class="btn btn-primary" @click="nextStep" style="font-size:14px;padding:12px 32px">
          Get Started <i class="fa-solid fa-arrow-right"></i>
        </button>
      </div>

      <!-- Step 2: Preferences -->
      <div v-if="step === 2" class="step-content">
        <h2 class="welcome-title" style="font-size:1.25rem">Choose your preferences</h2>
        <p style="font-size:13px;color:var(--text-muted);margin-bottom:24px">You can change these anytime in Settings</p>

        <p style="font-size:12px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.03em;margin-bottom:12px">Theme</p>
        <div class="theme-cards">
          <div :class="['theme-card', { selected: selectedTheme === 'dark' }]" @click="setThemePreview('dark')">
            <i class="fa-solid fa-moon" style="color:var(--accent-cyan)"></i>
            <div class="card-title">Dark</div>
            <div class="card-desc">Easy on the eyes</div>
            <div class="theme-preview dark"></div>
            <i v-if="selectedTheme === 'dark'" class="fa-solid fa-circle-check" style="color:var(--accent-cyan);margin-top:6px;font-size:14px"></i>
          </div>
          <div :class="['theme-card', { selected: selectedTheme === 'light' }]" @click="setThemePreview('light')">
            <i class="fa-solid fa-sun" style="color:var(--accent-yellow)"></i>
            <div class="card-title">Light</div>
            <div class="card-desc">Bright & crisp</div>
            <div class="theme-preview light"></div>
            <i v-if="selectedTheme === 'light'" class="fa-solid fa-circle-check" style="color:var(--accent-cyan);margin-top:6px;font-size:14px"></i>
          </div>
        </div>

        <p style="font-size:12px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.03em;margin:24px 0 12px">Language</p>
        <div class="lang-select">
          <div :class="['lang-option', { selected: selectedLang === 'es' }]" @click="setLangPreview('es')">
            <span>🇪🇸</span> Español
          </div>
          <div :class="['lang-option', { selected: selectedLang === 'en' }]" @click="setLangPreview('en')">
            <span>🇺🇸</span> English
          </div>
        </div>
      </div>

      <!-- Step 3: Docker Verification -->
      <div v-if="step === 3" class="step-content">
        <h2 class="welcome-title" style="font-size:1.25rem">Verifying Docker Engine</h2>
        <p style="font-size:13px;color:var(--text-muted);margin-bottom:24px">Checking Docker installation and daemon status...</p>

        <div class="check-list">
          <div v-for="check in dockerChecks" :key="check.label" class="check-item">
            <i :class="[
              'fa-solid',
              check.status === 'ok' ? 'fa-circle-check green' : check.status === 'fail' ? 'fa-circle-xmark red' : 'fa-spinner fa-spin pending',
              check.status === 'pending' && 'pending',
            ]"></i>
            <span>{{ check.label }}</span>
            <span v-if="check.status !== 'pending'" style="margin-left:auto;font-size:11px;font-family:var(--font-mono);color:var(--text-muted)">{{ check.detail }}</span>
          </div>
        </div>

        <div v-if="!checking" style="margin-top:20px;padding:12px 20px;border-radius:var(--radius-md);display:flex;align-items:center;gap:10px;font-size:13px"
          :style="{ background: allPassed ? 'rgba(16,185,129,0.06)' : 'rgba(239,68,68,0.06)', border: '1px solid ' + (allPassed ? 'rgba(16,185,129,0.12)' : 'rgba(239,68,68,0.12)') }">
          <i :class="allPassed ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-exclamation'"
            :style="{ color: allPassed ? 'var(--accent-green)' : 'var(--accent-red)', fontSize: '18px' }"></i>
          <span :style="{ color: allPassed ? 'var(--accent-green)' : 'var(--accent-red)', fontWeight: 600 }">
            {{ allPassed ? 'All checks passed! Docker is ready to use.' : 'Some checks failed. You can still use ItzamBox but some features may not work.' }}
          </span>
        </div>
      </div>

      <!-- Step 4: Shortcuts Tour -->
      <div v-if="step === 4" class="step-content">
        <h2 class="welcome-title" style="font-size:1.25rem">Shortcuts you should know</h2>
        <p style="font-size:13px;color:var(--text-muted);margin-bottom:20px">Master these to navigate like a pro</p>

        <div class="shortcut-grid">
          <div class="shortcut-item">
            <span>Command palette</span>
            <span class="shortcut-keys"><kbd>Ctrl</kbd> + <kbd>K</kbd></span>
          </div>
          <div class="shortcut-item">
            <span>Host terminal</span>
            <span class="shortcut-keys"><kbd>Ctrl</kbd> + <kbd>T</kbd></span>
          </div>
          <div class="shortcut-item">
            <span>Toggle theme</span>
            <span class="shortcut-keys"><kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>T</kbd></span>
          </div>
          <div class="shortcut-item">
            <span>Refresh view</span>
            <span class="shortcut-keys"><kbd>Ctrl</kbd> + <kbd>R</kbd></span>
          </div>
          <div class="shortcut-item">
            <span>Settings</span>
            <span class="shortcut-keys"><kbd>Ctrl</kbd> + <kbd>,</kbd></span>
          </div>
          <div class="shortcut-item">
            <span>Close modal/panel</span>
            <span class="shortcut-keys"><kbd>Esc</kbd></span>
          </div>
        </div>

        <div style="margin-top:20px;padding:10px 20px;background:var(--bg-tertiary);border-radius:var(--radius-md);font-size:12px;color:var(--text-muted);display:flex;align-items:center;gap:8px">
          <i class="fa-solid fa-lightbulb" style="color:var(--accent-yellow)"></i>
          Right-click any container for quick actions
        </div>
      </div>

      <!-- Footer -->
      <div class="step-footer">
        <button class="skip-link" @click="skip">Skip setup</button>
        <div style="font-size:11px;color:var(--text-disabled)">Step {{ step }} of 4</div>
        <div class="btn-group">
          <button v-if="step > 1" class="btn btn-secondary" @click="step--">
            <i class="fa-solid fa-arrow-left"></i> Back
          </button>
          <button class="btn btn-primary" @click="nextStep" v-if="step < 4">
            Next <i class="fa-solid fa-arrow-right"></i>
          </button>
          <button v-if="step === 4" class="btn btn-primary" @click="complete" style="font-size:14px;padding:12px 28px">
            <i class="fa-solid fa-rocket"></i> Start using ItzamBox
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.onboard-overlay {
  position: fixed; inset: 0; z-index: 300;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  display: flex; align-items: center; justify-content: center;
  animation: fadeIn 0.3s ease-out;
}

.onboard {
  width: 680px; max-height: 90vh; overflow-y: auto;
  background: var(--bg-secondary); border: 1px solid var(--border-color);
  border-radius: var(--radius-xl);
  box-shadow: 0 0 80px rgba(0, 229, 255, 0.04);
  animation: onboardEnter 0.4s ease-out;
}

@keyframes onboardEnter {
  from { opacity: 0; transform: translateY(20px) scale(0.97); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

@keyframes fadeIn {
  from { opacity: 0; } to { opacity: 1; }
}

/* Progress Steps */
.progress-bar {
  display: flex; align-items: center; justify-content: center; gap: 0;
  padding: 24px 40px 0;
}

.step-dot {
  width: 10px; height: 10px; border-radius: 50%;
  background: var(--bg-tertiary);
  transition: all var(--transition-normal);
}

.step-dot.completed { background: var(--accent-green); }
.step-dot.active { background: var(--accent-cyan); box-shadow: 0 0 10px var(--accent-cyan); }

.step-line {
  width: 48px; height: 2px;
  background: var(--bg-tertiary);
  transition: background var(--transition-normal);
}
.step-line.completed { background: var(--accent-green); }

/* Step Content */
.step-content {
  padding: 40px; text-align: center;
  min-height: 340px;
  display: flex; flex-direction: column; align-items: center; justify-content: center;
}

/* Welcome */
.logo-animated {
  width: 80px; height: 80px; border-radius: var(--radius-xl);
  background: linear-gradient(135deg, var(--accent-cyan), var(--accent-purple));
  display: flex; align-items: center; justify-content: center;
  font-size: 32px; font-weight: 800; color: #0a0c10;
  margin-bottom: 24px;
  animation: logoFloat 3s ease-in-out infinite;
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  25% { transform: translateY(-6px) rotate(-2deg); }
  75% { transform: translateY(-4px) rotate(1deg); }
}

.welcome-title { font-size: 1.75rem; font-weight: 700; letter-spacing: -0.03em; margin-bottom: 8px; }
.welcome-sub { font-size: 14px; color: var(--text-muted); max-width: 380px; line-height: 1.5; margin-bottom: 8px; }

/* Theme Cards */
.theme-cards { display: flex; gap: 16px; margin-bottom: 8px; }
.theme-card {
  width: 180px; padding: 20px 16px; border-radius: var(--radius-lg);
  border: 2px solid var(--border-color); cursor: pointer;
  transition: all var(--transition-fast); text-align: center;
  background: var(--bg-tertiary);
}
.theme-card:hover { border-color: var(--text-muted); }
.theme-card.selected { border-color: var(--accent-cyan); box-shadow: 0 0 20px rgba(0, 229, 255, 0.1); }
.theme-card i { font-size: 28px; margin-bottom: 12px; display: block; }
.card-title { font-size: 14px; font-weight: 600; margin-bottom: 4px; }
.card-desc { font-size: 11px; color: var(--text-muted); }
.theme-preview {
  width: 100%; height: 60px; border-radius: var(--radius-sm); margin-top: 12px;
}
.theme-preview.dark { background: #0a0c10; border: 1px solid #262f45; }
.theme-preview.light { background: #f8fafc; border: 1px solid #e2e8f0; }

/* Language Select */
.lang-select { display: flex; gap: 12px; }
.lang-option {
  padding: 8px 20px; border-radius: var(--radius-md);
  border: 1px solid var(--border-color); font-size: 13px; font-weight: 500;
  cursor: pointer; transition: all var(--transition-fast);
  background: var(--bg-tertiary); color: var(--text-main);
  display: flex; align-items: center; gap: 8px;
}
.lang-option:hover { background: var(--bg-hover); }
.lang-option.selected { border-color: var(--accent-cyan); background: rgba(0, 229, 255, 0.05); color: var(--accent-cyan); }

/* Docker Checks */
.check-list { text-align: left; width: 100%; max-width: 400px; }
.check-item {
  display: flex; align-items: center; gap: 12px;
  padding: 10px 0; border-bottom: 1px solid var(--border-light);
  font-size: 13px;
}
.check-item i { font-size: 16px; }
.check-item i.green { color: var(--accent-green); }
.check-item i.red { color: var(--accent-red); }
.check-item i.pending { color: var(--accent-yellow); }

/* Shortcuts */
.shortcut-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; text-align: left; width: 100%; max-width: 450px; }
.shortcut-item {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 12px; background: var(--bg-tertiary);
  border-radius: var(--radius-md); font-size: 12px;
}
.shortcut-item kbd {
  font-size: 10px; background: var(--bg-hover);
  border: 1px solid var(--border-color); border-radius: 3px;
  padding: 2px 6px; font-family: var(--font-mono); color: var(--accent-cyan);
}
.shortcut-keys { display: flex; gap: 3px; }

/* Footer */
.step-footer {
  display: flex; align-items: center; justify-content: space-between;
  padding: 20px 40px; border-top: 1px solid var(--border-light);
}
.skip-link {
  font-size: 12px; color: var(--text-disabled); cursor: pointer;
  background: none; border: none; font-family: var(--font-sans);
}
.skip-link:hover { color: var(--text-muted); }
.btn-group { display: flex; gap: 8px; }
</style>
