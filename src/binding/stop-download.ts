import { emit } from '@tauri-apps/api/event'
import { info } from '@tauri-apps/plugin-log'

export function stopDownload(filePath: string) {
  info(`stop-download: ${filePath}`)
  return emit('stop-download', { filePath })
}
