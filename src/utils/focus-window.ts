import { getCurrentWindow } from '@tauri-apps/api/window'

export async function focusWindow() {
  return getCurrentWindow()
    .show()
    .finally(() => getCurrentWindow().unminimize())
    .finally(() => getCurrentWindow().setFocus())
}
