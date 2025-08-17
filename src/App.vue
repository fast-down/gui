<template>
  <header class="header">
    <Button
      label="新建任务"
      variant="text"
      icon="pi pi-plus"
      @click="createTaskVisible = true"
    />
    <Button
      label="全部开始"
      @click="store.resumeAll"
      variant="text"
      icon="pi pi-play"
    />
    <Button
      label="全部暂停"
      @click="store.pauseAll"
      variant="text"
      icon="pi pi-pause"
    />
    <Button
      label="全部删除"
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
    >
    </DownloadItem>
  </TransitionGroup>
  <CreateTask v-model:visible="createTaskVisible" />
  <SettingsPage v-model:visible="settingsPageVisible" />
  <UpdatePage v-model:visible="updatePageVisible" />
  <Toast />
</template>

<script lang="ts" setup>
import { useToast } from 'primevue'
import { DownloadEntry } from './stores/app'
import { error } from '@tauri-apps/plugin-log'
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { exit, relaunch } from '@tauri-apps/plugin-process'
import { focusWindow } from './utils/focus-window'
import { getCurrentWindow } from '@tauri-apps/api/window'

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
;(async () => {
  const menu = await Menu.new({
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
        async action() {
          await store.pauseAll()
          await relaunch()
        },
      },
      {
        id: 'quit',
        text: '退出',
        async action() {
          await store.pauseAll()
          await exit(0)
        },
      },
    ],
  })
  TrayIcon.new({
    icon: (await defaultWindowIcon()) || undefined,
    menuOnLeftClick: false,
    menu,
    action: e => {
      if (e.type === 'Click' && e.button === 'Left' && e.buttonState === 'Up') {
        console.log(e)
        return focusWindow()
      }
    },
  })
  if (store.showAppMenu) menu.setAsAppMenu()
  const window = getCurrentWindow()
  window.onCloseRequested(e => {
    e.preventDefault()
    return window.hide()
  })
})()
</script>

<style scoped>
.header {
  display: flex;
  gap: 8px;
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
