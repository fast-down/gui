export function buildHeaders(str: string) {
  const headers: Record<string, string> = {}
  for (const [k, v] of str
    .split('\n')
    .map(e => e.trim())
    .filter(Boolean)
    .map(e => e.split(':').map(e => e.trim()))) {
    headers[k] = v
  }
  return headers
}
