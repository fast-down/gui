<template>
  <Dialog
    v-if="updateInfo"
    :visible="props.visible"
    @update:visible="onUpdateVisible"
    modal
    :header="`更新 ${updateInfo.currentVersion} -> ${updateInfo.version}`"
    :style="{ width: '25rem' }"
    :closable="false"
  >
    <div style="overflow-wrap: break-word">
      <div>修复了一些已知问题</div>
      <Divider />
      <div v-if="updateInfo.date">
        发布时间：{{
          formatDate(new Date(updateInfo.date * 1000), 'YYYY-MM-DD HH:mm:ss')
        }}
      </div>
    </div>
    <div class="action">
      <Button
        type="button"
        label="取消"
        severity="secondary"
        @click="emit('update:visible', false)"
      ></Button>
      <Button type="submit" label="重启应用程序" @click="restart"></Button>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { UpdateInfo } from '../utils/updater'
import { formatDate } from '@vueuse/core'
import { acceptUpdate } from '../utils/accept-update'

const props = defineProps<{
  visible: boolean
}>()
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()
const store = useAppStore()

function onUpdateVisible(v: boolean) {
  emit('update:visible', v)
}

async function restart() {
  await store.pauseAll()
  await acceptUpdate()
}

const updateInfo = ref<UpdateInfo | null>(null)

listen<UpdateInfo>('update', event => {
  console.log('update event', event)
  updateInfo.value = event.payload
  onUpdateVisible(true)
})
</script>

<style scoped>
.action {
  display: flex;
  margin-top: 16px;
  justify-content: end;
  gap: 8px;
}
</style>
