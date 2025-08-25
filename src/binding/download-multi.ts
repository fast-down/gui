import { Channel, InvokeArgs } from '@tauri-apps/api/core'
import { info } from '@tauri-apps/plugin-log'
import { WriteMethod } from '../interface/create-download-options'
import { DownloadEvent } from '../interface/event'

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
  writeMethod: WriteMethod
  initProgress: [number, number][][]
  initDownloaded: number
}

export async function downloadMulti(options: DownloadMulti) {
  info(`downloadMulti: ${JSON.stringify(options)}`)
  await invoke('download_multi', options as unknown as InvokeArgs)
}
