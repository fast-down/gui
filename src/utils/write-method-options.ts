import { WriteMethod } from '../interface/create-download-options'

export const writeMethodOptions: { name: string; code: WriteMethod }[] = [
  { name: '内存映射文件 (推荐)', code: 'mmap' },
  { name: '标准库 (兼容性好)', code: 'std' },
]
