<template>
  <header>
    <Button
      label="新建"
      variant="text"
      icon="pi pi-plus"
      @click="emit('createTask')"
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
      @click="emit('openSettings')"
    />
    <Button
      label="重启"
      @click="store.restart"
      variant="text"
      icon="pi pi-refresh"
    />
    <Button
      label="退出"
      @click="store.quit"
      variant="text"
      icon="pi pi-power-off"
    />
    <SelectButton
      v-model="showTypes"
      :options="showOptions"
      optionValue="value"
      optionLabel="name"
      multiple
      :invalid="showTypes.length === 0"
      size="small"
    />
  </header>
  <main ref="main">
    <div :style="{ height: itemHeight + 'px' }" style="position: relative">
      <template
        v-for="[index, item] in showList.entries()"
        :key="item.filePath"
      >
        <DownloadItem
          v-if="itemVisible[index]"
          class="download-item"
          :style="{ top: itemVisibleRange[index].start + 'px' }"
          :downloaded="item.downloaded"
          :elapsed-ms="item.elapsedMs"
          :file-name="item.fileName"
          :file-size="item.fileSize"
          :speed="item.speed"
          :status="item.status"
          :file-path="item.filePath"
          :read-progress="item.readProgress"
          :opened="item.opened"
          @remove="store.remove(item.filePath)"
          @pause="store.pause(item.filePath)"
          @resume="store.resume(item.filePath)"
          @update="updateEntry(item, $event)"
          @detail="showDetail(item.filePath)"
          @toggle-open="item.opened = !item.opened"
        />
      </template>
    </div>
  </main>
  <UpdatePage v-model:visible="updatePageVisible" />
  <DetailPage v-model:visible="detailPageVisible" :file-path="detailItem" />
</template>

<script lang="ts" setup>
import { DownloadEntry, DownloadStatus } from '../stores/app'

const store = useAppStore()
const emit = defineEmits(['createTask', 'openSettings'])
const updatePageVisible = ref(false)
const detailPageVisible = ref(false)
let detailItem: Ref<string> = ref('')

function showDetail(filePath: string) {
  detailItem.value = filePath
  detailPageVisible.value = true
}

const showTypes = ref<DownloadStatus[]>(['paused', 'downloading', 'pending'])
const showOptions = computed<{ name: string; value: DownloadStatus }[]>(() => [
  { name: `下载中 (${store.runningCount})`, value: 'downloading' },
  { name: `等待中 (${store.pendingCount})`, value: 'pending' },
  { name: `已暂停 (${store.pausedCount})`, value: 'paused' },
])
const showList = computed(() =>
  store.list.filter(e => showTypes.value.includes(e.status)),
)
const itemSize = computed(() =>
  showList.value.map(item => {
    let height = 172
    if (item.fileSize && item.readProgress.length) {
      height += 8 // footer gap
      if (item.opened) height += item.readProgress.length * 12
      else height += 12
    }
    return height
  }),
)
const itemHeight = computed(() => itemSize.value.reduce((a, b) => a + b, 0))
const itemVisibleRange = computed(() => {
  let current = 0
  return itemSize.value.map(size => {
    const start = current
    current += size
    return { start, end: current }
  })
})
const el = useTemplateRef<HTMLElement>('main')
const { y: scrollY } = useScroll(el)
const { height } = useElementSize(el)
const overflow = 200
const itemVisible = computed(() =>
  itemVisibleRange.value.map(
    range =>
      range.start < scrollY.value + height.value + overflow &&
      range.end > scrollY.value - overflow,
  ),
)

function updateEntry(
  item: DownloadEntry,
  data: { elapsedMs: number; speed: number },
) {
  item.elapsedMs = data.elapsedMs
  item.speed = data.speed
}
</script>

<style scoped lang="postcss">
header {
  display: flex;
  padding: 8px;
  padding-bottom: 4px;
  overflow-x: auto;

  & > * {
    flex-shrink: 0;
  }
}
main {
  flex: 1;
  overflow: hidden auto;
}
.download-item {
  position: absolute;
  left: 8px;
  right: 8px;
  transition: top 0.2s ease;
}
</style>
