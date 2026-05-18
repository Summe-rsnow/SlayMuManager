<script setup lang="ts">
import { ref, computed, h } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import {
  NModal, NCard, NButton, NTag, NIcon, NSpace, NCheckbox, NRadioGroup, NRadio,
  NSpin, NDropdown, NProgress, useMessage,
} from "naive-ui"
import {
  FileArchive, FolderOpen, AlertTriangle, CheckCircle2, XCircle,
  PackageOpen, Download, HelpCircle, ChevronDown,
} from "lucide-vue-next"
import DropZone from "./DropZone.vue"
import type { BatchImportPreview, BatchInstallResult, ConflictResolution } from "../types"

const { t } = useI18n()

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{
  (e: "close"): void
  (e: "installed"): void
}>()

const message = useMessage()

// --- 状态机 ---
type Stage = "idle" | "previewing" | "ready" | "installing" | "done"
const stage = ref<Stage>("idle")
const loading = ref(false)

// 反重入保护：扫描/安装中阻止新的拖放触发
const isBusy = computed(() =>
  stage.value === "previewing" || stage.value === "installing",
)
const preview = ref<BatchImportPreview | null>(null)

// 用户选择
const selectedIds = ref<Set<string>>(new Set())
const resolutionMap = ref<Record<string, ConflictResolution>>({})
const importPaths = ref<string[]>([])
const enableAfterInstall = ref(true)

// 安装进度
const installProgress = ref({ current: 0, total: 0, name: "", status: "" })

// --- 计算属性 ---
const readyMods = computed(() =>
  preview.value?.discoveredMods.filter(
    (m) => m.status === "ready" || m.status === "conflict",
  ) ?? [],
)

const errorMods = computed(() =>
  preview.value?.discoveredMods.filter(
    (m) => m.status === "error" || m.status === "unsupported_format",
  ) ?? [],
)

const hasConflicts = computed(() =>
  preview.value?.discoveredMods.some((m) => m.conflicts.length > 0) ?? false,
)

// --- 操作 ---

async function pickSingleArchive() {
  const path = await invoke<string | null>("pick_archive_file")
  if (path) {
    importPaths.value = [path]
    await doPreview()
  }
}

async function pickMultipleArchives() {
  const paths = await invoke<string[]>("pick_archive_files")
  if (paths.length > 0) {
    importPaths.value = paths
    await doPreview()
  }
}

async function pickFolder() {
  const path = await invoke<string | null>("pick_import_folder")
  if (path) {
    importPaths.value = [path]
    await doPreview()
  }
}

const importOptions = computed(() => [
  { label: t("import.multiZip"), key: "multi", icon: () => h(NIcon, null, { default: () => h(FileArchive, { size: 16 }) }) },
  { label: t("import.fromFolder"), key: "folder", icon: () => h(NIcon, null, { default: () => h(FolderOpen, { size: 16 }) }) },
])

function handleImportSelect(key: string) {
  if (key === "multi") pickMultipleArchives()
  else if (key === "folder") pickFolder()
}

function handleDropFiles(paths: string[]) {
  importPaths.value = paths
  doPreview()
}

async function doPreview() {
  if (importPaths.value.length === 0) return
  stage.value = "previewing"
  loading.value = true

  try {
    const result = await invoke<BatchImportPreview>("process_import_targets", {
      paths: importPaths.value,
      enableNow: enableAfterInstall.value,
    })
    preview.value = result

    // 默认选中所有 ready 的 mod
    selectedIds.value = new Set(
      result.discoveredMods
        .filter((m) => m.status === "ready" || m.status === "conflict")
        .map((m) => m.modId),
    )

    // 默认冲突策略：skip
    for (const m of result.discoveredMods) {
      if (m.conflicts.length > 0) {
        resolutionMap.value[m.modId] = "skip"
      }
    }

    stage.value = "ready"
  } catch (e: unknown) {
    message.error(t("import.error.previewFailed", { e }))
    stage.value = "idle"
  } finally {
    loading.value = false
  }
}

function toggleSelect(modId: string) {
  const s = new Set(selectedIds.value)
  if (s.has(modId)) {
    s.delete(modId)
  } else {
    s.add(modId)
  }
  selectedIds.value = s
}

function toggleAll() {
  if (selectedIds.value.size === readyMods.value.length) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(readyMods.value.map((m) => m.modId))
  }
}

async function doInstall() {
  if (importPaths.value.length === 0) return
  stage.value = "installing"
  loading.value = true
  installProgress.value = { current: 0, total: 0, name: t("import.progress.preparing"), status: "starting" }

  const selected = Array.from(selectedIds.value)
  const resolutions: [string, string][] = selected.map((id) => [
    id,
    resolutionMap.value[id] ?? "skip",
  ])

  // 监听后端进度事件
  const unlisten = await listen<{ current: number; total: number; name: string; status: string }>(
    "install-progress",
    (event) => {
      installProgress.value = event.payload
    },
  )

  try {
    const result = await invoke<BatchInstallResult>("batch_install_mods", {
      paths: importPaths.value,
      enableNow: enableAfterInstall.value,
      hasConflicts: hasConflicts.value,
      selectedIds: selected,
      resolutions,
    })

    if (result.successCount > 0) {
      message.success(t("import.success.installedCount", { count: result.successCount }))
    }
    if (result.failureCount > 0) {
      message.warning(t("import.warning.failedCount", { count: result.failureCount }))
    }

    stage.value = "done"
    emit("installed")
  } catch (e: unknown) {
    message.error(t("import.error.installFailed", { e }))
    stage.value = "ready"
  } finally {
    loading.value = false
    unlisten()
  }
}

function reset() {
  stage.value = "idle"
  preview.value = null
  importPaths.value = []
  selectedIds.value = new Set()
  resolutionMap.value = {}
  loading.value = false
}

function handleClose() {
  reset()
  emit("close")
}

// --- 工具 ---
function statusIcon(status: string) {
  switch (status) {
    case "ready":
      return CheckCircle2
    case "conflict":
      return AlertTriangle
    case "error":
    case "unsupported_format":
      return XCircle
    default:
      return HelpCircle
  }
}

function statusType(status: string): "success" | "warning" | "error" | "info" {
  switch (status) {
    case "ready":
      return "success"
    case "conflict":
      return "warning"
    case "error":
    case "unsupported_format":
      return "error"
    default:
      return "info"
  }
}

function statusText(status: string) {
  const map: Record<string, string> = {
    ready: t("import.status.ready"),
    conflict: t("import.status.conflict"),
    error: t("import.status.error"),
    unsupported_format: t("import.status.unsupportedFormat"),
  }
  return map[status] ?? status
}
</script>

<template>
  <NModal
    :show="show"
    @update:show="(v: boolean) => !v && handleClose()"
  >
    <NCard
      style="width: 720px; max-height: 85vh"
      :bordered="false"
      role="dialog"
    >
      <!-- 标题栏 -->
      <template #header>
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <NIcon :size="20"><PackageOpen /></NIcon>
            <span class="text-lg font-semibold">{{ t("import.title") }}</span>
          </div>
        </div>
      </template>

      <!-- === 阶段 1: 选择导入方式 === -->
      <template v-if="stage === 'idle'">
        <NSpace vertical :size="16" class="py-4">
          <DropZone :busy="isBusy" @files-dropped="handleDropFiles" />
          <div class="flex items-center justify-center">
            <NButton
              size="large"
              style="border-radius: 4px 0 0 4px"
              @click="pickSingleArchive"
            >
              <template #icon><NIcon :size="20"><FileArchive /></NIcon></template>
              {{ t("import.singleZip") }}
            </NButton>
            <NDropdown trigger="click" :options="importOptions" @select="handleImportSelect">
              <NButton
                size="large"
                style="border-radius: 0 4px 4px 0; padding: 0 8px; width: 36px"
              >
                <NIcon :size="16"><ChevronDown /></NIcon>
              </NButton>
            </NDropdown>
          </div>
        </NSpace>
      </template>

      <!-- === 阶段 2: 预览中=== -->
      <div v-else-if="stage === 'previewing'" class="flex flex-col items-center py-12 gap-4">
        <NSpin size="large" />
        <p class="text-c-secondary">{{ t("import.scanning") }}</p>
      </div>

      <!-- === 阶段 3: 预览结果 === -->
      <template v-else-if="stage === 'ready' && preview">
        <div class="flex items-center justify-between mb-3">
          <span class="text-sm text-c-secondary">
            {{ t("import.preview.foundMods", { ready: readyMods.length }) }}
            <template v-if="errorMods.length > 0">
              · {{ t("import.preview.errorCount", { count: errorMods.length }) }}
            </template>
          </span>
          <NButton size="small" text @click="toggleAll">
            {{ selectedIds.size === readyMods.length ? t("import.deselectAll") : t("import.selectAll") }}
          </NButton>
        </div>

        <div class="max-h-80 overflow-auto border border-c-default rounded-lg">
          <!-- 有冲突提示 -->
          <div
            v-if="hasConflicts"
            class="flex items-center gap-2 px-3 py-2 bg-c-warning border-b border-c-warning text-sm"
          >
            <NIcon :size="14" color="#f0a020"><AlertTriangle /></NIcon>
            <span class="text-c-warning">{{ t("import.conflict.detected") }}</span>
          </div>

          <div
            v-for="mod in preview.discoveredMods"
            :key="mod.modId"
            class="border-b border-c-default last:border-b-0"
          >
            <!-- Mod 行 -->
            <div
              class="flex items-center gap-3 px-3 py-3"
              :class="{
                'opacity-50': mod.status === 'error' || mod.status === 'unsupported_format',
              }"
            >
              <NCheckbox
                v-if="mod.status === 'ready' || mod.status === 'conflict'"
                :checked="selectedIds.has(mod.modId)"
                @update:checked="() => toggleSelect(mod.modId)"
              />
              <div v-else class="w-4" />

              <NIcon :size="16" :color="statusType(mod.status) === 'success' ? '#18a058' : statusType(mod.status) === 'warning' ? '#f0a020' : '#d03050'">
                <component :is="statusIcon(mod.status)" />
              </NIcon>

              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="font-medium text-sm truncate">{{ mod.name }}</span>
                  <span v-if="mod.version" class="text-xs text-c-muted font-mono">{{ mod.version }}</span>
                </div>
                <div v-if="mod.statusMessage" class="text-xs text-c-muted mt-0.5">
                  {{ mod.statusMessage }}
                </div>
              </div>

              <NTag :type="statusType(mod.status)" size="small" :bordered="false">
                {{ statusText(mod.status) }}
              </NTag>
            </div>

            <!-- 冲突解析选项 -->
            <div
              v-if="mod.conflicts.length > 0 && selectedIds.has(mod.modId)"
              class="px-3 pb-3 ml-10"
            >
              <div class="text-xs text-c-secondary mb-1">{{ t("import.conflict.details") }}</div>
              <div v-for="c in mod.conflicts" :key="c" class="text-xs text-c-warning mb-1">
                · {{ c }}
              </div>
              <NRadioGroup
                :value="resolutionMap[mod.modId] ?? 'skip'"
                size="small"
                @update:value="(v: string) => resolutionMap[mod.modId] = v as ConflictResolution"
              >
                <NSpace :size="16">
                  <NRadio value="skip">{{ t("import.conflict.skip") }}</NRadio>
                  <NRadio value="replace">{{ t("import.conflict.replace") }}</NRadio>
                  <NRadio value="rename">{{ t("import.conflict.rename") }}</NRadio>
                </NSpace>
              </NRadioGroup>
            </div>
          </div>
        </div>

        <!-- 底部按钮 -->
        <div class="flex items-center justify-between mt-4">
          <NCheckbox v-model:checked="enableAfterInstall">
            {{ t("import.enableAfterInstall") }}
          </NCheckbox>
          <NSpace>
            <NButton @click="handleClose">{{ t("common.cancel") }}</NButton>
            <NButton
              type="primary"
              :disabled="selectedIds.size === 0"
              @click="doInstall"
            >
              <template #icon><NIcon :size="16"><Download /></NIcon></template>
              {{ t("import.installWithCount", { count: selectedIds.size }) }}
            </NButton>
          </NSpace>
        </div>
      </template>

      <!-- === 阶段 4: 安装中 === -->
      <div v-else-if="stage === 'installing'" class="flex flex-col items-center py-8 gap-4">
        <NProgress
          type="circle"
          :percentage="installProgress.total > 0 ? Math.round((installProgress.current / installProgress.total) * 100) : 0"
          :status="installProgress.status === 'done' ? 'success' : undefined"
          :processing="installProgress.status === 'installing'"
        />
        <div class="text-center">
          <p class="text-c-primary font-medium">{{ installProgress.name || t("import.progress.installing") }}</p>
          <p v-if="installProgress.total > 0" class="text-xs text-c-muted mt-1">
            {{ installProgress.current }} / {{ installProgress.total }}
          </p>
        </div>
      </div>

      <!-- === 阶段 5: 完成 === -->
      <template v-else-if="stage === 'done'">
        <div class="flex flex-col items-center py-8 gap-3">
          <NIcon :size="48" color="#18a058"><CheckCircle2 /></NIcon>
          <p class="text-lg font-medium">{{ t("import.done") }}</p>
          <NButton type="primary" @click="handleClose">{{ t("common.close") }}</NButton>
        </div>
      </template>
    </NCard>
  </NModal>
</template>
