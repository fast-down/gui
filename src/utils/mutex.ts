export class Mutex {
  isLocked = false
  queue: ((value: () => void) => void)[] = []

  constructor() {}

  lock() {
    const unlock = () => {
      if (this.queue.length) this.queue.shift()!(unlock)
      else this.isLocked = false
    }
    if (this.isLocked)
      return new Promise<() => void>(resolve => this.queue.push(resolve))
    this.isLocked = true
    return Promise.resolve(unlock)
  }
}
