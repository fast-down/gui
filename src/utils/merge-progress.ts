export function canMerge(a: [number, number], b: [number, number]) {
  return a[0] === b[1] || b[0] === a[1]
}

export function mergeProgress(progresses: [number, number][][]) {
  const list = progresses.flatMap(e => structuredClone(e))
  list.sort((a, b) => a[0] - b[0])
  for (let i = 1; i < list.length; i++) {
    if (canMerge(list[i - 1], list[i])) {
      list[i - 1][1] = list[i][1]
      list.splice(i--, 1)
    }
  }
  return list
}
