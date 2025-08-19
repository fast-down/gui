import { path } from '@tauri-apps/api'
import { exists } from '@tauri-apps/plugin-fs'
import { openPath } from '@tauri-apps/plugin-opener'
import { platform } from '@tauri-apps/plugin-os'
import { Command } from '@tauri-apps/plugin-shell'

export async function openFile(filePath: string) {
  if (!(await exists(filePath))) throw new Error(`"${filePath}" not exists`)
  await openPath(filePath)
}
export async function openFolder(filePath: string) {
  if (!(await exists(filePath))) throw new Error(`"${filePath}" not exists`)
  const currentPlatform = platform()
  if (currentPlatform === 'windows') {
    await openPath(`/select,${filePath}`, 'explorer.exe')
  } else if (currentPlatform === 'macos') {
    await Command.create('open', ['-R', filePath]).execute()
  } else if (currentPlatform === 'linux') {
    const dir = await path.dirname(filePath)
    await Command.create('nautilus', [dir]).execute()
  }
}
