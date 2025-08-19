export type RemoveUndefined<T> = {
  [K in keyof T as T[K] extends undefined ? never : K]: T[K]
}

export type UndefinedAble<T> = {
  [K in keyof T]: T[K] | undefined
}

export function removeUndefined<T extends Record<string, unknown>>(obj: T) {
  for (const key in obj) {
    if (obj[key] === undefined) {
      delete obj[key]
    }
  }
  return obj as RemoveUndefined<T>
}
