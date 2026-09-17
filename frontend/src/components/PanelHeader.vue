<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
  pinned: boolean
  settings?: boolean
}>()

const emit = defineEmits<{
  pin: []
  settings: []
  /** 收起为桌面胶囊（与胶囊上的展开是同一语义的两端） */
  toggleForm: []
  drag: []
}>()
</script>

<template>
  <header class="header" @mousedown.left="emit('drag')">
    <div class="brand">
      <svg
        class="logo"
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
      >
        <rect x="3" y="4" width="18" height="12" rx="2" />
        <path d="M2 19h20" />
      </svg>
      <div class="titles">
        <div class="title">{{ title }}</div>
        <div v-if="subtitle" class="subtitle">{{ subtitle }}</div>
      </div>
    </div>

    <div class="actions">
      <!-- 面板 → 胶囊：与胶囊上的展开钮同一位置语义 -->
      <button
        class="icon-btn"
        title="收起为桌面胶囊"
        aria-label="收起为桌面胶囊"
        @click.stop="emit('toggleForm')"
        @mousedown.stop
      >
        <!-- 长条胶囊 + 下箭头，比细横线更易识别 -->
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2.5" y="9" width="19" height="7.5" rx="3.75" />
          <path d="M12 4.5v3.2" />
          <path d="M9.8 6.2 12 4l2.2 2.2" />
        </svg>
      </button>

      <button
        class="icon-btn"
        :class="{ active: settings }"
        title="设置"
        aria-label="设置"
        @click.stop="emit('settings')"
        @mousedown.stop
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3.2" />
          <path d="M19.1 14.6a1.55 1.55 0 0 0 .31 1.71l.06.06a1.9 1.9 0 1 1-2.69 2.69l-.06-.06a1.55 1.55 0 0 0-1.71-.31 1.55 1.55 0 0 0-.94 1.42V21a1.9 1.9 0 1 1-3.8 0v-.09a1.55 1.55 0 0 0-1.01-1.42 1.55 1.55 0 0 0-1.71.31l-.06.06a1.9 1.9 0 1 1-2.69-2.69l.06-.06a1.55 1.55 0 0 0 .31-1.71 1.55 1.55 0 0 0-1.42-.94H3a1.9 1.9 0 1 1 0-3.8h.09a1.55 1.55 0 0 0 1.42-1.01 1.55 1.55 0 0 0-.31-1.71l-.06-.06a1.9 1.9 0 1 1 2.69-2.69l.06.06a1.55 1.55 0 0 0 1.71.31h.08a1.55 1.55 0 0 0 .94-1.42V3a1.9 1.9 0 1 1 3.8 0v.09a1.55 1.55 0 0 0 .94 1.42 1.55 1.55 0 0 0 1.71-.31l.06-.06a1.9 1.9 0 1 1 2.69 2.69l-.06.06a1.55 1.55 0 0 0-.31 1.71v.08a1.55 1.55 0 0 0 1.42.94H21a1.9 1.9 0 1 1 0 3.8h-.09a1.55 1.55 0 0 0-1.42.94z" />
        </svg>
      </button>

      <!-- 实心图钉，16px，比线框更清楚 -->
      <button
        class="icon-btn"
        :class="{ active: pinned }"
        :title="pinned ? '取消钉住' : '钉住（失焦不收起，可拖动）'"
        :aria-label="pinned ? '取消钉住' : '钉住'"
        :aria-pressed="pinned"
        @click.stop="emit('pin')"
        @mousedown.stop
      >
        <svg v-if="pinned" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M16 9V4h1a1 1 0 1 0 0-2H7a1 1 0 1 0 0 2h1v5c0 1.66-1.34 3-3 3v2h5.97v7l1 1 1-1v-7H19v-2c-1.66 0-3-1.34-3-3z" />
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 4h6" />
          <path d="M10 4v5.5L6.2 11.8A2 2 0 0 0 5.2 13.6L5.4 15a1.8 1.8 0 0 0 1.1 1.6L9 17.8V20" />
          <path d="M14 4v5.5l3.8 2.3a2 2 0 0 1 1 1.8L18.6 15a1.8 1.8 0 0 1-1.1 1.6L15 17.8V20" />
          <path d="M8.2 16.2 5 19.5" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 14px;
  background: var(--bg-header);
  border: 1px solid var(--border);
  cursor: grab;
}

.header:active {
  cursor: grabbing;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.logo {
  color: var(--text-mid);
  flex-shrink: 0;
}

.titles {
  min-width: 0;
}

.title {
  font-weight: 650;
  font-size: 14px;
  letter-spacing: -0.01em;
}

.subtitle {
  color: var(--text-dim);
  font-size: 11px;
  margin-top: 1px;
}

.actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
}

.icon-btn:hover {
  background: var(--hover);
  color: var(--text);
}

.icon-btn.active {
  color: var(--accent);
  background: var(--accent-soft);
}
</style>
