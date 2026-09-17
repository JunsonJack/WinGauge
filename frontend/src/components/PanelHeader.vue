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
  // 不能用 dragstart 这个名字：它是 DOM 原生保留事件，Vue 会按 HTML5 拖拽处理，
  // 自定义事件永远不触发。改名 header-mousedown
  'header-mousedown': [e: MouseEvent]
}>()
</script>

<template>
  <header class="header" @mousedown="emit('header-mousedown', $event)">
    <div class="titles">
      <div class="title">{{ title }}</div>
      <div v-if="subtitle" class="subtitle">{{ subtitle }}</div>
    </div>
    <div class="actions">
      <button
        class="icon-btn"
        :class="{ active: settings }"
        title="设置"
        @click.stop="emit('settings')"
        @mousedown.stop
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.9.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.9 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3h.1a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.9-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.9v.1a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
        </svg>
      </button>
      <button
        class="icon-btn"
        :class="{ active: pinned }"
        :title="pinned ? '取消钉住' : '钉住（失焦不收起，可拖动）'"
        @click.stop="emit('pin')"
        @mousedown.stop
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 4h6l-1 7 4 3-2 2-4-3-4 3-2-2 4-3z" />
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
  padding: 6px 8px;
  border-radius: 10px;
  background: var(--bg-card);
  cursor: grab;
}

.header:active {
  cursor: grabbing;
}

.titles {
  min-width: 0;
}

.title {
  font-weight: 600;
  font-size: 14px;
}

.subtitle {
  color: var(--text-dim);
  font-size: 11px;
}

.actions {
  display: flex;
  gap: 2px;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text);
}

.icon-btn.active {
  color: var(--accent);
  background: rgba(52, 199, 89, 0.14);
}
</style>
