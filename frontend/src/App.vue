<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'
import PanelHeader from './components/PanelHeader.vue'
import MetricCard from './components/MetricCard.vue'
import HealthCard from './components/HealthCard.vue'
import DeviceHeader from './components/DeviceHeader.vue'
import CoreBars from './components/CoreBars.vue'
import Sparkline from './components/Sparkline.vue'
import SettingsView from './components/SettingsView.vue'
import { Ring } from './lib/ring'
import {
  cpuStatus,
  diskStatus,
  formatBootLine,
  formatBytes,
  formatRate,
  formatUptime,
  formatUptimeShort,
  loadLabel,
  memStatus,
  statusBadge,
  type Snapshot,
} from './lib/metrics'

const panel = getCurrentWebviewWindow()
const pinned = ref(false)
const paused = ref(false)
const showSettings = ref(false)
const snap = ref<Snapshot | null>(null)

const cpuHist = new Ring(60, 0)
const memHist = new Ring(60, 0)
const netDownHist = new Ring(60, 0)
const netUpHist = new Ring(60, 0)
const cpuSeries = ref<number[]>([])
const memSeries = ref<number[]>([])
const netDownSeries = ref<number[]>([])
const netUpSeries = ref<number[]>([])

let unlistenTick: UnlistenFn | undefined
let unlistenOpened: UnlistenFn | undefined
let unlistenPause: UnlistenFn | undefined

const device = computed(() => snap.value?.device ?? null)
const health = computed(() => snap.value?.health ?? null)
const cpu = computed(() => snap.value?.cpu ?? null)
const memory = computed(() => snap.value?.memory ?? null)
const network = computed(() => snap.value?.network ?? null)
const disk = computed(() => snap.value?.disk?.systemDrive ?? null)

const hostLine = computed(() => device.value?.host ?? 'WinGauge')
const osLine = computed(() => {
  if (!device.value) return 'Windows'
  const { os, osVersion } = device.value
  return osVersion ? `${os} ${osVersion}` : os
})
const cpuLine = computed(() => {
  if (!device.value) return ''
  return `${device.value.cpuBrand} · ${device.value.logicalCores} 核`
})
const memLine = computed(() => {
  if (!device.value) return ''
  return formatBytes(device.value.totalMemoryBytes)
})
const uptimeLine = computed(() => {
  if (!device.value) return '—'
  return formatUptime(device.value.uptimeSecs)
})
const uptimeShort = computed(() =>
  device.value ? formatUptimeShort(device.value.uptimeSecs) : undefined,
)
const bootLine = computed(() =>
  device.value ? formatBootLine(device.value.uptimeSecs) : undefined,
)

const cpuS = computed(() => cpu.value ? cpuStatus(cpu.value.usage) : 'ok')
const memS = computed(() => memory.value ? memStatus(memory.value.usage) : 'ok')
const diskPct = computed(() => {
  if (!disk.value) return 0
  return (disk.value.usedBytes / Math.max(1, disk.value.totalBytes)) * 100
})
const diskS = computed(() => diskStatus(diskPct.value))

watch(snap, (s) => {
  if (!s) return
  if (s.cpu) cpuHist.push(s.cpu.usage)
  if (s.memory) memHist.push(s.memory.usage)
  if (s.network) {
    netDownHist.push(s.network.downloadBps)
    netUpHist.push(s.network.uploadBps)
  }
  cpuSeries.value = cpuHist.values()
  memSeries.value = memHist.values()
  netDownSeries.value = netDownHist.values()
  netUpSeries.value = netUpHist.values()
})

onMounted(async () => {
  unlistenTick = await listen<Snapshot>('snapshot://tick', (e) => {
    snap.value = e.payload
  })
  unlistenOpened = await listen('panel://opened', () => {})
  unlistenPause = await listen<boolean>('tray://pause-toggled', (e) => {
    paused.value = e.payload === true
  })
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  unlistenTick?.()
  unlistenOpened?.()
  unlistenPause?.()
  window.removeEventListener('keydown', onKeyDown)
})

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (showSettings.value) {
      showSettings.value = false
      return
    }
    void panel.hide()
  }
}

/**
 * 原生窗口拖动：由 OS 合成移动，丝滑且支持跨显示器。
 * 必须在 mousedown 里调用（不能 mousemove），否则会被系统忽略。
 */
function startNativeDrag() {
  // 不 await：startDragging 会进入系统拖动循环，await 会卡住 UI 事件
  void panel.startDragging()
}

async function togglePin() {
  pinned.value = !pinned.value
  await panel.emit('panel/pin', pinned.value)
  if (pinned.value) {
    await panel.setAlwaysOnTop(true)
  }
}

function quit() {
  void invoke('quit_app')
}
</script>

<template>
  <PanelHeader
    :pinned="pinned"
    :settings="showSettings"
    :title="hostLine"
    :subtitle="showSettings ? '偏好设置' : paused ? '采样已暂停' : '实时监控'"
    @pin="togglePin"
    @settings="showSettings = !showSettings"
    @drag="startNativeDrag"
  />

  <SettingsView v-if="showSettings" @close="showSettings = false" />

  <template v-else>
    <DeviceHeader
      :host="hostLine"
      :os-line="osLine"
      :cpu-line="cpuLine"
      :mem-line="memLine"
      :uptime-line="uptimeLine"
    />

    <HealthCard :health="health" :boot-line="bootLine" :uptime-short="uptimeShort" />

    <div class="grid">
      <MetricCard
        v-if="cpu"
        icon="cpu"
        label="CPU"
        :value="`${cpu.usage.toFixed(0)}%`"
        :badge="statusBadge(cpuS)"
        :badge-tone="cpuS === 'ok' ? 'ok' : cpuS"
        :bar="cpu.usage"
        :bar-tone="cpuS"
      >
        <template #body>
          <CoreBars :cores="cpu.perCore" />
        </template>
        <template #note>
          {{ loadLabel(cpu.usage) }} · {{ cpu.perCore.length }} 逻辑核
          <template v-if="cpu.peak != null"> · 峰值 {{ cpu.peak.toFixed(0) }}%</template>
          <Sparkline :values="cpuSeries" :max="100" color="#2fbf71" :height="28" />
        </template>
      </MetricCard>

      <MetricCard
        v-if="memory"
        icon="mem"
        label="内存"
        :value="`${memory.usage.toFixed(0)}%`"
        :badge="statusBadge(memS)"
        :badge-tone="memS === 'ok' ? 'ok' : memS"
        :bar="memory.usage"
        :bar-tone="memS"
      >
        <template #note>
          {{ formatBytes(memory.usedBytes) }} / {{ formatBytes(memory.totalBytes) }}
          <template v-if="memory.committedBytes != null && memory.committedLimitBytes != null">
            · 已提交 {{ formatBytes(memory.committedBytes) }}
          </template>
          <Sparkline :values="memSeries" :max="100" color="#e8a317" :height="28" />
        </template>
      </MetricCard>

      <MetricCard
        v-if="network"
        wide
        icon="net"
        label="网络"
        :value="formatRate(network.downloadBps)"
        value-unit="下行"
        :badge="network.friendlyName"
        badge-tone="info"
      >
        <template #body>
          <div class="net-up">
            <span class="up-dot" />
            上行 {{ formatRate(network.uploadBps) }}
          </div>
          <Sparkline
            :values="netDownSeries"
            :values2="netUpSeries"
            color="#3b9eff"
            color2="#2fbf71"
            :height="44"
          />
          <div class="net-legend">
            <span><i class="dot down" />下行</span>
            <span><i class="dot up" />上行</span>
          </div>
        </template>
        <template #note>
          ↓ {{ formatRate(network.downloadBps) }} · ↑ {{ formatRate(network.uploadBps) }} · {{ network.friendlyName }}
        </template>
      </MetricCard>

      <MetricCard
        v-if="disk"
        wide
        icon="disk"
        label="磁盘"
        :value="`${diskPct.toFixed(0)}%`"
        :badge="statusBadge(diskS)"
        :badge-tone="diskS === 'ok' ? 'ok' : diskS"
        :bar="diskPct"
        :bar-tone="diskS"
      >
        <template #note>
          系统盘 {{ disk.letter }} · 已用 {{ formatBytes(disk.usedBytes) }} / {{ formatBytes(disk.totalBytes) }}
          · 剩余 {{ formatBytes(Math.max(0, disk.totalBytes - disk.usedBytes)) }}
        </template>
      </MetricCard>

      <div v-if="!cpu && !memory && !network && !disk" class="empty">
        正在等待第一帧采样…
      </div>
    </div>
  </template>

  <footer class="foot">
    <button class="exit" title="退出" @click="quit">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 3v10M7 8l5-5 5 5" />
        <path d="M5 14v5a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-5" />
      </svg>
      退出 WinGauge
    </button>
    <span class="ver">v0.1.0 · {{ paused ? '已暂停' : showSettings ? '设置' : '1s 刷新' }}<template v-if="pinned"> · 已钉住</template></span>
  </footer>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  align-content: start;
  padding-bottom: 2px;
}

.empty {
  grid-column: 1 / -1;
  padding: 24px 0;
  text-align: center;
  color: var(--text-dim);
  font-size: 12px;
}

.net-up {
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: flex-end;
  margin-top: -2px;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
}

.up-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
}

.net-legend {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-dim);
}

.net-legend .dot {
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

.foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-top: 2px;
}

.exit {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: none;
  background: transparent;
  color: var(--text-dim);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  padding: 4px 6px;
  border-radius: 8px;
}

.exit:hover {
  background: rgba(229, 72, 77, 0.1);
  color: var(--bad);
}

.ver {
  font-size: 10px;
  color: var(--text-dim);
}
</style>
