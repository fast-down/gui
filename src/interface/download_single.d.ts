import { Channel } from '@tauri-apps/api/core'

interface DownloadSingle {
  options: DownloadSingleOptions
  tx: Channel
}

interface DownloadSingleOptions {
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
