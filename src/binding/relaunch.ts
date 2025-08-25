import { info } from '@tauri-apps/plugin-log'

export function relaunch() {
  info('Relaunching app')
  return invoke<void>('relaunch')
}
