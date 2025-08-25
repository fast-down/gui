export type WriteMethod = 'mmap' | 'std'

export interface CreateDownloadOptions {
  url: string
  filename: string | null
  acceptInvalidCerts: boolean | null
  acceptInvalidHostnames: boolean | null
  headers: string | null
  proxy: string | null
  minChunkSize: number | null
  multiplexing: boolean | null
  retryGap: number | null
  saveDir: string | null
  threads: number | null
  writeBufferSize: number | null
  writeMethod: WriteMethod | null
  writeQueueCap: number | null
}
