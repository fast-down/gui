import { emit } from '@tauri-apps/api/event'
import { Channel } from '@tauri-apps/api/core'

export interface DownloadEntry {
  url: string
  filePath: string
  fileName: string
  fileSize: number
  speed: number
  readProgress: [number, number][][]
  writeProgress: [number, number][][]
  elapsedMs: number
  status: 'pending' | 'downloading' | 'paused'
  downloaded: number
  etag: string | null
  lastModified: string | null
}

export const useAppStore = defineStore(
  'app',
  () => {
    const list = ref<DownloadEntry[]>([])
    const threads = ref(8)
    const saveDir = ref('')
    const headers = ref(String.raw`sec-ch-ua-mobile: ?0
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36
sec-ch-ua: "Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"
sec-ch-ua-platform: "Windows"`)
    const proxy = ref<string | null>(null)
    const writeBufferSize = ref(8 * 1024 * 1024)
    const writeQueueCap = ref(10240)
    const retryGap = ref(500)
    const acceptInvalidCerts = ref(false)
    const acceptInvalidHostnames = ref(false)
    const minChunkSize = ref(8 * 1024)
    const multiplexing = ref(true)
    const writeMethod = ref<'mmap' | 'std'>('mmap')

    function remove(filePath: string) {
      const p = emit('stop-download', { file_path: filePath })
      const i = list.value.findIndex(e => e.filePath === filePath)
      if (i != -1) list.value.splice(i, 1)
      return p
    }

    function removeAll() {
      const p = Promise.all(
        list.value.map(e => emit('stop-download', { file_path: e.filePath })),
      )
      list.value = []
      return p
    }

    function pause(filePath: string) {
      const p = emit('stop-download', { file_path: filePath })
      return p
    }

    function pauseAll() {
      const p = Promise.all(
        list.value.map(e => emit('stop-download', { file_path: e.filePath })),
      )
      list.value
        .filter(e => e.status === 'pending')
        .forEach(e => (e.status = 'paused'))
      return p
    }

    async function resume(filePathOrEntry: string | DownloadEntry) {
      const entry =
        typeof filePathOrEntry === 'string'
          ? list.value.find(e => e.filePath === filePathOrEntry)
          : filePathOrEntry
      if (!entry) return
      const headersObj = buildHeaders(headers.value)
      const info = await prefetch({
        url: entry.url,
        headers: headersObj,
        proxy: proxy.value,
        acceptInvalidCerts: acceptInvalidCerts.value,
        acceptInvalidHostnames: acceptInvalidHostnames.value,
      })
      if (!info.fastDownload || entry.downloaded >= info.size)
        return add(entry.url, info)
      entry.status = 'downloading'
      const channel = new Channel<DownloadEvent>(res => {
        if (res.event === 'allFinished') {
          entry.status = 'paused'
          console.log('completed', mergeProgress(entry.writeProgress))
        } else if (res.event === 'pullProgress') {
          entry.readProgress = res.data[0]
          entry.downloaded = res.data[1]
        } else if (res.event === 'pushProgress') {
          entry.writeProgress = res.data
        } else {
          console.log(res)
        }
      })
      console.log(
        'resume',
        invertProgress(mergeProgress(entry.writeProgress), info.size),
        entry.writeProgress,
      )
      downloadMulti({
        options: {
          url: entry.url,
          filePath: entry.filePath,
          fileSize: info.size,
          threads: threads.value,
          writeBufferSize: writeBufferSize.value,
          writeQueueCap: writeQueueCap.value,
          minChunkSize: minChunkSize.value,
          retryGap: retryGap.value,
          downloadChunks: invertProgress(
            mergeProgress(entry.writeProgress),
            info.size,
          ),
          headers: headersObj,
          multiplexing: multiplexing.value,
          acceptInvalidCerts: acceptInvalidCerts.value,
          acceptInvalidHostnames: acceptInvalidHostnames.value,
          proxy: proxy.value,
          writeMethod: writeMethod.value,
          initProgress: entry.writeProgress,
          initDownloaded: entry.downloaded,
        },
        tx: channel,
      })
    }

    async function resumeAll() {
      Promise.all(
        list.value.filter(item => item.status === 'paused').map(resume),
      )
    }

    async function add(url: string, info?: UrlInfo) {
      const headersObj = buildHeaders(headers.value)
      if (!info)
        info = await prefetch({
          url,
          headers: headersObj,
          proxy: proxy.value,
          acceptInvalidCerts: acceptInvalidCerts.value,
          acceptInvalidHostnames: acceptInvalidHostnames.value,
        })
      const filePath = await genUniquePath(saveDir.value, info.name)
      await remove(filePath.path)
      list.value.unshift({
        url,
        filePath: filePath.path,
        fileName: filePath.name,
        fileSize: info.size,
        speed: 0,
        readProgress: [],
        writeProgress: [],
        elapsedMs: 0,
        status: 'downloading',
        downloaded: 0,
        etag: info.etag,
        lastModified: info.lastModified,
      })
      const entry = list.value[0]
      const channel = new Channel<DownloadEvent>(res => {
        if (res.event === 'allFinished') {
          entry.status = 'paused'
          console.log('completed', mergeProgress(entry.writeProgress))
        } else if (res.event === 'pullProgress') {
          entry.readProgress = res.data[0]
          entry.downloaded = res.data[1]
        } else if (res.event === 'pushProgress') {
          entry.writeProgress = res.data
        } else {
          console.log(res)
        }
      })
      if (info.fastDownload) {
        await downloadMulti({
          options: {
            url: info.finalUrl,
            acceptInvalidCerts: acceptInvalidCerts.value,
            acceptInvalidHostnames: acceptInvalidHostnames.value,
            downloadChunks: [[0, info.size]],
            headers: headersObj,
            proxy: proxy.value,
            filePath: filePath.path,
            fileSize: info.size,
            writeBufferSize: writeBufferSize.value,
            writeQueueCap: writeQueueCap.value,
            retryGap: retryGap.value,
            minChunkSize: minChunkSize.value,
            multiplexing: multiplexing.value,
            threads: threads.value,
            writeMethod: writeMethod.value,
            initProgress: [],
            initDownloaded: 0,
          },
          tx: channel,
        })
      } else {
        await downloadSingle({
          options: {
            url: info.finalUrl,
            acceptInvalidCerts: acceptInvalidCerts.value,
            acceptInvalidHostnames: acceptInvalidHostnames.value,
            headers: headersObj,
            proxy: proxy.value,
            filePath: filePath.path,
            writeBufferSize: writeBufferSize.value,
            writeQueueCap: writeQueueCap.value,
            multiplexing: multiplexing.value,
            retryGap: retryGap.value,
          },
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
      minChunkSize,
      acceptInvalidCerts,
      acceptInvalidHostnames,
      multiplexing,
      writeMethod,
      add,
      remove,
      removeAll,
      resume,
      resumeAll,
      pause,
      pauseAll,
    }
  },
  {
    persist: true,
  },
)
