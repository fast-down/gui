import { InvokeArgs } from '@tauri-apps/api/core'
import { info } from '@tauri-apps/plugin-log'

export interface Prefetch {
  url: string
  headers: Record<string, string>
  proxy: string | null
  acceptInvalidCerts: boolean
  acceptInvalidHostnames: boolean
}

export interface UrlInfo {
  size: number
  name: string
  supportsRange: boolean
  fastDownload: boolean
  finalUrl: string
  etag: string | null
  lastModified: string | null
}

export async function prefetch(options: Prefetch) {
  info(`prefetch(${JSON.stringify(options)})`)
  const res = await invoke<UrlInfo>(
    'prefetch',
    options as unknown as InvokeArgs,
  )
  info(`prefetch(${JSON.stringify(options)}) => ${JSON.stringify(res)}`)
  return res
}
