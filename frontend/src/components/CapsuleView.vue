<script setup lang="ts">
import { computed } from 'vue'
import type { CpuSnapshot, HealthSnapshot, MemorySnapshot, NetworkSnapshot } from '../lib/metrics'
import { cpuStatus, formatRate, memStatus, tempBadge, tempTone } from '../lib/metrics'

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
const tempText = computed(() => tempBadge(props.cpu?.tempC))
const tempCls = computed(() => tempTone(props.cpu?.tempC))
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
  <!-- 填满 #app（已是 pill 圆角），自身不再叠一层背景，避免双描边/四角杂色 -->
  <div
    class="capsule"
    :class="{ pinned }"
    @mousedown.left="emit('drag')"
    @dblclick="emit('expand')"
  >
    <span class="health-dot" :data-tone="healthTone" :title="`健康度 ${health?.score ?? '—'}`" />

    <div class="metric" :data-tone="cpuTone">
      <div class="v">{{ cpu ? `${cpu.usage.toFixed(0)}%` : '—' }}</div>
      <div class="k">
        CPU<template v-if="tempText">
          · <span :class="['temp', tempCls]">{{ tempText }}</span></template
        >
      </div>
    </div>

    <div class="sep" />

    <div class="metric" :data-tone="memTone">
      <div class="v">{{ memory ? `${memory.usage.toFixed(0)}%` : '—' }}</div>
      <div class="k">内存</div>
    </div>

    <div class="sep" />

    <div class="metric net">
      <div class="v tiny">
        <span class="dir down">↓</span>{{ network ? formatRate(network.downloadBps) : '—' }}
      </div>
      <div class="v tiny upv">
        <span class="dir up">↑</span>{{ network ? formatRate(network.uploadBps) : '—' }}
      </div>
      <div class="k">上下行</div>
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
  justify-content: center;
  width: 100%;
  height: 100%;
  gap: 4px;
  padding: 0 10px 0 14px;
  position: relative;
  cursor: grab;
  user-select: none;
  /* 不画自己的 background —— 圆角与底色由 #app.is-capsule 承担 */
}

.capsule:active {
  cursor: grabbing;
}

.health-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #b0b5b2;
  flex-shrink: 0;
  margin-right: 4px;
}

.health-dot[data-tone='excellent'],
.health-dot[data-tone='good'] {
  background: var(--accent);
  box-shadow: 0 0 0 3px rgba(47, 191, 113, 0.16);
}

.health-dot[data-tone='watch'] {
  background: var(--warn);
  box-shadow: 0 0 0 3px rgba(232, 163, 23, 0.16);
}

.health-dot[data-tone='critical'] {
  background: var(--bad);
  box-shadow: 0 0 0 3px rgba(229, 72, 77, 0.16);
}

.metric {
  min-width: 52px;
  text-align: center;
  padding: 0 2px;
}

.metric.net {
  min-width: 88px;
}

.v {
  font-size: 15px;
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

.v.tiny {
  font-size: 11px;
  font-weight: 700;
  line-height: 1.15;
}

.v.tiny.upv {
  color: var(--accent);
}

.dir.down {
  color: var(--info);
  margin-right: 2px;
  font-size: 10px;
}

.dir.up {
  color: var(--accent);
  margin-right: 2px;
  font-size: 10px;
}

.k {
  font-size: 9px;
  font-weight: 600;
  color: var(--text-dim);
  margin-top: 1px;
  letter-spacing: 0.02em;
}

.temp {
  font-weight: 700;
}

.temp.ok {
  color: var(--accent);
}

.temp.warn {
  color: var(--warn);
}

.temp.bad {
  color: var(--bad);
}

.sep {
  width: 1px;
  height: 20px;
  background: rgba(20, 40, 30, 0.1);
  margin: 0 2px;
  flex-shrink: 0;
}

.actions {
  display: flex;
  gap: 0;
  margin-left: 4px;
  flex-shrink: 0;
}

.icon-btn {
  width: 28px;
  height: 28px;
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
  right: 72px;
  top: 6px;
  font-size: 9px;
  font-weight: 700;
  color: var(--warn);
  background: var(--warn-soft);
  padding: 1px 6px;
  border-radius: 999px;
}

.capsule.pinned .health-dot {
  /* 钉住时用健康点已有光圈，不再叠窗口描边 */
}
</style>
