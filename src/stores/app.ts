import { Channel, invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { DownloadMulti } from '../interface/download_multi'
import { DownloadSingle } from '../interface/download_single'

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
    const proxy = ref(null as string | null)
    const runningCount = computed(
      () => list.value.filter(e => e.status === 'downloading').length,
    )
    const maxRunningCount = ref(3)
    const writeBufferSize = ref(8 * 1024 * 1024)
    const writeQueueCap = ref(10240)
    const retryGap = ref(500)
    const acceptInvalidCerts = ref(false)
    const acceptInvalidHostnames = ref(false)
    const minChunkSize = ref(8 * 1024)
    const multiplexing = ref(true)
    const writeMethod = ref<'mmap' | 'std'>('mmap')

    async function remove(filePath: string) {
      for (let i = 0; i < list.value.length; i++) {
        if (list.value[i].filePath === filePath) {
          list.value.splice(i, 1)
          await pause(filePath)
          break
        }
      }
    }

    function pause(filePath: string) {
      for (let i = 0; i < list.value.length; i++) {
        if (list.value[i].filePath === filePath) {
          list.value[i].status = 'paused'
        }
      }
      return emit('stop-download', { file_path: filePath })
    }

    function resume(filePath: string) {}

    async function add(url: string) {
      console.log(`add url: ${url}`)
      const headersObj = buildHeaders(headers.value)
      const info: UrlInfo = await invoke('prefetch', {
        url,
        headers: headersObj,
        proxy: proxy.value,
        acceptInvalidCerts: acceptInvalidCerts.value,
        acceptInvalidHostnames: acceptInvalidHostnames.value,
      } as Prefetch as Record<string, any>)
      console.log(info)
      const filePath: UniquePath = await invoke('gen_unique_path', {
        dir: saveDir.value,
        name: info.name,
      })
      console.log(filePath)
      await remove(filePath.path)
      const status =
        runningCount.value < maxRunningCount.value ? 'downloading' : 'pending'
      list.value.push({
        url,
        filePath: filePath.path,
        fileName: filePath.name,
        fileSize: info.size,
        speed: 0,
        readProgress: [],
        writeProgress: [],
        elapsedMs: 0,
        status,
        downloaded: 0,
        etag: info.etag,
        lastModified: info.lastModified,
      })
      const entry = list.value[list.value.length - 1]
      if (status !== 'downloading') return
      const channel = new Channel<DownloadEvent>(res => {
        if (res.event === 'allFinished') {
          entry.status = 'completed'
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
        await invoke('download_multi', {
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
          },
          tx: channel,
        } as DownloadMulti as Record<string, any>)
      } else {
        await invoke('download_single', {
          options: {
            url: info.finalUrl,
            acceptInvalidCerts: acceptInvalidCerts.value,
            acceptInvalidHostnames: acceptInvalidHostnames.value,
            headers: headersObj,
            proxy: proxy.value,
            filePath: filePath.path,
            fileSize: info.size,
            writeBufferSize: writeBufferSize.value,
            writeQueueCap: writeQueueCap.value,
            multiplexing: multiplexing.value,
            retryGap: retryGap.value,
          },
          tx: channel,
        } as DownloadSingle as Record<string, any>)
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
      acceptInvalidCerts,
      acceptInvalidHostnames,
      minChunkSize,
      multiplexing,
      runningCount,
      maxRunningCount,
      writeMethod,
      add,
      remove,
      resume,
      pause,
    }
  },
  {
    persist: true,
  },
)
