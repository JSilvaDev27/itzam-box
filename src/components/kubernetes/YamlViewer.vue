<!-- ItzamBox — Syntax-highlighted YAML Viewer (Read-Only)
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  yaml: string
  loading?: boolean
}>()

const YAML_KEYWORDS = ['apiVersion', 'kind', 'metadata', 'spec', 'status', 'data', 'items']

// Simple line-based YAML syntax highlighting
const highlightedLines = computed(() => {
  if (!props.yaml) return []
  return props.yaml.split('\n').map((line) => {
    const parts: { text: string; class: string }[] = []

    // Comment
    if (line.trimStart().startsWith('#')) {
      parts.push({ text: line, class: 'yaml-comment' })
      return parts
    }

    // Key-value pair
    const keyMatch = line.match(/^(\s*)([\w-]+(?:\.[\w-]+)?)(\s*:\s*)(.*)$/)
    if (keyMatch) {
      const [, indent, key, colon, value] = keyMatch

      if (indent) parts.push({ text: indent, class: '' })
      
      // Check if it's a YAML keyword
      if (YAML_KEYWORDS.includes(key)) {
        parts.push({ text: key, class: 'yaml-keyword' })
      } else {
        parts.push({ text: key, class: 'yaml-key' })
      }

      parts.push({ text: colon, class: 'yaml-punctuation' })

      // Highlight the value
      if (value) {
        const trimmedValue = value.trim()
        if (trimmedValue === '|' || trimmedValue === '>-') {
          parts.push({ text: value, class: 'yaml-punctuation' })
        } else if (trimmedValue === 'true' || trimmedValue === 'false') {
          parts.push({ text: value, class: 'yaml-boolean' })
        } else if (/^\d+$/.test(trimmedValue)) {
          parts.push({ text: value, class: 'yaml-number' })
        } else if (trimmedValue.startsWith('"') || trimmedValue.startsWith("'")) {
          parts.push({ text: value, class: 'yaml-string' })
        } else if (trimmedValue === 'null' || trimmedValue === '~') {
          parts.push({ text: value, class: 'yaml-null' })
        } else {
          parts.push({ text: value, class: 'yaml-value' })
        }
      }

      return parts
    }

    // List item
    const listMatch = line.match(/^(\s*)(- )(.*)$/)
    if (listMatch) {
      const [, indent, dash, rest] = listMatch
      if (indent) parts.push({ text: indent, class: '' })
      parts.push({ text: dash, class: 'yaml-punctuation' })
      parts.push({ text: rest, class: 'yaml-value' })
      return parts
    }

    // Default
    parts.push({ text: line, class: '' })
    return parts
  })
})
</script>

<template>
  <div class="yaml-viewer">
    <div v-if="loading" class="yaml-loading">
      <i class="fa-solid fa-spinner fa-spin"></i>
      <span>Loading YAML...</span>
    </div>
    <pre v-else-if="yaml" class="yaml-pre"><code><span v-for="(parts, i) in highlightedLines" :key="i" class="yaml-line"><span v-for="(part, j) in parts" :key="j" :class="part.class">{{ part.text }}</span></span></code></pre>
    <div v-else class="yaml-empty">
      <i class="fa-solid fa-file-code"></i>
      <p>No YAML data available</p>
    </div>
  </div>
</template>

<style scoped>
.yaml-viewer {
  position: relative;
  height: 100%;
  background: var(--bg-primary);
  border-radius: var(--radius-md);
  overflow: auto;
}

.yaml-pre {
  padding: 16px;
  margin: 0;
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.6;
  white-space: pre;
  tab-size: 2;
  color: var(--text-main);
  min-height: 100%;
}

.yaml-line {
  display: block;
}

.yaml-keyword { color: var(--accent-cyan); font-weight: 600; }
.yaml-key { color: var(--accent-blue); }
.yaml-punctuation { color: var(--text-disabled); }
.yaml-value { color: var(--text-main); }
.yaml-string { color: var(--accent-green); }
.yaml-number { color: var(--accent-purple); }
.yaml-boolean { color: var(--accent-yellow); }
.yaml-null { color: var(--text-disabled); font-style: italic; }
.yaml-comment { color: var(--text-disabled); font-style: italic; }

.yaml-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 32px;
  color: var(--text-muted);
  font-size: 13px;
}

.yaml-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px;
  color: var(--text-disabled);
  gap: 8px;
}

.yaml-empty i {
  font-size: 24px;
}

.yaml-empty p {
  font-size: 12px;
}
</style>
