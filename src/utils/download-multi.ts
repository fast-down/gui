import { Channel } from '@tauri-apps/api/core'

export interface DownloadMulti {
  options: DownloadMultiOptions
  tx: Channel<DownloadEvent>
}

export interface DownloadMultiOptions {
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
  initProgress: [number, number][][]
  initDownloaded: number
}

export async function downloadMulti(options: DownloadMulti) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  await invoke('download_multi', options as any)
}
