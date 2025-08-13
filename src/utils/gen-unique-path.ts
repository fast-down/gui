import { invoke } from '@tauri-apps/api/core'

export interface UniquePath {
  dir: string
  name: string
  path: string
}

export async function genUniquePath(dir: string, name: string) {
  return (await invoke('gen_unique_path', { dir, name })) as UniquePath
}
