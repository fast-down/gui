import { Channel } from '@tauri-apps/api/core'
import { stopDownload } from '../utils/stop-download'
import { disable, enable } from '@tauri-apps/plugin-autostart'
import { info } from '@tauri-apps/plugin-log'
import { Mutex } from '../utils/mutex'

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
  count: number
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
    const autoStart = ref(false)
    const maxConcurrentTasks = ref(3)
    const showAppMenu = ref(false)

    const runningCount = computed(
      () => list.value.filter(e => e.status === 'downloading').length,
    )

    watch([runningCount, maxConcurrentTasks], async ([curr, max]) => {
      if (curr >= max) return
      const entry = list.value.find(e => e.status === 'pending')
      if (!entry) return
      await resume(entry)
    })

    watch(autoStart, async v => {
      if (v) {
        await enable()
      } else {
        await disable()
      }
    })

    function remove(filePath: string) {
      const i = list.value.findIndex(e => e.filePath === filePath)
      if (i != -1) list.value.splice(i, 1)[0].count++
      return stopDownload(filePath)
    }

    function removeAll() {
      const p: Promise<void>[] = []
      list.value = list.value.filter(e => {
        const r = e.status === 'paused' && e.downloaded >= e.fileSize
        if (r) {
          e.count++
          p.push(stopDownload(e.filePath))
        }
        return !r
      })
      return Promise.all(p)
    }

    function pause(filePath: string) {
      const entry = list.value.find(e => e.filePath === filePath)
      if (!entry) return
      entry.count++
      if (entry.status === 'pending') entry.status = 'paused'
      else return stopDownload(filePath)
    }

    function pauseAll() {
      list.value
        .filter(e => e.status === 'pending')
        .forEach(e => (e.status = 'paused'))
      return Promise.all(
        list.value.map(e => {
          e.count++
          return stopDownload(e.filePath)
        }),
      )
    }

    async function resume(filePathOrEntry: string | DownloadEntry) {
      const entry =
        typeof filePathOrEntry === 'string'
          ? list.value.find(e => e.filePath === filePathOrEntry)
          : filePathOrEntry
      if (!entry || entry.status === 'downloading') return
      entry.status = 'pending'
      if (runningCount.value >= maxConcurrentTasks.value) return
      const localCount = ++entry.count
      const headersObj = buildHeaders(headers.value)
      const urlInfo = await prefetch({
        url: entry.url,
        headers: headersObj,
        proxy: proxy.value,
        acceptInvalidCerts: acceptInvalidCerts.value,
        acceptInvalidHostnames: acceptInvalidHostnames.value,
      }).finally(() => (entry.status = 'paused'))
      if (localCount !== entry.count) return (entry.status = 'paused')
      if (runningCount.value >= maxConcurrentTasks.value) return
      if (!urlInfo.fastDownload || entry.downloaded >= urlInfo.size) {
        entry.status = 'paused'
        return add(entry.url, { urlInfo })
      }
      entry.status = 'downloading'
      const channel = new Channel<DownloadEvent>(res => {
        if (res.event === 'allFinished') {
          entry.status = 'paused'
        } else if (res.event === 'pullProgress') {
          entry.readProgress = res.data[0]
          entry.downloaded = res.data[1]
        } else if (res.event === 'pushProgress') {
          entry.writeProgress = res.data
        } else {
          info(`Event: ${res.event}, Data: ${JSON.stringify(res.data)}`)
        }
      })
      await downloadMulti({
        options: {
          url: urlInfo.finalUrl,
          filePath: entry.filePath,
          fileSize: urlInfo.size,
          threads: threads.value,
          writeBufferSize: writeBufferSize.value,
          writeQueueCap: writeQueueCap.value,
          minChunkSize: minChunkSize.value,
          retryGap: retryGap.value,
          downloadChunks: invertProgress(
            mergeProgress(toRaw(entry.writeProgress)),
            urlInfo.size,
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

    function resumeAll() {
      return Promise.all(
        list.value
          .filter(e => e.status === 'paused' && e.downloaded < e.fileSize)
          .map(resume),
      )
    }

    interface AddOptions {
      urlInfo?: UrlInfo
      paused?: boolean
    }

    const mutex = new Mutex()
    async function add(url: string, options: AddOptions = {}) {
      const unlock = await mutex.lock()
      try {
        const headersObj = buildHeaders(headers.value)
        const urlInfo =
          options.urlInfo ||
          (await prefetch({
            url,
            headers: headersObj,
            proxy: proxy.value,
            acceptInvalidCerts: acceptInvalidCerts.value,
            acceptInvalidHostnames: acceptInvalidHostnames.value,
          }))
        const filePath = await genUniquePath(saveDir.value, urlInfo.name)
        await remove(filePath.path)
        list.value.unshift({
          url: urlInfo.finalUrl,
          filePath: filePath.path,
          fileName: filePath.name,
          fileSize: urlInfo.size,
          speed: 0,
          readProgress: [],
          writeProgress: [],
          elapsedMs: 0,
          status: options.paused
            ? 'paused'
            : runningCount.value < maxConcurrentTasks.value
            ? 'downloading'
            : 'pending',
          downloaded: 0,
          etag: urlInfo.etag,
          lastModified: urlInfo.lastModified,
          count: 0,
        })
        const entry = list.value[0]
        if (options.paused || entry.status !== 'downloading') {
          return
        }
        const channel = new Channel<DownloadEvent>(res => {
          if (res.event === 'allFinished') {
            entry.status = 'paused'
          } else if (res.event === 'pullProgress') {
            entry.readProgress = res.data[0]
            entry.downloaded = res.data[1]
          } else if (res.event === 'pushProgress') {
            entry.writeProgress = res.data
          } else {
            info(`Event: ${res.event}, Data: ${JSON.stringify(res.data)}`)
          }
        })
        if (urlInfo.fastDownload) {
          await downloadMulti({
            options: {
              url: urlInfo.finalUrl,
              acceptInvalidCerts: acceptInvalidCerts.value,
              acceptInvalidHostnames: acceptInvalidHostnames.value,
              downloadChunks: [[0, urlInfo.size]],
              headers: headersObj,
              proxy: proxy.value,
              filePath: filePath.path,
              fileSize: urlInfo.size,
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
              url: urlInfo.finalUrl,
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
      } finally {
        unlock()
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
      autoStart,
      maxConcurrentTasks,
      runningCount,
      showAppMenu,
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
