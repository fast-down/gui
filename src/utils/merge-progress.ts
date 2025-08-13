export function canMerge(a: [number, number], b: [number, number]) {
  return a[0] === b[1] || b[0] === a[1]
}

export function mergeProgress(
  progresses: [number, number][],
  newProgress: [number, number],
) {
  const [newStart, newEnd] = newProgress
  let left = 0
  let right = progresses.length
  while (left < right) {
    const mid = (left + right) >> 1
    if (progresses[mid][0] < newStart) {
      left = mid + 1
    } else {
      right = mid
    }
  }
  const i = left
  if (i === progresses.length) {
    if (progresses.length > 0) {
      const last = progresses[progresses.length - 1]
      if (last[1] === newStart) {
        last[1] = newEnd
        return
      }
    }
    progresses.push(newProgress)
  } else {
    const u1 = i > 0 && canMerge(progresses[i - 1], newProgress)
    const u2 = canMerge(newProgress, progresses[i])
    if (u1 && u2) {
      progresses[i - 1][1] = progresses[i][1]
      progresses.splice(i, 1)
    } else if (u1) {
      progresses[i - 1][1] = newEnd
    } else if (u2) {
      progresses[i][0] = newStart
    } else {
      progresses.splice(i, 0, newProgress)
    }
  }
}
