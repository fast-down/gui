import { getCurrentWindow } from '@tauri-apps/api/window'
import { info } from '@tauri-apps/plugin-log'

export async function focusWindow() {
  info('focusWindow')
  return getCurrentWindow()
    .show()
    .finally(() => getCurrentWindow().unminimize())
    .finally(() => getCurrentWindow().setFocus())
}
