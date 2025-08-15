import { format_time } from './util'

export function formatTime(time_secs: number | bigint): string {
  return format_time(BigInt(time_secs))
}
