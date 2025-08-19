<template>
  <Card class="card" @click="clickHandler">
    <template #title>
      <div class="title">
        <div class="single-line-text">
          {{ props.fileName }}
        </div>
        <div class="action">
          <Button
            v-if="props.status === 'downloading'"
            size="small"
            variant="text"
            icon="pi pi-pause"
            aria-label="暂停"
            @click="emit('pause')"
          />
          <Button
            v-else-if="props.status === 'pending'"
            size="small"
            variant="text"
            icon="pi pi-flag"
            aria-label="暂停"
            @click="emit('pause')"
          />
          <Button
            v-else
            size="small"
            variant="text"
            icon="pi pi-play"
            aria-label="开始"
            @click="emit('resume')"
          />
          <Button
            size="small"
            severity="help"
            variant="text"
            icon="pi pi-info-circle"
            aria-label="详情"
            @click="emit('detail')"
          />
          <Button
            size="small"
            severity="info"
            variant="text"
            icon="pi pi-file"
            aria-label="打开"
            @click="openFile(props.filePath)"
          />
          <Button
            size="small"
            severity="info"
            variant="text"
            icon="pi pi-folder-open"
            aria-label="打开文件夹"
            @click="openFolder(props.filePath)"
          />
          <Button
            size="small"
            severity="danger"
            variant="text"
            icon="pi pi-trash"
            aria-label="删除"
            @click="emit('remove')"
          />
        </div>
      </div>
    </template>
    <template #subtitle>
      <div class="single-line-text">{{ props.filePath }}</div>
    </template>
    <template #content>
      <table class="table">
        <thead class="thead">
          <tr>
            <th>速度</th>
            <th>用时</th>
            <th>剩余</th>
            <th>进度</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>{{ formatSize(props.speed) }}/s</td>
            <td>{{ formatTime(props.elapsedMs / 1000) }}</td>
            <td>{{ formatTime(eta) }}</td>
            <td>
              {{ formatSize(props.downloaded) }} /
              {{ formatSize(props.fileSize) }}
            </td>
          </tr>
        </tbody>
      </table>
    </template>
    <template #footer v-if="detailProgressRaw.length">
      <div class="details" :class="{ open: isShow }">
        <div
          v-for="info in detailProgress"
          :style="info"
          :key="info.left"
        ></div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { formatSize } from '../utils/format-size'
import { formatTime } from '../utils/format-time'
import { lerp } from '../utils/lerp'
import { oklchToRgb } from '../utils/oklch2rgb'
import { openFile, openFolder } from '../utils/open'

const props = defineProps<{
  downloaded: number
  elapsedMs: number
  fileName: string
  filePath: string
  fileSize: number
  readProgress: [number, number][][]
  speed: number
  status: 'pending' | 'downloading' | 'paused'
}>()
const emit = defineEmits(['resume', 'pause', 'remove', 'update', 'detail'])

const isShow = ref(false)
const eta = computed(() =>
  props.speed ? (props.fileSize - props.downloaded) / props.speed : 0,
)
const bgProgress = computed(() =>
  props.fileSize ? (props.downloaded / props.fileSize) * 100 + '%' : '0%',
)
const detailProgressRaw = computed(() => {
  if (!props.fileSize) return []
  return props.readProgress
    .map((progress, i, arr) =>
      progress
        .map(p => ({
          width: ((p[1] - p[0]) / props.fileSize) * 100,
          left: (p[0] / props.fileSize) * 100,
          '--rgb': oklchToRgb(0.8, 0.18, lerp(0, 360, i / arr.length)),
          backgroundColor: `oklch(0.8 0.18 ${lerp(0, 360, i / arr.length)})`,
        }))
        .filter(e => e.width >= 1),
    )
    .filter(e => e.length)
    .map((e, i) =>
      e.map(e => ({
        ...e,
        top: isShow.value ? i * 12 : 0,
      })),
    )
})
const detailProgress = computed(() => {
  if (!props.fileSize) return []
  const t = detailProgressRaw.value.flat()
  t.sort((a, b) => a.left - b.left)
  return t.map(e => ({
    ...e,
    left: e.left + '%',
    top: e.top + 'px',
    width: e.width + '%',
  }))
})
const detailProgressHeight = computed(() =>
  isShow.value ? detailProgressRaw.value.length * 12 + 'px' : '12px',
)

let timer: number | null = null
watch(
  () => props.status,
  newStatus => {
    if (newStatus === 'downloading') {
      let lastTime = Date.now()
      let oldDownloaded = props.downloaded
      timer = setInterval(() => {
        const dTime = Date.now() - lastTime
        emit('update', {
          elapsedMs: dTime + props.elapsedMs,
          speed: ((props.downloaded - oldDownloaded) / dTime) * 1000,
        })
        lastTime = Date.now()
        oldDownloaded = props.downloaded
      }, 1000)
    } else if (timer) {
      clearInterval(timer)
      timer = null
    }
  },
  {
    immediate: true,
  },
)

async function clickHandler(event: MouseEvent) {
  let target = event.target as HTMLElement
  while (target != document.body) {
    if (target instanceof HTMLButtonElement) return
    target = target.parentElement as HTMLElement
  }
  isShow.value = !isShow.value
}
</script>

<style scoped>
.single-line-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.action {
  display: flex;
  margin-left: auto;
}
.title {
  display: flex;
  width: 100%;
  align-items: center;
}
.table {
  width: 100%;
}
.thead th {
  text-align: start;
}
.card {
  background-image: linear-gradient(var(--p-primary-100), var(--p-primary-100));
  background-repeat: no-repeat;
  background-size: v-bind('bgProgress') 100%;
}
@media (prefers-color-scheme: dark) {
  .card {
    background-image: linear-gradient(
      var(--p-primary-900),
      var(--p-primary-900)
    );
  }
}
.details {
  position: relative;
  height: v-bind('detailProgressHeight');
  transition: height 0.2s ease;
}
.details > div {
  position: absolute;
  height: 12px;
  transition: top 0.2s ease, border-radius 0.2s ease;
  background-color: var(--rgb);
}
.details.open > div {
  border-radius: 6px;
}
.details > div:first-child {
  border-top-left-radius: 6px;
  border-bottom-left-radius: 6px;
}
.details.open > div:first-child {
  border-radius: 6px;
}
.details > div:last-child {
  border-top-right-radius: 6px;
  border-bottom-right-radius: 6px;
}
.details.open > div:last-child {
  border-radius: 6px;
}
.card :deep(.p-card-caption) {
  gap: 0;
}
</style>
