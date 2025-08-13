interface Prefetch {
  url: string
  headers: Record<string, string>
  proxy: string | null
  acceptInvalidCerts: boolean
  acceptInvalidHostnames: boolean
}

interface UrlInfo {
  size: number
  name: string
  supportsRange: boolean
  fastDownload: boolean
  finalUrl: string
  etag: string | null
  lastModified: string | null
}
