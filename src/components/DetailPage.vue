<template>
  <Dialog
    :visible="props.visible"
    @update:visible="onUpdateVisible"
    modal
    header="详情"
    :style="{ width: '60vw' }"
    :closable="false"
  >
    <Form v-slot="$form" :initial-values :resolver @submit="onFormSubmit">
      <div class="fields">
        <div>
          <IftaLabel style="display: flex">
            <Textarea name="url" rows="5" auto-resize style="width: 100%" />
            <label for="url">URL</label>
          </IftaLabel>
          <Message
            v-if="$form.url?.invalid"
            severity="error"
            size="small"
            variant="simple"
            style="margin-top: 4px"
            >{{ $form.url.error?.message }}</Message
          >
        </div>
        <IftaLabel>
          <InputNumber name="threads" :min="1" fluid />
          <label for="threads">线程数</label>
        </IftaLabel>
        <div>
          <div class="save-dir-container">
            <IftaLabel style="flex: 1">
              <InputText name="saveDir" fluid id="save-dir-input" />
              <label for="saveDir">保存目录</label>
            </IftaLabel>
            <Button
              variant="text"
              icon="pi pi-folder-open"
              aria-label="选取文件夹"
              size="large"
              @click="selectDir"
            />
          </div>
          <Message
            v-if="$form.saveDir?.invalid"
            severity="error"
            size="small"
            variant="simple"
            >{{ $form.saveDir.error?.message }}</Message
          >
        </div>
        <div>
          <IftaLabel style="display: flex">
            <Textarea name="headers" rows="5" auto-resize style="width: 100%" />
            <label for="headers">请求头 (Key: Value)</label>
          </IftaLabel>
          <Message
            v-if="$form.headers?.invalid"
            severity="error"
            size="small"
            variant="simple"
            style="margin-top: 4px"
            >{{ $form.headers.error?.message }}</Message
          >
        </div>
        <div>
          <IftaLabel>
            <InputText name="proxy" fluid />
            <label for="proxy">代理</label>
          </IftaLabel>
          <Message
            v-if="$form.proxy?.invalid"
            severity="error"
            size="small"
            variant="simple"
            >{{ $form.proxy.error?.message }}</Message
          >
        </div>
        <IftaLabel>
          <InputNumber name="writeBufferSize" :min="0" fluid />
          <label for="writeBufferSize">写入缓冲区大小 (字节)</label>
        </IftaLabel>
        <IftaLabel>
          <InputNumber name="writeQueueCap" :min="0" fluid />
          <label for="writeQueueCap">写入队列容量</label>
        </IftaLabel>
        <IftaLabel>
          <InputNumber name="retryGap" :min="0" fluid />
          <label for="retryGap">重试间隔 (ms)</label>
        </IftaLabel>
        <IftaLabel>
          <InputNumber name="minChunkSize" :min="0" fluid />
          <label for="minChunkSize">最小分块大小 (字节)</label>
        </IftaLabel>
        <Select
          name="writeMethod"
          :options="writeMethodOptions"
          option-label="name"
          option-value="code"
          placeholder="写入方式"
          fluid
        />
        <label for="multiplexing">是否启用多路复用 (建议速度慢时关闭)</label>
        <ToggleSwitch name="multiplexing" />
        <label for="acceptInvalidCerts">是否接受无效证书 (不安全)</label>
        <ToggleSwitch name="acceptInvalidCerts" />
        <label for="acceptInvalidHostnames">是否接受无效主机名 (不安全)</label>
        <ToggleSwitch name="acceptInvalidHostnames" />
      </div>
      <div class="action">
        <Button
          type="button"
          label="取消"
          severity="secondary"
          @click="emit('update:visible', false)"
        />
        <Button type="submit" label="保存" />
      </div>
    </Form>
  </Dialog>
</template>

<script setup lang="ts">
import { Form, FormResolverOptions, FormSubmitEvent } from '@primevue/forms'
import { open } from '@tauri-apps/plugin-dialog'
import { writeMethodOptions } from '../utils/write-method-options'
import { mergeConfig } from '../utils/merge-config'

const props = defineProps<{
  visible: boolean
  filePath: string
}>()
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()
const store = useAppStore()

const itemIndex = computed(() =>
  store.list.findIndex(e => e.filePath === props.filePath),
)

const initialValues = ref({
  url: '',
  threads: 8,
  saveDir: '',
  headers: '',
  proxy: '',
  writeBufferSize: 8 * 1024 * 1024,
  writeQueueCap: 10240,
  retryGap: 500,
  minChunkSize: 8 * 1024,
  acceptInvalidCerts: false,
  acceptInvalidHostnames: false,
  multiplexing: true,
  writeMethod: 'mmap',
})
watchEffect(() => {
  initialValues.value = {
    ...store.globalConfig,
    ...store.list[itemIndex.value]?.config,
    proxy:
      store.list[itemIndex.value]?.config?.proxy ||
      store.globalConfig.proxy ||
      '',
    url: store.list[itemIndex.value]?.url || '',
  }
})

async function resolver({ values }: FormResolverOptions) {
  const errors = {} as {
    [k in keyof typeof values]: { message: string }[]
  }
  if (typeof values.url !== 'string' || !values.url.trim())
    errors.url = [{ message: '请输入下载链接' }]
  else {
    try {
      const url = new URL(values.url)
      if (!['http:', 'https:'].includes(url.protocol)) {
        errors.url = [{ message: `URL 协议不正确` }]
      }
    } catch {
      errors.url = [{ message: `URL 格式不正确` }]
    }
  }
  if (typeof values.saveDir !== 'string' || !values.saveDir.trim())
    errors.saveDir = [{ message: '请选择一个保存目录' }]
  else {
    try {
      const res = await formatDir(values.saveDir)
      if (!res) errors.saveDir = [{ message: '目录不存在' }]
    } catch {
      errors.saveDir = [{ message: '目录格式不正确' }]
    }
  }
  if (typeof values.proxy === 'string' && values.proxy) {
    try {
      const url = new URL(values.proxy)
      if (!['http:', 'https:', 'socks:', 'socks5:'].includes(url.protocol))
        errors.proxy = [{ message: '不支持的协议' }]
    } catch {
      errors.proxy = [{ message: '代理格式不正确' }]
    }
  }
  const headers: string[] = values.headers
    .split('\n')
    .map((e: string) => e.trim())
  for (const [i, item] of headers.entries()) {
    if (!item) continue
    if (!item.match(/^\s*([^:]+?)\s*:\s*(.+)\s*$/)) {
      errors.headers ??= []
      errors.headers.push({ message: `第 ${i + 1} 行请求头格式不正确` })
    }
  }
  return { errors }
}

function onFormSubmit(event: FormSubmitEvent) {
  if (!event.valid) return
  emit('update:visible', false)
  const formData = event.states
  if (!store.list[itemIndex.value]) return
  store.list[itemIndex.value].url = formData.url.value
  store.list[itemIndex.value].config = mergeConfig(store.globalConfig, {
    acceptInvalidCerts: formData.acceptInvalidCerts.value,
    acceptInvalidHostnames: formData.acceptInvalidHostnames.value,
    headers: formData.headers.value,
    minChunkSize: formData.minChunkSize.value,
    multiplexing: formData.multiplexing.value,
    proxy: formData.proxy.value || null,
    retryGap: formData.retryGap.value,
    saveDir: formData.saveDir.value,
    threads: formData.threads.value,
    writeBufferSize: formData.writeBufferSize.value,
    writeMethod: formData.writeMethod.value,
    writeQueueCap: formData.writeQueueCap.value,
  })
}

async function selectDir() {
  const dir = await open({
    directory: true,
    title: '选择保存文件夹',
  })
  const saveDirInput = document.getElementById('save-dir-input') as
    | HTMLInputElement
    | undefined
  if (dir && saveDirInput) {
    saveDirInput.value = dir
    saveDirInput.dispatchEvent(new Event('input'))
  }
}

function onUpdateVisible(v: boolean) {
  emit('update:visible', v)
}
</script>

<style scoped>
.fields {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.action {
  display: flex;
  margin-top: 16px;
  justify-content: end;
  gap: 8px;
}
.save-dir-container {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
