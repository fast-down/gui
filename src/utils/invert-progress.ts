export function invertProgress(
  progress: [number, number][],
  total_size: number,
): [number, number][] {
  if (progress.length === 0) return [[0, total_size]]
  const result: [number, number][] = []
  let prev_end = 0
  for (const range of progress) {
    if (range[0] > prev_end) result.push([prev_end, range[0]])
    prev_end = range[1]
  }
  if (prev_end < total_size) result.push([prev_end, total_size])
  return result
}
