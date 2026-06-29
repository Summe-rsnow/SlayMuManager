import { defineStore } from "pinia"
import { computed } from "vue"
import { useStorage } from "@/composables/useStorage"

export const PRESET_TAGS = [
  { id: "visual-enhancement", label: { "zh-CN": "视觉增强", en: "Visual Enhancement" } },
  { id: "gameplay-expansion", label: { "zh-CN": "玩法扩展", en: "Gameplay Expansion" } },
  { id: "utility-tools", label: { "zh-CN": "工具辅助", en: "Utility Tools" } },
  { id: "quality-of-life", label: { "zh-CN": "体验优化", en: "Quality of Life" } },
  { id: "performance", label: { "zh-CN": "性能优化", en: "Performance" } },
  { id: "content-mod", label: { "zh-CN": "内容 Mod", en: "Content Mod" } },
  { id: "character-skin", label: { "zh-CN": "角色皮肤", en: "Character Skin" } },
  { id: "cheat-trainer", label: { "zh-CN": "修改/训练", en: "Cheat / Trainer" } },
  { id: "multiplayer-compatible", label: { "zh-CN": "联机兼容", en: "Multiplayer Compatible" } },
] as const

export type PresetTagId = (typeof PRESET_TAGS)[number]["id"]

export const useTagStore = defineStore("tags", () => {
  const tagMap = useStorage<Record<string, string[]>>("slaymumanager_mod_tags", {})

  function getTags(modId: string): string[] { return tagMap.value[modId] ?? [] }
  function setTags(modId: string, tags: string[]) { tagMap.value = { ...tagMap.value, [modId]: [...new Set(tags)] } }
  function addTag(modId: string, tag: string) {
    const current = getTags(modId)
    if (!current.includes(tag)) tagMap.value = { ...tagMap.value, [modId]: [...current, tag] }
  }
  function removeTag(modId: string, tag: string) {
    tagMap.value = { ...tagMap.value, [modId]: getTags(modId).filter((t) => t !== tag) }
  }
  function toggleTag(modId: string, tag: string) {
    if (getTags(modId).includes(tag)) removeTag(modId, tag)
    else addTag(modId, tag)
  }

  const usedTags = computed(() => {
    const set = new Set<string>()
    for (const tags of Object.values(tagMap.value)) tags.forEach((t) => set.add(t))
    return set
  })

  function getTagLabel(tagId: string, locale = "zh-CN"): string {
    const preset = PRESET_TAGS.find((t) => t.id === tagId)
    if (preset) return locale === "zh-CN" ? preset.label["zh-CN"] : preset.label.en
    return tagId
  }

  function isPresetTag(tagId: string): boolean { return PRESET_TAGS.some((t) => t.id === tagId) }

  return { getTags, setTags, addTag, removeTag, toggleTag, usedTags, getTagLabel, isPresetTag }
})
