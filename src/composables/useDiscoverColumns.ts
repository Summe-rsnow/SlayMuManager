import { useStorage } from "./useStorage"

const STORAGE_KEY = "discover-column-count"

export const discoverColumns = useStorage<number>(STORAGE_KEY, 3)

export function setDiscoverColumns(val: number) {
  discoverColumns.value = val
}
