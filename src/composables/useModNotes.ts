import { ref, watch } from "vue"

const STORAGE_KEY = "slaymumanager_mod_notes"

const noteMap = ref<Record<string, string>>(loadNotes())

function loadNotes(): Record<string, string> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? JSON.parse(raw) : {}
  } catch {
    return {}
  }
}

function saveNotes() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(noteMap.value))
}

watch(noteMap, saveNotes, { deep: true })

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
