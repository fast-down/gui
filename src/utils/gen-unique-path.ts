import { info } from '@tauri-apps/plugin-log'

export interface UniquePath {
  dir: string
  name: string
  path: string
}

export async function genUniquePath(dir: string, name: string) {
  info(`genUniquePath(${JSON.stringify(dir)}, ${JSON.stringify(name)})`)
  const res = await invoke<UniquePath>('gen_unique_path', { dir, name })
  info(
    `genUniquePath(${JSON.stringify(dir)}, ${JSON.stringify(
      name,
    )}) => ${JSON.stringify(res)}`,
  )
  return res
}
