<template>
  <header class="header">
    <Button
      label="新建任务"
      variant="text"
      icon="pi pi-plus"
      @click="createTaskVisible = true"
    />
    <Button label="全部开始" variant="text" icon="pi pi-play" />
    <Button label="全部暂停" variant="text" icon="pi pi-pause" />
    <Button label="全部删除" variant="text" icon="pi pi-trash" />
    <Button label="设置" variant="text" icon="pi pi-cog" />
  </header>
  <main class="main">
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
    >
    </DownloadItem>
  </main>
  <CreateTask v-model:visible="createTaskVisible" />
  <Toast />
</template>

<script lang="ts" setup>
const store = useAppStore()
for (const e of store.list) {
  e.status = 'paused'
  e.readProgress = deepClone(e.writeProgress)
  e.speed = e.elapsedMs ? (e.downloaded / e.elapsedMs) * 1000 : 0
}
const createTaskVisible = ref(false)
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
