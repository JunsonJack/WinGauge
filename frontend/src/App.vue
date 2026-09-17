<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'
import { LogicalSize, PhysicalPosition } from '@tauri-apps/api/dpi'
import { currentMonitor } from '@tauri-apps/api/window'
import PanelHeader from './components/PanelHeader.vue'
import MetricCard from './components/MetricCard.vue'
import HealthCard from './components/HealthCard.vue'
import DeviceHeader from './components/DeviceHeader.vue'
import CoreBars from './components/CoreBars.vue'
import Sparkline from './components/Sparkline.vue'
import NetworkCard from './components/NetworkCard.vue'
import SettingsView from './components/SettingsView.vue'
import CapsuleView from './components/CapsuleView.vue'
import { Ring } from './lib/ring'
import { applyUiPrefs, loadUiPrefs, watchSystemTheme, type UiPrefs } from './lib/prefs'
import {
  cpuStatus,
  diskStatus,
  formatBootLine,
  formatBytes,
  formatUptime,
  formatUptimeShort,
  loadLabel,
  memStatus,
  statusBadge,
  tempBadge,
  tempTone,
  type Snapshot,
} from './lib/metrics'

const PANEL_SIZE = new LogicalSize(380, 620)
const CAPSULE_SIZE = new LogicalSize(320, 48)

const panel = getCurrentWebviewWindow()
const pinned = ref(false)
const paused = ref(false)
/** panel | capsule | settings（settings 仅在 panel 模式下有意义） */
const mode = ref<'panel' | 'capsule' | 'settings'>('panel')
const snap = ref<Snapshot | null>(null)
const uiPrefs = ref<UiPrefs>(loadUiPrefs())

const isCapsule = computed(() => mode.value === 'capsule')
const isSettings = computed(() => mode.value === 'settings')

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
let unlistenExpand: UnlistenFn | undefined
let unwatchSystemTheme: (() => void) | undefined

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

const cpuS = computed(() => (cpu.value ? cpuStatus(cpu.value.usage) : 'ok'))
const memS = computed(() => (memory.value ? memStatus(memory.value.usage) : 'ok'))
const cpuTemp = computed(() => cpu.value?.tempC ?? null)
const cpuTempBadge = computed(() => tempBadge(cpuTemp.value))
const cpuTempTone = computed(() => tempTone(cpuTemp.value))
const thermal = computed(() => snap.value?.thermal ?? null)
const fans = computed(() => thermal.value?.fans ?? [])
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
  applyUiPrefs(uiPrefs.value)
  syncRootClass()
  unwatchSystemTheme = watchSystemTheme(
    () => uiPrefs.value,
    () => applyUiPrefs(uiPrefs.value),
  )

  unlistenTick = await listen<Snapshot>('snapshot://tick', (e) => {
    snap.value = e.payload
  })
  unlistenOpened = await listen('panel://opened', () => {})
  unlistenPause = await listen<boolean>('tray://pause-toggled', (e) => {
    paused.value = e.payload === true
  })
  // 托盘点胶囊时：Rust 请求展开
  unlistenExpand = await listen('panel://expand-from-capsule', () => {
    void expandFromCapsule()
  })
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  unlistenTick?.()
  unlistenOpened?.()
  unlistenPause?.()
  unlistenExpand?.()
  unwatchSystemTheme?.()
  window.removeEventListener('keydown', onKeyDown)
})

watch(mode, () => syncRootClass())

/** 胶囊态把整窗变成 pill 圆角（#app.is-capsule） */
function syncRootClass() {
  const root = document.getElementById('app')
  if (!root) return
  root.classList.toggle('is-capsule', isCapsule.value)
}

function onUiPrefsChange(prefs: UiPrefs) {
  uiPrefs.value = prefs
  applyUiPrefs(prefs)
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (mode.value === 'settings') {
      mode.value = 'panel'
      return
    }
    // 胶囊/面板都收起
    void panel.hide()
  }
}

function startNativeDrag() {
  void panel.startDragging()
}

/** 同步尺寸 + 模式到 Rust / 窗口 */
async function applyMode(next: 'panel' | 'capsule' | 'settings') {
  const toCapsule = next === 'capsule'
  mode.value = next

  await panel.emit('panel/capsule', toCapsule)
  await panel.setAlwaysOnTop(toCapsule || pinned.value)

  if (toCapsule) {
    await panel.setSize(CAPSULE_SIZE)
  } else {
    await panel.setSize(PANEL_SIZE)
    // 胶囊 → 面板：把过大面板收回工作区
    try {
      const pos = await panel.outerPosition()
      const mon = await currentMonitor()
      if (pos && mon) {
        const scale = await panel.scaleFactor()
        const w = PANEL_SIZE.width * scale
        const h = PANEL_SIZE.height * scale
        const area = mon.workArea
        let x = pos.x
        let y = pos.y
        if (x + w > area.position.x + area.size.width) {
          x = area.position.x + area.size.width - w
        }
        if (y + h > area.position.y + area.size.height) {
          y = Math.max(area.position.y, area.position.y + area.size.height - h)
        }
        await panel.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)))
      }
    } catch {
      /* 越界修正失败不阻塞展开 */
    }
  }
}

async function collapseToCapsule() {
  if (isCapsule.value) return
  if (isSettings.value) mode.value = 'panel'
  await applyMode('capsule')
}

async function expandFromCapsule() {
  await applyMode('panel')
  await panel.show()
  await panel.setFocus()
}

async function toggleSettings() {
  if (isCapsule.value) {
    await expandFromCapsule()
    mode.value = 'settings'
    return
  }
  mode.value = isSettings.value ? 'panel' : 'settings'
}

async function togglePin() {
  pinned.value = !pinned.value
  await panel.emit('panel/pin', pinned.value)
  await panel.setAlwaysOnTop(pinned.value || isCapsule.value)
}

function quit() {
  void invoke('quit_app')
}
</script>

<template>
  <!-- 胶囊：桌面常驻横条 -->
  <CapsuleView
    v-if="isCapsule"
    :cpu="cpu"
    :memory="memory"
    :network="network"
    :health="health"
    :pinned="pinned"
    :paused="paused"
    @expand="expandFromCapsule"
    @pin="togglePin"
    @drag="startNativeDrag"
  />

  <!-- 完整面板 / 设置 -->
  <template v-else>
    <PanelHeader
      :pinned="pinned"
      :settings="isSettings"
      :title="hostLine"
      :subtitle="paused ? '采样已暂停' : isSettings ? '偏好设置' : '实时监控'"
      @pin="togglePin"
      @settings="toggleSettings"
      @collapse="collapseToCapsule"
      @drag="startNativeDrag"
    />

    <SettingsView
      v-if="isSettings"
      :prefs="uiPrefs"
      @close="mode = 'panel'"
      @prefs="onUiPrefsChange"
    />

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
          :badge="cpuTempBadge || statusBadge(cpuS)"
          :badge-tone="cpuTempBadge ? cpuTempTone : cpuS === 'ok' ? 'ok' : cpuS"
          :bar="cpu.usage"
          :bar-tone="cpuS"
        >
          <template #body>
            <CoreBars :cores="cpu.perCore" />
          </template>
          <template #note>
            {{ loadLabel(cpu.usage) }} · {{ cpu.perCore.length }} 逻辑核
            <template v-if="cpuTempBadge"> · {{ cpuTempBadge }}</template>
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

        <!-- 厂商风扇：仅联想 EC 命中时显示 -->
        <MetricCard
          v-if="fans.length"
          icon="fan"
          label="风扇"
          :value="fans.length === 1 ? `${fans[0].rpm}` : `${Math.max(...fans.map((f) => f.rpm))}`"
          value-unit="RPM"
          badge="实测"
          badge-tone="info"
        >
          <template #note>
            <span v-for="f in fans" :key="f.id" class="fan-item">F{{ f.id }} {{ f.rpm }} RPM</span>
          </template>
        </MetricCard>

        <NetworkCard
          v-if="network"
          :network="network"
          :down-series="netDownSeries"
          :up-series="netUpSeries"
        />

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
            系统盘 {{ disk.letter }} · 已用 {{ formatBytes(disk.usedBytes) }} /
            {{ formatBytes(disk.totalBytes) }} · 剩余
            {{ formatBytes(Math.max(0, disk.totalBytes - disk.usedBytes)) }}
          </template>
        </MetricCard>

        <div v-if="!cpu && !memory && !network && !disk" class="empty">正在等待第一帧采样…</div>
      </div>
    </template>

    <footer class="foot">
      <button class="exit" title="退出" @click="quit">
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M12 3v10M7 8l5-5 5 5" />
          <path d="M5 14v5a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-5" />
        </svg>
        退出 WinGauge
      </button>
      <span class="ver"
        >v0.1.0 · {{ paused ? '已暂停' : isSettings ? '设置' : '1s 刷新' }}
        <template v-if="pinned">· 已钉住 </template></span
      >
    </footer>
  </template>
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

.fan-item {
  display: inline-block;
  margin-right: 10px;
  font-variant-numeric: tabular-nums;
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

.grid::-webkit-scrollbar {
  width: 4px;
}

.grid::-webkit-scrollbar-thumb {
  background: var(--scrollbar);
  border-radius: 2px;
}
</style>
