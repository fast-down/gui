<template>
  <Card class="card">
    <template #title>
      <div class="title">
        {{ props.info.fileName }}
        <div class="action">
          <Button
            v-if="['pending', 'downloading'].includes(props.info.status)"
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
          />
        </div>
      </div>
    </template>
    <template #subtitle>{{ props.info.filePath }}</template>
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
            <td>{{ formatSize(props.info.speed) }}/s</td>
            <td>{{ formatTime(props.info.elapsedMs / 1000) }}</td>
            <td>{{ formatTime(eta) }}</td>
            <td>
              {{ formatSize(props.info.downloaded) }} /
              {{ formatSize(props.info.fileSize) }}
            </td>
          </tr>
        </tbody>
      </table>
    </template>
    <template #footer>
      <div class="details">
        <ProgressBar
          :show-value="false"
          :value="(props.info.downloaded / props.info.fileSize) * 100"
        />
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

const props = defineProps<{
  info: DownloadEntry
}>()
const eta = computed(
  () => (props.info.fileSize - props.info.downloaded) / props.info.speed,
)
const bgProgress = computed(
  () => (props.info.downloaded / props.info.fileSize) * 100 + '%',
)

const emit = defineEmits(['resume', 'pause', 'remove'])
const toast = useToast()

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
  if (!(await checkFileExists(props.info.filePath))) return
  await openPath(props.info.filePath)
}
async function openFolder() {
  if (!(await checkFileExists(props.info.filePath))) return
  const currentPlatform = platform()
  if (currentPlatform === 'windows') {
    await openPath(`/select,${props.info.filePath}`, 'explorer.exe')
  } else if (currentPlatform === 'macos') {
    await Command.create('open-mac', ['-R', props.info.filePath]).execute()
  } else if (currentPlatform === 'linux') {
    const dir = await path.dirname(props.info.filePath)
    await Command.create('open-linux', [dir]).execute()
  }
}
</script>

<style scoped>
.action {
  margin-left: auto;
  display: flex;
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
  background-image: linear-gradient(var(--p-primary-200), var(--p-primary-200));
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
  max-height: 300px;
  overflow: auto;
}
</style>
