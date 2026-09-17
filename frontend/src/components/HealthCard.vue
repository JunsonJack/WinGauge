<script setup lang="ts">
import { computed } from 'vue'
import type { HealthSnapshot } from '../lib/metrics'
import { bandLabel } from '../lib/metrics'

const props = defineProps<{
  health: HealthSnapshot | null
}>()

const score = computed(() => props.health?.score ?? null)
const band = computed(() => bandLabel(props.health?.band))
const summary = computed(() => props.health?.summary ?? '采集中…')
const tone = computed(() => {
  const s = score.value
  if (s == null) return 'dim'
  if (s >= 90) return 'excellent'
  if (s >= 70) return 'good'
  if (s >= 50) return 'watch'
  return 'critical'
})
</script>

<template>
  <section class="health" :data-tone="tone">
    <div class="score-col">
      <div class="score">{{ score ?? '—' }}</div>
      <div class="score-cap">健康度</div>
    </div>
    <div class="meta">
      <div class="band">{{ band }}</div>
      <div class="summary">{{ summary }}</div>
      <ul v-if="health?.issues?.length" class="issues">
        <li v-for="issue in health.issues" :key="issue.metric + issue.reason">
          −{{ issue.points }} · {{ issue.reason }}
        </li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
.health {
  display: flex;
  gap: 14px;
  align-items: center;
  padding: 12px 14px;
  border-radius: var(--radius);
  background: var(--bg-card);
  border: 1px solid var(--border);
}

.health[data-tone='excellent'] {
  border-color: rgba(52, 199, 89, 0.4);
}

.health[data-tone='good'] {
  border-color: rgba(48, 209, 88, 0.35);
}

.health[data-tone='watch'] {
  border-color: rgba(255, 214, 10, 0.45);
}

.health[data-tone='critical'] {
  border-color: rgba(255, 69, 58, 0.5);
}

.score-col {
  min-width: 64px;
  text-align: center;
}

.score {
  font-size: 36px;
  font-weight: 700;
  line-height: 1;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.02em;
}

.health[data-tone='excellent'] .score {
  color: var(--accent);
}

.health[data-tone='good'] .score {
  color: #30d158;
}

.health[data-tone='watch'] .score {
  color: var(--warn);
}

.health[data-tone='critical'] .score {
  color: var(--bad);
}

.score-cap {
  margin-top: 4px;
  font-size: 10px;
  color: var(--text-dim);
}

.meta {
  flex: 1;
  min-width: 0;
}

.band {
  font-weight: 600;
  font-size: 14px;
}

.summary {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-dim);
}

.issues {
  margin-top: 6px;
  padding: 0;
  list-style: none;
  font-size: 11px;
  color: var(--text-dim);
}

.issues li {
  padding: 1px 0;
}
</style>
