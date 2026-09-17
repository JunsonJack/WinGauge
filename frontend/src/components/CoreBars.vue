<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  cores: number[]
}>()

/** 最多显示 16 核，更多时抽稀 */
const bars = computed(() => {
  const src = props.cores ?? []
  if (src.length <= 16) return src
  const step = src.length / 16
  const out: number[] = []
  for (let i = 0; i < 16; i += 1) {
    out.push(src[Math.floor(i * step)] ?? 0)
  }
  return out
})
</script>

<template>
  <div class="cores" :title="`共 ${cores.length} 逻辑核`">
    <div
      v-for="(v, i) in bars"
      :key="i"
      class="core"
      :style="{ height: Math.max(8, Math.min(100, v)) + '%' }"
      :data-hot="v >= 80"
    />
  </div>
</template>

<style scoped>
.cores {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  height: 36px;
  margin-top: 8px;
}

.core {
  flex: 1;
  min-width: 4px;
  border-radius: 3px 3px 2px 2px;
  background: linear-gradient(180deg, #6ee7a8, var(--accent));
  transition: height 0.28s ease;
}

.core[data-hot='true'] {
  background: linear-gradient(180deg, #ff9aa0, var(--bad));
}
</style>
