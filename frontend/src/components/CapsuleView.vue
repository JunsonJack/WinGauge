<script setup lang="ts">
import { computed } from 'vue'
import type { CpuSnapshot, HealthSnapshot, MemorySnapshot, NetworkSnapshot } from '../lib/metrics'
import { cpuStatus, formatRate, memStatus } from '../lib/metrics'

const props = defineProps<{
  cpu: CpuSnapshot | null
  memory: MemorySnapshot | null
  network: NetworkSnapshot | null
  health: HealthSnapshot | null
  pinned: boolean
  paused: boolean
}>()

const emit = defineEmits<{
  expand: []
  pin: []
  drag: []
}>()

const cpuTone = computed(() => (props.cpu ? cpuStatus(props.cpu.usage) : 'ok'))
const memTone = computed(() => (props.memory ? memStatus(props.memory.usage) : 'ok'))
const healthTone = computed(() => {
  const s = props.health?.score
  if (s == null) return 'dim'
  if (s >= 90) return 'excellent'
  if (s >= 70) return 'good'
  if (s >= 50) return 'watch'
  return 'critical'
})
</script>

<template>
  <div
    class="capsule"
    :class="{ pinned }"
    @mousedown.left="emit('drag')"
    @dblclick="emit('expand')"
  >
    <span class="health-dot" :data-tone="healthTone" :title="`健康度 ${health?.score ?? '—'}`" />

    <div class="metric" :data-tone="cpuTone">
      <div class="v">{{ cpu ? `${cpu.usage.toFixed(0)}%` : '—' }}</div>
      <div class="k">CPU</div>
    </div>

    <div class="sep" />

    <div class="metric" :data-tone="memTone">
      <div class="v">{{ memory ? `${memory.usage.toFixed(0)}%` : '—' }}</div>
      <div class="k">内存</div>
    </div>

    <div class="sep" />

    <div class="metric net">
      <div class="v">{{ network ? formatRate(network.downloadBps) : '—' }}</div>
      <div class="k">下行</div>
    </div>

    <div class="actions">
      <button
        class="icon-btn"
        :class="{ active: pinned }"
        :title="pinned ? '取消钉住' : '钉住'"
        @click.stop="emit('pin')"
        @mousedown.stop
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 4h6l-1 7 4 3-2 2-4-3-4 3-2-2 4-3z" />
        </svg>
      </button>
      <button
        class="icon-btn expand"
        title="展开完整面板"
        @click.stop="emit('expand')"
        @mousedown.stop
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M8 3H3v5M16 3h5v5M8 21H3v-5M16 21h5v-5" />
        </svg>
      </button>
    </div>

    <span v-if="paused" class="pause-tag">暂停</span>
  </div>
</template>

<style scoped>
.capsule {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 100%;
  padding: 0 8px 0 12px;
  border-radius: 999px;
  background:
    linear-gradient(120deg, rgba(220, 245, 235, 0.95), rgba(245, 250, 248, 0.94) 50%, rgba(228, 240, 255, 0.92));
  border: 1px solid rgba(255, 255, 255, 0.75);
  box-shadow:
    0 8px 28px rgba(20, 40, 30, 0.16),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  cursor: grab;
  user-select: none;
  overflow: hidden;
}

.capsule:active {
  cursor: grabbing;
}

.capsule.pinned {
  box-shadow:
    0 8px 28px rgba(20, 40, 30, 0.16),
    0 0 0 1.5px rgba(47, 191, 113, 0.45),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
}

.health-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #b0b5b2;
  flex-shrink: 0;
  margin-right: 6px;
}

.health-dot[data-tone='excellent'],
.health-dot[data-tone='good'] {
  background: var(--accent);
  box-shadow: 0 0 0 3px rgba(47, 191, 113, 0.18);
}

.health-dot[data-tone='watch'] {
  background: var(--warn);
  box-shadow: 0 0 0 3px rgba(232, 163, 23, 0.18);
}

.health-dot[data-tone='critical'] {
  background: var(--bad);
  box-shadow: 0 0 0 3px rgba(229, 72, 77, 0.18);
}

.metric {
  min-width: 48px;
  text-align: center;
  padding: 0 4px;
}

.metric.net {
  min-width: 58px;
}

.v {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.15;
  font-variant-numeric: tabular-nums;
  color: var(--text);
}

.metric[data-tone='warn'] .v {
  color: var(--warn);
}

.metric[data-tone='bad'] .v {
  color: var(--bad);
}

.k {
  font-size: 9px;
  font-weight: 600;
  color: var(--text-dim);
  margin-top: 1px;
  letter-spacing: 0.02em;
}

.sep {
  width: 1px;
  height: 18px;
  background: rgba(20, 40, 30, 0.1);
  margin: 0 2px;
  flex-shrink: 0;
}

.actions {
  display: flex;
  gap: 2px;
  margin-left: 6px;
  flex-shrink: 0;
}

.icon-btn {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  background: rgba(20, 40, 30, 0.07);
  color: var(--text);
}

.icon-btn.active {
  color: var(--accent);
  background: var(--accent-soft);
}

.icon-btn.expand {
  color: var(--text-mid);
}

.pause-tag {
  position: absolute;
  right: 70px;
  top: 4px;
  font-size: 8px;
  font-weight: 700;
  color: var(--warn);
  background: var(--warn-soft);
  padding: 1px 5px;
  border-radius: 999px;
}

.capsule {
  position: relative;
}
</style>
