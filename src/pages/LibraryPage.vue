<script setup lang="ts">
defineOptions({ name: "LibraryPage" })
import { ref, computed, onMounted, onUnmounted, watch } from "vue"
import { storeToRefs } from "pinia"
import { useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { getCurrentWebview } from "@tauri-apps/api/webview"
import {
  Alert,
  Badge,
  Button,
  Checkbox,
  EmptyState,
  Input,
  PageHeader,
  SectionHeader,
  Switch,
  Textarea,
  useConfirm,
  useToast,
} from "@miuui/vue"
import {
  Search,
  Download,
  RefreshCw,
  FolderOpen,
  Bookmark,
  BookmarkPlus,
  FolderHeart,
  AlertTriangle,
  Filter,
  X,
  PackageOpen,
  ArrowUp,
  HardDrive,
  CheckCircle,
  Check,
  AlertCircle,
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
import ModVersionConflictModal from "@/components/ModVersionConflictModal.vue"
import AppDialog from "@/components/AppDialog.vue"
import PresetManageModal from "@/components/PresetManageModal.vue"
import SkeletonCard from "@/components/SkeletonCard.vue"
import type {
  InstalledMod,
  ModProfile,
  AppBootstrap,
  BatchImportPreview,
  BatchInstallResult,
} from "../types"
import { useIsActive } from "@/composables/useIsActive"
import { useModOperations } from "@/composables/useModOperations"

const { t } = useI18n()
const { toast } = useToast()
const { confirm } = useConfirm()
const router = useRouter()
const highlightStore = useHighlightStore()
const modCacheStore = useModCacheStore()
const { enabledMods, disabledMods, loading } = storeToRefs(modCacheStore)
const { fetchMods } = modCacheStore
const tagStore = useTagStore()

// --- 多版本共存管理 ---
const showVersionConflictModal = ref(false)
const selectedConflictModId = ref<string | null>(null)
const filterOnlyConflicts = ref(false)

function openVersionConflictModal(mod: InstalledMod) {
  selectedConflictModId.value = mod.id
  showVersionConflictModal.value = true
}

async function onCardToggle(mod: InstalledMod) {
  // 如果当前是禁用状态，用户尝试启用，但此时该模组已有另一个版本处于启用状态
  if (mod.state === "disabled" && modCacheStore.hasConflict(mod.id)) {
    const group = modCacheStore.getConflictGroup(mod.id)
    const alreadyEnabled = group?.instances.find((i) => i.state === "enabled")
    if (alreadyEnabled) {
      toast({
        title: `该模组已有版本 (v${alreadyEnabled.version ?? alreadyEnabled.folderName}) 处于生效状态，请在版本面板中切换！`,
        variant: "warning",
      })
      openVersionConflictModal(mod)
      return
    }
  }
  await handleToggle(mod)
}

// --- 组件生命周期守卫（防止切换页面时异步回调卡死）---
const { isActive } = useIsActive()

// --- 从共享 composable 获取侧边栏 & 游戏启动/预设状态 ---
const sidebarStore = useSidebarStore()
const {
  quickPresetId,
  activePresetName,
  activePresetId,
  presetSnapshot,
  presetAppliedTick,
  vanillaLaunch,
} = storeToRefs(sidebarStore)
const { loadQuickPresets, handleToggleVanillaLaunch } = sidebarStore

const {
  busyId,
  batchBusy,
  showSaveGuardDialog,
  saveGuardInfo,
  handleToggle,
  handleUninstall,
  handleOpenFolder,
  handleOpenModsDir,
  enableAllMods,
  disableAllMods,
  dismissSaveGuard,
} = useModOperations()

// --- 导入（直通流程：选文件 → 检测冲突 → 安装）---
const importPaths = ref<string[]>([])
const showImportConflictDialog = ref(false)
const importPreviewData = ref<BatchImportPreview | null>(null)
const importResolutions = ref<Record<string, "skip" | "replace">>({})
const importBusy = ref(false)

/** 弹窗引导前往设置 */
async function showSettingsPrompt(type: "game-path" | "nexus") {
  const isGamePath = type === "game-path"
  const ok = await confirm({
    title: t(isGamePath ? "settings.prompt.gamePathRequired" : "settings.prompt.apiKeyRequired"),
    description: t(
      isGamePath ? "settings.prompt.gamePathRequiredDesc" : "settings.prompt.apiKeyRequiredDesc",
    ),
    confirmLabel: t("settings.prompt.goToSettings"),
    cancelLabel: t("common.cancel"),
  })
  if (ok) {
    highlightStore.highlight(type)
    router.push("/settings")
  }
}

/** 带游戏路径守卫的导入 */
async function handleImport() {
  try {
    const b = await invoke<AppBootstrap>("get_app_bootstrap")
    if (!b.gameDirectory) {
      showSettingsPrompt("game-path")
      return
    }
  } catch {
    /* ignore */
  }
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
    const conflicts = preview.discoveredMods.filter((m) => m.status === "conflict")
    if (conflicts.length > 0) {
      importPaths.value = paths
      importPreviewData.value = preview
      importResolutions.value = Object.fromEntries(conflicts.map((m) => [m.modId, "skip"] as const))
      showImportConflictDialog.value = true
    } else {
      const allIds = preview.discoveredMods
        .filter((m) => m.status !== "error" && m.status !== "unsupported_format")
        .map((m) => m.modId)
      if (allIds.length === 0) {
        if (!isActive.value) return
        toast({ title: t("common.noData"), variant: "warning" })
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
      toast({
        title: t("import.success.installedCount", { count: allIds.length }),
        variant: "success",
      })
    }
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("import.error.installFailed", { e: String(e) }), variant: "error" })
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
      .filter((m) => m.status !== "error" && m.status !== "unsupported_format")
      .map((m) => m.modId)
    const resolutions = Object.entries(importResolutions.value).filter(
      ([_, v]) => v === "replace",
    ) as Array<[string, string]>
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
    toast({
      title: t("import.success.installedCount", { count: allIds.length }),
      variant: "success",
    })
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("import.error.installFailed", { e: String(e) }), variant: "error" })
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
const {
  loadCachedUpdates,
  checkUpdates: unsafeCheckUpdates,
  hasUpdate,
  getUpdateInfo,
  openUpdateUrl,
} = updateStore

/** 组件本地控制按钮 loading，不依赖 store 的响应式绑定 */
const localChecking = ref(false)

/** 同步 store 完成状态：store 切回 false 时同步本地状态 */
watch(
  () => updateStore.checkingUpdates,
  (v) => {
    if (!v) localChecking.value = false
  },
)

/** 带游戏路径 + API Key 守卫的更新检查 */
async function checkUpdates() {
  localChecking.value = true
  try {
    const b = await invoke<AppBootstrap>("get_app_bootstrap")
    if (!isActive.value) {
      localChecking.value = false
      return
    }
    if (!b.gameDirectory) {
      showSettingsPrompt("game-path")
      localChecking.value = false
      return
    }
    if (!b.nexusApiKey) {
      showSettingsPrompt("nexus")
      localChecking.value = false
      return
    }
  } catch {
    /* ignore */
  }
  unsafeCheckUpdates()
}

// --- 更新检查结果弹窗 ---
const showResultDialog = ref(false)
const acknowledgedReqId = ref(0)

const updatableMods = computed(() =>
  (lastCheckResult.value?.results ?? []).filter((r) => r.hasUpdate),
)

const resultDialogIcon = computed(() => {
  if (!lastCheckResult.value) return CheckCircle
  if (!lastCheckResult.value.success) return AlertCircle
  if (lastCheckResult.value.summary.updatedMods === 0) return CheckCircle
  return PackageOpen
})

const resultDialogIconColor = computed(() => {
  if (!lastCheckResult.value) return "#22c55e"
  if (!lastCheckResult.value.success) return "#f0a020"
  if (lastCheckResult.value.summary.updatedMods === 0) return "#22c55e"
  return "var(--primary-color)"
})

const resultDialogTitle = computed(() => {
  if (!lastCheckResult.value) return ""
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

// --- 存为预设与预设管理 ---
const showSaveAsPresetDialog = ref(false)
const saveAsPresetName = ref("")
const saveAsPresetDescription = ref("")
const savingAsPreset = ref(false)
const showPresetManageModal = ref(false)

function openSaveAsPreset() {
  saveAsPresetName.value = ""
  saveAsPresetDescription.value = ""
  showSaveAsPresetDialog.value = true
}

async function handleSaveAsPreset() {
  const name = saveAsPresetName.value.trim()
  if (!name) {
    toast({ title: t("library.warning.enterPresetName"), variant: "warning" })
    return
  }
  savingAsPreset.value = true
  try {
    const modIds = enabledMods.value.map((m) => m.id)
    await invoke("create_profile", {
      name,
      description: saveAsPresetDescription.value.trim() || null,
      modIds,
    })
    const profiles = await invoke<ModProfile[]>("list_profiles")
    const created = profiles.find((p) => p.name === name)
    if (created) {
      await invoke("apply_profile", { id: created.id })
      if (!isActive.value) return
      activePresetId.value = created.id
      activePresetName.value = created.name
      quickPresetId.value = created.id
      presetSnapshot.value = new Set(modIds)
      toast({ title: t("library.success.savedAsPreset", { name }), variant: "success" })
      await fetchMods()
      loadQuickPresets()
    }
    showSaveAsPresetDialog.value = false
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: `${t("profiles.error.saveFailed")}: ${String(e)}`, variant: "error" })
  } finally {
    savingAsPreset.value = false
  }
}

function onPresetApplied() {
  fetchMods()
  loadQuickPresets()
}

// --- 创意工坊取消订阅 ---
const unsubscribing = ref<Set<string>>(new Set())
async function handleUnsubscribe(workshopId: string) {
  if (unsubscribing.value.has(workshopId)) return
  unsubscribing.value = new Set(unsubscribing.value).add(workshopId)
  try {
    await invoke("unsubscribe_workshop_mod", { publishedFileId: Number(workshopId) })
    if (!isActive.value) return
    toast({ title: t("library.mod.unsubscribed"), variant: "success" })
    await fetchMods()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("library.error.operationFailed", { e }), variant: "error" })
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
  if (filterOnlyConflicts.value) n++
  if (!filterShowEnabled.value) n++
  if (!filterShowDisabled.value) n++
  if (filterTagIds.value.size > 0) n++
  return n
})

function clearFilters() {
  filterAffectsGameplay.value = false
  filterOnlyConflicts.value = false
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
const usedPresetTags = computed(() => PRESET_TAGS.filter((t) => tagStore.usedTags.has(t.id)))

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
  if (filterOnlyConflicts.value) {
    list = list.filter((m) => modCacheStore.hasConflict(m.id))
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
  if (filterOnlyConflicts.value) {
    list = list.filter((m) => modCacheStore.hasConflict(m.id))
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
const anyFilterActive = computed(
  () =>
    !filterShowEnabled.value ||
    !filterShowDisabled.value ||
    filterAffectsGameplay.value ||
    filterOnlyConflicts.value ||
    filterTagIds.value.size > 0,
)
const hasGamePath = ref<boolean | null>(null)
const emptyReason = computed(() => {
  if (hasGamePath.value === false && !hasMods.value) return "noGamePath"
  if (!hasMods.value) return "noMods"
  if (anyFilterActive.value && filteredEnabled.value.length + filteredDisabled.value.length === 0)
    return "filtered"
  if (hasSearch.value && filteredEnabled.value.length + filteredDisabled.value.length === 0)
    return "search"
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
      const active = profiles.find((p) => p.id === bootstrap.activeProfileId)
      if (active) {
        activePresetId.value = active.id
        activePresetName.value = active.name
        quickPresetId.value = active.id
        presetSnapshot.value = new Set(active.modIds)
      }
    }
  } catch {
    /* ignore */
  }
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
    <PageHeader :title="t('library.title')" :description="t('library.subtitle')" class="mb-5">
      <template #actions>
        <Badge dot variant="success" size="sm">
          {{ t("library.enabledCountLabel") }} {{ enabledMods.length }}
        </Badge>
        <Badge dot variant="neutral" size="sm">
          {{ t("library.disabledCountLabel") }} {{ disabledMods.length }}
        </Badge>
        <span
          v-if="activePresetName"
          class="flex items-center gap-1 text-c-muted flex-shrink-0 whitespace-nowrap"
        >
          <Bookmark :size="14" />
          <span>{{ activePresetName }}</span>
        </span>
        <span v-if="loading" class="text-xs text-c-muted animate-pulse">{{
          t("library.refreshing")
        }}</span>
        <FloatingTip :label="t('library.vanillaLaunch')" :text="t('library.vanillaLaunchHint')" />
        <Switch
          :model-value="vanillaLaunch"
          size="sm"
          @update:model-value="handleToggleVanillaLaunch"
        />
        <div class="flex gap-2 flex-wrap">
          <Button size="sm" variant="secondary" @click="handleOpenModsDir">
            <template #icon><FolderOpen :size="14" /></template>
            {{ t("library.openModsDir") }}
          </Button>
          <Button size="sm" variant="secondary" @click="openSaveAsPreset">
            <template #icon><BookmarkPlus :size="14" /></template>
            {{ t("library.saveAsPreset") }}
          </Button>
          <Button size="sm" variant="secondary" @click="showPresetManageModal = true">
            <template #icon><FolderHeart :size="14" /></template>
            {{ t("library.managePresets") }}
          </Button>
          <Button size="sm" variant="secondary" :loading="localChecking" @click="checkUpdates">
            <template #icon><ArrowUp :size="14" /></template>
            {{ localChecking ? t("library.updateCheck.checking") : t("library.updateCheck.check") }}
            <FloatingTip :text="t('library.updateCheck.supportHint')" :width="240" />
          </Button>
          <Button size="sm" variant="secondary" :loading="loading" @click="() => fetchMods()">
            <template #icon><RefreshCw :size="14" /></template>
            {{ t("common.refresh") }}
          </Button>
          <Button size="sm" variant="solid" @click="handleImport">
            <template #icon><Download :size="14" /></template>
            {{ t("library.importMod") }}
          </Button>
        </div>
      </template>
    </PageHeader>

    <!-- 搜索栏 + 筛选 -->
    <div class="relative mb-6">
      <Input
        v-model="searchInput"
        :placeholder="t('library.searchPlaceholder')"
        clearable
        @clear="clearSearch"
        @enter="applySearch"
      >
        <template #prefix>
          <Search :size="16" />
        </template>
        <template #suffix>
          <div class="flex items-center gap-0.5">
            <button
              v-if="activeFilterCount > 0"
              type="button"
              class="filter-btn"
              @click="clearFilters"
              :title="t('library.filter.clear')"
            >
              <X :size="14" />
            </button>
            <button
              type="button"
              class="filter-btn"
              :class="{ active: showFilterPanel }"
              @click="showFilterPanel = !showFilterPanel"
            >
              <Filter :size="16" />
            </button>
          </div>
        </template>
      </Input>

      <!-- 浮动筛选面板 -->
      <Transition name="filter-dropdown">
        <div
          v-if="showFilterPanel"
          class="glass-floating absolute right-0 top-full mt-1 z-50 w-64 p-3 rounded-xl"
        >
          <div class="flex flex-col gap-3">
            <!-- 显示 -->
            <div>
              <div class="text-xs font-medium mb-1.5 text-c-secondary">
                {{ t("library.filter.show") }}
              </div>
              <div class="flex gap-3">
                <Checkbox v-model="filterShowEnabled" size="sm">
                  <span class="text-xs">{{ t("library.filter.enabled") }}</span>
                </Checkbox>
                <Checkbox v-model="filterShowDisabled" size="sm">
                  <span class="text-xs">{{ t("library.filter.disabled") }}</span>
                </Checkbox>
              </div>
            </div>
            <!-- 影响联机 -->
            <Checkbox v-model="filterAffectsGameplay" size="sm">
              <span class="text-xs">{{ t("library.filter.affectsGameplay") }}</span>
            </Checkbox>
            <!-- 多版本冲突 -->
            <Checkbox v-model="filterOnlyConflicts" size="sm">
              <span class="text-xs text-amber-500 font-medium">仅看多版本共存模组</span>
            </Checkbox>
            <!-- 标签 -->
            <div v-if="usedPresetTags.length > 0">
              <div class="text-xs font-medium mb-1.5 text-c-secondary">
                {{ t("library.filter.tags") }}
              </div>
              <div class="flex flex-wrap gap-x-3 gap-y-1">
                <Checkbox
                  v-for="tag in usedPresetTags"
                  :key="tag.id"
                  :model-value="filterTagIds.has(tag.id)"
                  size="sm"
                  @update:model-value="() => toggleFilterTag(tag.id)"
                >
                  <span class="text-xs">{{ tagStore.getTagLabel(tag.id, currentLocale) }}</span>
                </Checkbox>
              </div>
            </div>
            <span v-else class="text-xs italic text-c-muted">{{ t("library.filter.noTags") }}</span>
            <!-- 计数 + 清除 -->
            <div
              v-if="activeFilterCount > 0"
              class="flex justify-between items-center pt-2 border-t border-c-default"
            >
              <span class="text-xs text-c-muted">{{
                t("library.filter.activeFilterCount", { n: activeFilterCount })
              }}</span>
              <Button variant="ghost" size="xs" class="text-amber-500" @click="clearFilters">
                <template #icon><X :size="12" /></template>
                {{ t("library.filter.clear") }}
              </Button>
            </div>
          </div>
        </div>
      </Transition>
    </div>

    <!-- 多版本冲突全局横幅 -->
    <Alert v-if="modCacheStore.conflictCount > 0" variant="warning" material="glass" class="mb-4">
      <template #icon><AlertTriangle :size="18" class="text-amber-500" /></template>
      <div class="flex items-center justify-between gap-4">
        <span>
          检测到
          <strong>{{ modCacheStore.conflictCount }}</strong>
          个模组存在多版本共存冲突，可能导致游戏加载异常。
        </span>
        <Button
          size="xs"
          :variant="filterOnlyConflicts ? 'solid' : 'secondary'"
          @click="filterOnlyConflicts = !filterOnlyConflicts"
        >
          {{ filterOnlyConflicts ? "显示全部模组" : "仅看冲突模组" }}
        </Button>
      </div>
    </Alert>

    <Transition name="preset-fade" mode="out-in">
      <div :key="presetAppliedTick">
        <div v-if="emptyReason === 'noGamePath'" class="py-16">
          <EmptyState
            :title="t('library.empty.noGamePath')"
            :description="t('library.empty.noGamePathHint')"
          >
            <template #icon><HardDrive :size="40" /></template>
          </EmptyState>
        </div>
        <div v-else-if="emptyReason === 'noMods'" class="py-16">
          <EmptyState
            :title="t('library.empty.noMods')"
            :description="t('library.empty.noModsHint')"
          >
            <template #icon><PackageOpen :size="40" /></template>
          </EmptyState>
        </div>
        <div v-else-if="emptyReason === 'filtered'" class="py-16">
          <EmptyState :title="t('library.empty.filterNoResults')">
            <template #icon><Filter :size="40" /></template>
            <template #actions>
              <Button size="xs" variant="solid" @click="clearFilters">清除筛选</Button>
            </template>
          </EmptyState>
        </div>
        <div v-else-if="emptyReason === 'search'" class="py-16">
          <EmptyState :title="t('library.empty.searchNoMatch', { q: searchQuery })">
            <template #icon><Search :size="40" /></template>
            <template #actions>
              <Button size="xs" variant="solid" @click="clearSearch">清除搜索</Button>
            </template>
          </EmptyState>
        </div>

        <template v-else>
          <div v-if="filterShowEnabled" class="mb-6">
            <SectionHeader :title="t('library.section.enabled')" class="mb-3">
              <template #actions>
                <Badge variant="neutral" size="xs">{{ filteredEnabled.length }}</Badge>
                <Button
                  v-if="filteredEnabled.length > 0 && !loading"
                  size="xs"
                  variant="secondary"
                  :disabled="batchBusy"
                  @click="disableAllMods"
                >
                  {{ t("library.disableAll") }}
                </Button>
              </template>
            </SectionHeader>
            <div v-if="loading" class="flex flex-col gap-2">
              <SkeletonCard v-for="i in 3" :key="'skel-e-' + i" />
            </div>
            <div v-else-if="filteredEnabled.length === 0" class="text-center py-8 text-c-muted">
              <p v-if="hasSearch || filterAffectsGameplay">
                {{ t("library.empty.filterNoResults") }}
              </p>
              <p v-else>{{ t("library.empty.noEnabledMods") }}</p>
            </div>
            <div v-if="filteredEnabled.length > 0">
              <TransitionGroup name="stagger" tag="div" class="flex flex-col gap-2">
                <ModCard
                  v-for="mod in filteredEnabled"
                  :key="mod.installDir || mod.id"
                  :mod="mod"
                  :enabled="true"
                  :busy="busyId === mod.id || busyId === mod.installDir"
                  :toggle-disabled="false"
                  :has-update="hasUpdate(mod.id)"
                  :update-info="getUpdateInfo(mod.id) ?? null"
                  @toggle="onCardToggle"
                  @open-folder="handleOpenFolder"
                  @uninstall="handleUninstall"
                  @open-update-url="openUpdateUrl"
                  @unsubscribe="handleUnsubscribe"
                  @manage-versions="openVersionConflictModal"
                />
              </TransitionGroup>
            </div>
          </div>

          <div v-if="filterShowDisabled" class="mb-6">
            <SectionHeader :title="t('library.section.disabled')" class="mb-3">
              <template #actions>
                <Badge variant="neutral" size="xs">{{ filteredDisabled.length }}</Badge>
                <Button
                  v-if="filteredDisabled.length > 0 && !loading"
                  size="xs"
                  variant="secondary"
                  :disabled="batchBusy"
                  @click="enableAllMods"
                >
                  {{ t("library.enableAll") }}
                </Button>
              </template>
            </SectionHeader>
            <div v-if="loading" class="flex flex-col gap-2"></div>
            <div v-else-if="filteredDisabled.length === 0" class="text-center py-8 text-c-muted">
              <p v-if="hasSearch || filterAffectsGameplay">
                {{ t("library.empty.filterNoResults") }}
              </p>
              <p v-else>{{ t("library.empty.noDisabledMods") }}</p>
            </div>
            <div v-else>
              <TransitionGroup name="stagger" tag="div" class="flex flex-col gap-2">
                <ModCard
                  v-for="mod in filteredDisabled"
                  :key="mod.installDir || mod.id"
                  :mod="mod"
                  :enabled="false"
                  :busy="busyId === mod.id || busyId === mod.installDir"
                  :toggle-disabled="false"
                  :has-update="hasUpdate(mod.id)"
                  :update-info="getUpdateInfo(mod.id) ?? null"
                  @toggle="onCardToggle"
                  @open-folder="handleOpenFolder"
                  @uninstall="handleUninstall"
                  @open-update-url="openUpdateUrl"
                  @unsubscribe="handleUnsubscribe"
                  @manage-versions="openVersionConflictModal"
                />
              </TransitionGroup>
            </div>
          </div>
        </template>
      </div>
    </Transition>

    <!-- 导入冲突处理弹窗 -->
    <AppDialog v-model:show="showImportConflictDialog" :title="t('import.title')" width="520px">
      <div class="flex flex-col gap-3">
        <p class="text-sm text-c-secondary">{{ t("import.conflict.detected") }}</p>
        <div
          v-for="mod in importPreviewData?.discoveredMods.filter((m) => m.status === 'conflict') ??
          []"
          :key="mod.modId"
          class="flex p-2.5 rounded-xl gap-3"
          :style="{
            backgroundColor: 'color-mix(in srgb, var(--color-bg-card) 50%, transparent)',
            border: '1px solid var(--color-border)',
          }"
        >
          <div class="min-w-0 flex-1">
            <div class="text-sm font-medium truncate">{{ mod.name }}</div>
            <div class="text-xs text-c-muted mt-1 space-y-0.5">
              <div v-for="reason in dedupe(mod.conflicts)" :key="reason">• {{ reason }}</div>
            </div>
          </div>
          <div class="flex gap-1.5 shrink-0 items-center">
            <Button
              size="xs"
              :variant="importResolutions[mod.modId] === 'skip' ? 'solid' : 'ghost'"
              @click="setConflictResolution(mod.modId, 'skip')"
            >
              <template #icon>
                <Check
                  :size="12"
                  :class="importResolutions[mod.modId] === 'skip' ? '' : 'invisible'"
                />
              </template>
              {{ t("import.conflict.skip") }}
            </Button>
            <Button
              size="xs"
              :variant="importResolutions[mod.modId] === 'replace' ? 'danger' : 'ghost'"
              @click="setConflictResolution(mod.modId, 'replace')"
            >
              <template #icon>
                <Check
                  :size="12"
                  :class="importResolutions[mod.modId] === 'replace' ? '' : 'invisible'"
                />
              </template>
              {{ t("import.conflict.replace") }}
            </Button>
          </div>
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <Button variant="ghost" size="sm" @click="showImportConflictDialog = false">{{
            t("common.cancel")
          }}</Button>
          <Button
            variant="solid"
            size="sm"
            :loading="importBusy"
            @click="confirmImportWithConflicts"
          >
            {{
              t("import.installWithCount", {
                count:
                  importPreviewData?.discoveredMods.filter(
                    (m) => m.status !== "error" && m.status !== "unsupported_format",
                  ).length ?? 0,
              })
            }}
          </Button>
        </div>
      </div>
    </AppDialog>

    <!-- 预设管理弹窗 -->
    <PresetManageModal v-model:show="showPresetManageModal" @applied="onPresetApplied" />

    <!-- 存为预设对话框 -->
    <AppDialog
      v-model:show="showSaveAsPresetDialog"
      :title="t('library.saveAsPreset')"
      width="420px"
    >
      <div class="flex flex-col gap-3">
        <div>
          <label class="text-sm text-c-secondary mb-1 block">{{
            t("library.savePreset.nameLabel")
          }}</label>
          <Input
            v-model="saveAsPresetName"
            :placeholder="t('library.savePreset.namePlaceholder')"
            @enter="handleSaveAsPreset"
          />
        </div>
        <div>
          <label class="text-sm text-c-secondary mb-1 block">{{
            t("profiles.form.description")
          }}</label>
          <Textarea
            v-model="saveAsPresetDescription"
            :placeholder="t('profiles.form.descriptionPlaceholder')"
            :rows="2"
          />
        </div>
        <div class="text-xs text-c-muted">
          {{ t("library.saveAsPresetHint", { n: enabledMods.length }) }}
        </div>
        <div class="flex justify-end gap-2">
          <Button variant="ghost" size="sm" @click="showSaveAsPresetDialog = false">{{
            t("common.cancel")
          }}</Button>
          <Button variant="solid" size="sm" :loading="savingAsPreset" @click="handleSaveAsPreset">
            {{ t("common.save") }}
          </Button>
        </div>
      </div>
    </AppDialog>

    <!-- Save Guard 警告弹窗 -->
    <AppDialog v-model:show="showSaveGuardDialog" width="440px">
      <template #header>
        <div class="flex items-center gap-2">
          <AlertTriangle :size="18" class="text-amber-500" />
          <span class="font-semibold">{{ t("library.saveGuard.title") }}</span>
        </div>
      </template>
      <div v-if="saveGuardInfo" class="flex flex-col gap-2">
        <p v-if="saveGuardInfo.saveGuard.pathSwitched" class="text-sm text-c-secondary">
          {{
            saveGuardInfo.saveGuard.direction === "modded_to_vanilla"
              ? t("library.saveGuard.toVanilla")
              : t("library.saveGuard.toModded")
          }}
        </p>
        <p v-if="saveGuardInfo.saveGuard.hadPairs" class="text-sm text-c-secondary">
          {{
            t("library.saveGuard.syncResult", {
              synced: saveGuardInfo.saveGuard.savesSynced,
              backups: saveGuardInfo.saveGuard.backupsCreated,
            })
          }}
        </p>
        <div class="flex justify-end mt-2">
          <Button variant="solid" size="sm" @click="dismissSaveGuard">{{
            t("library.saveGuard.gotIt")
          }}</Button>
        </div>
      </div>
    </AppDialog>

    <!-- 检查更新结果弹窗 -->
    <AppDialog v-model:show="showResultDialog" width="480px" :mask-closable="false">
      <template #header>
        <div class="flex items-center gap-2">
          <component :is="resultDialogIcon" :size="18" :style="{ color: resultDialogIconColor }" />
          <span class="font-semibold">{{ resultDialogTitle }}</span>
        </div>
      </template>

      <template v-if="lastCheckResult">
        <!-- 成功且无更新 -->
        <div
          v-if="lastCheckResult.success && lastCheckResult.summary.updatedMods === 0"
          class="text-sm text-c-secondary py-4 text-center"
        >
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
            :style="{
              backgroundColor: 'color-mix(in srgb, var(--color-text-muted) 6%, transparent)',
            }"
          >
            <div class="min-w-0 flex-1">
              <div class="font-medium truncate">{{ info.name }}</div>
              <div class="text-xs text-c-muted mt-0.5">
                {{ info.localVersion ?? "—" }}
                <span class="mx-1">→</span>
                <span class="font-semibold" style="color: var(--primary-color)">{{
                  info.remoteVersion
                }}</span>
              </div>
            </div>
            <Button
              variant="ghost"
              size="xs"
              class="text-primary-theme"
              @click="openUpdateUrl({ id: info.modId } as InstalledMod)"
            >
              {{ t("library.updateCheck.openNexus") }}
            </Button>
          </div>
        </div>

        <!-- 失败 -->
        <div v-else class="text-sm text-c-secondary py-4">
          {{ lastCheckResult.error ?? t("library.updateCheck.error", { e: "" }) }}
        </div>
      </template>

      <template #footer>
        <div class="flex justify-end">
          <Button size="sm" variant="secondary" @click="dismissResultDialog">{{
            t("common.close")
          }}</Button>
        </div>
      </template>
    </AppDialog>

    <!-- 拖拽导入遮罩 -->
    <DragOverlay :accept-ext="['zip', '7z', 'rar']" title="松开以导入模组" />

    <!-- 模组多版本共存管理弹窗 -->
    <ModVersionConflictModal
      v-model:show="showVersionConflictModal"
      :mod-id="selectedConflictModId"
    />
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
