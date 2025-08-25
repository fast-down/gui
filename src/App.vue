<template>
  <HomePage
    @create-task="createTaskVisible = true"
    @open-settings="settingsPageVisible = true"
  />
  <CreateTask v-model:visible="createTaskVisible" />
  <SettingsPage v-model:visible="settingsPageVisible" />
  <Toast />
</template>

<script lang="ts" setup>
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { focusWindow } from './binding/focus-window'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link'
import { removeUndefined, UndefinedAble } from './utils/remove-undefined'
import { UrlInfo } from './binding/prefetch'
import { useToast } from 'primevue'
import { error } from '@tauri-apps/plugin-log'
import { listen } from '@tauri-apps/api/event'
import { CreateDownloadOptions } from './interface/create-download-options'

const toast = useToast()
window.addEventListener('error', event => {
  error(
    `Error captured by addEventListener: ${JSON.stringify({
      message: event.message,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      timeStamp: event.timeStamp,
    })}`,
  )
  toast.add({
    severity: 'error',
    summary: '错误',
    detail: event.message,
  })
})
window.addEventListener('unhandledrejection', event => {
  error(
    `Unhandled rejection captured by addEventListener: ${JSON.stringify({
      type: event.type,
      reason: event.reason,
      timeStamp: event.timeStamp,
    })}`,
  )
  toast.add({
    severity: 'error',
    summary: '错误',
    detail: event.reason,
  })
})

const store = useAppStore()
for (const e of store.list) {
  e.count = 0
  e.status = 'paused'
  e.readProgress = structuredClone(toRaw(e.writeProgress))
  e.speed = e.elapsedMs ? (e.downloaded / e.elapsedMs) * 1000 : 0
}

const createTaskVisible = ref(false)
const settingsPageVisible = ref(false)

Menu.new({
  items: [
    {
      id: 'createTask',
      text: '添加任务',
      async action() {
        focusWindow()
        createTaskVisible.value = true
      },
    },
    {
      id: 'resumeAll',
      text: '全部开始',
      action: () => store.resumeAll(),
    },
    {
      id: 'pauseAll',
      text: '全部暂停',
      action: () => store.pauseAll(),
    },
    {
      id: 'removeAll',
      text: '全部删除',
      action: () => store.removeAll(),
    },
    {
      id: 'settings',
      text: '设置',
      action() {
        focusWindow()
        settingsPageVisible.value = true
      },
    },
    {
      id: 'relaunch',
      text: '重启',
      action: () => store.restart(),
    },
    {
      id: 'quit',
      text: '退出',
      action: () => store.quit(),
    },
  ],
}).then(async menu => {
  TrayIcon.new({
    icon: (await defaultWindowIcon()) || undefined,
    menuOnLeftClick: false,
    menu,
    action: e => {
      if (e.type === 'Click' && e.button === 'Left' && e.buttonState === 'Up') {
        focusWindow()
      }
    },
  })
  if (store.showAppMenu) menu.setAsAppMenu()
})
const currWindow = getCurrentWindow()
currWindow.onCloseRequested(e => {
  e.preventDefault()
  return currWindow.hide()
})

getCurrent().then(urls => {
  if (urls) parseDeepLink(urls)
})
onOpenUrl(parseDeepLink)

function parseDeepLink(urls: string[]) {
  for (const urlRaw of urls) {
    const url = new URL(urlRaw)
    if (url.hostname === 'download') {
      const downloadUrl = url.searchParams.get('url')
      if (!downloadUrl) continue
      store.add(downloadUrl, {
        needPrefetch: true,
        urlInfo: removeUndefined<Partial<UrlInfo>>({
          name: url.searchParams.get('filename') || undefined,
        }),
        config: removeUndefined<UndefinedAble<DownloadConfig>>({
          acceptInvalidCerts: maybeBool(
            url.searchParams.get('acceptInvalidCerts'),
          ),
          acceptInvalidHostnames: maybeBool(
            url.searchParams.get('acceptInvalidHostnames'),
          ),
          headers: url.searchParams.get('headers') || undefined,
          proxy: url.searchParams.get('proxy') || undefined,
          minChunkSize:
            maybeInt(url.searchParams.get('minChunkSize')) || undefined,
          multiplexing: maybeBool(url.searchParams.get('multiplexing')),
          retryGap: maybeInt(url.searchParams.get('retryGap')),
          saveDir: url.searchParams.get('saveDir') || undefined,
          threads: maybeInt(url.searchParams.get('threads')) || undefined,
          writeBufferSize:
            maybeInt(url.searchParams.get('writeBufferSize')) || undefined,
          writeMethod: maybeWriteMethod(url.searchParams.get('writeMethod')),
          writeQueueCap:
            maybeInt(url.searchParams.get('writeQueueCap')) || undefined,
        }),
      })
    } else if (url.hostname === 'pause') {
      const filePath = url.searchParams.get('filePath')
      if (filePath) store.pause(filePath)
    } else if (url.hostname === 'resume') {
      const filePath = url.searchParams.get('filePath')
      if (filePath) store.resume(filePath)
    } else if (url.hostname === 'remove') {
      const filePath = url.searchParams.get('filePath')
      if (filePath) store.remove(filePath)
    } else if (url.hostname === 'pauseAll') store.pauseAll()
    else if (url.hostname === 'resumeAll') store.resumeAll()
    else if (url.hostname === 'removeAll') store.removeAll()
    else if (url.hostname === 'relaunch') store.restart()
    else if (url.hostname === 'exit') store.quit()
  }
}

function maybeInt(str: string | null) {
  if (!str) return undefined
  return parseInt(str) ?? undefined
}

function maybeBool(str: string | null) {
  if (['true', '1'].includes(str || '')) return true
  if (['false', '0'].includes(str || '')) return false
  return undefined
}

function maybeWriteMethod(str: string | null) {
  if (str === 'std') return 'std'
  if (str === 'mmap') return 'mmap'
  return undefined
}

listen<CreateDownloadOptions>('download-request', event => {
  store.add(event.payload.url, {
    needPrefetch: true,
    urlInfo: removeUndefined<Partial<UrlInfo>>({
      name: event.payload.filename || undefined,
    }),
    config: removeUndefined<UndefinedAble<DownloadConfig>>({
      acceptInvalidCerts: event.payload.acceptInvalidCerts ?? undefined,
      acceptInvalidHostnames: event.payload.acceptInvalidHostnames ?? undefined,
      headers: event.payload.headers || undefined,
      proxy: event.payload.proxy || undefined,
      minChunkSize: event.payload.minChunkSize || undefined,
      multiplexing: event.payload.multiplexing ?? undefined,
      retryGap: event.payload.retryGap ?? undefined,
      saveDir: event.payload.saveDir || undefined,
      threads: event.payload.threads || undefined,
      writeBufferSize: event.payload.writeBufferSize || undefined,
      writeMethod: maybeWriteMethod(event.payload.writeMethod),
      writeQueueCap: event.payload.writeQueueCap || undefined,
    }),
  })
})
type DownloadItemId = { filePath: string }
listen<DownloadItemId>('pause-request', event =>
  store.pause(event.payload.filePath),
)
listen<DownloadItemId>('resume-request', event =>
  store.resume(event.payload.filePath),
)
listen<DownloadItemId>('remove-request', event =>
  store.remove(event.payload.filePath),
)
listen('pause-all-request', () => store.pauseAll())
listen('resume-all-request', () => store.resumeAll())
listen('remove-all-request', () => store.removeAll())
</script>
