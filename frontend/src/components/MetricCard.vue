<script setup lang="ts">
defineProps<{
  label: string
  icon?: 'cpu' | 'mem' | 'net' | 'disk' | 'gpu' | 'input' | 'fan'
  value: string
  valueUnit?: string
  /** 右上角状态胶囊文案，如 偏高 / 正常 */
  badge?: string
  badgeTone?: 'ok' | 'warn' | 'bad' | 'info'
  /** 0–100，有则显示进度条 */
  bar?: number | null
  barTone?: 'ok' | 'warn' | 'bad'
  /** 全宽卡片（网络等） */
  wide?: boolean
}>()
</script>

<template>
  <section class="card" :class="{ wide }" :data-badge="badgeTone || 'ok'">
    <div class="head">
      <span class="label">
        <svg v-if="icon === 'cpu'" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="6" y="6" width="12" height="12" rx="2"/><path d="M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4"/></svg>
        <svg v-else-if="icon === 'mem'" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="7" width="18" height="10" rx="2"/><path d="M7 17v2M12 17v2M17 17v2"/></svg>
        <svg v-else-if="icon === 'net'" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/></svg>
        <svg v-else-if="icon === 'disk'" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="M3 12h18"/></svg>
        <svg v-else-if="icon === 'gpu'" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="4" y="6" width="16" height="12" rx="2"/><circle cx="12" cy="12" r="2.5"/></svg>
        <svg v-else-if="icon === 'input'" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M8 14h8"/></svg>
        <svg v-else-if="icon === 'fan'" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="3"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4M5 5l3 3M16 16l3 3M19 5l-3 3M8 16l-3 3"/></svg>
        {{ label }}
      </span>
      <span v-if="badge" class="badge">{{ badge }}</span>
    </div>

    <div class="value-row">
      <span class="value">{{ value }}</span>
      <span v-if="valueUnit" class="unit">{{ valueUnit }}</span>
    </div>

    <div v-if="bar != null" class="bar-track" :data-tone="barTone || 'ok'">
      <div class="bar-fill" :style="{ width: Math.min(100, Math.max(0, bar)) + '%' }" />
    </div>

    <slot name="body" />

    <div class="note"><slot name="note" /></div>
  </section>
</template>

<style scoped>
.card {
  background: var(--bg-card-strong);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: var(--radius);
  padding: 12px 14px;
  box-shadow: var(--shadow);
  min-width: 0;
}

.card.wide {
  grid-column: 1 / -1;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  margin-bottom: 4px;
}

.label {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-mid);
}

.badge {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 8px;
  border-radius: 999px;
  white-space: nowrap;
}

.card[data-badge='warn'] .badge {
  color: var(--warn);
  background: var(--warn-soft);
}

.card[data-badge='bad'] .badge {
  color: var(--bad);
  background: var(--bad-soft);
}

.card[data-badge='info'] .badge {
  color: var(--info);
  background: var(--info-soft);
}

.value-row {
  display: flex;
  align-items: baseline;
  gap: 4px;
  margin: 2px 0 6px;
}

.value {
  font-size: 28px;
  font-weight: 750;
  letter-spacing: -0.03em;
  line-height: 1.1;
  font-variant-numeric: tabular-nums;
}

.unit {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-mid);
}

.bar-track {
  height: 5px;
  border-radius: 999px;
  background: rgba(20, 40, 30, 0.07);
  overflow: hidden;
  margin-bottom: 2px;
}

.bar-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #3dd68c, var(--accent));
  transition: width 0.3s ease;
}

.bar-track[data-tone='warn'] .bar-fill {
  background: linear-gradient(90deg, #ffd76a, var(--warn));
}

.bar-track[data-tone='bad'] .bar-fill {
  background: linear-gradient(90deg, #ff8a8e, var(--bad));
}

.note {
  margin-top: 8px;
  color: var(--text-dim);
  font-size: 11px;
  line-height: 1.45;
}
</style>
