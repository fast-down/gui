import {
  isPermissionGranted,
  Options,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification'

export async function showNotification(options: Options) {
  let permissionGranted = await isPermissionGranted()
  if (!permissionGranted) {
    const permission = await requestPermission()
    permissionGranted = permission === 'granted'
  }
  if (permissionGranted) {
    sendNotification(options)
  } else {
    throw new Error('Notification permission not granted')
  }
}
