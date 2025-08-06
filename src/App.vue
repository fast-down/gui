<template>
  <header class="header">
    <Button label="新建任务" variant="text" icon="pi pi-plus" />
    <Button label="全部开始" variant="text" icon="pi pi-play" />
    <Button label="全部暂停" variant="text" icon="pi pi-pause" />
    <Button label="全部删除" variant="text" icon="pi pi-trash" />
    <Button label="设置" variant="text" icon="pi pi-cog" />
  </header>
  <main class="main">
    <DownloadItem
      v-for="item in store.list"
      :info="item"
      :key="item.filePath"
      class="download-item"
    >
    </DownloadItem>
  </main>
</template>

<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const store = useAppStore();
for (const e of store.list) {
  e.status = "paused";
  e.readProgress = [];
  e.readProgress.concat(e.writeProgress);
  e.speed = (e.downloaded / e.elapsedMs) * 1000;
}

const valid = ref(false);
const rawUrls = ref("");
const urlRules = [
  (value?: string) => {
    if (!value?.trim()) return "请输入 URL";
    const urls = value.split("\n").map((e) => e.trim());
    for (const [i, item] of urls.entries()) {
      if (!item) continue;
      try {
        const url = new URL(item);
        if (!["http:", "https:"].includes(url.protocol)) {
          return `第 ${i + 1} 行 URL 协议不正确`;
        }
      } catch (error) {
        console.error(error);
        return `第 ${i + 1} 行 URL 格式不正确`;
      }
    }
    return true;
  },
];
const dirRules = [
  async (value?: string) => {
    if (!value?.trim()) return "请选择一个保存目录";
    try {
      const res: string | null = await invoke("format_dir", { dir: value });
      if (!res) return "目录不存在";
      console.log(res);
      return true;
    } catch (error) {
      console.error(error);
      return "目录格式不正确";
    }
  },
];
async function selectDir() {
  const dir = await open({
    directory: true,
    title: "选择保存文件夹",
  });
  if (dir) store.saveDir = dir;
}

function createTask(isActive: Ref<boolean>) {
  if (!valid.value) {
    return;
  }
  isActive.value = false;
  const urls = rawUrls.value.split("\n").map((e) => e.trim());
  rawUrls.value = "";
  const headers: Record<string, string> = {};
  for (const [k, v] of store.headers
    .split("\n")
    .map((e) => e.trim())
    .filter(Boolean)
    .map((e) => e.split(":").map((e) => e.trim()))) {
    headers[k] = v;
  }
  for (const url of urls) {
    store.addEntry({
      url,
      headers,
      threads: store.threads,
      saveDir: store.saveDir,
      proxy: store.proxy,
    });
  }
}
</script>

<style scoped>
.header {
  display: flex;
  gap: 8px;
  padding: 8px;
  overflow-x: auto;
}

.header > * {
  flex-shrink: 0;
}

.main {
  flex: 1;
  overflow: auto;
}

.download-item {
  margin: 8px;
}
.download-item:first-child {
  margin-top: 0;
}
</style>
