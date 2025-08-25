export function diffConfig(
  globalConfig: DownloadConfig,
  itemConfig: Partial<DownloadConfig>,
) {
  const result: Partial<DownloadConfig> = {}
  for (const key in globalConfig) {
    // @ts-expect-error 乱报错
    if (globalConfig[key] !== itemConfig[key]) result[key] = itemConfig[key]
  }
  return result
}
