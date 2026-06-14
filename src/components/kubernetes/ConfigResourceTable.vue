<!-- ItzamBox — Kubernetes ConfigMap + Secret Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { K8sConfigMap, K8sSecretMeta } from '../../composables/useKubernetes'
import SecretValueMask from './SecretValueMask.vue'

const props = defineProps<{
  configmaps: K8sConfigMap[]
  secrets: K8sSecretMeta[]
  loading: boolean
  activeSubTab: string
}>()

const emit = defineEmits<{
  'switch-subtab': [tab: string]
  'inspect-cm': [cm: K8sConfigMap]
  'inspect-secret': [secret: K8sSecretMeta]
}>()

const filterQuery = ref('')
const expandedCms = ref<Set<string>>(new Set())

const filteredConfigMaps = computed(() => {
  let list = [...props.configmaps]
  if (filterQuery.value) {
    const q = filterQuery.value.toLowerCase()
    list = list.filter(cm => cm.name.toLowerCase().includes(q) || cm.namespace.toLowerCase().includes(q))
  }
  return list
})

const filteredSecrets = computed(() => {
  let list = [...props.secrets]
  if (filterQuery.value) {
    const q = filterQuery.value.toLowerCase()
    list = list.filter(s => s.name.toLowerCase().includes(q) || s.namespace.toLowerCase().includes(q))
  }
  return list
})

function toggleExpand(key: string) {
  if (expandedCms.value.has(key)) expandedCms.value.delete(key)
  else expandedCms.value.add(key)
}

function copyKeyValue(key: string, value: string) {
  navigator.clipboard?.writeText(value).catch(() => { /* ok */ })
}
</script>

<template>
  <div class="k8s-table-wrapper">
    <!-- Sub-tabs -->
    <div class="k8s-subtabs">
      <button class="k8s-subtab" :class="{ active: activeSubTab === 'configmaps' }" @click="emit('switch-subtab', 'configmaps')">
        <i class="fa-solid fa-file-lines"></i> ConfigMaps
      </button>
      <button class="k8s-subtab" :class="{ active: activeSubTab === 'secrets' }" @click="emit('switch-subtab', 'secrets')">
        <i class="fa-solid fa-lock"></i> Secrets
      </button>
    </div>

    <!-- Filter -->
    <div class="k8s-table-toolbar">
      <div class="k8s-table-filter">
        <i class="fa-solid fa-search"></i>
        <input v-model="filterQuery" :placeholder="'Filter ' + (activeSubTab === 'configmaps' ? 'configmaps...' : 'secrets...')" />
      </div>
      <span class="k8s-table-count">
        {{ activeSubTab === 'configmaps' ? filteredConfigMaps.length : filteredSecrets.length }}
        {{ activeSubTab === 'configmaps' ? 'configmap' + (filteredConfigMaps.length !== 1 ? 's' : '') : 'secret' + (filteredSecrets.length !== 1 ? 's' : '') }}
      </span>
    </div>

    <!-- ConfigMaps View -->
    <template v-if="activeSubTab === 'configmaps' && !loading">
      <div class="k8s-col-headers">
        <span class="k8s-col k8s-col--name">Name</span>
        <span class="k8s-col k8s-col--ns">Namespace</span>
        <span class="k8s-col k8s-col--keys" style="flex: 0 0 80px; justify-content: center;">Keys</span>
        <span class="k8s-col k8s-col--age" style="flex: 0 0 80px;">Age</span>
        <span class="k8s-col" style="flex: 0 0 40px;"></span>
      </div>
      <div class="k8s-table-body">
        <template v-for="cm in filteredConfigMaps" :key="cm.name + '/' + cm.namespace">
          <div class="k8s-data-row" @click="toggleExpand(cm.name + '/' + cm.namespace)">
            <span class="k8s-cell k8s-cell--name"><i class="fa-solid fa-file-lines k8s-cell-icon"></i>{{ cm.name }}</span>
            <span class="k8s-cell k8s-cell--ns">{{ cm.namespace }}</span>
            <span class="k8s-cell k8s-cell--keys">{{ cm.keys_count }}</span>
            <span class="k8s-cell k8s-cell--age">{{ cm.age }}</span>
            <span class="k8s-cell" style="flex:0 0 40px;justify-content:center">
              <i :class="expandedCms.has(cm.name + '/' + cm.namespace) ? 'fa-solid fa-chevron-up' : 'fa-solid fa-chevron-down'" style="font-size:10px;color:var(--text-disabled)"></i>
            </span>
          </div>
          <!-- Expanded keys -->
          <div v-if="expandedCms.has(cm.name + '/' + cm.namespace)" class="k8s-expanded-keys">
            <div v-if="cm.data_keys.length === 0" class="k8s-expanded-empty">No data keys</div>
            <div v-for="key in cm.data_keys" :key="key" class="k8s-key-row">
              <span class="k8s-key-name">{{ key }}</span>
              <button class="k8s-copy-btn" @click.stop="copyKeyValue(key, `[configmap ${cm.name}]`)" title="Copy key name">
                <i class="fa-solid fa-copy"></i>
              </button>
            </div>
          </div>
        </template>
        <div v-if="filteredConfigMaps.length === 0" class="k8s-table-empty">
          <i class="fa-solid fa-file-lines"></i>
          <p>No configmaps found</p>
        </div>
      </div>
    </template>

    <!-- Secrets View -->
    <template v-else-if="activeSubTab === 'secrets' && !loading">
      <div class="k8s-col-headers">
        <span class="k8s-col k8s-col--name">Name</span>
        <span class="k8s-col k8s-col--ns">Namespace</span>
        <span class="k8s-col" style="flex: 1; min-width: 0;">Type</span>
        <span class="k8s-col" style="flex: 0 0 60px; justify-content: center;">Keys</span>
        <span class="k8s-col" style="flex: 0 0 80px;">Age</span>
      </div>
      <div class="k8s-table-body">
        <div v-for="sec in filteredSecrets" :key="sec.name + '/' + sec.namespace"
          class="k8s-data-row" @click="emit('inspect-secret', sec)">
          <span class="k8s-cell k8s-cell--name"><i class="fa-solid fa-lock k8s-cell-icon k8s-cell-icon--secret"></i>{{ sec.name }}</span>
          <span class="k8s-cell k8s-cell--ns">{{ sec.namespace }}</span>
          <span class="k8s-cell" style="flex:1;min-width:0">
            <span class="k8s-type-badge k8s-type--cip" v-if="sec.secret_type">{{ sec.secret_type }}</span>
            <span v-else class="k8s-type--default">Opaque</span>
          </span>
          <span class="k8s-cell" style="flex:0 0 60px;justify-content:center;font-family:var(--font-mono)">{{ sec.keys_count }}</span>
          <span class="k8s-cell" style="flex:0 0 80px;font-family:var(--font-mono);font-size:11px;color:var(--text-muted)">{{ sec.age }}</span>
        </div>
        <div v-if="filteredSecrets.length === 0" class="k8s-table-empty">
          <i class="fa-solid fa-lock"></i>
          <p>No secrets found</p>
        </div>
      </div>
    </template>

    <!-- Loading -->
    <div v-else-if="loading" class="k8s-table-body">
      <div v-for="i in 5" :key="i" class="k8s-data-row k8s-skeleton-row">
        <span class="k8s-cell" style="flex:2"><span class="skeleton skeleton-text-md" style="width:140px"></span></span>
        <span class="k8s-cell" style="flex:1"><span class="skeleton skeleton-text-sm" style="width:80px"></span></span>
        <span class="k8s-cell" style="flex:1"><span class="skeleton skeleton-tag"></span></span>
        <span class="k8s-cell" style="flex:0 0 60px"><span class="skeleton skeleton-text-xs"></span></span>
        <span class="k8s-cell" style="flex:0 0 80px"><span class="skeleton skeleton-text-xs"></span></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.k8s-table-wrapper { display: flex; flex-direction: column; overflow: hidden; }

.k8s-subtabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  padding: 0 16px;
  gap: 0;
}

.k8s-subtab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  background: none;
  color: var(--text-muted);
  font-size: 12px;
  font-family: var(--font-sans);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
  margin-bottom: -1px;
}

.k8s-subtab:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.k8s-subtab.active {
  color: var(--accent-cyan);
  border-bottom-color: var(--accent-cyan);
}

.k8s-subtab i {
  font-size: 12px;
}

.k8s-table-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; border-bottom: 1px solid var(--border-light); }
.k8s-table-filter { display: flex; align-items: center; gap: 6px; background: var(--bg-tertiary); border: 1px solid var(--border-color); border-radius: 6px; padding: 5px 10px; }
.k8s-table-filter i { color: var(--text-disabled); font-size: 12px; }
.k8s-table-filter input { background: none; border: none; outline: none; color: var(--text-main); font-size: 12px; font-family: var(--font-sans); width: 200px; }
.k8s-table-filter input::placeholder { color: var(--text-disabled); }
.k8s-table-count { font-size: 11px; color: var(--text-muted); font-family: var(--font-mono); }

.k8s-col-headers { display: flex; align-items: center; padding: 6px 16px; font-size: 10px; font-weight: 600; color: var(--text-disabled); text-transform: uppercase; letter-spacing: 0.04em; background: var(--bg-tertiary); border-bottom: 1px solid var(--border-light); gap: 8px; }
.k8s-col { display: flex; align-items: center; gap: 3px; }
.k8s-col--name { flex: 2; min-width: 0; }
.k8s-col--ns { flex: 1; min-width: 0; }
.k8s-col--keys { flex: 0 0 80px; justify-content: center; }
.k8s-col--age { flex: 0 0 80px; }

.k8s-table-body { flex: 1; overflow-y: auto; }
.k8s-data-row { display: flex; align-items: center; padding: 10px 16px; border-bottom: 1px solid var(--border-light); gap: 8px; cursor: pointer; transition: background var(--transition-fast); }
.k8s-data-row:hover { background: var(--bg-hover); }
.k8s-skeleton-row { cursor: default; pointer-events: none; }

.k8s-cell { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-main); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.k8s-cell--name { flex: 2; min-width: 0; }
.k8s-cell--ns { flex: 1; min-width: 0; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell--keys { flex: 0 0 80px; justify-content: center; font-family: var(--font-mono); }
.k8s-cell--age { flex: 0 0 80px; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell-icon { color: var(--accent-cyan); font-size: 12px; flex-shrink: 0; }
.k8s-cell-icon--secret { color: var(--accent-red); }

.k8s-type-badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; }
.k8s-type--cip { background: rgba(59, 130, 246, 0.08); color: var(--accent-blue); border: 1px solid rgba(59, 130, 246, 0.15); }
.k8s-type--default { background: var(--bg-tertiary); color: var(--text-muted); border: 1px solid var(--border-color); }

/* Expanded keys section */
.k8s-expanded-keys {
  padding: 8px 16px 8px 48px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
}

.k8s-expanded-empty {
  font-size: 11px;
  color: var(--text-disabled);
  padding: 4px 0;
}

.k8s-key-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}

.k8s-key-name {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-main);
}

.k8s-copy-btn {
  background: none;
  border: none;
  color: var(--text-disabled);
  cursor: pointer;
  font-size: 10px;
  padding: 2px;
  border-radius: 3px;
  transition: all var(--transition-fast);
}

.k8s-copy-btn:hover {
  color: var(--accent-cyan);
  background: var(--bg-hover);
}

.k8s-table-empty { display: flex; flex-direction: column; align-items: center; padding: 48px 20px; color: var(--text-disabled); gap: 8px; }
.k8s-table-empty i { font-size: 28px; color: var(--text-disabled); }
.k8s-table-empty p { font-size: 13px; }
</style>
