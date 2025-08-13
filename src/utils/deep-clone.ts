export function deepClone<T>(source: T, hash = new WeakMap()): T {
  if (source === null || typeof source !== 'object') return source
  if (hash.has(source)) return hash.get(source)
  if (source instanceof Date) {
    const copy = new Date(source.getTime())
    hash.set(source, copy)
    return copy as T
  }
  if (source instanceof RegExp) {
    const copy = new RegExp(source.source, source.flags)
    hash.set(source, copy)
    return copy as T
  }
  if (source instanceof Map) {
    const copy = new Map()
    hash.set(source, copy)
    source.forEach((value, key) => {
      copy.set(deepClone(key, hash), deepClone(value, hash))
    })
    return copy as T
  }
  if (source instanceof Set) {
    const copy = new Set()
    hash.set(source, copy)
    source.forEach(value => {
      copy.add(deepClone(value, hash))
    })
    return copy as T
  }
  if (Array.isArray(source)) {
    const copy: any[] = []
    hash.set(source, copy)
    source.forEach(item => {
      copy.push(deepClone(item, hash))
    })
    return copy as T
  }
  if (source instanceof Object) {
    const copy = Object.create(Object.getPrototypeOf(source))
    hash.set(source, copy)
    for (const key in source) {
      if (source.hasOwnProperty(key)) {
        copy[key] = deepClone(source[key], hash)
      }
    }
    return copy as T
  }
  return source
}
