import { defineStore } from "pinia"
import { useStorage } from "@/composables/useStorage"

export const useNoteStore = defineStore("notes", () => {
  const noteMap = useStorage<Record<string, string>>("slaymumanager_mod_notes", {})

  function getNote(modId: string): string { return noteMap.value[modId] ?? "" }
  function setNote(modId: string, text: string) { noteMap.value = { ...noteMap.value, [modId]: text } }
  function hasNote(modId: string): boolean { return (noteMap.value[modId] ?? "").trim().length > 0 }

  return { getNote, setNote, hasNote }
})
