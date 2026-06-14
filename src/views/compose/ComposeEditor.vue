<!-- ItzamBox — Docker Compose Monaco Editor View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  readComposeFile,
  writeComposeFile,
  validateComposeFile,
  formatComposeFile,
} from '../../composables/useDocker'
import { useNotifications } from '../../composables/useNotifications'
import * as monaco from 'monaco-editor'

const route = useRoute()
const router = useRouter()
const { success, error, warning, info } = useNotifications()

const projectName = computed(() => route.params.name as string)
const projectPath = computed(() => (route.query.path as string) || '')

const editorContainer = ref<HTMLDivElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

// Status & UI State
const loading = ref(true)
const saving = ref(false)
const validating = ref(false)
const formatting = ref(false)
const yamlValid = ref<boolean | null>(null)
const yamlError = ref<string | null>(null)
const cursorPosition = ref({ line: 1, column: 1 })

// Load file on mount
onMounted(async () => {
  if (!projectPath.value) {
    error('Missing Path', 'No compose project path was specified.')
    router.push('/compose')
    return
  }

  try {
    const content = await readComposeFile(projectPath.value)
    loading.value = false

    // Initialize Monaco after DOM updates
    setTimeout(() => {
      if (editorContainer.value) {
        editor = monaco.editor.create(editorContainer.value, {
          value: content,
          language: 'yaml',
          theme: 'vs-dark',
          automaticLayout: true,
          minimap: { enabled: true },
          fontSize: 13,
          fontFamily: 'var(--font-mono, monospace)',
          lineHeight: 20,
          padding: { top: 12, bottom: 12 },
          scrollbar: {
            vertical: 'visible',
            horizontal: 'visible',
          },
        })

        // Track cursor position
        editor.onDidChangeCursorPosition((e) => {
          cursorPosition.value = {
            line: e.position.lineNumber,
            column: e.position.column,
          }
        })

        // Keyboard Shortcut: Save (Ctrl+S or Cmd+S)
        editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
          handleSave()
        })
      }
    }, 50)
  } catch (e: any) {
    error('Read Error', `Failed to load compose file: ${e.toString()}`)
    loading.value = false
  }
})

onUnmounted(() => {
  if (editor) {
    editor.dispose()
  }
})

// Actions
async function handleSave() {
  if (!editor || saving.value) return
  saving.value = true
  const content = editor.getValue()
  try {
    await writeComposeFile(projectPath.value, content)
    success('Compose Saved', 'Changes written to disk successfully.')
    // Auto-validate after save
    await handleValidate(false)
  } catch (e: any) {
    error('Save Failed', e.toString())
  } finally {
    saving.value = false
  }
}

async function handleValidate(showToast = true) {
  if (validating.value) return
  validating.value = true
  yamlValid.value = null
  yamlError.value = null

  try {
    // Save current editor content to temp file or directly save first
    if (editor) {
      await writeComposeFile(projectPath.value, editor.getValue())
    }
    const res = await validateComposeFile(projectPath.value)
    yamlValid.value = res.valid
    if (res.valid) {
      if (showToast) {
        success('Validation Passed', 'Docker Compose configuration is valid.')
      }
    } else {
      yamlError.value = res.error
      if (showToast) {
        warning('Validation Failed', 'Compose file contains syntax or structure errors.')
      }
    }
  } catch (e: any) {
    yamlValid.value = false
    yamlError.value = e.toString()
  } finally {
    validating.value = false
  }
}

async function handleFormat() {
  if (!editor || formatting.value) return
  formatting.value = true
  try {
    // Save current content first
    await writeComposeFile(projectPath.value, editor.getValue())
    const formatted = await formatComposeFile(projectPath.value)
    editor.setValue(formatted)
    success('Formatted', 'YAML code formatted successfully.')
  } catch (e: any) {
    error('Format Failed', e.toString())
  } finally {
    formatting.value = false
  }
}

async function handleReload() {
  if (!editor) return
  if (confirm('Discard changes and reload file from disk?')) {
    try {
      const content = await readComposeFile(projectPath.value)
      editor.setValue(content)
      info('Reloaded', 'Loaded file state from disk.')
      yamlValid.value = null
      yamlError.value = null
    } catch (e: any) {
      error('Reload Failed', e.toString())
    }
  }
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i>
    <span @click="router.push('/')" style="cursor:pointer">Home</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span @click="router.push('/compose')" style="cursor:pointer">Compose</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span @click="router.push({ name: 'ComposeDetail', params: { name: projectName }, query: { path: projectPath } })" style="cursor:pointer">
      {{ projectName }}
    </span>
    <i class="fa-solid fa-chevron-right"></i>
    <span class="current">Edit</span>
  </div>

  <div class="editor-wrapper">
    <!-- Toolbar -->
    <div class="editor-toolbar">
      <div class="toolbar-left">
        <button class="btn btn-secondary btn-sm" @click="router.push({ name: 'ComposeDetail', params: { name: projectName }, query: { path: projectPath } })">
          <i class="fa-solid fa-arrow-left"></i> Back
        </button>
        <div class="file-name-title">
          <i class="fa-solid fa-file-code" style="color:var(--accent-purple)"></i>
          <span>docker-compose.yml</span>
        </div>
      </div>
      <div class="toolbar-right">
        <button class="btn btn-primary btn-sm" :disabled="saving" @click="handleSave">
          <i class="fa-solid fa-floppy-disk" :class="{ 'fa-spin': saving }"></i> Save (Ctrl+S)
        </button>
        <button class="btn btn-secondary btn-sm" :disabled="formatting" @click="handleFormat">
          <i class="fa-solid fa-wand-magic-sparkles" :class="{ 'fa-spin': formatting }"></i> Format
        </button>
        <button class="btn btn-secondary btn-sm" :disabled="validating" @click="handleValidate(true)">
          <i class="fa-solid fa-shield-check" :class="{ 'fa-spin': validating }"></i> Validate
        </button>
        <button class="btn btn-ghost btn-sm" @click="handleReload">
          <i class="fa-solid fa-rotate"></i> Reload
        </button>
      </div>
    </div>

    <!-- Main Workspace Container -->
    <div class="editor-container-main">
      <div v-if="loading" class="editor-loading">
        <i class="fa-solid fa-circle-notch fa-spin"></i>
        <span>Loading editor...</span>
      </div>
      <div ref="editorContainer" class="monaco-target"></div>
    </div>

    <!-- Error Pane -->
    <div v-if="yamlValid === false && yamlError" class="editor-error-pane">
      <div class="error-pane-header">
        <i class="fa-solid fa-triangle-exclamation"></i>
        <span>YAML / Compose Validation Errors</span>
      </div>
      <pre class="error-pane-content">{{ yamlError }}</pre>
    </div>

    <!-- Status Bar -->
    <div class="editor-status-bar">
      <div class="status-left">
        <span class="status-item path" :title="projectPath">{{ projectPath }}</span>
      </div>
      <div class="status-right">
        <span class="status-item validation" :class="{ 'valid': yamlValid === true, 'invalid': yamlValid === false }">
          <template v-if="yamlValid === true">
            <i class="fa-solid fa-circle-check"></i> Valid
          </template>
          <template v-else-if="yamlValid === false">
            <i class="fa-solid fa-circle-xmark"></i> Invalid
          </template>
          <template v-else>
            <i class="fa-solid fa-circle-info"></i> Unvalidated
          </template>
        </span>
        <span class="status-item position">
          Ln {{ cursorPosition.line }}, Col {{ cursorPosition.column }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.editor-wrapper {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 120px);
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  margin-top: 12px;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
}

.toolbar-left, .toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-name-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
}

.editor-container-main {
  flex: 1;
  position: relative;
  min-height: 200px;
}

.editor-loading {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-size: 14px;
  z-index: 10;
}

.editor-loading i {
  font-size: 24px;
  color: var(--accent-purple);
}

.monaco-target {
  width: 100%;
  height: 100%;
}

.editor-error-pane {
  background: rgba(239, 68, 68, 0.05);
  border-top: 1px solid rgba(239, 68, 68, 0.2);
  max-height: 150px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.error-pane-header {
  padding: 6px 16px;
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-red);
  background: rgba(239, 68, 68, 0.1);
  display: flex;
  align-items: center;
  gap: 6px;
}

.error-pane-content {
  margin: 0;
  padding: 10px 16px;
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-main);
  white-space: pre-wrap;
  word-break: break-all;
}

.editor-status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 16px;
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-muted);
}

.status-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.status-item.path {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-right {
  display: flex;
  gap: 16px;
}

.status-item.validation.valid {
  color: var(--accent-green);
}

.status-item.validation.invalid {
  color: var(--accent-red);
}
</style>
