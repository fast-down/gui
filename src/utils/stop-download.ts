import { emit } from '@tauri-apps/api/event'

export function stopDownload(filePath: string) {
  return emit('stop-download', { file_path: filePath })
}
