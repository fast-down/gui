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
      :info="item"
      :key="item.filePath"
      class="download-item"
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
  e.readProgress = []
  e.readProgress.concat(e.writeProgress)
  e.speed = (e.downloaded / e.elapsedMs) * 1000
}

const createTaskVisible = ref(false)

// function createTask(isActive: Ref<boolean>) {
//   if (!valid.value) {
//     return
//   }
//   isActive.value = false
//   const urls = rawUrls.value.split('\n').map(e => e.trim())
//   rawUrls.value = ''
//   const headers: Record<string, string> = {}
//   for (const [k, v] of store.headers
//     .split('\n')
//     .map(e => e.trim())
//     .filter(Boolean)
//     .map(e => e.split(':').map(e => e.trim()))) {
//     headers[k] = v
//   }
//   for (const url of urls) {
//     store.addEntry({
//       url,
//       headers,
//       threads: store.threads,
//       saveDir: store.saveDir,
//       proxy: store.proxy,
//     })
//   }
// }
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
