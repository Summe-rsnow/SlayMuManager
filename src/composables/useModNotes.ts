import { useStorage } from "./useStorage"

const STORAGE_KEY = "slaymumanager_mod_notes"

const noteMap = useStorage<Record<string, string>>(STORAGE_KEY, {})

export function useModNotes() {
  function getNote(modId: string): string {
    return noteMap.value[modId] ?? ""
  }

  function setNote(modId: string, text: string) {
    noteMap.value = { ...noteMap.value, [modId]: text }
  }

  function hasNote(modId: string): boolean {
    return (noteMap.value[modId] ?? "").trim().length > 0
  }

  return { getNote, setNote, hasNote }
}
