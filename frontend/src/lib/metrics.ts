export type HealthBand = 'excellent' | 'good' | 'watch' | 'critical'

export interface HealthIssue {
  metric: string
  reason: string
  points: number
}

export interface HealthSnapshot {
  score: number
  band: HealthBand
  summary: string
  issues: HealthIssue[]
}

export interface DeviceSnapshot {
  host: string
  os: string
  osVersion: string
  cpuBrand: string
  logicalCores: number
  totalMemoryBytes: number
  uptimeSecs: number
}

export interface CpuSnapshot {
  usage: number
  perCore: number[]
  queueLength: number | null
  peak: number | null
}

export interface MemorySnapshot {
  usage: number
  totalBytes: number
  usedBytes: number
  committedBytes: number | null
  committedLimitBytes: number | null
}

export interface NetworkSnapshot {
  ifIndex: number
  friendlyName: string
  downloadBps: number
  uploadBps: number
}

export interface DriveSnapshot {
  letter: string
  totalBytes: number
  usedBytes: number
}

export interface DiskSnapshot {
  systemDrive: DriveSnapshot
}

export interface Snapshot {
  tsMs: number
  device: DeviceSnapshot | null
  health: HealthSnapshot | null
  cpu: CpuSnapshot | null
  memory: MemorySnapshot | null
  network: NetworkSnapshot | null
  disk: DiskSnapshot | null
}

export function formatBytes(n: number, digits = 1): string {
  if (!Number.isFinite(n) || n < 0) return '—'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let v = n
  let i = 0
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024
    i += 1
  }
  if (i === 0) return `${Math.round(v)} ${units[i]}`
  return `${v.toFixed(digits)} ${units[i]}`
}

export function formatRate(bps: number): string {
  if (!Number.isFinite(bps) || bps < 0) return '—'
  // bytes/s → 带单位的速率
  return `${formatBytes(bps, bps >= 1024 * 1024 ? 1 : 0)}/s`
}

export function formatUptime(secs: number): string {
  if (!Number.isFinite(secs) || secs <= 0) return '—'
  const d = Math.floor(secs / 86400)
  const h = Math.floor((secs % 86400) / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (d > 0) return `${d}天${h}小时`
  if (h > 0) return `${h}小时${m}分`
  return `${m}分`
}

export function bandLabel(band: HealthBand | null | undefined): string {
  switch (band) {
    case 'excellent':
      return '很好'
    case 'good':
      return '良好'
    case 'watch':
      return '需注意'
    case 'critical':
    default:
      return '—'
  }
}

export function cpuStatus(usage: number): 'ok' | 'warn' | 'bad' {
  if (usage >= 90) return 'bad'
  if (usage >= 70) return 'warn'
  return 'ok'
}

export function memStatus(usage: number): 'ok' | 'warn' | 'bad' {
  if (usage >= 95) return 'bad'
  if (usage >= 85) return 'warn'
  return 'ok'
}

export function diskStatus(usage: number): 'ok' | 'warn' | 'bad' {
  if (usage >= 95) return 'bad'
  if (usage >= 90) return 'warn'
  return 'ok'
}
