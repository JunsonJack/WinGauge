<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  cores: number[]
}>()

/** 最多显示 16 核，更多时抽稀以免柱子过密 */
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
      :style="{ height: Math.max(2, Math.min(100, v)) + '%' }"
      :data-hot="v >= 80"
    />
  </div>
</template>

<style scoped>
.cores {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 28px;
  margin-top: 8px;
}

.core {
  flex: 1;
  min-width: 3px;
  border-radius: 2px 2px 0 0;
  background: rgba(52, 199, 89, 0.55);
  transition: height 0.25s ease;
}

.core[data-hot='true'] {
  background: rgba(255, 69, 58, 0.75);
}
</style>
