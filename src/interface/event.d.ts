type WorkerId = number
interface ProgressEntry {
  start: number
  end: number
}
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
  data: [WorkerId, ProgressEntry]
}
interface PushError {
  event: 'pushError'
  data: [WorkerId, string]
}
interface PushProgress {
  event: 'pushProgress'
  data: [WorkerId, ProgressEntry]
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
type DownloadEvent =
  | Pulling
  | PullError
  | PullProgress
  | PushError
  | PushProgress
  | FlushError
  | Finished
  | AllFinished
