import { Channel } from '@tauri-apps/api/core'

interface DownloadMulti {
  options: DownloadMultiOptions
  tx: Channel
}

interface DownloadMultiOptions {
  url: string
  filePath: string
  fileSize: number
  threads: number
  writeBufferSize: number
  writeQueueCap: number
  minChunkSize: number
  retryGap: number
  downloadChunks: [number, number][]
  headers: Record<string, string>
  multiplexing: boolean
  acceptInvalidCerts: boolean
  acceptInvalidHostnames: boolean
  proxy: string | null
  writeMethod: string
}
