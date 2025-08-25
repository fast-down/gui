export interface UpdateInfo {
  /** Update description */
  body: string | null
  /** Version used to check for update */
  currentVersion: string
  /** Version announced */
  version: string
  /** Update publish date */
  date: number | null
  /** Target */
  target: string
  /** Download URL announced */
  downloadUrl: string
  /** Signature announced */
  signature: string
}
