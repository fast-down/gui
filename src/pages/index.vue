<template>
  {{ time }}
  {{ size }}
</template>

<script lang="ts" setup>
import { format_time, format_size } from '@/utils/util'
const time = ref(format_time(1n))
const size = ref(format_size(0x0d000721))
// const content = ref(format_time(1n))
</script>