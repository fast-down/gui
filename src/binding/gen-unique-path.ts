import { info } from '@tauri-apps/plugin-log'

export interface UniquePath {
  dir: string
  name: string
  path: string
}

export async function genUniquePath(dir: string, name: string) {
  const start = `genUniquePath(${JSON.stringify(dir)}, ${JSON.stringify(name)})`
  info(start)
  const res = await invoke<UniquePath>('gen_unique_path', { dir, name })
  info(`${start} => ${JSON.stringify(res)}`)
  return res
}
