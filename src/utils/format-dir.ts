export async function formatDir(dir: string) {
  return (await invoke('format_dir', { dir })) as string | null
}
