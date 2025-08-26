import { Channel } from '@tauri-apps/api/core'
import { stopDownload } from '../binding/stop-download'
import { disable, enable } from '@tauri-apps/plugin-autostart'
import { info } from '@tauri-apps/plugin-log'
import { Mutex } from '../utils/mutex'
import { UrlInfo } from '../binding/prefetch'
import { exit } from '@tauri-apps/plugin-process'
import { WriteMethod } from '../interface/create-download-options'
import { DownloadEvent } from '../interface/event'
import { path } from '@tauri-apps/api'
import { showNotification } from '../binding/notification'
import { isFocusWindow } from '../binding/focus-window'

export interface DownloadConfig {
  threads: number
  saveDir: string
  headers: string
  proxy: string
  writeBufferSize: number
  writeQueueCap: number
  retryGap: number
  acceptInvalidCerts: boolean
  acceptInvalidHostnames: boolean
  minChunkSize: number
  multiplexing: boolean
  writeMethod: WriteMethod
}

export type DownloadStatus = 'pending' | 'downloading' | 'paused'

export interface DownloadEntry {
  url: string
  filePath: string
  fileName: string
  fileSize: number
  speed: number
  readProgress: [number, number][][]
  writeProgress: [number, number][][]
  elapsedMs: number
  status: DownloadStatus
  downloaded: number
  etag: string | null
  lastModified: string | null
  count: number
  config?: Partial<DownloadConfig>
}

export type AddOptions =
  | {
      urlInfo?: Partial<UrlInfo>
      paused?: boolean
      config?: Partial<DownloadConfig>
      needPrefetch: true
    }
  | {
      urlInfo: UrlInfo
      paused?: boolean
      config?: Partial<DownloadConfig>
      needPrefetch?: false
    }

const downloadDir = await path.downloadDir()

export const useAppStore = defineStore(
  'app',
  () => {
    const list = ref<DownloadEntry[]>([])
    const globalConfig = ref<DownloadConfig>({
      threads: 8,
      saveDir: downloadDir,
      headers: String.raw`sec-ch-ua-mobile: ?0
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36
sec-ch-ua: "Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"
sec-ch-ua-platform: "Windows"`,
      proxy: '',
      writeBufferSize: 8 * 1024 * 1024,
      writeQueueCap: 10240,
      retryGap: 500,
      acceptInvalidCerts: false,
      acceptInvalidHostnames: false,
      minChunkSize: 8 * 1024,
      multiplexing: true,
      writeMethod: 'mmap',
    })
    const autoStart = ref(false)
    const maxConcurrentTasks = ref(3)
    const showAppMenu = ref(false)

    const runningCount = computed(() =>
      list.value.reduce((acc, e) => {
        if (e.status === 'downloading') return acc + 1
        return acc
      }, 0),
    )
    const pendingCount = computed(() =>
      list.value.reduce((acc, e) => {
        if (e.status === 'pending') return acc + 1
        return acc
      }, 0),
    )
    const pausedCount = computed(
      () => list.value.length - runningCount.value - pendingCount.value,
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
      const config = {
        ...globalConfig.value,
        ...entry.config,
      }
      const headersObj = buildHeaders(config.headers)
      try {
        const urlInfo = await prefetch({
          url: entry.url,
          headers: headersObj,
          proxy: config.proxy,
          acceptInvalidCerts: config.acceptInvalidCerts,
          acceptInvalidHostnames: config.acceptInvalidHostnames,
        })
        if (localCount !== entry.count) return (entry.status = 'paused')
        if (runningCount.value >= maxConcurrentTasks.value) return
        if (!urlInfo.fastDownload || entry.downloaded >= urlInfo.size) {
          entry.status = 'paused'
          return add(entry.url, { urlInfo, config: entry.config })
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
            threads: config.threads,
            writeBufferSize: config.writeBufferSize,
            writeQueueCap: config.writeQueueCap,
            minChunkSize: config.minChunkSize,
            retryGap: config.retryGap,
            downloadChunks: invertProgress(
              mergeProgress(toRaw(entry.writeProgress)),
              urlInfo.size,
            ),
            headers: headersObj,
            multiplexing: config.multiplexing,
            acceptInvalidCerts: config.acceptInvalidCerts,
            acceptInvalidHostnames: config.acceptInvalidHostnames,
            proxy: config.proxy,
            writeMethod: config.writeMethod,
            initProgress: entry.writeProgress,
            initDownloaded: entry.downloaded,
          },
          tx: channel,
        })
      } catch (e) {
        entry.status = 'paused'
        throw e
      }
    }

    function resumeAll() {
      return Promise.all(
        list.value
          .filter(e => e.status === 'paused' && e.downloaded < e.fileSize)
          .map(resume),
      )
    }

    const mutex = new Mutex()
    async function add(
      url: string,
      options: AddOptions = { needPrefetch: true },
    ) {
      const unlock = await mutex.lock()
      const config = {
        ...globalConfig.value,
        ...options.config,
      }
      try {
        const headersObj = buildHeaders(config.headers)
        const urlInfo = {
          ...(options.needPrefetch
            ? await prefetch({
                url,
                headers: headersObj,
                proxy: config.proxy,
                acceptInvalidCerts: config.acceptInvalidCerts,
                acceptInvalidHostnames: config.acceptInvalidHostnames,
              })
            : {}),
          ...options.urlInfo,
        } as UrlInfo
        const filePath = await genUniquePath(config.saveDir, urlInfo.name)
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
          config: options.config,
        })
        isFocusWindow().then(isFocus => {
          if (isFocus) return
          showNotification({ title: '添加任务成功', body: urlInfo.name })
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
              acceptInvalidCerts: config.acceptInvalidCerts,
              acceptInvalidHostnames: config.acceptInvalidHostnames,
              downloadChunks: [[0, urlInfo.size]],
              headers: headersObj,
              proxy: config.proxy,
              filePath: filePath.path,
              fileSize: urlInfo.size,
              writeBufferSize: config.writeBufferSize,
              writeQueueCap: config.writeQueueCap,
              retryGap: config.retryGap,
              minChunkSize: config.minChunkSize,
              multiplexing: config.multiplexing,
              threads: config.threads,
              writeMethod: config.writeMethod,
              initProgress: [],
              initDownloaded: 0,
            },
            tx: channel,
          })
        } else {
          await downloadSingle({
            options: {
              url: urlInfo.finalUrl,
              acceptInvalidCerts: config.acceptInvalidCerts,
              acceptInvalidHostnames: config.acceptInvalidHostnames,
              headers: headersObj,
              proxy: config.proxy,
              filePath: filePath.path,
              writeBufferSize: config.writeBufferSize,
              writeQueueCap: config.writeQueueCap,
              multiplexing: config.multiplexing,
              retryGap: config.retryGap,
            },
            tx: channel,
          })
        }
      } finally {
        unlock()
      }
    }

    async function restart() {
      await pauseAll()
      await relaunch()
    }

    async function quit() {
      await pauseAll()
      await exit(0)
    }

    return {
      list,
      globalConfig,
      autoStart,
      maxConcurrentTasks,
      runningCount,
      pendingCount,
      pausedCount,
      showAppMenu,
      add,
      remove,
      removeAll,
      resume,
      resumeAll,
      pause,
      pauseAll,
      restart,
      quit,
    }
  },
  {
    persist: true,
  },
)
