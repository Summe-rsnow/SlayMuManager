<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue"
import { useI18n } from "vue-i18n"
import { currentLocale } from "../i18n"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import {
  NSpace, NCard, NTag, NButton, NInput, NIcon,
  NModal, NCheckbox, NPopover, NSelect, useMessage,
} from "naive-ui"
import {
  Search, Download, RefreshCw, FolderOpen, Bookmark,
  AlertTriangle, Filter, X, Tag, Play, PackageOpen, Check,
} from "lucide-vue-next"
import ImportDialog from "../components/ImportDialog.vue"
import ModCard from "../components/ModCard.vue"
import { useModCache } from "../composables/useModCache"
import { useModTags, PRESET_TAGS } from "../composables/useModTags"
import { useRouter } from "vue-router"
import type { InstalledMod, ModProfile, ModToggleResult, CloudSaveStatus, AppBootstrap } from "../types"
import { useIsActive } from "../composables/useIsActive"

/** 内置原版预设 ID（与服务端 BUILTIN_VANILLA_ID 对应） */
const BUILTIN_VANILLA_ID = "__builtin__vanilla"

const { t } = useI18n()
const message = useMessage()
const router = useRouter()
const { enabledMods, disabledMods, loading, fetchMods } = useModCache()
const { getTags, usedTags, getTagLabel } = useModTags()

// --- 组件生命周期守卫（防止切换页面时异步回调卡死）---
const { isActive } = useIsActive()

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
const isActivePresetBuiltin = computed(() => activePresetId.value === BUILTIN_VANILLA_ID)

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

// --- 新增预设（空预设 + 切换）---
const showNewPresetDialog = ref(false)
const newPresetName = ref("")
const creatingNewPreset = ref(false)

function openNewPreset() {
  newPresetName.value = ""
  showNewPresetDialog.value = true
}

async function handleCreateNewPreset() {
  const name = newPresetName.value.trim()
  if (!name) {
    message.warning(t("library.warning.enterPresetName"))
    return
  }
  creatingNewPreset.value = true
  try {
    // 创建空预设
    await invoke("create_profile", {
      name,
      description: null,
      modIds: [] as string[],
    })
    // 查询所有预设找到刚创建的
    const profiles = await invoke<ModProfile[]>("list_profiles")
    const created = profiles.find(p => p.name === name)
    if (created) {
      // 切换到新预设
      await invoke("apply_profile", { id: created.id })
      if (!isActive.value) return
      activePresetId.value = created.id
      activePresetName.value = created.name
      quickPresetId.value = created.id
      presetSnapshot.value = new Set()
      message.success(t("library.success.presetApplied", { name }))
      await fetchMods()
      loadQuickPresets()
    }
    showNewPresetDialog.value = false
  } catch (e: any) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.applyFailed")}: ${e}`)
  } finally {
    creatingNewPreset.value = false
  }
}

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
  presetName.value = t("library.savePreset.defaultName") + " " + new Date().toLocaleDateString(currentLocale.value)
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
    loadQuickPresets()
  } catch (e: any) {
    message.error(t("library.error.saveFailed", { e }))
  } finally {
    savingPreset.value = false
  }
}

// --- 全部启用 / 全部禁用 ---
const batchBusy = ref(false)

async function enableAllMods() {
  const targets = disabledMods.value
  if (targets.length === 0) {
    message.info(t("library.info.allAlreadyEnabled"))
    return
  }
  batchBusy.value = true
  let success = 0
  for (const mod of targets) {
    try {
      await invoke<ModToggleResult>("enable_mod", { modId: mod.id })
      success++
    } catch { /* skip failed */ }
    if (!isActive.value) break
  }
  batchBusy.value = false
  if (!isActive.value) return
  message.success(t("library.success.batchEnabled", { n: success }))
  await fetchMods()
}

async function disableAllMods() {
  const targets = enabledMods.value
  if (targets.length === 0) {
    message.info(t("library.info.allAlreadyDisabled"))
    return
  }
  batchBusy.value = true
  let success = 0
  for (const mod of targets) {
    try {
      await invoke<ModToggleResult>("disable_mod", { modId: mod.id })
      success++
    } catch { /* skip failed */ }
    if (!isActive.value) break
  }
  batchBusy.value = false
  if (!isActive.value) return
  message.success(t("library.success.batchDisabled", { n: success }))
  await fetchMods()
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

// --- 搜索（实时 debounce 200ms + 回车立即搜索）---
const searchInput = ref("")
const searchQuery = ref("")
let searchDebounce: ReturnType<typeof setTimeout> | null = null

watch(searchInput, (val) => {
  if (searchDebounce) clearTimeout(searchDebounce)
  searchDebounce = setTimeout(() => {
    searchQuery.value = val
  }, 200)
})

function applySearch() {
  if (searchDebounce) clearTimeout(searchDebounce)
  searchQuery.value = searchInput.value
}

function clearSearch() {
  if (searchDebounce) clearTimeout(searchDebounce)
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
    <div class="flex items-center justify-between mb-6">
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
      <div class="flex flex-wrap gap-2">
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
          <div class="text-xs max-w-48 space-y-2">
            <p>{{ t("library.success.presetDirtyTip", { name: activePresetName }) }}</p>
            <NButton size="tiny" secondary @click="openSavePreset">
              {{ t("library.savePreset.title") }}
            </NButton>
          </div>
        </NPopover>
        <NButton size="small" type="success" @click="handleLaunchGame" :loading="launchingGame">
          <template #icon><NIcon :size="14"><Play /></NIcon></template>
          {{ t("library.launchGame") }}
        </NButton>
        <div class="w-px self-stretch bg-gray-200" />
        <NButton size="small" secondary @click="handleOpenModsDir">
          <template #icon><NIcon :size="14"><FolderOpen /></NIcon></template>
          {{ t("library.openModsDir") }}
        </NButton>
        <NButton size="small" secondary @click="openNewPreset">
          <template #icon><NIcon :size="14"><Bookmark /></NIcon></template>
          {{ t("library.newPreset") }}
        </NButton>
        <NButton size="small" secondary :loading="loading" @click="fetchMods">
          <template #icon><NIcon :size="14"><RefreshCw /></NIcon></template>
          {{ t("common.refresh") }}
        </NButton>
        <NButton size="small" type="primary" @click="handleImport">
          <template #icon><NIcon :size="14"><Download /></NIcon></template>
          {{ t("library.importMod") }}
        </NButton>
      </div>
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
      <div class="w-48 flex-shrink-0">
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
          <div class="space-y-2">
            <div class="text-xs text-gray-500 font-medium flex items-center gap-1">
              <NIcon :size="12"><Tag /></NIcon>
              {{ t("library.filter.tags") }}
            </div>
            <template v-if="usedPresetTags.length > 0">
              <NCheckbox
                v-for="t in usedPresetTags"
                :key="t.id"
                :checked="filterTagIds.has(t.id)"
                size="small"
                @update:checked="() => toggleFilterTag(t.id)"
              >
                <span class="text-xs">{{ getTagLabel(t.id) }}</span>
              </NCheckbox>
            </template>
            <p v-else class="text-xs text-gray-300 italic">{{ t("library.filter.noTags") }}</p>
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
            <NIcon :size="48" class="c-gray-300 mb-3"><PackageOpen /></NIcon>
            <p class="text-lg">{{ t("library.empty.noMods") }}</p>
            <p class="text-sm mt-1">{{ t("library.empty.noModsHint") }}</p>
          </template>
          <template v-else-if="emptyReason === 'filtered'">
            <NIcon :size="48" class="c-gray-300 mb-3"><Filter /></NIcon>
            <p>{{ t("library.empty.filterNoResults") }}</p>
            <p class="text-sm mt-1">
              <NButton text size="tiny" @click="clearFilters">{{ t("library.empty.clearAllFilters") }}</NButton>
            </p>
          </template>
          <template v-else-if="emptyReason === 'search'">
            <NIcon :size="48" class="c-gray-300 mb-3"><Search /></NIcon>
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
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span>{{ t("library.section.enabled") }}</span>
                  <NTag :type="filteredEnabled.length > 0 ? 'success' : 'default'" size="small" round>
                    {{ t("library.section.count", { n: filteredEnabled.length }) }}
                  </NTag>
                </div>
                <NButton
                  v-if="filteredEnabled.length > 0 && !isActivePresetBuiltin"
                  size="small"
                  secondary
                  :disabled="batchBusy"
                  :loading="batchBusy"
                  @click="disableAllMods"
                >
                  <template #icon><NIcon :size="12"><X /></NIcon></template>
                  {{ t("library.disableAll") }}
                </NButton>
              </div>
            </template>

            <div v-if="filteredEnabled.length === 0" class="text-center py-8 text-gray-400">
              <p v-if="hasSearch || filterAffectsGameplay">{{ t("library.empty.filterNoResults") }}</p>
              <p v-else>{{ t("library.empty.noEnabledMods") }}</p>
            </div>

            <NSpace v-else vertical :size="8">
              <ModCard
                v-for="mod in filteredEnabled"
                :key="mod.id"
                :mod="mod"
                :enabled="true"
                :busy="busyId === mod.id"
                :toggle-disabled="isActivePresetBuiltin"
                @toggle="handleToggle"
                @open-folder="handleOpenFolder"
                @uninstall="handleUninstall"
              />
            </NSpace>
          </NCard>

          <!-- 已禁用 Mod -->
          <NCard v-if="filterShowDisabled" size="small">
            <template #header>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span>{{ t("library.section.disabled") }}</span>
                  <NTag type="default" size="small" round>
                    {{ t("library.section.count", { n: filteredDisabled.length }) }}
                  </NTag>
                </div>
                <NButton
                  v-if="filteredDisabled.length > 0 && !isActivePresetBuiltin"
                  size="small"
                  secondary
                  :disabled="batchBusy"
                  :loading="batchBusy"
                  @click="enableAllMods"
                >
                  <template #icon><NIcon :size="12"><Check /></NIcon></template>
                  {{ t("library.enableAll") }}
                </NButton>
              </div>
            </template>

            <div v-if="filteredDisabled.length === 0" class="text-center py-8 text-gray-400">
              <p v-if="hasSearch || filterAffectsGameplay">{{ t("library.empty.filterNoResults") }}</p>
              <p v-else>{{ t("library.empty.noDisabledMods") }}</p>
            </div>

            <NSpace v-else vertical :size="8">
              <ModCard
                v-for="mod in filteredDisabled"
                :key="mod.id"
                :mod="mod"
                :enabled="false"
                :busy="busyId === mod.id"
                :toggle-disabled="isActivePresetBuiltin"
                @toggle="handleToggle"
                @open-folder="handleOpenFolder"
                @uninstall="handleUninstall"
              />
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

    <!-- 新增预设对话框 -->
    <NModal :show="showNewPresetDialog" @update:show="(v: boolean) => !v && (showNewPresetDialog = false)">
      <NCard style="width: 420px" :bordered="false" role="dialog" :title="t('library.newPreset')">
        <NSpace vertical :size="12">
          <div>
            <label class="text-sm text-gray-500 mb-1 block">{{ t("library.savePreset.nameLabel") }}</label>
            <NInput
              v-model:value="newPresetName"
              :placeholder="t('library.savePreset.namePlaceholder')"
              @keyup.enter="handleCreateNewPreset"
            />
          </div>
          <div class="text-xs text-gray-400">
            {{ t("library.newPresetHint") }}
          </div>
          <div class="flex justify-end gap-2">
            <NButton @click="showNewPresetDialog = false">{{ t("common.cancel") }}</NButton>
            <NButton type="primary" :loading="creatingNewPreset" @click="handleCreateNewPreset">
              {{ t("common.confirm") }}
            </NButton>
          </div>
        </NSpace>
      </NCard>
    </NModal>

    <!-- 保存为预设对话框 -->
    <NModal :show="showSavePresetDialog" @update:show="(v: boolean) => !v && (showSavePresetDialog = false)">
      <NCard style="width: 480px" :bordered="false" role="dialog" :title="t('library.savePreset.title')">
        <NSpace vertical :size="12">
          <div>
            <label class="text-sm text-gray-500 mb-1 block">{{ t("library.savePreset.nameLabel") }}</label>
            <NInput v-model:value="presetName" :placeholder="t('library.savePreset.namePlaceholder')" />
          </div>
          <div>
            <label class="text-sm text-gray-500 mb-1 block">{{ t("library.savePreset.descriptionLabel") }}</label>
            <NInput v-model:value="presetDescription" :placeholder="t('library.savePreset.descriptionPlaceholder')" />
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
