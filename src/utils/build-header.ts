export const headerRegex = /^\s*([^:]+?)\s*:\s*(.+?)\s*$/

export function buildHeaders(str: string) {
  const headers: Record<string, string> = {}
  for (const [k, v] of <[string, string][]>str
    .split('\n')
    .map(e => {
      const res = e.match(headerRegex)
      return res && [res[1], res[2]]
    })
    .filter(Boolean)) {
    headers[k] = v
  }
  return headers
}
