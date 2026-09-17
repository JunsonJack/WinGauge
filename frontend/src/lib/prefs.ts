/** UI 偏好：面板不透明度等，localStorage 持久化 */
export interface UiPrefs {
  /** 面板/胶囊底色不透明度 0.5–1.0 */
  opacity: number
}

const KEY = 'wingauge.ui'

const DEFAULTS: UiPrefs = {
  opacity: 0.92,
}

export function loadUiPrefs(): UiPrefs {
  try {
    const raw = localStorage.getItem(KEY)
    if (!raw) return { ...DEFAULTS }
    const parsed = JSON.parse(raw) as Partial<UiPrefs>
    const opacity = clampOpacity(parsed.opacity ?? DEFAULTS.opacity)
    return { opacity }
  } catch {
    return { ...DEFAULTS }
  }
}

export function saveUiPrefs(prefs: UiPrefs) {
  try {
    localStorage.setItem(KEY, JSON.stringify({ opacity: clampOpacity(prefs.opacity) }))
  } catch {
    /* ignore quota */
  }
}

export function clampOpacity(v: number): number {
  if (!Number.isFinite(v)) return DEFAULTS.opacity
  return Math.min(1, Math.max(0.45, Math.round(v * 100) / 100))
}

/** 应用到 #app 的 CSS 变量 */
export function applyUiPrefs(prefs: UiPrefs) {
  const el = document.documentElement
  el.style.setProperty('--panel-alpha', String(clampOpacity(prefs.opacity)))
}
