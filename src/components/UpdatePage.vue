<template>
  <Dialog
    v-if="updateInfo"
    :visible="props.visible"
    @update:visible="onUpdateVisible"
    modal
    :header="`更新 v${updateInfo.currentVersion} -> v${updateInfo.version}`"
    :style="{ width: '60vw' }"
    :closable="false"
  >
    <div>
      <div class="markdown-body" v-html="updateInfo.body"></div>
      <Divider />
      <div v-if="updateInfo.date">
        发布时间：{{
          formatDate(new Date(updateInfo.date * 1000), 'YYYY-MM-DD HH:mm:ss')
        }}
      </div>
    </div>
    <div class="dialog-action">
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
import { formatDate } from '@vueuse/core'
import { acceptUpdate } from '../binding/accept-update'
import { UpdateInfo } from '../interface/updater'

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
  updateInfo.value = event.payload
  onUpdateVisible(true)
})
</script>
