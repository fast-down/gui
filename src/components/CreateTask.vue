<template>
  <Dialog
    :visible="props.visible"
    @update:visible="onUpdateVisible"
    modal
    header="新建任务"
    :style="{ width: '25rem' }"
    :closable="false"
  >
    <Form v-slot="$form" :initial-values :resolver @submit="onFormSubmit">
      <div class="fields">
        <div>
          <IftaLabel style="display: flex">
            <Textarea
              name="url"
              rows="5"
              style="resize: vertical; width: 100%"
            />
            <label for="url">URL (一行一个)</label>
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
      </div>
      <div class="action">
        <Button
          type="button"
          label="取消"
          severity="secondary"
          @click="emit('update:visible', false)"
        ></Button>
        <Button type="submit" label="新建"></Button>
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
  url: '',
  threads: 8,
  saveDir: '',
})
watchEffect(() => {
  initialValues.threads = store.threads
  initialValues.saveDir = store.saveDir
})

async function resolver({ values }: FormResolverOptions) {
  const errors = {} as {
    [k in keyof typeof values]: { message: string }[]
  }
  if (typeof values.url !== 'string' || !values.url.trim())
    errors.url = [{ message: '请输入下载链接' }]
  else {
    const urls = values.url.split('\n').map(e => e.trim())
    for (const [i, item] of urls.entries()) {
      if (!item) continue
      try {
        const url = new URL(item)
        if (!['http:', 'https:'].includes(url.protocol)) {
          errors.url ??= []
          errors.url.push({ message: `第 ${i + 1} 行 URL 协议不正确` })
        }
      } catch (error) {
        console.error(error)
        errors.url ??= []
        errors.url.push({ message: `第 ${i + 1} 行 URL 格式不正确` })
      }
    }
  }
  if (typeof values.saveDir !== 'string' || !values.saveDir.trim())
    errors.saveDir = [{ message: '请选择一个保存目录' }]
  else {
    try {
      const res = await formatDir(values.saveDir)
      if (!res) errors.saveDir = [{ message: '目录不存在' }]
    } catch (error) {
      console.error(error)
      errors.saveDir = [{ message: '目录格式不正确' }]
    }
  }
  return { errors }
}

function onFormSubmit(event: FormSubmitEvent) {
  if (!event.valid) return
  emit('update:visible', false)
  const formData = event.states
  const urls: string[] = formData.url.value
    .split('\n')
    .map((e: string) => e.trim())
    .filter(Boolean)
  store.saveDir = formData.saveDir.value
  store.threads = formData.threads.value
  urls.forEach(url => store.add(url))
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
