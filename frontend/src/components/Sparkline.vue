<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    values: number[]
    values2?: number[]
    max?: number
    color?: string
    color2?: string
    height?: number
  }>(),
  {
    color: '#3b9eff',
    color2: '#2fbf71',
    height: 48,
  },
)

const W = 280

function buildPath(vals: number[], h: number): string {
  if (!vals || vals.length < 2) return ''
  const cap = 60
  const slice = vals.length > cap ? vals.slice(-cap) : vals
  const maxV = Math.max(props.max ?? 100, ...slice, 1)
  const n = slice.length
  const step = W / (n - 1)
  let d = ''
  for (let i = 0; i < n; i += 1) {
    const x = i * step
    const y = h - (slice[i] / maxV) * (h - 3) - 1.5
    d += `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`
  }
  return d
}

const path1 = computed(() => buildPath(props.values, props.height))
const path2 = computed(() => (props.values2 ? buildPath(props.values2, props.height) : ''))
const area1 = computed(() => (path1.value ? `${path1.value}L${W},${props.height}L0,${props.height}Z` : ''))
</script>

<template>
  <svg
    class="spark"
    width="100%"
    :height="height"
    :viewBox="`0 0 ${W} ${height}`"
    preserveAspectRatio="none"
  >
    <path v-if="area1" :d="area1" :fill="color" opacity="0.18" />
    <path v-if="path1" :d="path1" fill="none" :stroke="color" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round" />
    <path v-if="path2" :d="path2" fill="none" :stroke="color2" stroke-width="1.4" stroke-linejoin="round" stroke-linecap="round" />
  </svg>
</template>

<style scoped>
.spark {
  display: block;
  width: 100%;
  margin-top: 6px;
}
</style>
