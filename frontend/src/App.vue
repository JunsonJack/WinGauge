<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
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
  formatBytes,
  formatRate,
  formatUptime,
  memStatus,
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
const cpuSeries = ref<number[]>([])
const memSeries = ref<number[]>([])
const netSeries = ref<number[]>([])

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
  if (!device.value) return '读取系统信息…'
  const { os, osVersion } = device.value
  return osVersion ? `${os} ${osVersion}` : os
})
const cpuLine = computed(() => {
  if (!device.value) return ''
  return `${device.value.cpuBrand} · ${device.value.logicalCores} 核`
})
const memLine = computed(() => {
  if (!device.value) return ''
  return `内存 ${formatBytes(device.value.totalMemoryBytes)}`
})
const uptimeLine = computed(() => {
  if (!device.value) return ''
  return `已运行 ${formatUptime(device.value.uptimeSecs)}`
})

const cpuLoadLabel = computed(() => {
  const u = cpu.value?.usage ?? 0
  if (u < 30) return '低负载'
  if (u < 70) return '中负载'
  return '高负载'
})

watch(snap, (s) => {
  if (!s) return
  if (s.cpu) cpuHist.push(s.cpu.usage)
  if (s.memory) memHist.push(s.memory.usage)
  if (s.network) netDownHist.push(s.network.downloadBps)
  cpuSeries.value = cpuHist.values()
  memSeries.value = memHist.values()
  netSeries.value = netDownHist.values()
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
  window.addEventListener('mousemove', onDragMove)
  window.addEventListener('mouseup', onDragEnd)
})

onUnmounted(() => {
  unlistenTick?.()
  unlistenOpened?.()
  unlistenPause?.()
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('mousemove', onDragMove)
  window.removeEventListener('mouseup', onDragEnd)
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

let dragging = false
let dragStart = { x: 0, y: 0 }

function startDrag(e: MouseEvent) {
  dragging = true
  dragStart = { x: e.screenX, y: e.screenY }
}

function onDragMove(e: MouseEvent) {
  if (!dragging) return
  const dx = e.screenX - dragStart.x
  const dy = e.screenY - dragStart.y
  if (dx === 0 && dy === 0) return
  dragStart = { x: e.screenX, y: e.screenY }
  void panel.emit('panel/drag-delta', [dx, dy])
}

function onDragEnd() {
  dragging = false
}

async function togglePin() {
  pinned.value = !pinned.value
  await panel.emit('panel/pin', pinned.value)
  if (pinned.value) {
    await panel.setAlwaysOnTop(true)
  }
}
</script>

<template>
  <PanelHeader
    :pinned="pinned"
    :settings="showSettings"
    title="WinGauge"
    :subtitle="showSettings ? '偏好设置' : paused ? '采样已暂停' : '实时监控'"
    @pin="togglePin"
    @settings="showSettings = !showSettings"
    @header-mousedown="startDrag"
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

    <HealthCard :health="health" />

    <div class="cards">
      <MetricCard
        v-if="cpu"
        label="CPU"
        :value="`${cpu.usage.toFixed(0)}%`"
        :status="cpuStatus(cpu.usage)"
        :bar="cpu.usage"
        :bar-label="cpu.peak != null ? `峰值 ${cpu.peak.toFixed(0)}%` : undefined"
      >
        <template #note>
          <div class="note-row">
            <span>{{ cpu.perCore.length }} 逻辑核 · {{ cpuLoadLabel }}</span>
          </div>
          <CoreBars :cores="cpu.perCore" />
          <Sparkline :values="cpuSeries" :max="100" color="#34c759" :height="24" />
        </template>
      </MetricCard>

      <MetricCard
        v-if="memory"
        label="内存"
        :value="`${memory.usage.toFixed(0)}%`"
        :status="memStatus(memory.usage)"
        :bar="memory.usage"
        :bar-label="`${formatBytes(memory.usedBytes)} / ${formatBytes(memory.totalBytes)}`"
      >
        <template #note>
          <span v-if="memory.committedBytes != null && memory.committedLimitBytes != null">
            已提交 {{ formatBytes(memory.committedBytes) }} / {{ formatBytes(memory.committedLimitBytes) }}
          </span>
          <span v-else>已用 {{ formatBytes(memory.usedBytes) }} / {{ formatBytes(memory.totalBytes) }}</span>
          <Sparkline :values="memSeries" :max="100" color="#0a84ff" :height="24" />
        </template>
      </MetricCard>

      <MetricCard
        v-if="network"
        label="网络"
        :value="formatRate(network.downloadBps)"
        status="ok"
      >
        <template #note>
          <div class="note-row">
            <span>↓ {{ formatRate(network.downloadBps) }} · ↑ {{ formatRate(network.uploadBps) }}</span>
            <span class="iface">{{ network.friendlyName }}</span>
          </div>
          <Sparkline :values="netSeries" color="#5e5ce6" :height="24" />
        </template>
      </MetricCard>

      <MetricCard
        v-if="disk"
        label="磁盘"
        :value="`${((disk.usedBytes / Math.max(1, disk.totalBytes)) * 100).toFixed(0)}%`"
        :status="diskStatus((disk.usedBytes / Math.max(1, disk.totalBytes)) * 100)"
        :bar="(disk.usedBytes / Math.max(1, disk.totalBytes)) * 100"
        :bar-label="`${formatBytes(disk.usedBytes)} / ${formatBytes(disk.totalBytes)}`"
      >
        <template #note>
          系统盘 {{ disk.letter }} · 剩余 {{ formatBytes(Math.max(0, disk.totalBytes - disk.usedBytes)) }}
        </template>
      </MetricCard>

      <div v-if="!cpu && !memory && !network && !disk" class="empty">
        正在等待第一帧采样…
      </div>
    </div>
  </template>

  <footer class="foot">
    <span>WinGauge 0.1.0 · {{ paused ? '已暂停' : showSettings ? '设置' : '1s 刷新' }}</span>
    <span v-if="pinned && !showSettings" class="pinned-tag">已钉住</span>
  </footer>
</template>

<style scoped>
.cards {
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
  padding-right: 2px;
  min-height: 0;
}

.empty {
  padding: 24px 0;
  text-align: center;
  color: var(--text-dim);
  font-size: 12px;
}

.note-row {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.iface {
  color: rgba(255, 255, 255, 0.4);
  max-width: 45%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.foot {
  display: flex;
  justify-content: space-between;
  color: var(--text-dim);
  font-size: 11px;
  padding: 2px 4px 0;
}

.pinned-tag {
  color: var(--accent);
}

.cards::-webkit-scrollbar {
  width: 4px;
}

.cards::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.12);
  border-radius: 2px;
}
</style>
