import { format_time } from './util'

export function formatTime(time_secs: number | bigint): string {
  if (typeof time_secs === 'number') return format_time(BigInt(time_secs|0));
  return format_time(time_secs)
}
