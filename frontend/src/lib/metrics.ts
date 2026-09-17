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
  /** 厂商 provider 读到的 CPU 温度 °C；通用假数据不会出现 */
  tempC: number | null
}

export interface ThermalSensor {
  id: number
  tempC: number
  label: string
}

export interface FanReading {
  id: number
  rpm: number
}

export interface ThermalSnapshot {
  cpuTempC: number | null
  gpuTempC: number | null
  otherSensors: ThermalSensor[]
  fans: FanReading[]
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
  thermal: ThermalSnapshot | null
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

export function formatUptimeShort(secs: number): string {
  if (!Number.isFinite(secs) || secs <= 0) return '—'
  const d = Math.floor(secs / 86400)
  if (d > 0) return `${d} 天`
  const h = Math.floor(secs / 3600)
  if (h > 0) return `${h} 小时`
  return `${Math.floor(secs / 60)} 分`
}

/** 根据 uptime 反推启动时间文案：「自 9月11日 09:09 启动」 */
export function formatBootLine(uptimeSecs: number): string {
  if (!Number.isFinite(uptimeSecs) || uptimeSecs <= 0) return ''
  const boot = new Date(Date.now() - uptimeSecs * 1000)
  const mo = boot.getMonth() + 1
  const d = boot.getDate()
  const hh = String(boot.getHours()).padStart(2, '0')
  const mm = String(boot.getMinutes()).padStart(2, '0')
  return `自 ${mo}月${d}日 ${hh}:${mm} 启动`
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
      return '异常'
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

export function statusBadge(status: 'ok' | 'warn' | 'bad'): string {
  if (status === 'bad') return '异常'
  if (status === 'warn') return '偏高'
  return '正常'
}

export function loadLabel(usage: number): string {
  if (usage < 30) return '低负载'
  if (usage < 70) return '中负载'
  return '高负载'
}

/** 温度徽标文案与色档：参考图 CPU 卡右上角的 51°C */
export function tempBadge(tempC: number | null | undefined): string | null {
  if (tempC == null || !Number.isFinite(tempC)) return null
  return `${Math.round(tempC)}°C`
}

export function tempTone(tempC: number | null | undefined): 'ok' | 'warn' | 'bad' {
  if (tempC == null) return 'ok'
  if (tempC >= 90) return 'bad'
  if (tempC >= 80) return 'warn'
  return 'ok'
}
