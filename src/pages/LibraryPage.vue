<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, onBeforeUnmount } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import {
  NSpace, NCard, NTag, NButton, NInput, NIcon, NSwitch,
  NPopconfirm, NModal, NCheckbox, NPopover, NSelect, useMessage,
} from "naive-ui"
import {
  Search, Download, RefreshCw, FolderOpen, Trash2, Bookmark,
  AlertTriangle, Filter, X, Tag, Plus, Play,
} from "lucide-vue-next"
import ImportDialog from "../components/ImportDialog.vue"
import { useModCache } from "../composables/useModCache"
import { useModTags, PRESET_TAGS } from "../composables/useModTags"
import { useRouter } from "vue-router"
import type { InstalledMod, ModProfile, ModToggleResult, CloudSaveStatus, AppBootstrap } from "../types"
import "../assets/library-effects.css"

const { t } = useI18n()
const message = useMessage()
const router = useRouter()
const { enabledMods, disabledMods, loading, fetchMods } = useModCache()
const { getTags, toggleTag, usedTags, getTagLabel, isPresetTag } = useModTags()

// --- 组件生命周期守卫（防止切换页面时异步回调卡死）---
const isActive = ref(true)
onBeforeUnmount(() => { isActive.value = false })

// --- 启动游戏 ---
const launchingGame = ref(false)

// 云存档差异确认弹窗
const showLaunchMismatchDialog = ref(false)
const launchMismatchStatus = ref<CloudSaveStatus | null>(null)

/** 弹窗：查看存档 → 跳转存档页面 */
function handleGoToSaves() {
  showLaunchMismatchDialog.value = false
  router.push("/saves")
}

/** 弹窗：强制启动 → 跳过云存档检查直接启动 */
async function handleLaunchAnyway() {
  showLaunchMismatchDialog.value = false
  await doLaunchGame()
}

/** 实际执行启动游戏（无检查） */
async function doLaunchGame() {
  launchingGame.value = true
  try {
    await invoke("launch_game")
    if (!isActive.value) return
    message.success(t("library.success.gameLaunched"))
  } catch (e: any) {
    if (!isActive.value) return
    message.error(t("library.error.launchFailed", { e }))
  } finally {
    if (isActive.value) launchingGame.value = false
  }
}

/** 启动游戏入口：先检测云存档状态 */
async function handleLaunchGame() {
  launchingGame.value = true
  try {
    const cloudStatus = await invoke<CloudSaveStatus>("get_cloud_save_status")
    if (!isActive.value) return

    if (cloudStatus.isAvailable && cloudStatus.hasMismatch) {
      launchMismatchStatus.value = cloudStatus
      showLaunchMismatchDialog.value = true
      return
    }

    await doLaunchGame()
  } catch {
    // 云存档检测失败时直接尝试启动
    await doLaunchGame()
  } finally {
    if (isActive.value) launchingGame.value = false
  }
}

// --- 快速预设选择 ---
const quickPresetId = ref<string | null>(null)
const quickPresetOptions = ref<Array<{ label: string; value: string }>>([])
async function loadQuickPresets() {
  try {
    const profiles = await invoke<ModProfile[]>("list_profiles")
    if (!isActive.value) return
    quickPresetOptions.value = profiles.map(p => ({ label: p.name, value: p.id }))
  } catch { /* ignore */ }
}
async function handleQuickPreset(presetId: string) {
  if (!presetId) return
  try {
    const label = quickPresetOptions.value.find(p => p.value === presetId)?.label ?? presetId
    await invoke("apply_profile", { id: presetId })
    if (!isActive.value) return
    // 记录激活预设，下拉框保持选中
    activePresetId.value = presetId
    activePresetName.value = label
    quickPresetId.value = presetId
    message.success(t("library.success.presetApplied", { name: label }))
    await fetchMods()
    // 快照预设声明的 mod ID（用于脏检测）
    try {
      const profiles = await invoke<ModProfile[]>("list_profiles")
      const profile = profiles.find(p => p.id === presetId)
      if (profile) presetSnapshot.value = new Set(profile.modIds)
    } catch { /* ignore */ }
  } catch (e: any) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.applyFailed")}: ${e}`)
  }
}

// --- 对话框 ---
const showImportDialog = ref(false)

// --- 当前激活预设跟踪 ---
const activePresetId = ref<string | null>(null)
const activePresetName = ref("")
const presetSnapshot = ref<Set<string>>(new Set())

/** 当前激活预设是否为原版（内置） */
const isActivePresetBuiltin = computed(() => activePresetId.value === "__builtin__vanilla")

/** 当前启用的 mod 是否偏离了激活预设 */
const isPresetDirty = computed(() => {
  if (!activePresetId.value || isActivePresetBuiltin.value) return false
  const currentIds = new Set(enabledMods.value.map(m => m.id))
  if (currentIds.size !== presetSnapshot.value.size) return true
  for (const id of currentIds) {
    if (!presetSnapshot.value.has(id)) return true
  }
  return false
})

// --- 保存为预设 ---
const showSavePresetDialog = ref(false)
const presetName = ref("")
const presetDescription = ref("")
const savingPreset = ref(false)

function openSavePreset() {
  if (enabledMods.value.length === 0) {
    message.warning(t("library.warning.noEnabledMods"))
    return
  }
  presetName.value = t("library.savePreset.defaultName") + " " + new Date().toLocaleDateString("zh-CN")
  presetDescription.value = t("library.savePreset.defaultDescription")
  showSavePresetDialog.value = true
}

async function handleSavePreset() {
  const name = presetName.value.trim()
  if (!name) {
    message.warning(t("library.warning.enterPresetName"))
    return
  }
  savingPreset.value = true
  try {
    await invoke("create_profile", {
      name,
      description: presetDescription.value.trim() || null,
      modIds: enabledMods.value.map((m) => m.id),
    })
    message.success(t("library.success.presetSaved", { name }))
    showSavePresetDialog.value = false
  } catch (e: any) {
    message.error(t("library.error.saveFailed", { e }))
  } finally {
    savingPreset.value = false
  }
}

// --- 侧边栏筛选 ---
const filterAffectsGameplay = ref(false)
const filterShowEnabled = ref(true)
const filterShowDisabled = ref(true)
const filterTagIds = ref<Set<string>>(new Set())

const activeFilterCount = computed(() => {
  let n = 0
  if (filterAffectsGameplay.value) n++
  if (!filterShowEnabled.value) n++
  if (!filterShowDisabled.value) n++
  if (filterTagIds.value.size > 0) n++
  return n
})

function clearFilters() {
  filterAffectsGameplay.value = false
  filterShowEnabled.value = true
  filterShowDisabled.value = true
  filterTagIds.value = new Set()
}

function toggleFilterTag(tagId: string) {
  const next = new Set(filterTagIds.value)
  if (next.has(tagId)) next.delete(tagId)
  else next.add(tagId)
  filterTagIds.value = next
}

// 已在使用的预设标签（用于侧边栏展示）
const usedPresetTags = computed(() =>
  PRESET_TAGS.filter((t) => usedTags.value.has(t.id))
)

// --- 搜索（回车提交）---
const searchInput = ref("")
const searchQuery = ref("")

function applySearch() {
  searchQuery.value = searchInput.value
}

function clearSearch() {
  searchInput.value = ""
  searchQuery.value = ""
}

// --- 行级操作锁 ---
const busyId = ref<string | null>(null)

// --- Save Guard 弹窗 ---
const showSaveGuardDialog = ref(false)
const saveGuardInfo = ref<ModToggleResult | null>(null)

// --- 搜索 + 筛选 ---
const filteredEnabled = computed(() => {
  let list = enabledMods.value
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter(
      (m) =>
        m.name.toLowerCase().includes(q) ||
        m.author?.toLowerCase().includes(q) ||
        m.id.toLowerCase().includes(q),
    )
  }
  if (filterAffectsGameplay.value) {
    list = list.filter((m) => m.affectsGameplay)
  }
  if (filterTagIds.value.size > 0) {
    list = list.filter((m) => {
      const tags = getTags(m.id)
      return tags.some((t) => filterTagIds.value.has(t))
    })
  }
  return list
})

const filteredDisabled = computed(() => {
  let list = disabledMods.value
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter(
      (m) =>
        m.name.toLowerCase().includes(q) ||
        m.author?.toLowerCase().includes(q) ||
        m.id.toLowerCase().includes(q),
    )
  }
  if (filterAffectsGameplay.value) {
    list = list.filter((m) => m.affectsGameplay)
  }
  if (filterTagIds.value.size > 0) {
    list = list.filter((m) => {
      const tags = getTags(m.id)
      return tags.some((t) => filterTagIds.value.has(t))
    })
  }
  return list
})

const hasSearch = computed(() => searchQuery.value.length > 0)
const hasMods = computed(() => enabledMods.value.length + disabledMods.value.length > 0)
const anyFilterActive = computed(() =>
  !filterShowEnabled.value || !filterShowDisabled.value ||
  filterAffectsGameplay.value || filterTagIds.value.size > 0
)
const emptyReason = computed(() => {
  if (!hasMods.value) return "noMods"
  if (anyFilterActive.value &&
      filteredEnabled.value.length + filteredDisabled.value.length === 0) return "filtered"
  if (hasSearch.value && filteredEnabled.value.length + filteredDisabled.value.length === 0) return "search"
  return null
})

// --- 卡片鼠标特效 ---
function onCardMouseMove(e: MouseEvent, el: HTMLElement) {
  const rect = el.getBoundingClientRect()
  el.style.setProperty("--mouse-x", `${e.clientX - rect.left}px`)
  el.style.setProperty("--mouse-y", `${e.clientY - rect.top}px`)
}

function onCardClick(e: MouseEvent, el: HTMLElement) {
  const rect = el.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top
  const ripple = document.createElement("span")
  ripple.className = "ripple-effect"
  ripple.style.left = `${x}px`
  ripple.style.top = `${y}px`
  ripple.style.width = ripple.style.height = `${Math.max(rect.width, rect.height)}px`
  el.appendChild(ripple)
  ripple.addEventListener("animationend", () => ripple.remove())
}

// --- 操作 ---
async function handleToggle(mod: InstalledMod) {
  if (busyId.value) return
  const isEnabling = mod.state === "disabled"
  busyId.value = mod.id
  try {
    const result = await invoke<ModToggleResult>(
      isEnabling ? "enable_mod" : "disable_mod",
      { modId: mod.id },
    )
    // Save Guard 检查：仅路径切换时弹窗提醒
    if (result.saveGuard.pathSwitched) {
      saveGuardInfo.value = result
      showSaveGuardDialog.value = true
    } else {
      message.success(
        t(isEnabling ? "library.success.enabled" : "library.success.disabled", { name: mod.name })
      )
    }
    await fetchMods()
    // 自动同步激活预设的本地快照（后端已更新预设，前端同步避免脏标记）
    if (activePresetId.value && !isActivePresetBuiltin.value) {
      const next = new Set(presetSnapshot.value)
      if (isEnabling) {
        next.add(mod.id)
      } else {
        next.delete(mod.id)
      }
      presetSnapshot.value = next
    }
  } catch (e: any) {
    message.error(t("library.error.operationFailed", { e }))
  } finally {
    busyId.value = null
  }
}

async function handleUninstall(mod: InstalledMod) {
  if (busyId.value) return
  busyId.value = mod.id
  try {
    await invoke("uninstall_mod", { modId: mod.id })
    message.success(t("library.success.uninstalled", { name: mod.name }))
    await fetchMods()
  } catch (e: any) {
    message.error(t("library.error.uninstallFailed", { e }))
  } finally {
    busyId.value = null
  }
}

async function handleOpenFolder(mod: InstalledMod) {
  try {
    await invoke("open_mod_folder", { modId: mod.id })
  } catch (e: any) {
    message.error(t("library.error.openFailed", { e }))
  }
}

async function handleOpenModsDir() {
  try {
    await invoke("open_mods_directory")
  } catch (e: any) {
    message.error(t("library.error.openFailed", { e }))
  }
}

function handleImport() {
  showImportDialog.value = true
}

function onImportDone() {
  fetchMods()
}

function dismissSaveGuard() {
  showSaveGuardDialog.value = false
  if (saveGuardInfo.value) {
    message.success(
      t("library.success.toggle", {
        action: saveGuardInfo.value.modItem.state === "enabled"
          ? t("common.enabled")
          : t("common.disabled"),
        name: saveGuardInfo.value.modItem.name,
      })
    )
  }
}

// --- 外部 Mod 变更监听 ---
let unlistenModsChanged: (() => void) | null = null

onMounted(async () => {
  await fetchMods()
  loadQuickPresets()
  // 恢复上次激活的预设状态
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    if (bootstrap.activeProfileName) {
      const profiles = await invoke<ModProfile[]>("list_profiles")
      const active = profiles.find(p => p.name === bootstrap.activeProfileName)
      if (active) {
        activePresetId.value = active.id
        activePresetName.value = active.name
        quickPresetId.value = active.id
        presetSnapshot.value = new Set(active.modIds)
      }
    }
  } catch { /* ignore */ }
  unlistenModsChanged = (await listen("slaymgr:mods-changed", () => {
    if (isActive.value) fetchMods()
  }).catch(() => null)) as (() => void) | null
})

onUnmounted(() => {
  unlistenModsChanged?.()
})
</script>

<template>
  <div>
    <!-- 头部 -->
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-bold text-gray-800">{{ t("library.title") }}</h1>
        <div class="flex items-center gap-4 mt-1 text-sm text-gray-500">
          <span class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-green-500 inline-block" />
            {{ t("library.enabledCountLabel") }} {{ enabledMods.length }}
          </span>
          <span class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-gray-400 inline-block" />
            {{ t("library.disabledCountLabel") }} {{ disabledMods.length }}
          </span>
          <span v-if="loading" class="text-xs text-gray-400 animate-pulse">{{ t("library.refreshing") }}</span>
        </div>
      </div>
      <NSpace>
        <NSelect
          v-model:value="quickPresetId"
          :options="quickPresetOptions"
          :placeholder="t('library.quickPresetPlaceholder')"
          style="width: 150px"
          size="small"
          :disabled="quickPresetOptions.length === 0"
          @update:value="handleQuickPreset"
        />
        <NPopover v-if="isPresetDirty" trigger="hover" placement="bottom">
          <template #trigger>
            <NTag type="warning" size="tiny" :bordered="false" class="cursor-default">
              {{ t("library.success.presetDirty") }}
            </NTag>
          </template>
          <span class="text-xs">{{ t("library.success.presetDirtyTip", { name: activePresetName }) }}</span>
        </NPopover>
        <NButton type="success" @click="handleLaunchGame" :loading="launchingGame">
          <template #icon><NIcon :size="16"><Play /></NIcon></template>
          {{ t("library.launchGame") }}
        </NButton>
        <div class="w-px h-6 bg-gray-200 self-center" />
        <NButton secondary @click="handleOpenModsDir">
          <template #icon><NIcon :size="16"><FolderOpen /></NIcon></template>
          {{ t("library.openModsDir") }}
        </NButton>
        <NButton secondary @click="openSavePreset">
          <template #icon><NIcon :size="16"><Bookmark /></NIcon></template>
          {{ t("library.saveAsPreset") }}
        </NButton>
        <NButton secondary :loading="loading" @click="fetchMods">
          <template #icon><NIcon :size="16"><RefreshCw /></NIcon></template>
          {{ t("common.refresh") }}
        </NButton>
        <NButton type="primary" @click="handleImport">
          <template #icon><NIcon :size="16"><Download /></NIcon></template>
          {{ t("library.importMod") }}
        </NButton>
      </NSpace>
    </div>

    <!-- 搜索栏 -->
    <div class="mb-4">
      <NInput
        v-model:value="searchInput"
        :placeholder="t('library.searchPlaceholder')"
        clearable
        @clear="clearSearch"
        @keydown.enter="applySearch"
      >
        <template #prefix>
          <NIcon :size="16"><Search /></NIcon>
        </template>
      </NInput>
    </div>

    <!-- 主布局：侧边栏 + 内容 -->
    <div class="flex gap-4">
      <!-- 侧边栏筛选 -->
      <div class="w-44 flex-shrink-0">
        <div class="sticky top-4 space-y-3 p-3 rounded-lg border border-gray-100 bg-gray-50/50">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-600 flex items-center gap-1.5">
              <NIcon :size="14"><Filter /></NIcon>
              {{ t("library.filter.title") }}
            </span>
            <NButton
              v-if="activeFilterCount > 0"
              text
              size="tiny"
              type="warning"
              @click="clearFilters"
            >
              <template #icon><NIcon :size="12"><X /></NIcon></template>
              {{ t("library.filter.clear") }}
            </NButton>
          </div>

          <div class="space-y-2">
            <div class="text-xs text-gray-500 font-medium">{{ t("library.filter.show") }}</div>
            <NCheckbox v-model:checked="filterShowEnabled" size="small">
              <span class="text-xs">{{ t("library.filter.enabled") }}</span>
            </NCheckbox>
            <NCheckbox v-model:checked="filterShowDisabled" size="small">
              <span class="text-xs">{{ t("library.filter.disabled") }}</span>
            </NCheckbox>
          </div>

          <div class="space-y-2">
            <div class="text-xs text-gray-500 font-medium">{{ t("library.filter.attributes") }}</div>
            <NCheckbox v-model:checked="filterAffectsGameplay" size="small">
              <span class="text-xs">{{ t("library.filter.affectsGameplay") }}</span>
            </NCheckbox>
          </div>

          <!-- 标签筛选 -->
          <div v-if="usedPresetTags.length > 0" class="space-y-2">
            <div class="text-xs text-gray-500 font-medium flex items-center gap-1">
              <NIcon :size="12"><Tag /></NIcon>
              {{ t("library.filter.tags") }}
            </div>
            <NCheckbox
              v-for="t in usedPresetTags"
              :key="t.id"
              :checked="filterTagIds.has(t.id)"
              size="small"
              @update:checked="() => toggleFilterTag(t.id)"
            >
              <span class="text-xs">{{ getTagLabel(t.id) }}</span>
            </NCheckbox>
          </div>

          <!-- 筛选计数徽章 -->
          <div v-if="activeFilterCount > 0" class="pt-2 border-t border-gray-100">
            <NTag type="warning" size="tiny" :bordered="false">
              {{ t("library.filter.activeFilterCount", { n: activeFilterCount }) }}
            </NTag>
          </div>
        </div>
      </div>

      <!-- 内容区 -->
      <div class="flex-1 min-w-0">
        <!-- 三层空状态 -->
        <div v-if="emptyReason" class="text-center py-16 text-gray-400">
          <template v-if="emptyReason === 'noMods'">
            <p class="text-lg">{{ t("library.empty.noMods") }}</p>
            <p class="text-sm mt-1">{{ t("library.empty.noModsHint") }}</p>
          </template>
          <template v-else-if="emptyReason === 'filtered'">
            <p>{{ t("library.empty.filterNoResults") }}</p>
            <p class="text-sm mt-1">
              <NButton text size="tiny" @click="clearFilters">{{ t("library.empty.clearAllFilters") }}</NButton>
            </p>
          </template>
          <template v-else-if="emptyReason === 'search'">
            <p>{{ t("library.empty.searchNoMatch", { q: searchQuery }) }}</p>
            <p class="text-sm mt-1">
              <NButton text size="tiny" @click="clearSearch">{{ t("library.empty.clearSearch") }}</NButton>
            </p>
          </template>
        </div>

        <template v-else>
          <!-- 已启用 Mod -->
          <NCard v-if="filterShowEnabled" size="small" class="mb-4">
            <template #header>
              <div class="flex items-center gap-2">
                <span>{{ t("library.section.enabled") }}</span>
                <NTag :type="filteredEnabled.length > 0 ? 'success' : 'default'" size="small" round>
                  {{ t("library.section.count", { n: filteredEnabled.length }) }}
                </NTag>
              </div>
            </template>

            <div v-if="filteredEnabled.length === 0" class="text-center py-8 text-gray-400">
              <p v-if="hasSearch || filterAffectsGameplay">{{ t("library.empty.filterNoResults") }}</p>
              <p v-else>{{ t("library.empty.noEnabledMods") }}</p>
            </div>

            <NSpace v-else vertical :size="8">
              <div
                v-for="mod in filteredEnabled"
                :key="mod.id"
                class="mod-card mod-card--enabled flex items-center justify-between p-3 rounded-lg border border-gray-100 bg-white transition-colors"
                :class="{ 'pointer-events-none opacity-60': busyId === mod.id }"
                @mousemove="(e: MouseEvent) => onCardMouseMove(e, (e.currentTarget as HTMLElement))"
                @click="(e: MouseEvent) => onCardClick(e, (e.currentTarget as HTMLElement))"
              >
                <div class="flex-1 min-w-0" style="position:relative;z-index:2">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-gray-800 truncate">{{ mod.name }}</span>
                    <span class="text-xs text-gray-400 font-mono truncate">{{ mod.version ?? "—" }}</span>
                    <NTag v-if="mod.affectsGameplay" type="warning" size="tiny" :bordered="false">
                      {{ t("library.mod.affectsGameplay") }}
                    </NTag>
                  </div>
                  <div class="text-xs text-gray-400 mt-0.5">
                    {{ mod.author ?? t("library.mod.unknownAuthor") }} · {{ mod.folderName }}
                  </div>
                  <!-- 标签行 -->
                  <div class="flex items-center gap-1 mt-1 flex-wrap">
                    <NTag
                      v-for="tagId in getTags(mod.id)"
                      :key="tagId"
                      size="tiny"
                      :bordered="false"
                      :type="isPresetTag(tagId) ? 'info' : 'default'"
                      closable
                      @close="() => toggleTag(mod.id, tagId)"
                    >
                      {{ getTagLabel(tagId) }}
                    </NTag>
                    <NPopover trigger="click" placement="bottom-start">
                      <template #trigger>
                        <NButton text size="tiny" class="opacity-0 group-hover:opacity-100 transition-opacity">
                          <template #icon><NIcon :size="12"><Plus /></NIcon></template>
                        </NButton>
                      </template>
                      <div class="w-52">
                        <div class="text-xs text-gray-500 mb-2">{{ t("library.mod.selectTag") }}</div>
                        <NSpace vertical :size="4">
                          <NCheckbox
                            v-for="t in PRESET_TAGS"
                            :key="t.id"
                            size="small"
                            :checked="getTags(mod.id).includes(t.id)"
                            @update:checked="() => toggleTag(mod.id, t.id)"
                          >
                            <span class="text-xs">{{ getTagLabel(t.id) }}</span>
                          </NCheckbox>
                        </NSpace>
                      </div>
                    </NPopover>
                  </div>
                </div>
                <div class="mod-actions flex items-center gap-2 flex-shrink-0 ml-4" style="position:relative;z-index:2">
                  <NButton text size="tiny" :disabled="busyId !== null" @click="handleOpenFolder(mod)">
                    <template #icon><NIcon :size="14"><FolderOpen /></NIcon></template>
                  </NButton>
                  <NPopconfirm @positive-click="() => handleUninstall(mod)">
                    <template #trigger>
                      <NButton text size="tiny" type="error" :disabled="busyId !== null">
                        <template #icon><NIcon :size="14"><Trash2 /></NIcon></template>
                      </NButton>
                    </template>
                    {{ t("library.mod.confirmUninstall", { name: mod.name }) }}
                  </NPopconfirm>
                  <NSwitch
                    :value="true"
                    :disabled="busyId !== null || isActivePresetBuiltin"
                    @update:value="() => handleToggle(mod)"
                  />
                </div>
              </div>
            </NSpace>
          </NCard>

          <!-- 已禁用 Mod -->
          <NCard v-if="filterShowDisabled" size="small">
            <template #header>
              <div class="flex items-center gap-2">
                <span>{{ t("library.section.disabled") }}</span>
                <NTag type="default" size="small" round>
                  {{ t("library.section.count", { n: filteredDisabled.length }) }}
                </NTag>
              </div>
            </template>

            <div v-if="filteredDisabled.length === 0" class="text-center py-8 text-gray-400">
              <p v-if="hasSearch || filterAffectsGameplay">{{ t("library.empty.filterNoResults") }}</p>
              <p v-else>{{ t("library.empty.noDisabledMods") }}</p>
            </div>

            <NSpace v-else vertical :size="8">
              <div
                v-for="mod in filteredDisabled"
                :key="mod.id"
                class="mod-card mod-card--disabled flex items-center justify-between p-3 rounded-lg border border-gray-100 bg-white transition-colors opacity-70"
                :class="{ 'pointer-events-none opacity-40': busyId === mod.id }"
                @mousemove="(e: MouseEvent) => onCardMouseMove(e, (e.currentTarget as HTMLElement))"
                @click="(e: MouseEvent) => onCardClick(e, (e.currentTarget as HTMLElement))"
              >
                <div class="flex-1 min-w-0" style="position:relative;z-index:2">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-gray-600 truncate">{{ mod.name }}</span>
                    <span class="text-xs text-gray-400 font-mono truncate">{{ mod.version ?? "—" }}</span>
                  </div>
                  <div class="text-xs text-gray-400 mt-0.5">
                    {{ mod.author ?? t("library.mod.unknownAuthor") }} · {{ mod.folderName }}
                  </div>
                  <!-- 标签行 -->
                  <div class="flex items-center gap-1 mt-1 flex-wrap">
                    <NTag
                      v-for="tagId in getTags(mod.id)"
                      :key="tagId"
                      size="tiny"
                      :bordered="false"
                      :type="isPresetTag(tagId) ? 'info' : 'default'"
                      closable
                      @close="() => toggleTag(mod.id, tagId)"
                    >
                      {{ getTagLabel(tagId) }}
                    </NTag>
                    <NPopover trigger="click" placement="bottom-start">
                      <template #trigger>
                        <NButton text size="tiny" class="opacity-0 group-hover:opacity-100 transition-opacity">
                          <template #icon><NIcon :size="12"><Plus /></NIcon></template>
                        </NButton>
                      </template>
                      <div class="w-52">
                        <div class="text-xs text-gray-500 mb-2">{{ t("library.mod.selectTag") }}</div>
                        <NSpace vertical :size="4">
                          <NCheckbox
                            v-for="t in PRESET_TAGS"
                            :key="t.id"
                            size="small"
                            :checked="getTags(mod.id).includes(t.id)"
                            @update:checked="() => toggleTag(mod.id, t.id)"
                          >
                            <span class="text-xs">{{ getTagLabel(t.id) }}</span>
                          </NCheckbox>
                        </NSpace>
                      </div>
                    </NPopover>
                  </div>
                </div>
                <div class="mod-actions flex items-center gap-2 flex-shrink-0 ml-4" style="position:relative;z-index:2">
                  <NButton text size="tiny" :disabled="busyId !== null" @click="handleOpenFolder(mod)">
                    <template #icon><NIcon :size="14"><FolderOpen /></NIcon></template>
                  </NButton>
                  <NPopconfirm @positive-click="() => handleUninstall(mod)">
                    <template #trigger>
                      <NButton text size="tiny" type="error" :disabled="busyId !== null">
                        <template #icon><NIcon :size="14"><Trash2 /></NIcon></template>
                      </NButton>
                    </template>
                    {{ t("library.mod.confirmUninstall", { name: mod.name }) }}
                  </NPopconfirm>
                  <NSwitch
                    :value="false"
                    :disabled="busyId !== null || isActivePresetBuiltin"
                    @update:value="() => handleToggle(mod)"
                  />
                </div>
              </div>
            </NSpace>
          </NCard>
        </template>
      </div>
    </div>

    <!-- 导入对话框 -->
    <ImportDialog
      :show="showImportDialog"
      @close="showImportDialog = false"
      @installed="onImportDone"
    />

    <!-- 保存为预设对话框 -->
    <NModal :show="showSavePresetDialog" @update:show="(v: boolean) => !v && (showSavePresetDialog = false)">
      <NCard style="width: 480px" :bordered="false" role="dialog" :title="t('library.savePreset.title')">
        <NSpace vertical :size="12">
          <div>
            <span class="text-sm text-gray-500">{{ t("library.savePreset.nameLabel") }}</span>
            <NInput v-model:value="presetName" :placeholder="t('library.savePreset.namePlaceholder')" class="mt-1" />
          </div>
          <div>
            <span class="text-sm text-gray-500">{{ t("library.savePreset.descriptionLabel") }}</span>
            <NInput v-model:value="presetDescription" :placeholder="t('library.savePreset.descriptionPlaceholder')" class="mt-1" />
          </div>
          <div class="text-xs text-gray-400">
            {{ t("library.savePreset.willSave", { n: enabledMods.length }) }}
          </div>
          <div class="flex justify-end gap-2">
            <NButton @click="showSavePresetDialog = false">{{ t("common.cancel") }}</NButton>
            <NButton type="primary" :loading="savingPreset" @click="handleSavePreset">
              {{ t("common.save") }}
            </NButton>
          </div>
        </NSpace>
      </NCard>
    </NModal>

    <!-- Save Guard 警告弹窗 -->
    <NModal :show="showSaveGuardDialog" @update:show="(v: boolean) => !v && dismissSaveGuard()">
      <NCard style="width: 440px" :bordered="false" role="dialog">
        <template #header>
          <div class="flex items-center gap-2">
            <NIcon :size="18" color="#f0a020"><AlertTriangle /></NIcon>
            <span class="font-semibold">{{ t("library.saveGuard.title") }}</span>
          </div>
        </template>
        <NSpace v-if="saveGuardInfo" vertical :size="8">
          <p v-if="saveGuardInfo.saveGuard.pathSwitched" class="text-sm text-gray-600">
            {{ t("library.saveGuard.pathSwitchWarning") }}
          </p>
          <p v-if="saveGuardInfo.saveGuard.hadPairs" class="text-sm text-gray-600">
            {{ t("library.saveGuard.syncResult", { synced: saveGuardInfo.saveGuard.savesSynced, backups: saveGuardInfo.saveGuard.backupsCreated }) }}
          </p>
          <div class="flex justify-end mt-2">
            <NButton type="primary" size="small" @click="dismissSaveGuard">{{ t("library.saveGuard.gotIt") }}</NButton>
          </div>
        </NSpace>
      </NCard>
    </NModal>

    <!-- 云存档差异确认弹窗 -->
    <NModal :show="showLaunchMismatchDialog" @update:show="(v: boolean) => !v && (showLaunchMismatchDialog = false)">
      <NCard style="width: 440px" :bordered="false" role="dialog">
        <template #header>
          <div class="flex items-center gap-2">
            <NIcon :size="18" color="#f0a020"><AlertTriangle /></NIcon>
            <span class="font-semibold">{{ t("library.launchMismatch.title") }}</span>
          </div>
        </template>
        <NSpace v-if="launchMismatchStatus" vertical :size="8">
          <p class="text-sm text-gray-600">{{ t("library.launchMismatch.warning") }}</p>
          <div class="text-xs text-gray-500 bg-amber-50 rounded p-2 space-y-1">
            <div class="flex justify-between" v-if="launchMismatchStatus.differentCount > 0">
              <span>{{ t("saves.cloud.mismatch.different", { n: launchMismatchStatus.differentCount }) }}</span>
            </div>
            <div class="flex justify-between" v-if="launchMismatchStatus.localOnlyCount > 0">
              <span>{{ t("saves.cloud.mismatch.localOnly", { n: launchMismatchStatus.localOnlyCount }) }}</span>
            </div>
            <div class="flex justify-between" v-if="launchMismatchStatus.cloudOnlyCount > 0">
              <span>{{ t("saves.cloud.mismatch.cloudOnly", { n: launchMismatchStatus.cloudOnlyCount }) }}</span>
            </div>
          </div>
          <div class="flex justify-between mt-2 gap-2">
            <NButton secondary size="small" @click="handleGoToSaves">
              {{ t("library.launchMismatch.goToSaves") }}
            </NButton>
            <NButton type="warning" size="small" @click="handleLaunchAnyway">
              {{ t("library.launchMismatch.forceLaunch") }}
            </NButton>
          </div>
        </NSpace>
      </NCard>
    </NModal>
  </div>
</template>
