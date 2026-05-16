<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  NCard, NButton, NTag, NIcon, NSpace, NModal, NInput,
  NPopconfirm, NCheckbox, useMessage,
} from "naive-ui"
import {
  Plus, FolderHeart, Edit3, Trash2, Play, Download, Upload, Save, Search,
} from "lucide-vue-next"
import type { ModProfile, ApplyProfileResult, BundlePreview, ConflictResolution, InstalledMod } from "../types"

const { t } = useI18n()
const message = useMessage()

// --- 状态 ---
const profiles = ref<ModProfile[]>([])
const showCreateDialog = ref(false)
const showApplyDialog = ref(false)
const showImportDialog = ref(false)
const applyResult = ref<ApplyProfileResult | null>(null)
const loading = ref(false)

// --- 组件生命周期守卫 ---
const isActive = ref(true)
onBeforeUnmount(() => { isActive.value = false })

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
  } catch (e: any) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.loadFailed")}: ${e}`)
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
  } catch (e: any) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.saveFailed")}: ${e}`)
  } finally {
    if (isActive.value) loading.value = false
  }
}

async function handleDelete(profile: ModProfile) {
  try {
    await invoke("delete_profile", { id: profile.id })
    message.success(t("profiles.success.deleted", { name: profile.name }))
    await loadProfiles()
  } catch (e: any) {
    message.error(`${t("profiles.error.deleteFailed")}: ${e}`)
  }
}

// --- 应用预设 ---
async function handleApply(profile: ModProfile) {
  loading.value = true
  try {
    applyResult.value = await invoke<ApplyProfileResult>("apply_profile", { id: profile.id })
    if (!isActive.value) return
    showApplyDialog.value = true
    await loadProfiles()
  } catch (e: any) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.applyFailed")}: ${e}`)
  } finally {
    if (isActive.value) loading.value = false
  }
}

// --- 导出 ---
async function handleExport(profile: ModProfile) {
  // 使用 rfd 选择保存路径
  try {
    const path = await invoke<string | null>("pick_archive_file")
    if (!path) return

    await invoke("export_preset_bundle", {
      profileId: profile.id,
      outputPath: path,
    })
    message.success(t("profiles.success.exported", { path }))
  } catch (e: any) {
    message.error(`${t("profiles.error.exportFailed")}: ${e}`)
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
  } catch (e: any) {
    message.error(`${t("profiles.error.importPreviewFailed")}: ${e}`)
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
    await loadProfiles()
  } catch (e: any) {
    message.error(`${t("profiles.error.importFailed")}: ${e}`)
  } finally {
    loading.value = false
  }
}

// --- 从模组库保存当前启用列表 ---
async function saveCurrentMods() {
  try {
    await loadInstalledMods()
    if (!isActive.value) return
    if (installedMods.value.length === 0) {
      message.warning(t("profiles.warning.noEnabledMods"))
      return
    }
    selectedModIds.value = installedMods.value
      .filter(m => m.state === "enabled")
      .map(m => m.id)
    formName.value = `${t("profiles.form.defaultName")} ${new Date().toLocaleDateString("zh-CN")}`
    formDescription.value = t("profiles.form.defaultDescription")
    editingId.value = null
    modSearchQuery.value = ""
    showCreateDialog.value = true
  } catch (e: any) {
    if (!isActive.value) return
    message.error(`${t("profiles.error.readModsFailed")}: ${e}`)
  }
}

onMounted(loadProfiles)
</script>

<template>
  <div>
    <!-- 头部 -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-gray-800">{{ t("profiles.title") }}</h1>
        <p class="text-sm text-gray-500 mt-1">{{ t("profiles.subtitle") }}</p>
      </div>
      <NSpace>
        <NButton secondary @click="saveCurrentMods">
          <template #icon><NIcon :size="16"><Save /></NIcon></template>
          {{ t("profiles.saveCurrent") }}
        </NButton>
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
    <div v-if="profiles.length > 0" class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <NCard
        v-for="p in profiles"
        :key="p.id"
        size="small"
        class="hover:shadow-md transition-shadow"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <NIcon :size="16" :color="p.builtin ? '#10b981' : '#6366f1'"><FolderHeart /></NIcon>
              <span class="font-semibold text-gray-800 truncate">{{ p.name }}</span>
              <NTag v-if="p.builtin" type="success" size="tiny" :bordered="false">
                {{ t("profiles.builtin") }}
              </NTag>
            </div>
            <p v-if="p.description" class="text-xs text-gray-400 mb-2 line-clamp-2">
              {{ p.description }}
            </p>
            <NSpace :size="4">
              <NTag size="small" :bordered="false">
                {{ t("profiles.modCount", { n: p.modIds.length }) }}
              </NTag>
            </NSpace>
          </div>
          <NSpace :size="4" class="ml-3 flex-shrink-0">
            <NButton text size="tiny" @click="() => handleApply(p)">
              <template #icon><NIcon :size="14"><Play /></NIcon></template>
            </NButton>
            <NButton v-if="!p.builtin" text size="tiny" @click="() => handleExport(p)">
              <template #icon><NIcon :size="14"><Download /></NIcon></template>
            </NButton>
            <NButton v-if="!p.builtin" text size="tiny" @click="() => openEdit(p)">
              <template #icon><NIcon :size="14"><Edit3 /></NIcon></template>
            </NButton>
            <NPopconfirm v-if="!p.builtin" @positive-click="() => handleDelete(p)">
              <template #trigger>
                <NButton text size="tiny" type="error">
                  <template #icon><NIcon :size="14"><Trash2 /></NIcon></template>
                </NButton>
              </template>
              {{ t("profiles.confirmDelete", { name: p.name }) }}
            </NPopconfirm>
          </NSpace>
        </div>
      </NCard>
    </div>

    <NCard v-else size="small">
      <div class="text-center py-12 text-gray-400">
        <NIcon :size="48" class="c-gray-300 mb-3"><FolderHeart /></NIcon>
        <p>{{ t("profiles.empty.noProfiles") }}</p>
        <p class="text-sm mt-1">{{ t("profiles.empty.noProfilesHint") }}</p>
      </div>
    </NCard>

    <!-- 创建/编辑对话框 -->
    <NModal
      :show="showCreateDialog"
      :mask-closable="false"
      @update:show="(v: boolean) => !v && (showCreateDialog = false)"
    >
      <NCard style="width: 520px" :bordered="false" role="dialog">
        <template #header>
          <span class="text-lg font-semibold">{{ formTitle }}</span>
        </template>
        <NSpace vertical :size="12">
          <div>
            <label class="text-sm text-gray-600 mb-1 block">{{ t("profiles.form.name") }}</label>
            <NInput v-model:value="formName" :placeholder="t('profiles.form.namePlaceholder')" />
          </div>
          <div>
            <label class="text-sm text-gray-600 mb-1 block">{{ t("profiles.form.description") }}</label>
            <NInput
              v-model:value="formDescription"
              type="textarea"
              :placeholder="t('profiles.form.descriptionPlaceholder')"
              :rows="2"
            />
          </div>
          <div>
            <label class="text-sm text-gray-600 mb-1 flex items-center gap-2">
              {{ t("profiles.form.modList") }}
              <NTag size="tiny" :bordered="false" :type="selectedModIds.length > 0 ? 'info' : 'default'">
                {{ t("profiles.form.selectedCount", { n: selectedModIds.length }) }}
              </NTag>
            </label>

            <!-- Mod 搜索 -->
            <NInput
              v-model:value="modSearchQuery"
              :placeholder="t('profiles.form.searchMods')"
              size="small"
              clearable
              class="mb-2"
            >
              <template #prefix><NIcon :size="14"><Search /></NIcon></template>
            </NInput>

            <!-- Mod 列表 -->
            <div class="max-h-60 overflow-y-auto border border-gray-200 rounded-lg p-2 space-y-1">
              <div v-if="loadingMods" class="text-center py-4 text-xs text-gray-400">
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
                    <span v-if="mod.version" class="text-gray-400 font-mono ml-1">{{ mod.version }}</span>
                  </span>
                </NCheckbox>
                <div v-if="filteredModsForPicker.length === 0 && !loadingMods" class="text-center py-4 text-xs text-gray-400">
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
      </NCard>
    </NModal>

    <!-- 应用结果对话框 -->
    <NModal
      :show="showApplyDialog"
      @update:show="(v: boolean) => !v && (showApplyDialog = false)"
    >
      <NCard v-if="applyResult" style="width: 480px" :bordered="false" role="dialog">
        <template #header>
          <span class="text-lg font-semibold">
            {{ t("profiles.apply.applied", { name: applyResult.profile.name }) }}
          </span>
        </template>
        <NSpace vertical :size="8">
          <div v-if="applyResult.enabledModIds.length > 0" class="flex items-center gap-2 text-sm">
            <NTag type="success" size="small" :bordered="false">+{{ applyResult.enabledModIds.length }}</NTag>
            <span class="text-gray-600">{{ t("profiles.apply.enabled") }}</span>
          </div>
          <div v-if="applyResult.disabledModIds.length > 0" class="flex items-center gap-2 text-sm">
            <NTag type="default" size="small" :bordered="false">-{{ applyResult.disabledModIds.length }}</NTag>
            <span class="text-gray-600">{{ t("profiles.apply.disabled") }}</span>
          </div>
          <div v-if="applyResult.missingModIds.length > 0" class="flex items-center gap-2 text-sm">
            <NTag type="warning" size="small" :bordered="false">!{{ applyResult.missingModIds.length }}</NTag>
            <span class="text-gray-600">{{ t("profiles.apply.missing") }}</span>
          </div>
        </NSpace>
        <template #footer>
          <NButton type="primary" @click="showApplyDialog = false">
            {{ t("profiles.apply.ok") }}
          </NButton>
        </template>
      </NCard>
    </NModal>

    <!-- 导入整合包对话框 -->
    <NModal
      :show="showImportDialog"
      :mask-closable="false"
      @update:show="(v: boolean) => !v && (showImportDialog = false)"
    >
      <NCard v-if="bundlePreview" style="width: 560px" :bordered="false" role="dialog">
        <template #header>
          <span class="text-lg font-semibold">
            {{ t("profiles.importBundle.title", { name: bundlePreview.manifest.profile.name }) }}
          </span>
        </template>

        <div class="mb-3">
          <span class="text-sm text-gray-500">
            {{ t("profiles.importBundle.summary", {
              n1: bundlePreview.manifest.mods.length,
              n2: bundlePreview.manifest.profile.modIds.length,
            }) }}
          </span>
        </div>

        <!-- 冲突提示 -->
        <div
          v-if="bundlePreview.conflicts.length > 0"
          class="mb-3 p-3 rounded-lg bg-amber-50 border border-amber-200"
        >
          <p class="text-sm font-medium text-amber-800 mb-2">
            {{ t("profiles.importBundle.conflictCount", { n: bundlePreview.conflicts.length }) }}
          </p>
          <div
            v-for="c in bundlePreview.conflicts"
            :key="c.modId"
            class="text-xs text-amber-700 mb-1"
          >
            · {{ c.name }}: {{ c.reason }}
          </div>
          <div class="mt-2 text-xs text-gray-500">
            {{ t("profiles.importBundle.conflictHint") }}
          </div>
        </div>

        <!-- 缺失提示 -->
        <div
          v-if="bundlePreview.missingIds.length > 0"
          class="mb-3 text-xs text-gray-500"
        >
          {{ t("profiles.importBundle.missingHint", { n: bundlePreview.missingIds.length }) }}
        </div>

        <template #footer>
          <NSpace justify="end">
            <NButton @click="showImportDialog = false">{{ t("common.cancel") }}</NButton>
            <NButton type="primary" :loading="loading" @click="confirmImport">
              {{ t("profiles.importBundle.importBtn") }}
            </NButton>
          </NSpace>
        </template>
      </NCard>
    </NModal>
  </div>
</template>
