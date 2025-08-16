<template>
  <Card class="card" @click="clickHandler">
    <template #title>
      <div class="title">
        <div class="single-line-text">
          {{ props.fileName }}
        </div>
        <div class="action">
          <Button
            v-if="['pending', 'downloading'].includes(props.status)"
            size="small"
            variant="text"
            icon="pi pi-pause"
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
            severity="info"
            variant="text"
            icon="pi pi-file"
            aria-label="打开"
            @click="openFile"
          />
          <Button
            size="small"
            severity="info"
            variant="text"
            icon="pi pi-folder-open"
            aria-label="打开文件夹"
            @click="openFolder"
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
    <template #footer v-if="detailProgress.length">
      <div class="details">
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
import { openPath } from '@tauri-apps/plugin-opener'
import { formatSize } from '../utils/format-size'
import { formatTime } from '../utils/format-time'
import { platform } from '@tauri-apps/plugin-os'
import { Command } from '@tauri-apps/plugin-shell'
import { path } from '@tauri-apps/api'
import { exists } from '@tauri-apps/plugin-fs'
import { useToast } from 'primevue'
import { lerp } from '../utils/lerp'
import { oklchToRgb } from '../utils/oklch2rgb'

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
const emit = defineEmits(['resume', 'pause', 'remove', 'update'])
const toast = useToast()

const isShow = ref(false)
const eta = computed(() =>
  props.speed ? (props.fileSize - props.downloaded) / props.speed : 0,
)
const bgProgress = computed(() =>
  props.fileSize ? (props.downloaded / props.fileSize) * 100 + '%' : '0%',
)
const detailProgress = computed(() =>
  props.fileSize
    ? props.readProgress.flatMap((progress, i, arr) =>
        progress
          .map(p => ({
            width: ((p[1] - p[0]) / props.fileSize) * 100,
            left: (p[0] / props.fileSize) * 100,
            top: isShow.value ? i * 12 : 0,
          }))
          .filter(e => e.width >= 1)
          .map(e => ({
            width: e.width + '%',
            left: e.left + '%',
            top: e.top + 'px',
            '--color': oklchToRgb(0.8, 0.18, lerp(0, 360, i / arr.length)),
          })),
      )
    : [],
)
const detailProgressHeight = computed(() =>
  isShow.value ? props.readProgress.length * 12 + 'px' : '12px',
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

async function checkFileExists(filePath: string) {
  if (!(await exists(filePath))) {
    toast.add({
      severity: 'error',
      summary: '文件不存在',
      detail: filePath,
      life: 3000,
    })
    return false
  }
  return true
}

async function openFile() {
  if (!(await checkFileExists(props.filePath))) return
  await openPath(props.filePath)
}
async function openFolder() {
  if (!(await checkFileExists(props.filePath))) return
  const currentPlatform = platform()
  if (currentPlatform === 'windows') {
    await openPath(`/select,${props.filePath}`, 'explorer.exe')
  } else if (currentPlatform === 'macos') {
    await Command.create('open-mac', ['-R', props.filePath]).execute()
  } else if (currentPlatform === 'linux') {
    const dir = await path.dirname(props.filePath)
    await Command.create('open-linux', [dir]).execute()
  }
}
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
  border-radius: 6px;
  background: var(--color);
  transition: top 0.2s ease;
}
.card :deep(.p-card-caption) {
  gap: 0;
}
</style>
