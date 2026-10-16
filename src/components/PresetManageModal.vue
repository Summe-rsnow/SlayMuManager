<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  NButton, NTag, NIcon, NSpace, NInput,
  NCheckbox, useMessage,
} from "naive-ui"
import {
  Plus, FolderHeart, Edit3, Trash2, Play, Download, Search, Sparkles,
} from "@lucide/vue"
import type {
  ModProfile,
  ApplyProfileResult,
  BundlePreview,
  ConflictResolution,
  InstalledMod,
} from "@/types"
import { useSidebarStore } from "@/stores/useSidebarStore"
import { useExportStore } from "@/stores/useExportStore"
import EmptyState from "@/components/EmptyState.vue"
import AppDialog from "@/components/AppDialog.vue"
import ConfirmBtn from "@/components/ConfirmBtn.vue"
import IconBtn from "@/components/IconBtn.vue"
import LoadingOverlay from "@/components/LoadingOverlay.vue"
import FloatingTip from "@/components/FloatingTip.vue"

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
  (e: "applied", profile: ModProfile): void
}>()

const { t } = useI18n()
const message = useMessage()

// --- 状态 ---
const profiles = ref<ModProfile[]>([])
const sidebarStore = useSidebarStore()
const { activePresetName, presetAppliedTick } = storeToRefs(sidebarStore)
const { loadQuickPresets } = sidebarStore

const showCreateDialog = ref(false)
const showApplyDialog = ref(false)
const showImportDialog = ref(false)
const applyResult = ref<ApplyProfileResult | null>(null)
const loading = ref(false)

// 创建/编辑表单
const editingId = ref<string | null>(null)
const formName = ref("")
const formDescription = ref("")
const selectedModIds = ref<string[]>([])

// Mod 选择器
const installedMods = ref<InstalledMod[]>([])
const modSearchQuery = ref("")
const loadingMods = ref(false)

const cardStyle = computed(() => ({
  backgroundColor: "var(--glass-surface-bg)",
  backdropFilter: "blur(var(--glass-surface-blur))",
  WebkitBackdropFilter: "blur(var(--glass-surface-blur))",
  borderColor: "var(--glass-surface-border)",
  boxShadow: "var(--shadow-glass-surface)",
}))

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
    installedMods.value = [...enabled, ...disabled]
  } catch {
    installedMods.value = []
  } finally {
    loadingMods.value = false
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
  selectedModIds.value = filteredModsForPicker.value
    .filter(m => m.source === "workshop")
    .map(m => m.id)
}

const exportStore = useExportStore()
const { exportingId } = storeToRefs(exportStore)

// 整合包导入
const bundlePath = ref("")
const bundlePreview = ref<BundlePreview | null>(null)
const bundleResolutions = ref<Record<string, ConflictResolution>>({})

const formTitle = computed(() =>
  editingId.value ? t("profiles.form.editTitle") : t("profiles.form.createTitle"),
)

async function loadProfiles() {
  try {
    const result = await invoke<ModProfile[]>("list_profiles")
    profiles.value = result
  } catch (e: unknown) {
    message.error(`${t("profiles.error.loadFailed")}: ${String(e)}`)
  }
}

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
      message.success(t("profiles.success.updated"))
    } else {
      await invoke("create_profile", {
        name,
        description: formDescription.value.trim() || null,
        modIds: ids,
      })
      message.success(t("profiles.success.created"))
    }
    showCreateDialog.value = false
    await loadProfiles()
    loadQuickPresets()
  } catch (e: unknown) {
    message.error(`${t("profiles.error.saveFailed")}: ${String(e)}`)
  } finally {
    loading.value = false
  }
}

async function handleDelete(profile: ModProfile) {
  try {
    await invoke("delete_profile", { id: profile.id })
    message.success(t("profiles.success.deleted", { name: profile.name }))
    await loadProfiles()
    loadQuickPresets()
  } catch (e: unknown) {
    message.error(`${t("profiles.error.deleteFailed")}: ${String(e)}`)
  }
}

async function handleApply(profile: ModProfile) {
  loading.value = true
  try {
    applyResult.value = await invoke<ApplyProfileResult>("apply_profile", { id: profile.id })
    activePresetName.value = applyResult.value.profile.name
    showApplyDialog.value = true
    await loadProfiles()
    presetAppliedTick.value++
    loadQuickPresets()
    emit("applied", profile)
  } catch (e: unknown) {
    message.error(`${t("profiles.error.applyFailed")}: ${String(e)}`)
  } finally {
    loading.value = false
  }
}

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

async function startImport() {
  try {
    const path = await invoke<string | null>("pick_preset_bundle")
    if (!path) return

    bundlePath.value = path
    loading.value = true
    bundlePreview.value = await invoke<BundlePreview>("preview_preset_bundle", {
      bundlePath: path,
    })
    const resolutions: Record<string, ConflictResolution> = {}
    for (const c of bundlePreview.value.conflicts) {
      resolutions[c.modId] = "skip"
    }
    bundleResolutions.value = resolutions
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
    loadQuickPresets()
    presetAppliedTick.value++
    emit("applied", result.profile)
  } catch (e: unknown) {
    message.error(`${t("profiles.error.importFailed")}: ${String(e)}`)
  } finally {
    loading.value = false
  }
}

watch(() => props.show, (val) => {
  if (val) {
    loadProfiles()
  }
})

onMounted(() => {
  if (props.show) {
    loadProfiles()
  }
})
</script>

<template>
  <AppDialog
    :show="show"
    width="660px"
    @update:show="(v: boolean) => emit('update:show', v)"
  >
    <template #header>
      <div class="flex items-center justify-between w-full pr-4">
        <div class="flex items-center gap-2">
          <NIcon :size="20" class="text-primary-theme"><FolderHeart /></NIcon>
          <span class="text-base font-bold text-c-primary">{{ t("profiles.manageTitle") }}</span>
        </div>
        <div class="flex items-center gap-2">
          <NButton size="small" secondary @click="startImport">
            <template #icon><NIcon :size="14"><Download /></NIcon></template>
            {{ t("profiles.importBundle") }}
          </NButton>
          <NButton size="small" type="primary" @click="openCreate">
            <template #icon><NIcon :size="14"><Plus /></NIcon></template>
            {{ t("profiles.create") }}
          </NButton>
        </div>
      </div>
    </template>

    <div class="max-h-[64vh] overflow-y-auto pr-1">
      <div v-if="profiles.length > 0" class="flex flex-col gap-2.5">
        <div
          v-for="p in profiles"
          :key="p.id"
          class="group preset-card flex items-center justify-between p-3.5 rounded-xl border transition-all duration-200 hover:-translate-y-0.5"
          :style="cardStyle"
          :class="p.name === activePresetName ? 'border-primary-theme/50 bg-primary-theme/6 shadow-glow' : ''"
        >
          <!-- 左侧信息 -->
          <div class="flex-1 min-w-0 pr-4">
            <div class="flex items-center gap-2 flex-wrap">
              <NIcon :size="16" :class="p.name === activePresetName ? 'text-primary-theme' : 'text-c-muted'"><FolderHeart /></NIcon>
              <span class="font-semibold text-[15px] text-c-primary truncate">{{ p.name }}</span>
              <NTag v-if="p.builtin" type="success" size="tiny" :bordered="false">
                {{ t("profiles.builtin") }}
              </NTag>
              <NTag v-if="p.name === activePresetName" type="info" size="tiny" :bordered="false" class="font-medium">
                <template #icon><NIcon :size="10"><Sparkles /></NIcon></template>
                {{ t("profiles.active") }}
              </NTag>
              <span class="bg-c-secondary/50 px-2 py-0.5 rounded-md text-xs text-c-muted font-mono">
                {{ t("profiles.modCount", { n: p.modIds.length }) }}
              </span>
            </div>

            <div class="text-xs text-c-muted mt-1 leading-relaxed truncate">
              <FloatingTip v-if="p.description" :text="p.description" truncated :max-lines="1" />
              <span v-else class="opacity-50 italic">{{ t("profiles.noDescription") }}</span>
            </div>
          </div>

          <!-- 右侧操作 -->
          <div class="preset-actions flex items-center gap-2 flex-shrink-0">
            <NButton
              size="small"
              secondary
              :type="p.name === activePresetName ? 'default' : 'primary'"
              :disabled="p.name === activePresetName"
              class="px-2.5"
              @click="handleApply(p)"
            >
              <template #icon><NIcon :size="13"><Play /></NIcon></template>
              {{ p.name === activePresetName ? t("profiles.active") : t("profiles.applyBtn") }}
            </NButton>

            <IconBtn
              v-if="!p.builtin"
              :icon="Download"
              :tip="t('profiles.exportBundle')"
              :disabled="exportingId === p.id"
              @click="handleExport(p)"
            />

            <IconBtn
              v-if="!p.builtin"
              :icon="Edit3"
              :tip="t('profiles.form.editTitle')"
              @click="openEdit(p)"
            />

            <ConfirmBtn
              v-if="!p.builtin"
              :icon="Trash2"
              :confirmText="t('profiles.confirmDelete', { name: p.name })"
              :tip="t('common.delete')"
              @confirm="handleDelete(p)"
            />
          </div>
        </div>
      </div>

      <EmptyState
        v-else
        :icon="FolderHeart"
        :title="t('profiles.empty.noProfiles')"
        :description="t('profiles.empty.noProfilesHint')"
        bordered
      />
    </div>

    <!-- 创建/编辑预设对话框 -->
    <AppDialog v-model:show="showCreateDialog" :title="formTitle" width="500px">
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

          <div class="max-h-56 overflow-y-auto border border-c-default rounded-lg p-2 space-y-1">
            <div v-if="loadingMods" class="text-center py-4 text-xs text-c-muted">
              {{ t("profiles.form.loadingMods") }}
            </div>
            <template v-else>
              <NCheckbox
                v-for="mod in filteredModsForPicker"
                :key="mod.id"
                :checked="mod.source === 'workshop' || selectedModIds.includes(mod.id)"
                :disabled="mod.source === 'workshop'"
                size="small"
                class="w-full"
                @update:checked="(v: boolean) => toggleModSelection(mod.id, v)"
              >
                <span class="text-xs">
                  {{ mod.name }}
                  <span v-if="mod.version" class="text-c-muted font-mono ml-1">{{ mod.version }}</span>
                  <NTag v-if="mod.source === 'workshop'" type="info" size="tiny" :bordered="false" class="ml-1">
                    {{ t("library.mod.workshop") }}
                  </NTag>
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
    <AppDialog v-if="applyResult" v-model:show="showApplyDialog" width="440px">
      <template #header>
        <span class="text-base font-semibold">
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
        <div class="flex justify-end">
          <NButton type="primary" size="small" @click="showApplyDialog = false">
            {{ t("profiles.apply.ok") }}
          </NButton>
        </div>
      </template>
    </AppDialog>

    <!-- 导入预设对话框 -->
    <AppDialog v-if="bundlePreview" v-model:show="showImportDialog" width="540px">
      <template #header>
        <span class="text-base font-semibold">
          {{ t("profiles.importDialog.title", { name: bundlePreview.manifest.profile.name }) }}
        </span>
      </template>

      <div class="mb-3 text-sm text-c-secondary">
        {{ t("profiles.importDialog.summary", {
          n1: bundlePreview.manifest.mods.length,
          n2: bundlePreview.manifest.profile.modIds.length,
        }) }}
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

    <LoadingOverlay :loading="loading && !showImportDialog" :text="t('common.loading')" />
  </AppDialog>
</template>

<style scoped>
.preset-card {
  transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.2s ease, border-color 0.2s ease, background-color 0.2s ease;
}
.preset-card:hover {
  background-color: var(--glass-surface-bg-hover) !important;
  box-shadow: var(--shadow-card-hover), var(--shadow-glow) !important;
  border-color: color-mix(in srgb, var(--primary-color) 35%, var(--glass-surface-border)) !important;
}
</style>
