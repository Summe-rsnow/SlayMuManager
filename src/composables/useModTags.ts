import { ref, computed } from "vue"

// 9 种预设标签
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

const STORAGE_KEY = "slaymumanager_mod_tags"

// Module-level state — singleton
const tagMap = ref<Record<string, string[]>>({})
const loaded = ref(false)

function load() {
  if (loaded.value) return
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    tagMap.value = raw ? JSON.parse(raw) : {}
  } catch {
    tagMap.value = {}
  }
  loaded.value = true
}

function save() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tagMap.value))
  } catch {
    // localStorage 不可用时静默失败
  }
}

export function useModTags() {
  load()

  /** 获取某个 Mod 的标签列表 */
  function getTags(modId: string): string[] {
    return tagMap.value[modId] ?? []
  }

  /** 设置某个 Mod 的标签（覆盖） */
  function setTags(modId: string, tags: string[]) {
    tagMap.value[modId] = [...new Set(tags)]
    save()
  }

  /** 为 Mod 添加一个标签 */
  function addTag(modId: string, tag: string) {
    const current = getTags(modId)
    if (!current.includes(tag)) {
      tagMap.value[modId] = [...current, tag]
      save()
    }
  }

  /** 为 Mod 移除一个标签 */
  function removeTag(modId: string, tag: string) {
    const current = getTags(modId)
    tagMap.value[modId] = current.filter((t) => t !== tag)
    save()
  }

  /** 切换标签（有则删，无则加） */
  function toggleTag(modId: string, tag: string) {
    if (getTags(modId).includes(tag)) {
      removeTag(modId, tag)
    } else {
      addTag(modId, tag)
    }
  }

  /** 获取所有正在使用的标签 ID 集合 */
  const usedTags = computed(() => {
    const set = new Set<string>()
    for (const tags of Object.values(tagMap.value)) {
      for (const t of tags) {
        set.add(t)
      }
    }
    return set
  })

  /** 获取预设标签的中文标签文本 */
  function getTagLabel(tagId: string, locale = "zh-CN"): string {
    const preset = PRESET_TAGS.find((t) => t.id === tagId)
    if (preset) {
      return locale === "zh-CN" ? preset.label["zh-CN"] : preset.label.en
    }
    // 自定义标签直接返回 ID
    return tagId
  }

  /** 判断标签是否为预设 */
  function isPresetTag(tagId: string): boolean {
    return PRESET_TAGS.some((t) => t.id === tagId)
  }

  return {
    getTags,
    setTags,
    addTag,
    removeTag,
    toggleTag,
    usedTags,
    getTagLabel,
    isPresetTag,
  }
}
