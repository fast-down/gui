<template>
  <Dialog
    v-model:visible="props.visible"
    modal
    header="新建任务"
    :style="{ width: '25rem' }"
    :closable="false"
  >
    <Form v-slot="$form" :initial-values :resolver @submit="onFormSubmit">
      <div class="fields">
        <div>
          <IftaLabel style="display: flex">
            <Textarea name="url" rows="5" class="url-input" />
            <label for="url">URL (一行一个)</label>
          </IftaLabel>
          <Message
            v-if="$form.url?.invalid"
            severity="error"
            size="small"
            variant="simple"
            style="margin-top: 4px"
          >
            {{ $form.url.error?.message }}
          </Message>
        </div>
        <div>
          <IftaLabel>
            <InputNumber name="threads" :min="1" fluid />
            <label for="threads">线程数</label>
          </IftaLabel>
          <Message
            v-if="$form.threads?.invalid"
            severity="error"
            size="small"
            variant="simple"
            >{{ $form.threads.error?.message }}</Message
          >
        </div>
        <div>
          <div class="save-dir-container">
            <IftaLabel class="save-dir-label">
              <InputText name="saveDir" class="save-dir-input" />
              <label for="saveDir">保存目录</label>
            </IftaLabel>
            <Button
              variant="text"
              icon="pi pi-folder-open"
              aria-label="选取文件夹"
              @click="selectDir"
            />
          </div>
          <Message
            v-if="$form.saveDir?.invalid"
            severity="error"
            size="small"
            variant="simple"
          >
            {{ $form.saveDir.error?.message }}
          </Message>
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
import { Form, FormResolverOptions } from '@primevue/forms'
import { invoke } from '@tauri-apps/api/core'
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
  threads: store.threads,
  saveDir: store.saveDir,
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
          errors.url = errors.url || []
          errors.url.push({ message: `第 ${i + 1} 行 URL 协议不正确` })
        }
      } catch (error) {
        console.error(error)
        errors.url = errors.url || []
        errors.url.push({ message: `第 ${i + 1} 行 URL 格式不正确` })
      }
    }
  }
  if (typeof values.saveDir !== 'string' || !values.saveDir.trim())
    errors.saveDir = [{ message: '请选择一个保存目录' }]
  else {
    try {
      const res: string | null = await invoke('format_dir', {
        dir: values.saveDir,
      })
      if (!res) errors.saveDir = [{ message: '目录不存在' }]
    } catch (error) {
      console.error(error)
      errors.saveDir = [{ message: '目录格式不正确' }]
    }
  }
  return { errors }
}

function onFormSubmit({ valid }: { valid: boolean }) {
  if (valid) emit('update:visible', false)
}

async function selectDir() {
  const dir = await open({
    directory: true,
    title: '选择保存文件夹',
  })
  if (dir) initialValues.saveDir = dir
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
.url-input {
  width: 100%;
  resize: none;
}
.save-dir-container {
  display: flex;
  align-items: center;
  gap: 8px;
}
.save-dir-label {
  flex: 1;
}
.save-dir-input {
  width: 100%;
}
</style>
