<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  /** 0..100 或任意非负值；内部按 max 归一 */
  values: number[]
  max?: number
  color?: string
  height?: number
}>()

const W = 120
const H = computed(() => props.height ?? 28)

const path = computed(() => {
  const vals = props.values ?? []
  if (vals.length < 2) return ''
  const cap = 60
  const slice = vals.length > cap ? vals.slice(-cap) : vals
  const maxV = Math.max(props.max ?? 100, ...slice, 1)
  const n = slice.length
  const step = W / (n - 1)
  let d = ''
  for (let i = 0; i < n; i += 1) {
    const x = i * step
    const y = H.value - (slice[i] / maxV) * (H.value - 2) - 1
    d += `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`
  }
  return d
})

const area = computed(() => {
  const p = path.value
  if (!p) return ''
  return `${p}L${W},${H.value}L0,${H.value}Z`
})

const stroke = computed(() => props.color ?? 'var(--accent)')
</script>

<template>
  <svg
    class="spark"
    :width="'100%'"
    :height="H"
    :viewBox="`0 0 ${W} ${H}`"
    preserveAspectRatio="none"
  >
    <path v-if="area" :d="area" :fill="stroke" opacity="0.15" />
    <path v-if="path" :d="path" fill="none" :stroke="stroke" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round" />
  </svg>
</template>

<style scoped>
.spark {
  display: block;
  width: 100%;
  margin-top: 6px;
  border-radius: 4px;
  overflow: hidden;
}
</style>
