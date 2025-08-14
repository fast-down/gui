import { Channel } from '@tauri-apps/api/core'

export interface DownloadSingle {
  options: DownloadSingleOptions
  tx: Channel<DownloadEvent>
}

export interface DownloadSingleOptions {
  url: string
  filePath: string
  writeBufferSize: number
  writeQueueCap: number
  retryGap: number
  headers: Record<string, string>
  multiplexing: boolean
  acceptInvalidCerts: boolean
  acceptInvalidHostnames: boolean
  proxy: string | null
}

export async function downloadSingle(options: DownloadSingle) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  await invoke('download_single', options as any)
}
