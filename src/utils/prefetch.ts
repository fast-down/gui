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
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const info: UrlInfo = await invoke('prefetch', options as any)
  try {
    info.name = decodeURIComponent(info.name)
  } catch {
    // ignore
  }
  return info
}
