<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { clampOpacity, saveUiPrefs, type UiPrefs } from '../lib/prefs'

const props = defineProps<{
  prefs: UiPrefs
}>()

const emit = defineEmits<{
  close: []
  prefs: [prefs: UiPrefs]
}>()

const autostart = ref(false)
const autostartBusy = ref(false)
const version = ref('0.1.0')
const error = ref('')
const opacity = ref(props.prefs.opacity)

watch(
  () => props.prefs,
  (p) => {
    opacity.value = p.opacity
  },
  { deep: true },
)

watch(opacity, (v) => {
  const next = { opacity: clampOpacity(v) }
  saveUiPrefs(next)
  emit('prefs', next)
})

onMounted(async () => {
  try {
    autostart.value = await invoke<boolean>('get_autostart')
  } catch (e) {
    error.value = String(e)
  }
  try {
    version.value = await invoke<string>('app_version')
  } catch {
    /* ignore */
  }
})

async function toggleAutostart() {
  if (autostartBusy.value) return
  autostartBusy.value = true
  error.value = ''
  try {
    const next = !autostart.value
    await invoke('set_autostart', { enabled: next })
    autostart.value = next
  } catch (e) {
    error.value = String(e)
  } finally {
    autostartBusy.value = false
  }
}

async function quitApp() {
  try {
    await invoke('quit_app')
  } catch (e) {
    error.value = String(e)
  }
}

function resetOpacity() {
  opacity.value = 0.92
}

const opacityPct = computed(() => `${Math.round(opacity.value * 100)}%`)
const year = computed(() => new Date().getFullYear())
</script>

<template>
  <div class="settings">
    <div class="head">
      <span class="title">设置</span>
      <button class="close" title="关闭" @click="emit('close')">✕</button>
    </div>

    <label class="row">
      <div>
        <div class="label">开机自启</div>
        <div class="hint">写入 HKCU Run，可被任务管理器管理</div>
      </div>
      <input type="checkbox" :checked="autostart" :disabled="autostartBusy" @change="toggleAutostart" />
    </label>

    <div class="row static col">
      <div class="row-top">
        <div>
          <div class="label">面板透明度</div>
          <div class="hint">面板与桌面胶囊的底色不透明度</div>
        </div>
        <span class="badge">{{ opacityPct }}</span>
      </div>
      <div class="slider-row">
        <input
          v-model.number="opacity"
          class="slider"
          type="range"
          min="0.45"
          max="1"
          step="0.01"
        />
        <button class="ghost" type="button" @click="resetOpacity">默认</button>
      </div>
      <div class="slider-marks">
        <span>更透</span>
        <span>更实</span>
      </div>
    </div>

    <div class="row static">
      <div>
        <div class="label">采样频率</div>
        <div class="hint">面板可见 1s / 隐藏 2s；隐藏时不停采</div>
      </div>
      <span class="badge">自动</span>
    </div>

    <div class="row static">
      <div>
        <div class="label">隐私</div>
        <div class="hint">本机自用 · 无云同步 · 无遥测 · 不记键鼠内容</div>
      </div>
    </div>

    <p v-if="error" class="err">{{ error }}</p>

    <div class="foot">
      <span>WinGauge v{{ version }} · © {{ year }}</span>
      <button class="quit" @click="quitApp">退出 WinGauge</button>
    </div>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 2px 0;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.title {
  font-weight: 650;
  font-size: 14px;
}

.close {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
}

.close:hover {
  background: rgba(20, 40, 30, 0.06);
  color: var(--text);
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: var(--radius-sm);
  background: var(--bg-card);
  border: 1px solid var(--border);
  box-shadow: var(--shadow);
  cursor: pointer;
}

.row.static {
  cursor: default;
}

.row.col {
  flex-direction: column;
  align-items: stretch;
  gap: 10px;
}

.row-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.label {
  font-size: 13px;
  font-weight: 600;
}

.hint {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-dim);
  line-height: 1.4;
}

.badge {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 8px;
  border-radius: 999px;
  font-variant-numeric: tabular-nums;
}

.slider-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.slider {
  flex: 1;
  accent-color: var(--accent);
  height: 4px;
  cursor: pointer;
}

.ghost {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  font-size: 11px;
  font-weight: 600;
  border-radius: 8px;
  padding: 4px 8px;
  cursor: pointer;
}

.ghost:hover {
  color: var(--text);
  background: rgba(20, 40, 30, 0.04);
}

.slider-marks {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: var(--text-dim);
}

input[type='checkbox'] {
  width: 18px;
  height: 18px;
  accent-color: var(--accent);
}

.err {
  color: var(--bad);
  font-size: 11px;
}

.foot {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-top: 8px;
  font-size: 11px;
  color: var(--text-dim);
}

.quit {
  border: 1px solid rgba(229, 72, 77, 0.3);
  background: var(--bad-soft);
  color: var(--bad);
  border-radius: 9px;
  padding: 5px 10px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.quit:hover {
  background: rgba(229, 72, 77, 0.22);
}
</style>
