<script setup lang="ts">
import Sparkline from './Sparkline.vue'
import { formatRate, type NetworkSnapshot } from '../lib/metrics'

defineProps<{
  network: NetworkSnapshot
  downSeries: number[]
  upSeries: number[]
}>()
</script>

<template>
  <section class="net-card">
    <div class="head">
      <span class="label">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <circle cx="12" cy="12" r="9" />
          <path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18" />
        </svg>
        网络
      </span>
      <span class="iface">{{ network.friendlyName }}</span>
    </div>

    <div class="rates">
      <div class="rate">
        <div class="dir down">
          <span class="arrow">↓</span>
          下行
        </div>
        <div class="val">{{ formatRate(network.downloadBps) }}</div>
      </div>
      <div class="rate">
        <div class="dir up">
          <span class="arrow">↑</span>
          上行
        </div>
        <div class="val">{{ formatRate(network.uploadBps) }}</div>
      </div>
    </div>

    <Sparkline
      :values="downSeries"
      :values2="upSeries"
      color="#3b9eff"
      color2="#2fbf71"
      :height="48"
    />

    <div class="legend">
      <span><i class="dot down" />下行</span>
      <span><i class="dot up" />上行</span>
      <span class="meta">↓ {{ formatRate(network.downloadBps) }} · ↑ {{ formatRate(network.uploadBps) }}</span>
    </div>
  </section>
</template>

<style scoped>
.net-card {
  grid-column: 1 / -1;
  background: var(--bg-card-strong);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: var(--radius);
  padding: 12px 14px;
  box-shadow: var(--shadow);
  min-width: 0;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 6px;
}

.label {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-mid);
}

.iface {
  font-size: 11px;
  font-weight: 600;
  color: var(--info);
  background: var(--info-soft);
  padding: 2px 8px;
  border-radius: 999px;
  max-width: 55%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rates {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-bottom: 2px;
}

.rate {
  min-width: 0;
}

.dir {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 600;
  margin-bottom: 2px;
}

.dir.down {
  color: var(--info);
}

.dir.up {
  color: var(--accent);
}

.arrow {
  font-size: 12px;
  line-height: 1;
}

.val {
  font-size: 22px;
  font-weight: 750;
  letter-spacing: -0.03em;
  line-height: 1.15;
  font-variant-numeric: tabular-nums;
  color: var(--text);
}

.legend {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 6px;
  font-size: 11px;
  color: var(--text-dim);
}

.legend .dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  margin-right: 4px;
}

.dot.down {
  background: #3b9eff;
}

.dot.up {
  background: #2fbf71;
}

.meta {
  margin-left: auto;
  font-variant-numeric: tabular-nums;
}
</style>
