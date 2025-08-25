import { info } from '@tauri-apps/plugin-log'

export async function formatDir(dir: string) {
  const start = `formatDir(${JSON.stringify(dir)})`
  info(start)
  const res = await invoke<string | null>('format_dir', { dir })
  info(`${start} => ${JSON.stringify(res)}`)
  return res
}
