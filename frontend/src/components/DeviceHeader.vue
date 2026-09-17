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
    <div class="host">{{ host }}</div>
    <div class="lines">
      <div>{{ osLine }}</div>
      <div>{{ cpuLine }}</div>
      <div>{{ memLine }}</div>
      <div class="uptime">{{ uptimeLine }}</div>
    </div>
  </section>
</template>

<style scoped>
.device {
  padding: 10px 12px;
  border-radius: var(--radius);
  background: var(--bg-card);
  border: 1px solid var(--border);
}

.host {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 4px;
}

.lines {
  display: flex;
  flex-direction: column;
  gap: 1px;
  font-size: 11px;
  color: var(--text-dim);
  line-height: 1.45;
}

.uptime {
  margin-top: 2px;
  color: rgba(255, 255, 255, 0.7);
}
</style>
