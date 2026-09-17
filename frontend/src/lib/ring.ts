/** 轻量环形缓冲，给迷你趋势图用 */
export class Ring {
  private buf: number[]
  private idx = 0
  private filled = false
  private capacity: number

  constructor(capacity: number, fill = 0) {
    this.capacity = capacity
    this.buf = new Array<number>(capacity).fill(fill)
  }

  push(v: number) {
    this.buf[this.idx] = v
    this.idx = (this.idx + 1) % this.capacity
    if (this.idx === 0) this.filled = true
  }

  values(): number[] {
    if (!this.filled) return this.buf.slice(0, this.idx)
    return [...this.buf.slice(this.idx), ...this.buf.slice(0, this.idx)]
  }

  last(): number | null {
    if (!this.filled && this.idx === 0) return null
    const i = (this.idx - 1 + this.capacity) % this.capacity
    return this.buf[i]
  }
}
