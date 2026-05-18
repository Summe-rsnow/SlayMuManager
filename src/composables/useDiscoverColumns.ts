import { ref } from "vue"

const STORAGE_KEY = "discover-column-count"

export const discoverColumns = ref<number>(Number(localStorage.getItem(STORAGE_KEY)) || 3)

export function setDiscoverColumns(val: number) {
  discoverColumns.value = val
  localStorage.setItem(STORAGE_KEY, String(val))
}
