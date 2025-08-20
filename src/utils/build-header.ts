export function buildHeaders(str: string) {
  const headers: Record<string, string> = {}
  for (const [k, v] of <[string, string][]>str
    .split('\n')
    .map(e => e.trim())
    .filter(Boolean)
    .map(e => {
      const res = e.match(/^([^:]+):(.+)$/)
      if (res) return [res[1], res[2]]
      return null
    })
    .filter(Boolean)) {
    headers[k] = v
  }
  return headers
}
