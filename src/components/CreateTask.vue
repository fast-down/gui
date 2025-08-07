<template>
  <Dialog
    v-model:visible="props.visible"
    modal
    header="新建任务"
    :style="{ width: '25rem' }"
    :closable="false"
  >
    <Form>
      <FloatLabel variant="on">
        <InputText id="url-input" />
        <label for="url-input">On Label</label>
      </FloatLabel>
    </Form>
    <div class="action">
      <Button
        type="button"
        label="取消"
        severity="secondary"
        @click="emit('update:visible', false)"
      ></Button>
      <Button
        type="button"
        label="新建"
        @click="emit('update:visible', false)"
      ></Button>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { Form } from '@primevue/forms'
const props = defineProps<{
  visible: boolean
}>()
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()
</script>

<style scoped>
.action {
  display: flex;
  justify-content: end;
  gap: 8px;
}
</style>
