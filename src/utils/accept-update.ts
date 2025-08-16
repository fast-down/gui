import { emit } from '@tauri-apps/api/event'

export function acceptUpdate() {
  return emit('accept_update')
}
