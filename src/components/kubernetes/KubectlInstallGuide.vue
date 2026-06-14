<!-- ItzamBox — kubectl Installation Guide
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface LinuxDistro {
  id: string
  name: string
  version: string
  id_like: string | null
  package_manager: string
  supported: boolean
}

const distro = ref<LinuxDistro | null>(null)
const detected = ref(false)

onMounted(async () => {
  try {
    distro.value = await invoke<LinuxDistro>('detect_linux_distro')
    detected.value = true
  } catch {
    detected.value = true
  }
})

const installCommands = [
  {
    os: 'Linux (Debian/Ubuntu)',
    condition: () => distro.value?.id === 'ubuntu' || distro.value?.id === 'debian' || distro.value?.id_like?.includes('debian'),
    commands: [
      '# Download the latest kubectl binary',
      'curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"',
      '# Make it executable',
      'chmod +x ./kubectl',
      '# Move to PATH',
      'sudo mv ./kubectl /usr/local/bin/kubectl',
      '# Verify installation',
      'kubectl version --client',
    ],
  },
  {
    os: 'Linux (Fedora/RHEL/CentOS)',
    condition: () => distro.value?.id === 'fedora' || distro.value?.id_like?.includes('fedora'),
    commands: [
      '# Install via dnf',
      'sudo dnf install kubectl',
      '# Or download binary manually',
      'curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"',
      'chmod +x ./kubectl',
      'sudo mv ./kubectl /usr/local/bin/kubectl',
    ],
  },
  {
    os: 'Linux (Arch)',
    condition: () => distro.value?.id === 'arch' || distro.value?.id_like?.includes('arch'),
    commands: ['sudo pacman -S kubectl'],
  },
  {
    os: 'macOS (Homebrew)',
    condition: () => false,
    commands: ['brew install kubectl'],
  },
  {
    os: 'Windows (Winget)',
    condition: () => false,
    commands: ['winget install Kubernetes.kubectl'],
  },
  {
    os: 'Any (curl binary)',
    condition: () => true,
    commands: [
      '# Download the latest release',
      'curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"',
      '# Validate checksum (optional)',
      'curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl.sha256"',
      'echo "$(cat kubectl.sha256)  kubectl" | sha256sum --check',
      '# Install',
      'sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl',
      '# Verify',
      'kubectl version --client',
    ],
  },
]

const matchedCommands = installCommands.filter(c => c.condition())
const displayCommands = matchedCommands.length > 0 ? matchedCommands : [installCommands[installCommands.length - 1]]
</script>

<template>
  <div class="kubectl-guide">
    <div class="kubectl-guide-icon">
      <i class="fa-solid fa-ship"></i>
    </div>
    <h2 class="kubectl-guide-title">kubectl Not Found</h2>
    <p class="kubectl-guide-desc">
      ItzamBox requires kubectl to interact with Kubernetes clusters.
      Please install kubectl to continue.
    </p>

    <div v-if="distro" class="kubectl-guide-distro">
      <span class="kubectl-guide-distro-label">Detected OS:</span>
      <span class="kubectl-guide-distro-value">{{ distro.name }} {{ distro.version }}</span>
    </div>

    <div v-for="(section, i) in displayCommands" :key="i" class="kubectl-guide-section">
      <h3 class="kubectl-guide-section-title">{{ section.os }}</h3>
      <div class="kubectl-guide-commands">
        <div v-for="(cmd, j) in section.commands" :key="j" class="kubectl-guide-command-line">
          <span class="kubectl-guide-command-prompt" v-if="!cmd.startsWith('#')">$</span>
          <span class="kubectl-guide-command-text" :class="{ 'kubectl-guide-comment': cmd.startsWith('#') }">{{ cmd }}</span>
        </div>
      </div>
    </div>

    <p class="kubectl-guide-note">
      After installing kubectl, configure a kubeconfig file at <code>~/.kube/config</code>
      and ensure it contains at least one active context.
    </p>

    <button class="btn btn-primary" @click="$emit('retry')">
      <i class="fa-solid fa-rotate"></i> Check Again
    </button>
  </div>
</template>

<style scoped>
.kubectl-guide {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px 32px;
  text-align: center;
  max-width: 640px;
  margin: 0 auto;
}

.kubectl-guide-icon {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  background: rgba(0, 229, 255, 0.06);
  border: 1px solid rgba(0, 229, 255, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  color: var(--accent-cyan);
  margin-bottom: 20px;
}

.kubectl-guide-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-main);
  margin-bottom: 8px;
}

.kubectl-guide-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.5;
  margin-bottom: 20px;
  max-width: 480px;
}

.kubectl-guide-distro {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  margin-bottom: 20px;
  font-size: 12px;
}

.kubectl-guide-distro-label {
  color: var(--text-disabled);
  font-weight: 500;
}

.kubectl-guide-distro-value {
  color: var(--text-main);
  font-family: var(--font-mono);
}

.kubectl-guide-section {
  width: 100%;
  text-align: left;
  margin-bottom: 16px;
}

.kubectl-guide-section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
  margin-bottom: 8px;
  padding-left: 4px;
}

.kubectl-guide-commands {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.kubectl-guide-command-line {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 5px 12px;
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.6;
  border-bottom: 1px solid var(--border-light);
}

.kubectl-guide-command-line:last-child {
  border-bottom: none;
}

.kubectl-guide-command-prompt {
  color: var(--accent-green);
  flex-shrink: 0;
  font-weight: 600;
  user-select: none;
}

.kubectl-guide-command-text {
  color: var(--text-main);
  word-break: break-all;
}

.kubectl-guide-comment {
  color: var(--text-disabled);
  font-style: italic;
}

.kubectl-guide-note {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
  margin-bottom: 20px;
}

.kubectl-guide-note code {
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--bg-tertiary);
  padding: 1px 5px;
  border-radius: 3px;
  color: var(--accent-cyan);
}
</style>
