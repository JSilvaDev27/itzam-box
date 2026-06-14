<!-- ItzamBox — Swarm Inactive CTA Cards
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
defineEmits<{
  (e: 'init'): void
  (e: 'join'): void
}>()
</script>

<template>
  <div class="swarm-cta-grid" role="region" aria-label="Swarm actions">
    <!-- Initialize New Swarm -->
    <div class="swarm-cta-card" role="article">
      <div class="swarm-cta-card__icon swarm-cta-card__icon--init">
        <i class="fa-solid fa-play" aria-hidden="true"></i>
      </div>
      <h3 class="swarm-cta-card__title">
        Initialize New Swarm
      </h3>
      <p class="swarm-cta-card__desc">
        Create a new Swarm cluster. This node will become the first manager node.
      </p>
      <button class="btn btn-primary" @click="$emit('init')" aria-label="Initialize Swarm">
        <i class="fa-solid fa-play" aria-hidden="true"></i>
        Initialize Swarm
      </button>
    </div>

    <!-- Join Existing Swarm -->
    <div class="swarm-cta-card" role="article">
      <div class="swarm-cta-card__icon swarm-cta-card__icon--join">
        <i class="fa-solid fa-right-to-bracket" aria-hidden="true"></i>
      </div>
      <h3 class="swarm-cta-card__title">
        Join Existing Swarm
      </h3>
      <p class="swarm-cta-card__desc">
        Join an existing Swarm cluster using a join token from a manager node.
      </p>
      <button class="btn btn-secondary" @click="$emit('join')" aria-label="Join Swarm">
        <i class="fa-solid fa-right-to-bracket" aria-hidden="true"></i>
        Join Swarm
      </button>
    </div>
  </div>
</template>

<style scoped>
.swarm-cta-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-5, 20px);
  max-width: 720px;
  margin: 24px auto 0;
}

.swarm-cta-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 32px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  text-align: center;
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.swarm-cta-card:hover {
  border-color: rgba(0, 229, 255, 0.2);
  box-shadow: var(--shadow-glow);
}

.swarm-cta-card__icon {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
}

.swarm-cta-card__icon--init {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
}

.swarm-cta-card__icon--join {
  background: rgba(168, 85, 247, 0.1);
  color: var(--accent-purple);
}

.swarm-cta-card__title {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-main);
  margin: 0;
}

.swarm-cta-card__desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
}
</style>
