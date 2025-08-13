import { invoke } from '@tauri-apps/api/core'

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
  return (await invoke('prefetch', options as any)) as UrlInfo
}
