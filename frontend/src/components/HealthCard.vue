<script setup lang="ts">
import { computed } from 'vue'
import type { HealthSnapshot } from '../lib/metrics'
import { bandLabel } from '../lib/metrics'

const props = defineProps<{
  health: HealthSnapshot | null
  bootLine?: string
  uptimeShort?: string
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
    <div class="head">
      <span class="label">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M12 21s-7-4.5-7-10a4 4 0 0 1 7-2.5A4 4 0 0 1 19 11c0 5.5-7 10-7 10z"/></svg>
        健康度
      </span>
      <span class="pill">{{ band }}</span>
    </div>

    <div class="body">
      <div class="score">{{ score ?? '—' }}</div>
      <div class="meta">
        <div class="summary">{{ summary }}</div>
        <div v-if="bootLine" class="boot">{{ bootLine }}</div>
      </div>
      <div class="check" aria-hidden="true">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M5 13l4 4L19 7" />
        </svg>
      </div>
    </div>

    <ul v-if="health?.issues?.length" class="issues">
      <li v-for="issue in health.issues" :key="issue.metric + issue.reason">
        −{{ issue.points }} · {{ issue.reason }}
      </li>
    </ul>

    <div v-if="uptimeShort" class="foot">已运行 {{ uptimeShort }}</div>
  </section>
</template>

<style scoped>
.health {
  padding: 12px 14px;
  border-radius: var(--radius);
  background: var(--bg-card-strong);
  border: 1px solid rgba(255, 255, 255, 0.8);
  box-shadow: var(--shadow);
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-mid);
}

.pill {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 8px;
  border-radius: 999px;
}

.health[data-tone='watch'] .pill {
  color: var(--warn);
  background: var(--warn-soft);
}

.health[data-tone='critical'] .pill {
  color: var(--bad);
  background: var(--bad-soft);
}

.body {
  display: flex;
  align-items: center;
  gap: 12px;
}

.score {
  font-size: 44px;
  font-weight: 750;
  line-height: 1;
  letter-spacing: -0.04em;
  color: var(--accent);
  min-width: 72px;
  font-variant-numeric: tabular-nums;
}

.health[data-tone='watch'] .score {
  color: var(--warn);
}

.health[data-tone='critical'] .score {
  color: var(--bad);
}

.meta {
  flex: 1;
  min-width: 0;
}

.summary {
  font-size: 15px;
  font-weight: 650;
  letter-spacing: -0.01em;
}

.boot {
  margin-top: 3px;
  font-size: 11px;
  color: var(--text-dim);
}

.check {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--accent);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 4px 12px rgba(47, 191, 113, 0.35);
}

.health[data-tone='watch'] .check {
  background: var(--warn);
  box-shadow: 0 4px 12px rgba(232, 163, 23, 0.3);
}

.health[data-tone='critical'] .check {
  background: var(--bad);
  box-shadow: 0 4px 12px rgba(229, 72, 77, 0.3);
}

.issues {
  margin-top: 8px;
  list-style: none;
  font-size: 11px;
  color: var(--text-dim);
}

.foot {
  margin-top: 8px;
  font-size: 11px;
  color: var(--text-dim);
}
</style>
