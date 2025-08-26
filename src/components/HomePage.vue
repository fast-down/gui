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
  <TransitionGroup
    name="list"
    tag="main"
    ref="main"
    @before-leave="onBeforeLeave"
  >
    <template v-for="[index, item] in showList.entries()" :key="item.filePath">
      <div class="download-item" :style="{ height: itemSize[index] + 'px' }">
        <DownloadItem
          v-if="itemVisible[index]"
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
      </div>
    </template>
  </TransitionGroup>
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
  showList.value.map((item, i, arr) => {
    let height = 164
    if (i !== 0) height += 8 // margin-top
    if (i === arr.length - 1) height += 8 // margin-bottom
    if (item.fileSize && item.readProgress.length) {
      height += 8 // footer gap
      if (item.opened) height += item.readProgress.length * 12
      else height += 12
    }
    return height
  }),
)
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

function onBeforeLeave(el: Element) {
  if (el instanceof HTMLElement) el.style.width = el.clientWidth + 'px'
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
  box-sizing: border-box;
  padding: 8px 8px 0 8px;

  &:first-child {
    padding-top: 0;
  }
  &:last-child {
    padding-bottom: 8px;
  }
}
</style>
