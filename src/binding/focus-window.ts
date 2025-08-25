import { getCurrentWindow } from '@tauri-apps/api/window'
import { info } from '@tauri-apps/plugin-log'

export async function focusWindow() {
  info('focusWindow')
  const window = getCurrentWindow()
  return window
    .show()
    .finally(() => window.unminimize())
    .finally(() => window.setFocus())
}

export async function isFocusWindow() {
  return getCurrentWindow().isFocused()
}
