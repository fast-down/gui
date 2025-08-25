const oneSecond = 1
const oneMinute = oneSecond * 60
const oneHour = oneMinute * 60
const oneDay = oneHour * 24

export function formatTime(sec: number): string {
  if (sec === Infinity) return '∞:∞:∞'
  if (Number.isNaN(sec)) sec = 0
  const isNegative = sec < 0
  if (sec < 0) sec = -sec
  if (sec < oneDay) {
    const seconds = sec % oneMinute
    const minutes = Math.floor((sec / oneMinute) % oneMinute)
    const hours = Math.floor(sec / oneHour)
    return `${isNegative ? '-' : ''}${hours
      .toString()
      .padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds
      .toFixed(0)
      .padStart(2, '0')}`
  }
  const remainder = sec % oneDay
  const days = Math.floor(sec / oneDay)
  const seconds = remainder % oneMinute
  const minutes = Math.floor((remainder / oneMinute) % oneMinute)
  const hours = Math.floor(remainder / oneHour)
  return `${isNegative ? '-' : ''}${days}d ${hours
    .toString()
    .padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds
    .toFixed(0)
    .padStart(2, '0')}`
}
