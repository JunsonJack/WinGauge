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
  collapse: []
  drag: []
}>()
</script>

<template>
  <header
    class="header"
    @mousedown.left="emit('drag')"
  >
    <div class="brand">
      <svg class="logo" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
        <rect x="3" y="4" width="18" height="12" rx="2" />
        <path d="M2 19h20" />
      </svg>
      <div class="titles">
        <div class="title">{{ title }}</div>
        <div v-if="subtitle" class="subtitle">{{ subtitle }}</div>
      </div>
    </div>
    <div class="actions">
      <button
        class="icon-btn"
        title="收起为桌面胶囊"
        @click.stop="emit('collapse')"
        @mousedown.stop
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="8" width="20" height="8" rx="4" />
          <path d="M8 12h8" />
        </svg>
      </button>
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
  padding: 8px 10px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.55);
  border: 1px solid rgba(255, 255, 255, 0.65);
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
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
}

.icon-btn:hover {
  background: rgba(20, 40, 30, 0.06);
  color: var(--text);
}

.icon-btn.active {
  color: var(--accent);
  background: var(--accent-soft);
}
</style>
