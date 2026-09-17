<script setup lang="ts">
defineProps<{
  label: string
  value: string
  status?: 'ok' | 'warn' | 'bad'
  /** 0–100，有则显示进度条 */
  bar?: number | null
  barLabel?: string
}>()
</script>

<template>
  <section class="card" :data-status="status || 'ok'">
    <div class="row">
      <span class="label">{{ label }}</span>
      <span class="value">{{ value }}</span>
    </div>
    <div v-if="bar != null" class="bar-wrap" :title="barLabel">
      <div class="bar-track">
        <div class="bar-fill" :style="{ width: Math.min(100, Math.max(0, bar)) + '%' }" />
      </div>
      <span v-if="barLabel" class="bar-label">{{ barLabel }}</span>
    </div>
    <div class="note"><slot name="note" /></div>
  </section>
</template>

<style scoped>
.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 10px 12px;
}

.card[data-status='warn'] {
  border-color: rgba(255, 214, 10, 0.4);
}

.card[data-status='bad'] {
  border-color: rgba(255, 69, 58, 0.45);
}

.row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}

.label {
  color: var(--text-dim);
  font-size: 12px;
}

.value {
  font-size: 20px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.bar-wrap {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bar-track {
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--accent);
  transition: width 0.25s ease;
}

.card[data-status='warn'] .bar-fill {
  background: var(--warn);
}

.card[data-status='bad'] .bar-fill {
  background: var(--bad);
}

.bar-label {
  font-size: 10px;
  color: var(--text-dim);
  font-variant-numeric: tabular-nums;
}

.note {
  margin-top: 6px;
  color: var(--text-dim);
  font-size: 11px;
  line-height: 1.4;
}
</style>
