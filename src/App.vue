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
import { DownloadEntry } from './stores/app'

const store = useAppStore()
for (const e of store.list) {
  e.isLocked = false
  e.status = 'paused'
  e.readProgress = deepClone(e.writeProgress)
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
