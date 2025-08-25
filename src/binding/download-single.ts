import { Channel, InvokeArgs } from '@tauri-apps/api/core'
import { info } from '@tauri-apps/plugin-log'
import { DownloadEvent } from '../interface/event'

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
  info(`downloadSingle: ${JSON.stringify(options)}`)
  await invoke('download_single', options as unknown as InvokeArgs)
}
