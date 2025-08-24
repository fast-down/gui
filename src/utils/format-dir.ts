import { info } from '@tauri-apps/plugin-log'

export async function formatDir(dir: string) {
  info(`formatDir(${JSON.stringify(dir)})`)
  const res = await invoke<string | null>('format_dir', { dir })
  info(`formatDir(${JSON.stringify(dir)}) => ${JSON.stringify(res)}`)
  return res
}
