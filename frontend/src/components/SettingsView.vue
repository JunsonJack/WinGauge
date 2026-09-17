<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits<{
  close: []
}>()

const autostart = ref(false)
const autostartBusy = ref(false)
const version = ref('0.1.0')
const error = ref('')

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
        <div class="hint">写入 HKCU\\…\\Run，可被任务管理器管理</div>
      </div>
      <input type="checkbox" :checked="autostart" :disabled="autostartBusy" @change="toggleAutostart" />
    </label>

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
        <div class="hint">本机自用，无云同步 / 无遥测 / 不记键鼠内容</div>
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
  padding: 4px 2px;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.title {
  font-weight: 600;
  font-size: 14px;
}

.close {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
}

.close:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text);
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  cursor: pointer;
}

.row.static {
  cursor: default;
}

.label {
  font-size: 13px;
  font-weight: 500;
}

.hint {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-dim);
  line-height: 1.4;
}

.badge {
  font-size: 11px;
  color: var(--accent);
  background: rgba(52, 199, 89, 0.14);
  padding: 2px 8px;
  border-radius: 999px;
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
  border: 1px solid rgba(255, 69, 58, 0.35);
  background: rgba(255, 69, 58, 0.1);
  color: #ff6961;
  border-radius: 8px;
  padding: 4px 10px;
  font-size: 11px;
  cursor: pointer;
}

.quit:hover {
  background: rgba(255, 69, 58, 0.18);
}
</style>
