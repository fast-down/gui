<script setup lang="ts"></script>

<template>
  <div class="d-flex flex-column h-screen">
    <div class="d-flex ga-2 pa-2 overflow-x-auto">
      <v-dialog max-width="500" persistent>
        <template #activator="{ props: activatorProps }">
          <v-btn prepend-icon="mdi-plus" stacked v-bind="activatorProps">
            新建任务
          </v-btn>
        </template>
        <template #default="{ isActive }">
          <v-card title="新建任务">
            <v-card-text>
              <v-form v-model="valid" @submit="createTask(isActive)">
                <v-container>
                  <v-row>
                    <v-textarea
                      v-model="rawUrls"
                      label="URL (一行一个)"
                      :rules="urlRules"
                    />
                  </v-row>
                  <v-row>
                    <v-text-field
                      v-model="store.saveDir"
                      label="保存文件夹"
                      :rules="dirRules"
                    >
                      <template #append>
                        <IconBtn
                          icon="mdi-folder"
                          text="选择文件夹"
                          @click="selectDir"
                        />
                      </template>
                    </v-text-field>
                  </v-row>
                  <v-row>
                    <v-number-input
                      v-model="store.threads"
                      label="线程数量"
                      :min="1"
                    />
                  </v-row>
                </v-container>
              </v-form>
            </v-card-text>
            <v-card-actions>
              <v-spacer />
              <v-btn @click="isActive.value = false"> 取消 </v-btn>
              <v-btn @click="createTask(isActive)"> 开始 </v-btn>
            </v-card-actions>
          </v-card>
        </template>
      </v-dialog>
      <v-btn prepend-icon="mdi-play" stacked> 全部开始 </v-btn>
      <v-btn prepend-icon="mdi-pause" stacked> 全部暂停 </v-btn>
      <v-btn prepend-icon="mdi-delete" stacked> 全部删除 </v-btn>
      <v-btn prepend-icon="mdi-cog" stacked> 设置 </v-btn>
    </div>
    <div ref="scrollRef" class="overflow-hidden" style="flex: 1">
      <v-virtual-scroll :height="height" :items="store.list">
        <template #default="{ item }">
          <v-card class="download-item ma-2">
            <v-card-item>
              <v-card-title> {{ item.fileName }} </v-card-title>
              <v-card-subtitle> {{ item.filePath }} </v-card-subtitle>
            </v-card-item>
            <v-card-text>
              <v-container class="pa-0">
                <v-row>
                  <v-col>
                    <div>速度</div>
                    <div>{{ formatSize(item.speed) }}/s</div>
                  </v-col>
                  <v-col>
                    <div>用时</div>
                    <div>{{ formatTime(item.elapsedMs / 1000) }}</div>
                  </v-col>
                  <v-col>
                    <div>速度</div>
                    <div>10MB/s</div>
                  </v-col>
                </v-row>
                <v-row>
                  <v-col class="py-0">
                    <v-progress-linear
                      :max="item.fileSize"
                      :model-value="1 * 1024 * 1024"
                    />
                  </v-col>
                </v-row>
              </v-container>
            </v-card-text>
            <v-card-actions>
              <IconBtn
                v-if="['downloading', 'pending'].includes(item.status!)"
                icon="mdi-pause"
                text="暂停"
              />
              <IconBtn v-else icon="mdi-play" text="开始" />
              <IconBtn icon="mdi-delete" text="删除" />
              <IconBtn icon="mdi-open-in-new" text="打开" />
              <IconBtn icon="mdi-folder" text="打开文件夹" />
            </v-card-actions>
          </v-card>
        </template>
      </v-virtual-scroll>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useElementSize } from '@vueuse/core'
import { useAppStore } from '@/stores/app'
import { formatSize } from '@/utils/format-size'
import { formatTime } from '@/utils/format-time'

const store = useAppStore()
for (const e of store.list) {
  e.status = 'paused'
  e.readProgress = []
  e.readProgress.concat(e.writeProgress)
  const downloaded = e.writeProgress.reduce((a, b) => a + b[1] - b[0], 0)
  e.speed = (downloaded / e.elapsedMs) * 1000
}

const scrollRef = useTemplateRef('scrollRef')
const { height } = useElementSize(scrollRef)

const valid = ref(false)
const rawUrls = ref('')
const urlRules = [
  (value?: string) => {
    if (!value?.trim()) return '请输入 URL'
    const urls = value.split('\n').map(e => e.trim())
    for (const [i, item] of urls.entries()) {
      if (!item) continue
      try {
        const url = new URL(item)
        if (!['http:', 'https:'].includes(url.protocol)) {
          return `第 ${i + 1} 行 URL 协议不正确`
        }
      } catch (error) {
        console.error(error)
        return `第 ${i + 1} 行 URL 格式不正确`
      }
    }
    return true
  },
]
const dirRules = [
  async (value?: string) => {
    if (!value?.trim()) return '请选择一个保存目录'
    try {
      const res: string | null = await invoke('format_dir', { dir: value })
      if (!res) return '目录不存在'
      console.log(res)
      return true
    } catch (error) {
      console.error(error)
      return '目录格式不正确'
    }
  },
]
async function selectDir() {
  const dir = await open({
    directory: true,
    title: '选择保存文件夹',
  })
  if (dir) store.saveDir = dir
}

function createTask(isActive: Ref<boolean>) {
  if (!valid.value) {
    return
  }
  isActive.value = false
  const urls = rawUrls.value.split('\n').map(e => e.trim())
  rawUrls.value = ''
  const headers: Record<string, string> = {}
  for (const [k, v] of store.headers
    .split('\n')
    .map(e => e.trim())
    .filter(Boolean)
    .map(e => e.split(':').map(e => e.trim()))) {
    headers[k] = v
  }
  for (const url of urls) {
    store.addEntry({
      url,
      headers,
      threads: store.threads,
      saveDir: store.saveDir,
      proxy: store.proxy,
    })
  }
}
</script>

<style>
.download-item:first-child {
  margin-top: 0 !important;
}
</style>
