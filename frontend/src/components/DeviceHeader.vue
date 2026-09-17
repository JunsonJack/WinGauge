<script setup lang="ts">
defineProps<{
  host: string
  osLine: string
  cpuLine: string
  memLine: string
  uptimeLine: string
}>()
</script>

<template>
  <section class="device">
    <div class="top">
      <div class="left">
        <svg class="icon" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="4" width="18" height="12" rx="2" />
          <path d="M2 19h20" stroke-linecap="round" />
        </svg>
        <div>
          <div class="host">{{ host }}</div>
          <div class="tags">
            <span class="tag">{{ osLine }}</span>
            <span class="tag">{{ cpuLine }}</span>
            <span class="tag">{{ memLine }}</span>
          </div>
        </div>
      </div>
      <div class="uptime">
        <div class="cap">运行时间</div>
        <div class="val">{{ uptimeLine }}</div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.device {
  padding: 12px 14px;
  border-radius: var(--radius);
  background: var(--bg-card-strong);
  border: 1px solid var(--card-edge);
  box-shadow: var(--shadow);
}

.top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.left {
  display: flex;
  gap: 10px;
  min-width: 0;
}

.icon {
  color: var(--text-mid);
  flex-shrink: 0;
  margin-top: 2px;
}

.host {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin-bottom: 6px;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag {
  font-size: 10px;
  color: var(--text-mid);
  background: var(--hover);
  border-radius: 999px;
  padding: 3px 8px;
  white-space: nowrap;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.uptime {
  text-align: right;
  flex-shrink: 0;
}

.cap {
  font-size: 10px;
  color: var(--text-dim);
}

.val {
  font-size: 15px;
  font-weight: 700;
  margin-top: 2px;
  letter-spacing: -0.02em;
}
</style>
