<template>
  <header class="header">
    <Button
      label="新建"
      variant="text"
      icon="pi pi-plus"
      @click="createTaskVisible = true"
    />
    <Button
      label="开始"
      @click="store.resumeAll"
      variant="text"
      icon="pi pi-play"
    />
    <Button
      label="暂停"
      @click="store.pauseAll"
      variant="text"
      icon="pi pi-pause"
    />
    <Button
      label="删除"
      @click="store.removeAll"
      variant="text"
      icon="pi pi-trash"
    />
    <Button
      label="设置"
      variant="text"
      icon="pi pi-cog"
      @click="settingsPageVisible = true"
    />
    <Button label="重启" @click="restart" variant="text" icon="pi pi-refresh" />
    <Button label="退出" @click="quit" variant="text" icon="pi pi-power-off" />
  </header>
  <TransitionGroup
    name="list"
    tag="main"
    class="main"
    @before-leave="onBeforeLeave"
  >
    <DownloadItem
      v-for="item in store.list"
      :downloaded="item.downloaded"
      :elapsed-ms="item.elapsedMs"
      :file-name="item.fileName"
      :file-size="item.fileSize"
      :speed="item.speed"
      :status="item.status"
      :file-path="item.filePath"
      :read-progress="item.readProgress"
      :key="item.filePath"
      class="download-item"
      @remove="store.remove(item.filePath)"
      @pause="store.pause(item.filePath)"
      @resume="store.resume(item.filePath)"
      @update="updateEntry(item, $event)"
      @detail="showDetail(item.filePath)"
    >
    </DownloadItem>
  </TransitionGroup>
  <CreateTask v-model:visible="createTaskVisible" />
  <SettingsPage v-model:visible="settingsPageVisible" />
  <UpdatePage v-model:visible="updatePageVisible" />
  <DetailPage v-model:visible="detailPageVisible" :file-path="detailItem" />
  <Toast />
</template>

<script lang="ts" setup>
import { useToast } from 'primevue'
import { AddOptions, DownloadEntry } from './stores/app'
import { error } from '@tauri-apps/plugin-log'
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { exit, relaunch } from '@tauri-apps/plugin-process'
import { focusWindow } from './utils/focus-window'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link'
import { removeUndefined, UndefinedAble } from './utils/remove-undefined'
import { UrlInfo } from './utils/prefetch'

const toast = useToast()
const store = useAppStore()
for (const e of store.list) {
  e.count = 0
  e.status = 'paused'
  e.readProgress = structuredClone(toRaw(e.writeProgress))
  e.speed = e.elapsedMs ? (e.downloaded / e.elapsedMs) * 1000 : 0
}
const createTaskVisible = ref(false)
const settingsPageVisible = ref(false)
const updatePageVisible = ref(false)
const detailPageVisible = ref(false)
let detailItem: Ref<string> = ref('')

function showDetail(filePath: string) {
  detailItem.value = filePath
  detailPageVisible.value = true
}

async function restart() {
  await store.pauseAll()
  await relaunch()
}

async function quit() {
  await store.pauseAll()
  await exit(0)
}

function updateEntry(
  item: DownloadEntry,
  data: { elapsedMs: number; speed: number },
) {
  item.elapsedMs = data.elapsedMs
  item.speed = data.speed
}

function onBeforeLeave(el: Element) {
  if (el instanceof HTMLElement) el.style.width = el.clientWidth + 'px'
}

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
      action() {
        return store.resumeAll()
      },
    },
    {
      id: 'pauseAll',
      text: '全部暂停',
      action() {
        return store.pauseAll()
      },
    },
    {
      id: 'removeAll',
      text: '全部删除',
      action() {
        return store.removeAll()
      },
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
      action() {
        return restart()
      },
    },
    {
      id: 'quit',
      text: '退出',
      action() {
        return quit()
      },
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
    console.log(url)
    if (url.protocol !== 'fast-down:') continue
    if (url.hostname === 'download') {
      const downloadUrl = url.searchParams.get('url')
      if (!downloadUrl) continue
      const options: AddOptions = {
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
          minChunkSize: maybeInt(url.searchParams.get('minChunkSize')),
          multiplexing: maybeBool(url.searchParams.get('multiplexing')),
          retryGap: maybeInt(url.searchParams.get('retryGap')),
          saveDir: url.searchParams.get('saveDir') || undefined,
          threads: maybeInt(url.searchParams.get('threads')),
          writeBufferSize: maybeInt(url.searchParams.get('writeBufferSize')),
          writeMethod: maybeWriteMethod(url.searchParams.get('writeMethod')),
          writeQueueCap: maybeInt(url.searchParams.get('writeQueueCap')),
        }),
      }
      console.log(downloadUrl, options)
      store.add(downloadUrl, options)
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
    else if (url.hostname === 'relaunch') relaunch()
    else if (url.hostname === 'exit') exit(0)
  }
}

function maybeInt(str: string | null) {
  if (!str) return undefined
  return parseInt(str) || undefined
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
</script>

<style scoped>
.header {
  display: flex;
  padding: 8px;
  overflow-x: auto;
}
.header > * {
  flex-shrink: 0;
}
.main {
  flex: 1;
  overflow: auto;
}
.download-item {
  margin: 8px;
}
.download-item:first-child {
  margin-top: 0;
}
</style>
