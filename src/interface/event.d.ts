export type WorkerId = number
interface Pulling {
  event: 'pulling'
  data: WorkerId
}
interface PullError {
  event: 'pullError'
  data: [WorkerId, string]
}
interface PullProgress {
  event: 'pullProgress'
  data: [[number, number][][], number]
}
interface PushError {
  event: 'pushError'
  data: [WorkerId, string]
}
interface PushProgress {
  event: 'pushProgress'
  data: [number, number][][]
}
interface FlushError {
  event: 'flushError'
  data: string
}
interface Finished {
  event: 'finished'
  data: WorkerId
}
interface AllFinished {
  event: 'allFinished'
}
export type DownloadEvent =
  | Pulling
  | PullError
  | PullProgress
  | PushError
  | PushProgress
  | FlushError
  | Finished
  | AllFinished
