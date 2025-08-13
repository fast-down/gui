import { Channel, invoke } from '@tauri-apps/api/core'

export interface DownloadEntry {
  url: string
  filePath: string
  fileName: string
  fileSize: number
  speed: number
  readProgress: [number, number][][]
  writeProgress: [number, number][][]
  elapsedMs: number
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error'
  downloaded: number
  etag?: string
  lastModified?: string
}

export const useAppStore = defineStore(
  'app',
  () => {
    const list = reactive([
      {
        url: 'https://www.example.com/file.zip',
        filePath: 'C:\\Down load\\1GB.bin',
        fileName: '1GB.bin',
        fileSize: 12 * 1024 * 1024,
        readProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        writeProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        etag: 'W/"123456789"',
        lastModified: '2022-01-01T00:00:00Z',
        elapsedMs: 1.3 * 1000,
        status: 'pending',
        downloaded: 0.7 * 1024 * 1024,
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: 'https://www.example.com/file.zip',
        filePath: '/path/to/file.zip',
        fileName: 'file.zip',
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        writeProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        etag: 'W/"123456789"',
        lastModified: '2022-01-01T00:00:00Z',
        elapsedMs: 1.3 * 1000,
        status: 'pending',
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: 'https://www.example.com/file.zip',
        filePath: '/path/to/file.zip',
        fileName: 'file.zip',
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        writeProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        etag: 'W/"123456789"',
        lastModified: '2022-01-01T00:00:00Z',
        elapsedMs: 1.3 * 1000,
        status: 'pending',
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: 'https://www.example.com/file.zip',
        filePath: '/path/to/file.zip',
        fileName: 'file.zip',
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        writeProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        etag: 'W/"123456789"',
        lastModified: '2022-01-01T00:00:00Z',
        elapsedMs: 1.3 * 1000,
        status: 'pending',
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: 'https://www.example.com/file.zip',
        filePath: '/path/to/file.zip',
        fileName: 'file.zip',
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        writeProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        etag: 'W/"123456789"',
        lastModified: '2022-01-01T00:00:00Z',
        elapsedMs: 1.3 * 1000,
        status: 'pending',
      },
    ] as DownloadEntry[])

    const threads = ref(8)
    const saveDir = ref('')
    const headers = ref(String.raw`sec-ch-ua-mobile: ?0
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36
sec-ch-ua: "Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"
sec-ch-ua-platform: "Windows"`)
    const proxy = ref(void 0 as string | undefined)
    const runningCount = computed(
      () => list.filter(e => e.status === 'downloading').length,
    )
    const maxRunningCount = ref(3)
    const writeBufferSize = ref(8 * 1024 * 1024)
    const writeQueueCap = ref(10240)
    const retryGap = ref(500)

    function removeEntry(filePath: string) {
      for (let i = 0; i < list.length; i++) {
        if (list[i].filePath === filePath) {
          list.splice(i, 1)
          break
        }
      }
    }

    async function addEntry(options: {
      url: string
      threads: number
      saveDir: string
      headers: Record<string, string>
      writeBufferSize: number
      writeQueueCap: number
      retryGap: number
      proxy?: string
    }) {
      const info: UrlInfo = await invoke('prefetch', {
        url: options.url,
        headers: options.headers,
        proxy: options.proxy,
      })
      console.log(info)
      let filePath: UniquePath = await invoke('gen_unique_path', {
        dir: options.saveDir,
        name: info.name,
      })
      console.log(filePath)
      removeEntry(filePath.path)
      const status =
        runningCount.value < maxRunningCount.value ? 'downloading' : 'pending'
      list.push({
        url: options.url,
        filePath: filePath.path,
        fileName: filePath.name,
        fileSize: info.size,
        speed: 0,
        readProgress: [],
        writeProgress: [],
        elapsedMs: 0,
        status,
        downloaded: 0,
      })
      if (status !== 'downloading') return
      let channel = new Channel<Event>()
      let stop_channel: Channel<void>
      if (info.fastDownload) {
        stop_channel = await invoke('download_multi', {
          url: info.finalUrl,
          filePath: filePath.path,
          fileSize: info.size,
          threads: options.threads,
          writeBufferSize: 1024 * 1024,
          writeQueueCap: options.writeQueueCap,
          downloadChunks: [0, info.size],
          retryGap: options.retryGap,
          headers: options.headers,
          proxy: options.proxy,
          tx: channel,
        })
      }
    }
    return {
      list,
      threads,
      saveDir,
      headers,
      proxy,
      writeBufferSize,
      writeQueueCap,
      retryGap,
      addEntry,
      removeEntry,
    }
  },
  {
    persist: true,
  },
)

export interface UrlInfo {
  size: number
  name: string
  supportsRange: boolean
  fastDownload: boolean
  finalUrl: string
  etag: string | null
  lastModified: string | null
}

export interface UniquePath {
  dir: string
  name: string
  path: string
}
