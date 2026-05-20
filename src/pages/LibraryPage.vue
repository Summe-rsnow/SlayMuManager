<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue"
import { useI18n } from "vue-i18n"
import { currentLocale } from "../i18n"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { getCurrentWebview } from "@tauri-apps/api/webview"
import {
  NSpace, NCard, NTag, NButton, NInput, NIcon,
  NCheckbox, useMessage, useDialog,
} from "naive-ui"
import {
  Search, Download, RefreshCw, FolderOpen, Bookmark,
  AlertTriangle, Filter, X, PackageOpen, Check, ArrowUp,
} from "lucide-vue-next"
import DragOverlay from "../components/DragOverlay.vue"
import ModCard from "../components/ModCard.vue"
import AppDialog from "../components/AppDialog.vue"
import { useModCache } from "../composables/useModCache"
import { useModUpdates } from "../composables/useModUpdates"
import { useModTags, PRESET_TAGS } from "../composables/useModTags"
import type { InstalledMod, ModProfile, AppBootstrap, BatchImportPreview, BatchInstallResult } from "../types"
import { useIsActive } from "../composables/useIsActive"
import { useSidebarActions } from "../composables/useSidebarActions"
import { useModOperations } from "../composables/useModOperations"

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()
const { enabledMods, disabledMods, loading, fetchMods } = useModCache()
const { getTags, usedTags, getTagLabel } = useModTags()

// --- 组件生命周期守卫（防止切换页面时异步回调卡死）---
const { isActive } = useIsActive()

// --- 从共享 composable 获取侧边栏 & 游戏启动/预设状态 ---
const {
  quickPresetId,
  loadQuickPresets,
  activePresetName,
  activePresetId,
  presetSnapshot,
  presetAppliedTick,
} = useSidebarActions()

const {
  busyId, batchBusy, showSaveGuardDialog, saveGuardInfo, isActivePresetBuiltin,
  handleToggle, handleUninstall, handleOpenFolder, handleOpenModsDir,
  enableAllMods, disableAllMods, dismissSaveGuard,
} = useModOperations()

// --- 原版预设冲突检测（重置后才重新弹）---
let shownVanillaConflict = false
async function checkVanillaConflict() {
  if (shownVanillaConflict || !isActivePresetBuiltin.value) return
  const enabled = enabledMods.value
  if (enabled.length === 0) return
  shownVanillaConflict = true
  dialog.warning({
    title: t("library.vanillaConflict.title"),
    content: t("library.vanillaConflict.content", { n: enabled.length }),
    positiveText: t("library.vanillaConflict.disableAll"),
    negativeText: t("library.vanillaConflict.later"),
    onPositiveClick: () => disableAllMods(),
    onNegativeClick: () => { /* 用户选择以后再说 */ },
    maskClosable: true,
  })
}

// --- 导入（直通流程：选文件 → 检测冲突 → 安装）---
const importPaths = ref<string[]>([])
const showImportConflictDialog = ref(false)
const importPreviewData = ref<BatchImportPreview | null>(null)
const importResolutions = ref<Record<string, "skip" | "replace">>({})
const importBusy = ref(false)

async function handleImport() {
  const paths = await invoke<string[]>("pick_archive_files")
  if (paths.length === 0) return
  await doImportFlow(paths)
}

async function doImportFlow(paths: string[]) {
  importBusy.value = true
  try {
    const preview = await invoke<BatchImportPreview>("process_import_targets", {
      paths,
      enableNow: false,
    })
    const conflicts = preview.discoveredMods.filter(m => m.status === "conflict")
    if (conflicts.length > 0) {
      importPaths.value = paths
      importPreviewData.value = preview
      importResolutions.value = Object.fromEntries(
        conflicts.map(m => [m.modId, "skip"] as const),
      )
      showImportConflictDialog.value = true
    } else {
      const allIds = preview.discoveredMods
        .filter(m => m.status !== "error" && m.status !== "unsupported_format")
        .map(m => m.modId)
      if (allIds.length === 0) {
        message.warning(t("common.noData"))
        return
      }
      await invoke<BatchInstallResult>("batch_install_mods", {
        paths,
        enableNow: false,
        hasConflicts: false,
        selectedIds: allIds,
        resolutions: [] as Array<[string, string]>,
      })
      fetchMods()
      message.success(t("import.success.installedCount", { count: allIds.length }))
    }
  } catch (e: unknown) {
    message.error(t("import.error.installFailed", { e: String(e) }))
  } finally {
    importBusy.value = false
  }
}

async function confirmImportWithConflicts() {
  if (!importPreviewData.value) return
  importBusy.value = true
  try {
    const preview = importPreviewData.value
    const allIds = preview.discoveredMods
      .filter(m => m.status !== "error" && m.status !== "unsupported_format")
      .map(m => m.modId)
    const resolutions = Object.entries(importResolutions.value)
      .filter(([_, v]) => v === "replace") as Array<[string, string]>
    await invoke<BatchInstallResult>("batch_install_mods", {
      paths: importPaths.value,
      enableNow: false,
      hasConflicts: true,
      selectedIds: allIds,
      resolutions,
    })
    showImportConflictDialog.value = false
    fetchMods()
    message.success(t("import.success.installedCount", { count: allIds.length }))
  } catch (e: unknown) {
    message.error(t("import.error.installFailed", { e: String(e) }))
  } finally {
    importBusy.value = false
  }
}

function setConflictResolution(modId: string, resolution: "skip" | "replace") {
  importResolutions.value = { ...importResolutions.value, [modId]: resolution }
}

function dedupe(arr: string[]): string[] {
  return [...new Set(arr)]
}

// --- 更新检测（托管于 useModUpdates composable）---
const {
  checkingUpdates,
  loadCachedUpdates,
  checkUpdates,
  hasUpdate,
  getUpdateInfo,
  openUpdateUrl,
} = useModUpdates()

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
    await invoke("create_profile", {
      name,
      description: null,
      modIds: [] as string[],
    })
    const profiles = await invoke<ModProfile[]>("list_profiles")
    const created = profiles.find(p => p.name === name)
    if (created) {
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
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.applyFailed")}: ${String(e)}`)
  } finally {
    creatingNewPreset.value = false
  }
}

// --- 侧边栏筛选 ---
const showFilterPanel = ref(false)
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

// --- 拖拽导入（Webview 级别，覆盖整个页面）---
let unlistenDragDrop: (() => void) | null = null

function isSupportedImport(path: string): boolean {
  const lower = path.toLowerCase()
  if (lower.endsWith(".zip") || lower.endsWith(".7z")) return true
  const base = path.split(/[\\/]/).pop() ?? ""
  if (!base.includes(".")) return true
  return false
}

async function setupDragDrop() {
  const webview = getCurrentWebview()
  unlistenDragDrop = await webview.onDragDropEvent((event) => {
    if (event.payload.type !== "drop") return
    if (importBusy.value) return
    const paths = event.payload.paths.filter(isSupportedImport)
    if (paths.length === 0) return
    doImportFlow(paths)
  })
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
        activePresetName.value = active.id === "__builtin__vanilla" ? t("profiles.builtinVanilla") : active.name
        quickPresetId.value = active.id
        presetSnapshot.value = new Set(active.modIds)
      }
    }
  } catch { /* ignore */ }
  // 加载更新检查缓存（无网络请求）
  await loadCachedUpdates()
  // 原版预设下检测到已启用模组时提示用户
  checkVanillaConflict()
  unlistenModsChanged = (await listen("slaymgr:mods-changed", () => {
    if (isActive.value) fetchMods()
  }).catch(() => null)) as (() => void) | null
  setupDragDrop()
})

onUnmounted(() => {
  unlistenModsChanged?.()
  unlistenDragDrop?.()
})

// 侧边栏切换预设后自动刷新
watch(presetAppliedTick, () => {
  fetchMods()
})
</script>

<template>
  <div>
    <!-- 头部 -->
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-c-primary">{{ t("library.title") }}</h1>
      <div class="flex items-center gap-4 text-sm text-c-secondary">
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-green-500 inline-block" />
          {{ t("library.enabledCountLabel") }} {{ enabledMods.length }}
        </span>
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full inline-block bg-c-muted" />
          {{ t("library.disabledCountLabel") }} {{ disabledMods.length }}
        </span>
        <span v-if="activePresetName" class="flex items-center gap-1 text-c-muted">
          <NIcon :size="14"><Bookmark /></NIcon>
          <span>{{ activePresetName }}</span>
        </span>
        <span v-if="loading" class="text-xs text-c-muted animate-pulse">{{ t("library.refreshing") }}</span>
        <div class="flex gap-2 ml-4">
          <NButton size="small" secondary @click="handleOpenModsDir">
            <template #icon><NIcon :size="14"><FolderOpen /></NIcon></template>
            {{ t("library.openModsDir") }}
          </NButton>
          <NButton size="small" secondary @click="openNewPreset">
            <template #icon><NIcon :size="14"><Bookmark /></NIcon></template>
            {{ t("library.newPreset") }}
          </NButton>
          <NButton size="small" secondary :loading="checkingUpdates" @click="checkUpdates">
            <template #icon><NIcon :size="14"><ArrowUp /></NIcon></template>
            {{ t("library.updateCheck.check") }}
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
    </div>

    <!-- 搜索栏 + 筛选 -->
    <div class="relative mb-4">
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
        <template #suffix>
          <div class="flex items-center gap-0.5">
            <button
              v-if="activeFilterCount > 0"
              class="filter-btn"
              @click="clearFilters"
              :title="t('library.filter.clear')"
            >
              <NIcon :size="14"><X /></NIcon>
            </button>
            <button
              class="filter-btn"
              :class="{ active: showFilterPanel }"
              @click="showFilterPanel = !showFilterPanel"
            >
              <NIcon :size="16"><Filter /></NIcon>
            </button>
          </div>
        </template>
      </NInput>

      <!-- 浮动筛选面板 -->
      <Transition name="filter-dropdown">
        <div
          v-if="showFilterPanel"
          class="absolute right-0 top-full mt-1 z-50 w-64 p-3 rounded-xl border shadow-lg backdrop-blur-xl"
          :style="{
            backgroundColor: 'color-mix(in srgb, var(--color-bg-primary) 95%, transparent)',
            borderColor: 'var(--color-border)',
          }"
        >
          <div class="flex flex-col gap-3">
            <!-- 显示 -->
            <div>
              <div class="text-xs font-medium mb-1.5 text-c-secondary">{{ t("library.filter.show") }}</div>
              <div class="flex gap-3">
                <NCheckbox v-model:checked="filterShowEnabled" size="small">
                  <span class="text-xs">{{ t("library.filter.enabled") }}</span>
                </NCheckbox>
                <NCheckbox v-model:checked="filterShowDisabled" size="small">
                  <span class="text-xs">{{ t("library.filter.disabled") }}</span>
                </NCheckbox>
              </div>
            </div>
            <!-- 影响联机 -->
            <NCheckbox v-model:checked="filterAffectsGameplay" size="small">
              <span class="text-xs">{{ t("library.filter.affectsGameplay") }}</span>
            </NCheckbox>
            <!-- 标签 -->
            <div v-if="usedPresetTags.length > 0">
              <div class="text-xs font-medium mb-1.5 text-c-secondary">{{ t("library.filter.tags") }}</div>
              <div class="flex flex-wrap gap-x-3 gap-y-1">
                <NCheckbox
                  v-for="tag in usedPresetTags"
                  :key="tag.id"
                  :checked="filterTagIds.has(tag.id)"
                  size="small"
                  @update:checked="() => toggleFilterTag(tag.id)"
                >
                  <span class="text-xs">{{ getTagLabel(tag.id, currentLocale) }}</span>
                </NCheckbox>
              </div>
            </div>
            <span v-else class="text-xs italic text-c-muted">{{ t("library.filter.noTags") }}</span>
            <!-- 计数 + 清除 -->
            <div v-if="activeFilterCount > 0" class="flex justify-between items-center pt-2 border-t border-c-default">
              <span class="text-xs text-c-muted">{{ t("library.filter.activeFilterCount", { n: activeFilterCount }) }}</span>
              <NButton text size="tiny" type="warning" @click="clearFilters">
                <template #icon><NIcon :size="12"><X /></NIcon></template>
                {{ t("library.filter.clear") }}
              </NButton>
            </div>
          </div>
        </div>
      </Transition>
    </div>

    <Transition name="preset-fade" mode="out-in">
      <div :key="presetAppliedTick">
        <!-- 三层空状态 -->
        <div v-if="emptyReason" class="text-center py-16 text-c-muted">
          <template v-if="emptyReason === 'noMods'">
            <NIcon :size="48" class="mb-3" :color="'var(--color-text-muted)'"><PackageOpen /></NIcon>
            <p class="text-lg">{{ t("library.empty.noMods") }}</p>
            <p class="text-sm mt-1">{{ t("library.empty.noModsHint") }}</p>
          </template>
          <template v-else-if="emptyReason === 'filtered'">
            <NIcon :size="48" class="mb-3" :color="'var(--color-text-muted)'"><Filter /></NIcon>
            <p>{{ t("library.empty.filterNoResults") }}</p>
            <p class="text-sm mt-1">
              <NButton text size="tiny" @click="clearFilters">{{ t("library.empty.clearAllFilters") }}</NButton>
            </p>
          </template>
          <template v-else-if="emptyReason === 'search'">
            <NIcon :size="48" class="mb-3" :color="'var(--color-text-muted)'"><Search /></NIcon>
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

            <div v-if="filteredEnabled.length === 0" class="text-center py-8 text-c-muted">
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
                :has-update="hasUpdate(mod.id)"
                :update-info="getUpdateInfo(mod.id) ?? null"
                @toggle="handleToggle"
                @open-folder="handleOpenFolder"
                @uninstall="handleUninstall"
                @open-update-url="openUpdateUrl"
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

            <div v-if="filteredDisabled.length === 0" class="text-center py-8 text-c-muted">
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
                :has-update="hasUpdate(mod.id)"
                :update-info="getUpdateInfo(mod.id) ?? null"
                @toggle="handleToggle"
                @open-folder="handleOpenFolder"
                @uninstall="handleUninstall"
                @open-update-url="openUpdateUrl"
              />
            </NSpace>
          </NCard>
        </template>
      </div>
    </Transition>

    <!-- 导入冲突处理弹窗 -->
    <AppDialog v-model:show="showImportConflictDialog" :title="t('import.title')" width="520px">
      <NSpace vertical :size="12">
        <p class="text-sm text-c-secondary">{{ t("import.conflict.detected") }}</p>
        <div
          v-for="mod in (importPreviewData?.discoveredMods.filter(m => m.status === 'conflict') ?? [])"
          :key="mod.modId"
          class="flex p-2.5 rounded-xl gap-3"
          :style="{ backgroundColor: 'color-mix(in srgb, var(--color-bg-card) 50%, transparent)', border: '1px solid var(--color-border)' }"
        >
          <div class="min-w-0 flex-1">
            <div class="text-sm font-medium truncate">{{ mod.name }}</div>
            <div class="text-xs text-c-muted mt-1 space-y-0.5">
              <div v-for="reason in dedupe(mod.conflicts)" :key="reason">
                • {{ reason }}
              </div>
            </div>
          </div>
          <div class="flex gap-1.5 shrink-0 items-center">
            <NButton
              size="tiny"
              :type="importResolutions[mod.modId] === 'skip' ? 'primary' : 'default'"
              :quaternary="importResolutions[mod.modId] !== 'skip'"
              @click="setConflictResolution(mod.modId, 'skip')"
            >
              <template #icon>
                <NIcon :size="12" :class="importResolutions[mod.modId] === 'skip' ? '' : 'invisible'"><Check /></NIcon>
              </template>
              {{ t("import.conflict.skip") }}
            </NButton>
            <NButton
              size="tiny"
              :type="importResolutions[mod.modId] === 'replace' ? 'warning' : 'default'"
              :quaternary="importResolutions[mod.modId] !== 'replace'"
              @click="setConflictResolution(mod.modId, 'replace')"
            >
              <template #icon>
                <NIcon :size="12" :class="importResolutions[mod.modId] === 'replace' ? '' : 'invisible'"><Check /></NIcon>
              </template>
              {{ t("import.conflict.replace") }}
            </NButton>
          </div>
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <NButton @click="showImportConflictDialog = false">{{ t("common.cancel") }}</NButton>
          <NButton
            type="primary"
            :loading="importBusy"
            @click="confirmImportWithConflicts"
          >
            {{ t("import.installWithCount", { count: importPreviewData?.discoveredMods.filter(m => m.status !== 'error' && m.status !== 'unsupported_format').length ?? 0 }) }}
          </NButton>
        </div>
      </NSpace>
    </AppDialog>

    <!-- 新增预设对话框 -->
    <AppDialog v-model:show="showNewPresetDialog" :title="t('library.newPreset')" width="420px">
      <NSpace vertical :size="12">
        <div>
          <label class="text-sm text-c-secondary mb-1 block">{{ t("library.savePreset.nameLabel") }}</label>
          <NInput
            v-model:value="newPresetName"
            :placeholder="t('library.savePreset.namePlaceholder')"
            @keyup.enter="handleCreateNewPreset"
          />
        </div>
        <div class="text-xs text-c-muted">
          {{ t("library.newPresetHint") }}
        </div>
        <div class="flex justify-end gap-2">
          <NButton @click="showNewPresetDialog = false">{{ t("common.cancel") }}</NButton>
          <NButton type="primary" :loading="creatingNewPreset" @click="handleCreateNewPreset">
            {{ t("common.confirm") }}
          </NButton>
        </div>
      </NSpace>
    </AppDialog>

    <!-- Save Guard 警告弹窗 -->
    <AppDialog v-model:show="showSaveGuardDialog" width="440px">
      <template #header>
        <div class="flex items-center gap-2">
          <NIcon :size="18" color="#f0a020"><AlertTriangle /></NIcon>
          <span class="font-semibold">{{ t("library.saveGuard.title") }}</span>
        </div>
      </template>
      <NSpace v-if="saveGuardInfo" vertical :size="8">
        <p v-if="saveGuardInfo.saveGuard.pathSwitched" class="text-sm text-c-secondary">
          {{ saveGuardInfo.saveGuard.direction === 'modded_to_vanilla' ? t("library.saveGuard.toVanilla") : t("library.saveGuard.toModded") }}
        </p>
        <p v-if="saveGuardInfo.saveGuard.hadPairs" class="text-sm text-c-secondary">
          {{ t("library.saveGuard.syncResult", { synced: saveGuardInfo.saveGuard.savesSynced, backups: saveGuardInfo.saveGuard.backupsCreated }) }}
        </p>
        <div class="flex justify-end mt-2">
          <NButton type="primary" size="small" @click="dismissSaveGuard">{{ t("library.saveGuard.gotIt") }}</NButton>
        </div>
      </NSpace>
    </AppDialog>

    <!-- 拖拽导入遮罩 -->
    <DragOverlay :accept-ext="['zip', '7z']" title="松开以导入模组" />

  </div>
</template>

<style scoped>
.filter-btn {
  padding: 4px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  outline: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.filter-btn:hover {
  background-color: color-mix(in srgb, var(--color-text-secondary) 8%, transparent);
  color: var(--color-text-secondary);
}
.filter-btn.active {
  color: var(--primary-color);
  background-color: color-mix(in srgb, var(--primary-color) 10%, transparent);
}

/* 筛选面板下拉动画 */
.filter-dropdown-enter-active {
  transition: all 0.2s ease-out;
}
.filter-dropdown-leave-active {
  transition: all 0.15s ease-in;
}
.filter-dropdown-enter-from {
  opacity: 0;
  transform: translateY(-4px) scale(0.97);
}
.filter-dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.97);
}
</style>
