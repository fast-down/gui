<template>
  <v-tooltip interactive location="top" :text="props.text">
    <template #activator="{ props: activatorProps }">
      <v-icon-btn
        v-ripple
        v-bind="activatorProps"
        :disabled="props.disabled"
        :icon="props.icon"
        @click="emit('click')"
      />
    </template>
  </v-tooltip>
</template>

<script lang="ts" setup>
const props = defineProps({
  disabled: {
    type: Boolean,
    default: false,
  },
  icon: {
    type: String,
    required: true,
  },
  text: {
    type: String,
    required: true,
  },
})
const emit = defineEmits(['click'])
</script>
