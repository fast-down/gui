const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB']

export function formatSize(size: number) {
  if (size === Infinity) return `∞ ${units[0]}`
  if (Number.isNaN(size)) size = 0
  const isNegative = size < 0
  if (size < 0) size = -size
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex += 1
  }
  return `${isNegative ? '-' : ''}${size.toFixed(2)} ${units[unitIndex]}`
}
