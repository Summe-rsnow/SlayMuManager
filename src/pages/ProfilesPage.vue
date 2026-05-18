<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  NCard, NButton, NTag, NIcon, NSpace, NInput,
  NPopconfirm, NCheckbox, NPopover, useMessage,
} from "naive-ui"
import {
  Plus, FolderHeart, Edit3, Trash2, Play, Download, Upload, Search, Loader2,
} from "lucide-vue-next"
import type { AppBootstrap, ModProfile, ApplyProfileResult, BundlePreview, ConflictResolution, InstalledMod } from "../types"
import { useIsActive } from "../composables/useIsActive"
import { useSidebarActions } from "../composables/useSidebarActions"
import { useExportState } from "../composables/useExportState"
import EmptyState from "../components/EmptyState.vue"
import AppDialog from "../components/AppDialog.vue"

const { t } = useI18n()
const message = useMessage()

// --- 状态 ---
const profiles = ref<ModProfile[]>([])
const { activePresetName, presetAppliedTick } = useSidebarActions()
const showCreateDialog = ref(false)
const showApplyDialog = ref(false)
const showImportDialog = ref(false)
const applyResult = ref<ApplyProfileResult | null>(null)
const loading = ref(false)

// --- 组件生命周期守卫 ---
const { isActive } = useIsActive()

// 创建/编辑表单
const editingId = ref<string | null>(null)
const formName = ref("")
const formDescription = ref("")
const selectedModIds = ref<string[]>([])

// Mod 选择器
const installedMods = ref<InstalledMod[]>([])
const modSearchQuery = ref("")
const loadingMods = ref(false)

const filteredModsForPicker = computed(() => {
  let list = installedMods.value
  if (modSearchQuery.value) {
    const q = modSearchQuery.value.toLowerCase()
    list = list.filter(m =>
      m.name.toLowerCase().includes(q) ||
      m.id.toLowerCase().includes(q) ||
      (m.author ?? "").toLowerCase().includes(q),
    )
  }
  return list
})

async function loadInstalledMods() {
  loadingMods.value = true
  try {
    const [enabled, disabled] = await Promise.all([
      invoke<InstalledMod[]>("list_installed_mods"),
      invoke<InstalledMod[]>("list_disabled_mods"),
    ])
    if (!isActive.value) return
    installedMods.value = [...enabled, ...disabled]
  } catch {
    if (!isActive.value) return
    installedMods.value = []
  } finally {
    if (isActive.value) loadingMods.value = false
  }
}

function toggleModSelection(modId: string, checked: boolean) {
  if (checked) {
    if (!selectedModIds.value.includes(modId)) {
      selectedModIds.value = [...selectedModIds.value, modId]
    }
  } else {
    selectedModIds.value = selectedModIds.value.filter(id => id !== modId)
  }
}

function selectAllMods() {
  selectedModIds.value = filteredModsForPicker.value.map(m => m.id)
}

function deselectAllMods() {
  selectedModIds.value = []
}

const { exportingId } = useExportState()

// 整合包导入
const bundlePath = ref("")
const bundlePreview = ref<BundlePreview | null>(null)
const bundleResolutions = ref<Record<string, ConflictResolution>>({})

// --- 计算 ---
const formTitle = computed(() =>
  editingId.value ? t("profiles.form.editTitle") : t("profiles.form.createTitle"),
)

// --- 加载 ---
async function loadProfiles() {
  try {
    const result = await invoke<ModProfile[]>("list_profiles")
    if (!isActive.value) return
    profiles.value = result
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.loadFailed")}: ${String(e)}`)
  }
}

// --- CRUD ---
function openCreate() {
  editingId.value = null
  formName.value = ""
  formDescription.value = ""
  selectedModIds.value = []
  modSearchQuery.value = ""
  loadInstalledMods()
  showCreateDialog.value = true
}

function openEdit(profile: ModProfile) {
  editingId.value = profile.id
  formName.value = profile.name
  formDescription.value = profile.description ?? ""
  selectedModIds.value = [...profile.modIds]
  modSearchQuery.value = ""
  loadInstalledMods()
  showCreateDialog.value = true
}

async function handleSave() {
  const name = formName.value.trim()
  if (!name) {
    message.warning(t("profiles.warning.nameRequired"))
    return
  }
  const ids = selectedModIds.value

  loading.value = true
  try {
    if (editingId.value) {
      await invoke("update_profile", {
        id: editingId.value,
        name,
        description: formDescription.value.trim() || null,
        modIds: ids,
      })
      if (!isActive.value) return
      message.success(t("profiles.success.updated"))
    } else {
      await invoke("create_profile", {
        name,
        description: formDescription.value.trim() || null,
        modIds: ids,
      })
      if (!isActive.value) return
      message.success(t("profiles.success.created"))
    }
    showCreateDialog.value = false
    await loadProfiles()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.saveFailed")}: ${String(e)}`)
  } finally {
    if (isActive.value) loading.value = false
  }
}

async function handleDelete(profile: ModProfile) {
  try {
    await invoke("delete_profile", { id: profile.id })
    message.success(t("profiles.success.deleted", { name: profile.name }))
    await loadProfiles()
  } catch (e: unknown) {
    message.error(`${t("profiles.error.deleteFailed")}: ${String(e)}`)
  }
}

// --- 应用预设 ---
async function handleApply(profile: ModProfile) {
  loading.value = true
  try {
    applyResult.value = await invoke<ApplyProfileResult>("apply_profile", { id: profile.id })
    if (!isActive.value) return
    activePresetName.value = applyResult.value.profile.name
    showApplyDialog.value = true
    await loadProfiles()
    presetAppliedTick.value++
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.applyFailed")}: ${String(e)}`)
  } finally {
    if (isActive.value) loading.value = false
  }
}

// --- 导出 ---
async function handleExport(profile: ModProfile) {
  try {
    const path = await invoke<string | null>("pick_save_bundle_path", {
      defaultName: profile.name,
    })
    if (!path) return

    exportingId.value = profile.id
    await invoke("export_preset_bundle", {
      profileId: profile.id,
      outputPath: path,
    })
    message.success(t("profiles.success.exported", { path }))
  } catch (e: unknown) {
    message.error(`${t("profiles.error.exportFailed")}: ${String(e)}`)
  } finally {
    exportingId.value = null
  }
}

// --- 导入 ---
async function startImport() {
  try {
    const path = await invoke<string | null>("pick_preset_bundle")
    if (!path) return

    bundlePath.value = path
    loading.value = true
    bundlePreview.value = await invoke<BundlePreview>("preview_preset_bundle", {
      bundlePath: path,
    })
    // 默认冲突跳过
    for (const c of bundlePreview.value.conflicts) {
      bundleResolutions.value[c.modId] = "skip"
    }
    showImportDialog.value = true
  } catch (e: unknown) {
    message.error(`${t("profiles.error.importPreviewFailed")}: ${String(e)}`)
  } finally {
    loading.value = false
  }
}

async function confirmImport() {
  const resolutions: [string, string][] = Object.entries(bundleResolutions.value)
  loading.value = true
  try {
    const result = await invoke<ApplyProfileResult>("confirm_import_preset_bundle", {
      bundlePath: bundlePath.value,
      applyProfile: true,
      resolutions,
    })
    showImportDialog.value = false
    bundlePreview.value = null
    message.success(t("profiles.success.bundleImported", { n: result.enabledModIds.length }))
    activePresetName.value = result.profile.name
    await loadProfiles()
  } catch (e: unknown) {
    message.error(`${t("profiles.error.importFailed")}: ${String(e)}`)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await loadProfiles()
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    activePresetName.value = bootstrap.activeProfileName || ""
  } catch { /* ignore */ }
})

// 侧边栏切换预设后自动刷新
watch(presetAppliedTick, () => {
  loadProfiles()
})
</script>

<template>
  <div>
    <!-- 头部 -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-c-primary">{{ t("profiles.title") }}</h1>
        <p class="text-sm text-c-secondary mt-1">{{ t("profiles.subtitle") }}</p>
      </div>
      <NSpace>
        <NButton secondary @click="startImport">
          <template #icon><NIcon :size="16"><Upload /></NIcon></template>
          {{ t("profiles.importBundle") }}
        </NButton>
        <NButton type="primary" @click="openCreate">
          <template #icon><NIcon :size="16"><Plus /></NIcon></template>
          {{ t("profiles.create") }}
        </NButton>
      </NSpace>
    </div>

    <!-- 预设列表 -->
    <Transition name="preset-fade" mode="out-in">
      <div :key="presetAppliedTick">
        <div v-if="profiles.length > 0" class="grid grid-cols-1 md:grid-cols-2 gap-3 auto-rows-fr">
          <NCard
            v-for="p in profiles"
            :key="p.id"
            size="small"
            class="hover:shadow-md transition-shadow"
          >
            <div class="flex items-start">
              <div class="flex-1 min-w-0">
                <!-- 名称 + 标签 -->
                <div class="flex items-center gap-2 flex-wrap">
                  <NIcon :size="16" :color="p.builtin ? 'var(--primary-500)' : 'var(--primary-color)'"><FolderHeart /></NIcon>
                  <span class="font-semibold text-c-primary truncate">{{ p.name }}</span>
                  <NTag v-if="p.builtin" type="success" size="tiny" :bordered="false">
                    {{ t("profiles.builtin") }}
                  </NTag>
                  <NTag v-if="p.name === activePresetName" type="info" size="tiny" :bordered="false">
                    {{ t("profiles.active") }}
                  </NTag>
                </div>

                <!-- 描述区域（固定高度，无描述时显示占位文字） -->
                <div style="height: 2.25rem" class="mt-1 mb-0.5">
                  <NPopover v-if="p.description" trigger="hover" placement="top" :width="320">
                    <template #trigger>
                      <p class="text-xs text-c-muted line-clamp-2 cursor-help">{{ p.description }}</p>
                    </template>
                    <div class="text-xs leading-relaxed">{{ p.description }}</div>
                  </NPopover>
                  <p v-else class="text-xs text-c-muted leading-5">{{ t("profiles.noDescription") }}</p>
                </div>

                <!-- Mod 数量 -->
                <div class="h-6 flex items-center">
                  <NTag size="small" :bordered="false">
                    {{ t("profiles.modCount", { n: p.modIds.length }) }}
                  </NTag>
                </div>
              </div>

              <!-- 右侧操作按钮 -->
              <NSpace :size="4" class="ml-3 flex-shrink-0">
                <NButton text size="tiny" @click="() => handleApply(p)">
                  <template #icon><NIcon :size="14"><Play /></NIcon></template>
                </NButton>
                <NButton v-if="!p.builtin" text size="tiny" :loading="exportingId === p.id" @click="() => handleExport(p)">
                  <template #icon><NIcon :size="14"><Download /></NIcon></template>
                </NButton>
                <NButton v-if="!p.builtin" text size="tiny" @click="() => openEdit(p)">
                  <template #icon><NIcon :size="14"><Edit3 /></NIcon></template>
                </NButton>
                <NPopconfirm v-if="!p.builtin && p.name !== activePresetName" @positive-click="() => handleDelete(p)">
                  <template #trigger>
                    <NButton text size="tiny" type="error">
                      <template #icon><NIcon :size="14"><Trash2 /></NIcon></template>
                    </NButton>
                  </template>
                  <template v-if="p.name === activePresetName">
                    {{ t("profiles.confirmDeleteActive", { name: p.name }) }}
                  </template>
                  <template v-else>
                    {{ t("profiles.confirmDelete", { name: p.name }) }}
                  </template>
                </NPopconfirm>
              </NSpace>
            </div>
          </NCard>
        </div>

        <EmptyState v-else :icon="FolderHeart" :title="t('profiles.empty.noProfiles')" :description="t('profiles.empty.noProfilesHint')" bordered />
      </div>
    </Transition>

    <!-- 创建/编辑对话框 -->
    <AppDialog v-model:show="showCreateDialog" :title="formTitle" width="520px" :mask-closable="false">
      <NSpace vertical :size="12">
        <div>
          <label class="text-sm text-c-secondary mb-1 block">{{ t("profiles.form.name") }}</label>
          <NInput v-model:value="formName" :placeholder="t('profiles.form.namePlaceholder')" />
        </div>
        <div>
          <label class="text-sm text-c-secondary mb-1 block">{{ t("profiles.form.description") }}</label>
          <NInput
            v-model:value="formDescription"
            type="textarea"
            :placeholder="t('profiles.form.descriptionPlaceholder')"
            :rows="2"
          />
        </div>
        <div>
          <label class="text-sm text-c-secondary mb-1 flex items-center gap-2">
            {{ t("profiles.form.modList") }}
            <NTag size="tiny" :bordered="false" :type="selectedModIds.length > 0 ? 'info' : 'default'">
              {{ t("profiles.form.selectedCount", { n: selectedModIds.length }) }}
            </NTag>
          </label>

          <NInput
            v-model:value="modSearchQuery"
            :placeholder="t('profiles.form.searchMods')"
            size="small"
            clearable
            class="mb-2"
          >
            <template #prefix><NIcon :size="14"><Search /></NIcon></template>
          </NInput>

          <div class="max-h-60 overflow-y-auto border border-c-default rounded-lg p-2 space-y-1">
            <div v-if="loadingMods" class="text-center py-4 text-xs text-c-muted">
              {{ t("profiles.form.loadingMods") }}
            </div>
            <template v-else>
              <NCheckbox
                v-for="mod in filteredModsForPicker"
                :key="mod.id"
                :checked="selectedModIds.includes(mod.id)"
                size="small"
                class="w-full"
                @update:checked="(v: boolean) => toggleModSelection(mod.id, v)"
              >
                <span class="text-xs">
                  {{ mod.name }}
                  <span v-if="mod.version" class="text-c-muted font-mono ml-1">{{ mod.version }}</span>
                </span>
              </NCheckbox>
              <div v-if="filteredModsForPicker.length === 0 && !loadingMods" class="text-center py-4 text-xs text-c-muted">
                {{ t("profiles.form.noModsMatch") }}
              </div>
            </template>
          </div>

          <NSpace :size="4" class="mt-2">
            <NButton size="tiny" text @click="selectAllMods">
              {{ t("profiles.form.selectAll") }}
            </NButton>
            <NButton size="tiny" text @click="deselectAllMods">
              {{ t("profiles.form.deselectAll") }}
            </NButton>
          </NSpace>
        </div>
      </NSpace>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="showCreateDialog = false">{{ t("common.cancel") }}</NButton>
          <NButton type="primary" :loading="loading" @click="handleSave">
            {{ editingId ? t("profiles.form.updateBtn") : t("profiles.form.createBtn") }}
          </NButton>
        </NSpace>
      </template>
    </AppDialog>

    <!-- 应用结果对话框 -->
    <AppDialog v-if="applyResult" v-model:show="showApplyDialog" width="480px">
      <template #header>
        <span class="text-lg font-semibold">
          {{ t("profiles.apply.applied", { name: applyResult.profile.name }) }}
        </span>
      </template>
      <NSpace vertical :size="8">
        <div v-if="applyResult.enabledModIds.length > 0" class="flex items-center gap-2 text-sm">
          <NTag type="success" size="small" :bordered="false">+{{ applyResult.enabledModIds.length }}</NTag>
          <span class="text-c-secondary">{{ t("profiles.apply.enabled") }}</span>
        </div>
        <div v-if="applyResult.disabledModIds.length > 0" class="flex items-center gap-2 text-sm">
          <NTag type="default" size="small" :bordered="false">-{{ applyResult.disabledModIds.length }}</NTag>
          <span class="text-c-secondary">{{ t("profiles.apply.disabled") }}</span>
        </div>
        <div v-if="applyResult.missingModIds.length > 0" class="flex items-center gap-2 text-sm">
          <NTag type="warning" size="small" :bordered="false">!{{ applyResult.missingModIds.length }}</NTag>
          <span class="text-c-secondary">{{ t("profiles.apply.missing") }}</span>
        </div>
      </NSpace>
      <template #footer>
        <NButton type="primary" @click="showApplyDialog = false">
          {{ t("profiles.apply.ok") }}
        </NButton>
      </template>
    </AppDialog>

    <!-- 分析导入文件时的加载遮罩 -->
    <div
      v-if="loading && !showImportDialog"
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background-color: rgba(0, 0, 0, 0.5); backdrop-filter: blur(4px);"
    >
      <div class="flex flex-col items-center gap-3">
        <NIcon :size="32" class="animate-spin text-white"><Loader2 /></NIcon>
        <span class="text-sm text-white/90">{{ t("common.loading") }}</span>
      </div>
    </div>

    <!-- 导入预设对话框 -->
    <AppDialog v-if="bundlePreview" v-model:show="showImportDialog" width="560px" :mask-closable="false">
      <template #header>
        <span class="text-lg font-semibold">
          {{ t("profiles.importDialog.title", { name: bundlePreview.manifest.profile.name }) }}
        </span>
      </template>

      <div class="mb-3">
        <span class="text-sm text-c-secondary">
          {{ t("profiles.importDialog.summary", {
            n1: bundlePreview.manifest.mods.length,
            n2: bundlePreview.manifest.profile.modIds.length,
          }) }}
        </span>
      </div>

      <div
        v-if="bundlePreview.conflicts.length > 0"
        class="mb-3 p-3 rounded-lg bg-c-warning border border-c-warning"
      >
        <p class="text-sm font-medium text-c-warning mb-2">
          {{ t("profiles.importDialog.conflictCount", { n: bundlePreview.conflicts.length }) }}
        </p>
        <div
          v-for="c in bundlePreview.conflicts"
          :key="c.modId"
          class="text-xs text-c-warning mb-1"
        >
          · {{ c.name }}: {{ c.reason }}
        </div>
        <div class="mt-2 text-xs text-c-secondary">
          {{ t("profiles.importDialog.conflictHint") }}
        </div>
      </div>

      <div
        v-if="bundlePreview.missingIds.length > 0"
        class="mb-3 text-xs text-c-secondary"
      >
        {{ t("profiles.importDialog.missingHint", { n: bundlePreview.missingIds.length }) }}
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showImportDialog = false">{{ t("common.cancel") }}</NButton>
          <NButton type="primary" :loading="loading" @click="confirmImport">
            {{ t("profiles.importDialog.importBtn") }}
          </NButton>
        </NSpace>
      </template>
    </AppDialog>
  </div>
</template>
