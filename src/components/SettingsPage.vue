<template>
  <Dialog
    :visible="props.visible"
    @update:visible="onUpdateVisible"
    modal
    header="设置"
    :style="{ width: '25rem' }"
    :closable="false"
  >
    <Form v-slot="$form" :initial-values :resolver @submit="onFormSubmit">
      <div class="fields">
        <div>
          <IftaLabel style="display: flex">
            <Textarea
              name="headers"
              rows="5"
              style="resize: vertical; width: 100%"
            />
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
        <label for="multiplexing">是否启用多路复用</label>
        <ToggleSwitch name="multiplexing" />
        <label for="acceptInvalidCerts">是否接受无效证书 (不安全)</label>
        <ToggleSwitch name="acceptInvalidCerts" />
        <label for="acceptInvalidHostnames">是否接受无效主机名 (不安全)</label>
        <ToggleSwitch name="acceptInvalidHostnames" />
        <label for="autoStart">是否开机自启动</label>
        <ToggleSwitch name="autoStart" />
      </div>
      <div class="action">
        <Button
          type="button"
          label="取消"
          severity="secondary"
          @click="emit('update:visible', false)"
        ></Button>
        <Button type="submit" label="保存"></Button>
      </div>
    </Form>
  </Dialog>
</template>

<script setup lang="ts">
import { Form, FormResolverOptions, FormSubmitEvent } from '@primevue/forms'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps<{
  visible: boolean
}>()
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()
const store = useAppStore()

const initialValues = reactive({
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
  autoStart: false,
})
watchEffect(() => {
  initialValues.threads = store.threads
  initialValues.saveDir = store.saveDir
  initialValues.headers = store.headers
  initialValues.proxy = store.proxy || ''
  initialValues.writeBufferSize = store.writeBufferSize
  initialValues.writeQueueCap = store.writeQueueCap
  initialValues.retryGap = store.retryGap
  initialValues.minChunkSize = store.minChunkSize
  initialValues.acceptInvalidCerts = store.acceptInvalidCerts
  initialValues.acceptInvalidHostnames = store.acceptInvalidHostnames
  initialValues.multiplexing = store.multiplexing
  initialValues.writeMethod = store.writeMethod
  initialValues.autoStart = store.autoStart
})

const writeMethodOptions = [
  { name: '内存映射文件 (推荐)', code: 'mmap' },
  { name: '标准库 (兼容性好)', code: 'std' },
]

async function resolver({ values }: FormResolverOptions) {
  const errors = {} as {
    [k in keyof typeof values]: { message: string }[]
  }
  const headers: string[] = values.headers
    .split('\n')
    .map((e: string) => e.trim())
  for (const [i, item] of headers.entries()) {
    if (!item) continue
    if (
      item
        .split(':')
        .map(e => e.trim())
        .filter(Boolean).length !== 2
    ) {
      errors.headers ??= []
      errors.headers.push({ message: `第 ${i + 1} 行 URL 格式不正确` })
    }
  }
  if (typeof values.saveDir === 'string' && values.saveDir) {
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
  return { errors }
}

function onFormSubmit(event: FormSubmitEvent) {
  if (!event.valid) return
  emit('update:visible', false)
  const formData = event.states
  store.threads = formData.threads.value
  store.saveDir = formData.saveDir.value
  store.headers = formData.headers.value
  store.proxy = formData.proxy.value || null
  store.writeBufferSize = formData.writeBufferSize.value
  store.writeQueueCap = formData.writeQueueCap.value
  store.retryGap = formData.retryGap.value
  store.minChunkSize = formData.minChunkSize.value
  store.acceptInvalidCerts = formData.acceptInvalidCerts.value
  store.acceptInvalidHostnames = formData.acceptInvalidHostnames.value
  store.multiplexing = formData.multiplexing.value
  store.writeMethod = formData.writeMethod.value
  store.autoStart = formData.autoStart.value
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
