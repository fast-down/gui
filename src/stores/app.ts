import { invoke } from "@tauri-apps/api/core";

export interface DownloadEntry {
  url: string;
  filePath: string;
  fileName: string;
  fileSize: number;
  speed: number;
  readProgress: [number, number][];
  writeProgress: [number, number][];
  elapsedMs: number;
  status: "pending" | "downloading" | "paused" | "completed" | "error";
  downloaded: number;
  etag?: string;
  lastModified?: string;
}

export const useAppStore = defineStore(
  "app",
  () => {
    const list = reactive([
      {
        url: "https://www.example.com/file.zip",
        filePath: "/path/to/file.zip",
        fileName: "file.zip",
        fileSize: 12 * 1024 * 1024,
        readProgress: [
          [
            [0, 0.3 * 1024 * 1024],
            [0.4 * 1024 * 1024, 0.5 * 1024 * 1024],
          ],
          [[1 * 1024 * 1024, 1.2 * 1024 * 1024]],
        ],
        writeProgress: [
          [0, 0.5 * 1024 * 1024],
          [1 * 1024 * 1024, 1.2 * 1024 * 1024],
        ],
        etag: 'W/"123456789"',
        lastModified: "2022-01-01T00:00:00Z",
        elapsedMs: 1.3 * 1000,
        status: "pending",
        downloaded: 0.7 * 1024 * 1024,
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: "https://www.example.com/file.zip",
        filePath: "/path/to/file.zip",
        fileName: "file.zip",
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [[0, 200]],
        writeProgress: [[0, 100]],
        etag: 'W/"123456789"',
        lastModified: "2022-01-01T00:00:00Z",
        elapsedMs: 1.3 * 1000,
        status: "pending",
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: "https://www.example.com/file.zip",
        filePath: "/path/to/file.zip",
        fileName: "file.zip",
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [[0, 200]],
        writeProgress: [[0, 100]],
        etag: 'W/"123456789"',
        lastModified: "2022-01-01T00:00:00Z",
        elapsedMs: 1.3 * 1000,
        status: "pending",
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: "https://www.example.com/file.zip",
        filePath: "/path/to/file.zip",
        fileName: "file.zip",
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [[0, 200]],
        writeProgress: [[0, 100]],
        etag: 'W/"123456789"',
        lastModified: "2022-01-01T00:00:00Z",
        elapsedMs: 1.3 * 1000,
        status: "pending",
      },
      {
        downloaded: 0.7 * 1024 * 1024,
        url: "https://www.example.com/file.zip",
        filePath: "/path/to/file.zip",
        fileName: "file.zip",
        fileSize: 1.2 * 1024 * 1024,
        readProgress: [[0, 200]],
        writeProgress: [[0, 100]],
        etag: 'W/"123456789"',
        lastModified: "2022-01-01T00:00:00Z",
        elapsedMs: 1.3 * 1000,
        status: "pending",
      },
    ] as DownloadEntry[]);

    const threads = ref(8);
    const saveDir = ref("");
    const headers = ref(String.raw`sec-ch-ua-mobile: ?0
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36
sec-ch-ua: "Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"
sec-ch-ua-platform: "Windows"`);
    const proxy = ref(void 0 as string | undefined);

    async function addEntry(options: {
      url: string;
      threads: number;
      saveDir: string;
      headers: Record<string, string>;
      proxy?: string;
    }) {
      const res = await invoke("prefetch", {
        url: options.url,
        headers: options.headers,
        proxy: options.proxy,
      });
      console.log(res);
      // list.push(entry)
    }
    return { list, threads, saveDir, headers, proxy, addEntry };
  },
  {
    persist: true,
  }
);
