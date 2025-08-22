import { InvokeArgs } from '@tauri-apps/api/core'

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

export function prefetch(options: Prefetch) {
  return invoke<UrlInfo>('prefetch', options as unknown as InvokeArgs)
}
