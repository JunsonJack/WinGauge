/** UI 偏好：主题、透明度等，localStorage 持久化 */

export type ThemeMode = 'light' | 'dark' | 'system'
export type ResolvedTheme = 'light' | 'dark'

export interface UiPrefs {
  /** 面板/胶囊底色不透明度 0.45–1.0 */
  opacity: number
  /** 主题：浅色 / 深色 / 跟随系统 */
  theme: ThemeMode
}

const KEY = 'wingauge.ui'

const DEFAULTS: UiPrefs = {
  opacity: 0.92,
  theme: 'system',
}

let mediaQuery: MediaQueryList | null = null
let onSystemChange: (() => void) | null = null

export function loadUiPrefs(): UiPrefs {
  try {
    const raw = localStorage.getItem(KEY)
    if (!raw) return { ...DEFAULTS }
    const parsed = JSON.parse(raw) as Partial<UiPrefs>
    return {
      opacity: clampOpacity(parsed.opacity ?? DEFAULTS.opacity),
      theme: normalizeTheme(parsed.theme),
    }
  } catch {
    return { ...DEFAULTS }
  }
}

export function saveUiPrefs(prefs: UiPrefs) {
  try {
    localStorage.setItem(
      KEY,
      JSON.stringify({
        opacity: clampOpacity(prefs.opacity),
        theme: normalizeTheme(prefs.theme),
      }),
    )
  } catch {
    /* ignore quota */
  }
}

export function clampOpacity(v: number): number {
  if (!Number.isFinite(v)) return DEFAULTS.opacity
  return Math.min(1, Math.max(0.45, Math.round(v * 100) / 100))
}

export function normalizeTheme(v: unknown): ThemeMode {
  return v === 'light' || v === 'dark' || v === 'system' ? v : DEFAULTS.theme
}

export function systemPrefersDark(): boolean {
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false
}

export function resolveTheme(mode: ThemeMode): ResolvedTheme {
  if (mode === 'system') return systemPrefersDark() ? 'dark' : 'light'
  return mode
}

/** 应用透明度 + 解析后的主题到 documentElement */
export function applyUiPrefs(prefs: UiPrefs) {
  const el = document.documentElement
  el.style.setProperty('--panel-alpha', String(clampOpacity(prefs.opacity)))
  const resolved = resolveTheme(prefs.theme)
  el.setAttribute('data-theme', resolved)
  // 同步给 WebView 滚动条/表单控件
  el.style.colorScheme = resolved
}

/** 主题为 system 时监听系统切换；返回清理函数 */
export function watchSystemTheme(getPrefs: () => UiPrefs, onChange: () => void): () => void {
  if (onSystemChange) {
    mediaQuery?.removeEventListener('change', onSystemChange)
    onSystemChange = null
    mediaQuery = null
  }
  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery = mq
  const handler = () => {
    if (getPrefs().theme === 'system') onChange()
  }
  onSystemChange = handler
  mq.addEventListener('change', handler)
  return () => {
    mq.removeEventListener('change', handler)
    if (onSystemChange === handler) {
      mediaQuery = null
      onSystemChange = null
    }
  }
}

export const THEME_LABELS: Record<ThemeMode, string> = {
  light: '浅色',
  dark: '深色',
  system: '跟随系统',
}
