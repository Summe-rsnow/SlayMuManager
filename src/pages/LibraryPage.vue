<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue"
import { storeToRefs } from "pinia"
import { useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { getCurrentWebview } from "@tauri-apps/api/webview"
import {
  NSpace, NButton, NInput, NIcon,
  NCheckbox, NSwitch, useMessage, useDialog,
} from "naive-ui"
import {
  Search, Download, RefreshCw, FolderOpen, Bookmark,
  AlertTriangle, Filter, X, PackageOpen, ArrowUp, HardDrive,
  CheckCircle, AlertCircle,
} from "@lucide/vue"
import { currentLocale } from "@/i18n"
import { useHighlightStore } from "@/stores/useHighlightStore"
import { useModCacheStore } from "@/stores/useModCacheStore"
import { useUpdateStore } from "@/stores/useUpdateStore"
import { useTagStore, PRESET_TAGS } from "@/stores/useTagStore"
import { useSidebarStore } from "@/stores/useSidebarStore"
import DragOverlay from "@/components/DragOverlay.vue"
import FloatingTip from "@/components/FloatingTip.vue"
import ModCard from "@/components/ModCard.vue"
import AppDialog from "@/components/AppDialog.vue"
import EmptyState from "@/components/EmptyState.vue"
import PageHeader from "@/components/PageHeader.vue"
import CountBadge from "@/components/CountBadge.vue"
import ListSection from "@/components/ListSection.vue"
import SkeletonCard from "@/components/SkeletonCard.vue"
import type { InstalledMod, ModProfile, AppBootstrap, BatchImportPreview, BatchInstallResult } from "../types"
import { useIsActive } from "@/composables/useIsActive"
import { useModOperations } from "@/composables/useModOperations"


const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()
const router = useRouter()
const highlightStore = useHighlightStore()
const modCacheStore = useModCacheStore()
const { enabledMods, disabledMods, loading } = storeToRefs(modCacheStore)
const { fetchMods } = modCacheStore
const tagStore = useTagStore()

// --- 组件生命周期守卫（防止切换页面时异步回调卡死）---
const { isActive } = useIsActive()

// --- 从共享 composable 获取侧边栏 & 游戏启动/预设状态 ---
const sidebarStore = useSidebarStore()
const { quickPresetId, activePresetName, activePresetId, presetSnapshot, presetAppliedTick, vanillaLaunch } = storeToRefs(sidebarStore)
const { loadQuickPresets, handleToggleVanillaLaunch } = sidebarStore

const {
  busyId, batchBusy, showSaveGuardDialog, saveGuardInfo,
  handleToggle, handleUninstall, handleOpenFolder, handleOpenModsDir,
  enableAllMods, disableAllMods, dismissSaveGuard,
} = useModOperations()

// --- 导入（直通流程：选文件 → 检测冲突 → 安装）---
const importPaths = ref<string[]>([])
const showImportConflictDialog = ref(false)
const importPreviewData = ref<BatchImportPreview | null>(null)
const importResolutions = ref<Record<string, "skip" | "replace">>({})
const importBusy = ref(false)

/** 弹窗引导前往设置 */
function showSettingsPrompt(type: "game-path" | "nexus") {
  const isGamePath = type === "game-path"
  dialog.warning({
    title: t(isGamePath ? "settings.prompt.gamePathRequired" : "settings.prompt.apiKeyRequired"),
    content: t(isGamePath ? "settings.prompt.gamePathRequiredDesc" : "settings.prompt.apiKeyRequiredDesc"),
    positiveText: t("settings.prompt.goToSettings"),
    negativeText: t("common.cancel"),
    onPositiveClick: () => {
      highlightStore.highlight(type)
      router.push("/settings")
    },
    maskClosable: true,
  })
}

/** 带游戏路径守卫的导入 */
async function handleImport() {
  try {
    const b = await invoke<AppBootstrap>("get_app_bootstrap")
    if (!b.gameDirectory) {
      showSettingsPrompt("game-path")
      return
    }
  } catch { /* ignore */ }
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
    if (!isActive.value) return
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
        if (!isActive.value) return
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
      if (!isActive.value) return
      fetchMods()
      loadCachedUpdates()
      message.success(t("import.success.installedCount", { count: allIds.length }))
    }
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("import.error.installFailed", { e: String(e) }))
  } finally {
    if (isActive.value) importBusy.value = false
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
    if (!isActive.value) return
    showImportConflictDialog.value = false
    fetchMods()
    loadCachedUpdates()
    message.success(t("import.success.installedCount", { count: allIds.length }))
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("import.error.installFailed", { e: String(e) }))
  } finally {
    if (isActive.value) importBusy.value = false
  }
}

function setConflictResolution(modId: string, resolution: "skip" | "replace") {
  importResolutions.value = { ...importResolutions.value, [modId]: resolution }
}

function dedupe(arr: string[]): string[] {
  return [...new Set(arr)]
}

// --- 更新检测（托管于 useUpdateStore Pinia store）---
const updateStore = useUpdateStore()
const { lastCheckResult } = storeToRefs(updateStore)
const { loadCachedUpdates, checkUpdates: unsafeCheckUpdates, hasUpdate, getUpdateInfo, openUpdateUrl } = updateStore

/** 组件本地控制按钮 loading，不依赖 store 的响应式绑定 */
const localChecking = ref(false)

/** 同步 store 完成状态：store 切回 false 时同步本地状态 */
watch(() => updateStore.checkingUpdates, (v) => { if (!v) localChecking.value = false })

/** 带游戏路径 + API Key 守卫的更新检查 */
async function checkUpdates() {
  localChecking.value = true
  try {
    const b = await invoke<AppBootstrap>("get_app_bootstrap")
    if (!isActive.value) { localChecking.value = false; return }
    if (!b.gameDirectory) { showSettingsPrompt("game-path"); localChecking.value = false; return }
    if (!b.nexusApiKey) { showSettingsPrompt("nexus"); localChecking.value = false; return }
  } catch { /* ignore */ }
  unsafeCheckUpdates()
}

// --- 更新检查结果弹窗 ---
const showResultDialog = ref(false)
const acknowledgedReqId = ref(0)

const updatableMods = computed(() =>
  (lastCheckResult.value?.results ?? []).filter((r) => r.hasUpdate)
)

const resultDialogIcon = computed(() => {
  if (!lastCheckResult.value) return CheckCircle
  if (!lastCheckResult.value.success) return AlertCircle
  if (lastCheckResult.value.summary.updatedMods === 0) return CheckCircle
  return PackageOpen
})

const resultDialogIconColor = computed(() => {
  if (!lastCheckResult.value) return '#22c55e'
  if (!lastCheckResult.value.success) return '#f0a020'
  if (lastCheckResult.value.summary.updatedMods === 0) return '#22c55e'
  return 'var(--primary-color)'
})

const resultDialogTitle = computed(() => {
  if (!lastCheckResult.value) return ''
  if (!lastCheckResult.value.success) return t("library.updateCheck.titleFail")
  if (lastCheckResult.value.summary.updatedMods === 0) return t("library.updateCheck.titleUpToDate")
  return t("library.updateCheck.titleFound", { n: lastCheckResult.value.summary.updatedMods })
})

function dismissResultDialog() {
  if (lastCheckResult.value) acknowledgedReqId.value = lastCheckResult.value.reqId
  showResultDialog.value = false
}

watch(localChecking, (val) => {
  if (!val && lastCheckResult.value && lastCheckResult.value.reqId !== acknowledgedReqId.value) {
    showResultDialog.value = true
  }
})

onMounted(() => {
  if (
    !localChecking.value &&
    lastCheckResult.value &&
    lastCheckResult.value.reqId !== acknowledgedReqId.value
  ) {
    showResultDialog.value = true
  }
})

// --- 新增/保存预设 ---
const showNewPresetDialog = ref(false)
const newPresetName = ref("")
const creatingNewPreset = ref(false)
/** 非空时表示"保存当前配置为新预设"，存的是要写入预设的 modId 列表 */
const saveAsPresetModIds = ref<string[] | null>(null)

function openNewPreset() {
  saveAsPresetModIds.value = null
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
    const modIds = saveAsPresetModIds.value ?? []
    await invoke("create_profile", {
      name,
      description: null,
      modIds,
    })
    const profiles = await invoke<ModProfile[]>("list_profiles")
    const created = profiles.find(p => p.name === name)
    if (created) {
      await invoke("apply_profile", { id: created.id })
      if (!isActive.value) return
      activePresetId.value = created.id
      activePresetName.value = created.name
      quickPresetId.value = created.id
      presetSnapshot.value = new Set(modIds)
      let msg = t("library.success.presetApplied", { name })
      if (saveAsPresetModIds.value) {
        // 保存当前配置后禁用原版预设的 mod
        await disableAllMods()
        msg = t("profiles.success.created")
      }
      message.success(msg)
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

// --- 创意工坊取消订阅 ---
const unsubscribing = ref<Set<string>>(new Set())
async function handleUnsubscribe(workshopId: string) {
  if (unsubscribing.value.has(workshopId)) return
  unsubscribing.value = new Set(unsubscribing.value).add(workshopId)
  try {
    await invoke("unsubscribe_workshop_mod", { publishedFileId: Number(workshopId) })
    if (!isActive.value) return
    message.success(t("library.mod.unsubscribed"))
    await fetchMods()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("library.error.operationFailed", { e }))
  } finally {
    if (isActive.value) {
      const next = new Set(unsubscribing.value)
      next.delete(workshopId)
      unsubscribing.value = next
    }
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
  PRESET_TAGS.filter((t) => tagStore.usedTags.has(t.id))
)

// --- 搜索（实时 debounce 200ms + 回车立即搜索）---
const searchInput = ref("")
const searchQuery = ref("")
let searchDebounce: ReturnType<typeof setTimeout> | null = null

watch(searchInput, (val) => {
  if (searchDebounce) clearTimeout(searchDebounce)
  searchDebounce = setTimeout(() => {
    searchDebounce = null
    searchQuery.value = val
  }, 200)
})

onUnmounted(() => {
  if (searchDebounce) clearTimeout(searchDebounce)
  searchDebounce = null
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
      const tags = tagStore.getTags(m.id)
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
      const tags = tagStore.getTags(m.id)
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
const hasGamePath = ref<boolean | null>(null)
const emptyReason = computed(() => {
  if (hasGamePath.value === false && !hasMods.value) return "noGamePath"
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
  if (lower.endsWith(".zip") || lower.endsWith(".7z") || lower.endsWith(".rar")) return true
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
    hasGamePath.value = !!bootstrap.gameDirectory
    if (bootstrap.activeProfileId) {
      const profiles = await invoke<ModProfile[]>("list_profiles")
      const active = profiles.find(p => p.id === bootstrap.activeProfileId)
      if (active) {
        activePresetId.value = active.id
        activePresetName.value = active.name
        quickPresetId.value = active.id
        presetSnapshot.value = new Set(active.modIds)
      }
    }
  } catch { /* ignore */ }
  // 加载更新检查缓存（无网络请求）
  await loadCachedUpdates()
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
    <PageHeader :title="t('library.title')" :subtitle="t('library.subtitle')">
      <CountBadge :label="t('library.enabledCountLabel')" :count="enabledMods.length" dot-color="bg-green-500" />
      <CountBadge :label="t('library.disabledCountLabel')" :count="disabledMods.length" />
      <span v-if="activePresetName" class="flex items-center gap-1 text-c-muted flex-shrink-0 whitespace-nowrap">
        <NIcon :size="14"><Bookmark /></NIcon>
        <span>{{ activePresetName }}</span>
      </span>
      <span v-if="loading" class="text-xs text-c-muted animate-pulse">{{ t("library.refreshing") }}</span>
      <FloatingTip :label="t('library.vanillaLaunch')" :text="t('library.vanillaLaunchHint')" />
      <NSwitch :value="vanillaLaunch" size="small" @update:value="handleToggleVanillaLaunch" />
      <div class="flex gap-2">
        <NButton size="small" secondary @click="handleOpenModsDir">
          <template #icon><NIcon :size="14"><FolderOpen /></NIcon></template>
          {{ t("library.openModsDir") }}
        </NButton>
        <NButton size="small" secondary @click="openNewPreset">
          <template #icon><NIcon :size="14"><Bookmark /></NIcon></template>
          {{ t("library.newPreset") }}
        </NButton>
        <NButton size="small" secondary :loading="localChecking" @click="checkUpdates">
          <template #icon><NIcon :size="14"><ArrowUp /></NIcon></template>
          {{ localChecking ? t("library.updateCheck.checking") : t("library.updateCheck.check") }}
          <FloatingTip :text="t('library.updateCheck.supportHint')" :width="240" />
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
    </PageHeader>

    <!-- 搜索栏 + 筛选 -->
    <div class="relative mb-6">
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
                  <span class="text-xs">{{ tagStore.getTagLabel(tag.id, currentLocale) }}</span>
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
        <div v-if="emptyReason === 'noGamePath'" class="py-16">
          <EmptyState :icon="HardDrive" :title="t('library.empty.noGamePath')" :description="t('library.empty.noGamePathHint')" />
        </div>
        <div v-else-if="emptyReason === 'noMods'" class="py-16">
          <EmptyState :icon="PackageOpen" :title="t('library.empty.noMods')" :description="t('library.empty.noModsHint')" />
        </div>
        <div v-else-if="emptyReason === 'filtered'" class="py-16">
          <EmptyState :icon="Filter" :title="t('library.empty.filterNoResults')" actionText="清除筛选" @action="clearFilters" />
        </div>
        <div v-else-if="emptyReason === 'search'" class="py-16">
          <EmptyState :icon="Search" :title="t('library.empty.searchNoMatch', { q: searchQuery })" actionText="清除搜索" @action="clearSearch" />
        </div>

        <template v-else>
          <ListSection v-if="filterShowEnabled" :title="t('library.section.enabled')" :count="filteredEnabled.length" :action-label="filteredEnabled.length > 0 && !loading ? t('library.disableAll') : undefined" :action-busy="batchBusy" @action="disableAllMods">
            <div v-if="loading" class="flex flex-col gap-2">
              <SkeletonCard v-for="i in 3" :key="'skel-e-'+i" />
            </div>
            <div v-else-if="filteredEnabled.length === 0" class="text-center py-8 text-c-muted">
              <p v-if="hasSearch || filterAffectsGameplay">{{ t("library.empty.filterNoResults") }}</p>
              <p v-else>{{ t("library.empty.noEnabledMods") }}</p>
            </div>
            <NSpace v-if="filteredEnabled.length > 0" vertical :size="8">
              <TransitionGroup name="stagger" tag="div" class="flex flex-col gap-2">
                <ModCard
                  v-for="mod in filteredEnabled"
                  :key="mod.id"
                  :mod="mod"
                  :enabled="true"
                  :busy="busyId === mod.id"
                  :toggle-disabled="false"
                  :has-update="hasUpdate(mod.id)"
                  :update-info="getUpdateInfo(mod.id) ?? null"
                  @toggle="handleToggle"
                  @open-folder="handleOpenFolder"
                  @uninstall="handleUninstall"
                  @open-update-url="openUpdateUrl"
                  @unsubscribe="handleUnsubscribe"
                />
              </TransitionGroup>
            </NSpace>
          </ListSection>

          <ListSection v-if="filterShowDisabled" :title="t('library.section.disabled')" :count="filteredDisabled.length" :action-label="filteredDisabled.length > 0 && !loading ? t('library.enableAll') : undefined" :action-busy="batchBusy" @action="enableAllMods">
            <div v-if="loading" class="flex flex-col gap-2">
              <SkeletonCard v-for="i in 2" :key="'skel-d-'+i" />
            </div>
            <div v-else-if="filteredDisabled.length === 0" class="text-center py-8 text-c-muted">
              <p v-if="hasSearch || filterAffectsGameplay">{{ t("library.empty.filterNoResults") }}</p>
              <p v-else>{{ t("library.empty.noDisabledMods") }}</p>
            </div>
            <NSpace v-else vertical :size="8">
              <TransitionGroup name="stagger" tag="div" class="flex flex-col gap-2">
                <ModCard
                  v-for="mod in filteredDisabled"
                  :key="mod.id"
                  :mod="mod"
                  :enabled="false"
                  :busy="busyId === mod.id"
                  :toggle-disabled="false"
                  :has-update="hasUpdate(mod.id)"
                  :update-info="getUpdateInfo(mod.id) ?? null"
                  @toggle="handleToggle"
                  @open-folder="handleOpenFolder"
                  @uninstall="handleUninstall"
                  @open-update-url="openUpdateUrl"
                  @unsubscribe="handleUnsubscribe"
                />
              </TransitionGroup>
            </NSpace>
          </ListSection>
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
    <AppDialog v-model:show="showNewPresetDialog" :title="saveAsPresetModIds ? t('library.saveAsPreset') : t('library.newPreset')" width="420px">
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
          {{ saveAsPresetModIds ? t("library.saveAsPresetHint", { n: saveAsPresetModIds.length }) : t("library.newPresetHint") }}
        </div>
        <div class="flex justify-end gap-2">
          <NButton @click="showNewPresetDialog = false">{{ t("common.cancel") }}</NButton>
          <NButton type="primary" :loading="creatingNewPreset" @click="handleCreateNewPreset">
            {{ saveAsPresetModIds ? t("common.save") : t("common.confirm") }}
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

    <!-- 检查更新结果弹窗 -->
    <AppDialog v-model:show="showResultDialog" width="480px" :mask-closable="false">
      <template #header>
        <div class="flex items-center gap-2">
          <NIcon :size="18" :color="resultDialogIconColor"><component :is="resultDialogIcon" /></NIcon>
          <span class="font-semibold">{{ resultDialogTitle }}</span>
        </div>
      </template>

      <template v-if="lastCheckResult">
        <!-- 成功且无更新 -->
        <div v-if="lastCheckResult.success && lastCheckResult.summary.updatedMods === 0" class="text-sm text-c-secondary py-4 text-center">
          {{ t("library.updateCheck.allUpToDate") }}
        </div>

        <!-- 成功且有更新 -->
        <div v-else-if="lastCheckResult.success" class="max-h-64 overflow-y-auto space-y-2">
          <p class="text-sm text-c-secondary mb-3">
            {{ t("library.updateCheck.foundUpdates", { n: lastCheckResult.summary.updatedMods }) }}
          </p>
          <div
            v-for="info in updatableMods"
            :key="info.modId"
            class="flex items-center justify-between px-3 py-2 rounded-lg text-sm"
            :style="{ backgroundColor: 'color-mix(in srgb, var(--color-text-muted) 6%, transparent)' }"
          >
            <div class="min-w-0 flex-1">
              <div class="font-medium truncate">{{ info.name }}</div>
              <div class="text-xs text-c-muted mt-0.5">
                {{ info.localVersion ?? '—' }}
                <span class="mx-1">→</span>
                <span class="font-semibold" style="color: var(--primary-color)">{{ info.remoteVersion }}</span>
              </div>
            </div>
            <NButton text size="tiny" style="color: var(--primary-color)" @click="openUpdateUrl({ id: info.modId } as InstalledMod)">
              {{ t("library.updateCheck.openNexus") }}
            </NButton>
          </div>
        </div>

        <!-- 失败 -->
        <div v-else class="text-sm text-c-secondary py-4">
          {{ lastCheckResult.error ?? t("library.updateCheck.error", { e: '' }) }}
        </div>
      </template>

      <template #footer>
        <NSpace justify="end">
          <NButton size="small" @click="dismissResultDialog">{{ t("common.close") }}</NButton>
        </NSpace>
      </template>
    </AppDialog>

    <!-- 拖拽导入遮罩 -->
    <DragOverlay :accept-ext="['zip', '7z', 'rar']" title="松开以导入模组" />

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
