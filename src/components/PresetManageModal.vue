<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  Alert,
  Badge,
  Button,
  Card,
  Checkbox,
  EmptyState,
  Input,
  LoadingOverlay,
  Textarea,
  useToast,
} from "@miuui/vue"
import { Plus, FolderHeart, Download, Search } from "@lucide/vue"
import type {
  ModProfile,
  ApplyProfileResult,
  BundlePreview,
  ConflictResolution,
  InstalledMod,
} from "@/types"
import { useSidebarStore } from "@/stores/useSidebarStore"
import { useExportStore } from "@/stores/useExportStore"
import AppDialog from "@/components/AppDialog.vue"
import PresetCard from "./PresetCard.vue"

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
  (e: "applied", profile: ModProfile): void
}>()

const { t } = useI18n()
const { toast } = useToast()

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

const filteredModsForPicker = computed(() => {
  let list = installedMods.value
  if (modSearchQuery.value) {
    const q = modSearchQuery.value.toLowerCase()
    list = list.filter(
      (m) =>
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
    selectedModIds.value = selectedModIds.value.filter((id) => id !== modId)
  }
}

function selectAllMods() {
  selectedModIds.value = filteredModsForPicker.value.map((m) => m.id)
}

function deselectAllMods() {
  selectedModIds.value = filteredModsForPicker.value
    .filter((m) => m.source === "workshop")
    .map((m) => m.id)
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
    toast({ title: `${t("profiles.error.loadFailed")}: ${String(e)}`, variant: "error" })
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
    toast({ title: t("profiles.warning.nameRequired"), variant: "warning" })
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
      toast({ title: t("profiles.success.updated"), variant: "success" })
    } else {
      await invoke("create_profile", {
        name,
        description: formDescription.value.trim() || null,
        modIds: ids,
      })
      toast({ title: t("profiles.success.created"), variant: "success" })
    }
    showCreateDialog.value = false
    await loadProfiles()
    loadQuickPresets()
  } catch (e: unknown) {
    toast({ title: `${t("profiles.error.saveFailed")}: ${String(e)}`, variant: "error" })
  } finally {
    loading.value = false
  }
}

async function handleDelete(profile: ModProfile) {
  try {
    await invoke("delete_profile", { id: profile.id })
    toast({ title: t("profiles.success.deleted", { name: profile.name }), variant: "success" })
    await loadProfiles()
    loadQuickPresets()
  } catch (e: unknown) {
    toast({ title: `${t("profiles.error.deleteFailed")}: ${String(e)}`, variant: "error" })
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
    toast({ title: `${t("profiles.error.applyFailed")}: ${String(e)}`, variant: "error" })
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
    toast({ title: t("profiles.success.exported", { path }), variant: "success" })
  } catch (e: unknown) {
    toast({ title: `${t("profiles.error.exportFailed")}: ${String(e)}`, variant: "error" })
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
    toast({ title: `${t("profiles.error.importPreviewFailed")}: ${String(e)}`, variant: "error" })
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
    toast({
      title: t("profiles.success.bundleImported", { n: result.enabledModIds.length }),
      variant: "success",
    })
    activePresetName.value = result.profile.name
    await loadProfiles()
    loadQuickPresets()
    presetAppliedTick.value++
    emit("applied", result.profile)
  } catch (e: unknown) {
    toast({ title: `${t("profiles.error.importFailed")}: ${String(e)}`, variant: "error" })
  } finally {
    loading.value = false
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      loadProfiles()
    }
  },
)

onMounted(() => {
  if (props.show) {
    loadProfiles()
  }
})
</script>

<template>
  <AppDialog :show="show" width="680px" @update:show="(v: boolean) => emit('update:show', v)">
    <template #header>
      <div class="flex items-center justify-between w-full pr-4">
        <div class="flex items-center gap-2">
          <FolderHeart :size="20" class="text-primary-theme" />
          <span class="text-base font-bold text-c-primary">{{ t("profiles.manageTitle") }}</span>
        </div>
        <div class="flex items-center gap-2">
          <Button size="sm" variant="secondary" @click="startImport">
            <template #icon><Download :size="14" /></template>
            {{ t("profiles.importBundle") }}
          </Button>
          <Button size="sm" variant="solid" @click="openCreate">
            <template #icon><Plus :size="14" /></template>
            {{ t("profiles.create") }}
          </Button>
        </div>
      </div>
    </template>

    <div class="max-h-[64vh] overflow-y-auto px-1 py-1 scrollbar-thin">
      <div v-if="profiles.length > 0" class="flex flex-col gap-2">
        <PresetCard
          v-for="p in profiles"
          :key="p.id"
          :profile="p"
          :is-active="p.name === activePresetName"
          :loading="loading"
          :is-exporting="exportingId === p.id"
          @apply="handleApply"
          @export="handleExport"
          @edit="openEdit"
          @delete="handleDelete"
        />
      </div>

      <Card v-else material="glass" padding="sm">
        <EmptyState
          :title="t('profiles.empty.noProfiles')"
          :description="t('profiles.empty.noProfilesHint')"
        >
          <template #icon><FolderHeart :size="36" /></template>
        </EmptyState>
      </Card>
    </div>

    <!-- 创建/编辑预设对话框 -->
    <AppDialog v-model:show="showCreateDialog" :title="formTitle" width="500px">
      <div class="flex flex-col gap-3">
        <div>
          <label class="text-sm text-c-secondary mb-1 block">{{ t("profiles.form.name") }}</label>
          <Input v-model="formName" :placeholder="t('profiles.form.namePlaceholder')" />
        </div>
        <div>
          <label class="text-sm text-c-secondary mb-1 block">{{
            t("profiles.form.description")
          }}</label>
          <Textarea
            v-model="formDescription"
            :placeholder="t('profiles.form.descriptionPlaceholder')"
            :rows="2"
          />
        </div>
        <div>
          <label class="text-sm text-c-secondary mb-1 flex items-center gap-2">
            {{ t("profiles.form.modList") }}
            <Badge size="xs" :variant="selectedModIds.length > 0 ? 'info' : 'neutral'">
              {{ t("profiles.form.selectedCount", { n: selectedModIds.length }) }}
            </Badge>
          </label>

          <Input
            v-model="modSearchQuery"
            :placeholder="t('profiles.form.searchMods')"
            size="sm"
            clearable
            class="mb-2"
          >
            <template #prefix><Search :size="14" /></template>
          </Input>

          <div class="max-h-56 overflow-y-auto border border-c-default rounded-lg p-2 space-y-1">
            <div v-if="loadingMods" class="text-center py-4 text-xs text-c-muted">
              {{ t("profiles.form.loadingMods") }}
            </div>
            <template v-else>
              <Checkbox
                v-for="mod in filteredModsForPicker"
                :key="mod.id"
                :model-value="mod.source === 'workshop' || selectedModIds.includes(mod.id)"
                :disabled="mod.source === 'workshop'"
                size="sm"
                class="w-full"
                @update:model-value="(v: boolean) => toggleModSelection(mod.id, v)"
              >
                <span class="text-xs inline-flex items-center gap-1 flex-wrap">
                  <span>{{ mod.name }}</span>
                  <span v-if="mod.version" class="text-c-muted font-mono">{{ mod.version }}</span>
                  <Badge v-if="mod.source === 'workshop'" variant="info" size="xs">
                    {{ t("library.mod.workshop") }}
                  </Badge>
                </span>
              </Checkbox>
              <div
                v-if="filteredModsForPicker.length === 0 && !loadingMods"
                class="text-center py-4 text-xs text-c-muted"
              >
                {{ t("profiles.form.noModsMatch") }}
              </div>
            </template>
          </div>

          <div class="flex items-center gap-2 mt-2">
            <Button size="xs" variant="ghost" @click="selectAllMods">
              {{ t("profiles.form.selectAll") }}
            </Button>
            <Button size="xs" variant="ghost" @click="deselectAllMods">
              {{ t("profiles.form.deselectAll") }}
            </Button>
          </div>
        </div>
      </div>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button variant="ghost" size="sm" @click="showCreateDialog = false">{{
            t("common.cancel")
          }}</Button>
          <Button variant="solid" size="sm" :loading="loading" @click="handleSave">
            {{ editingId ? t("profiles.form.updateBtn") : t("profiles.form.createBtn") }}
          </Button>
        </div>
      </template>
    </AppDialog>

    <!-- 应用结果对话框 -->
    <AppDialog v-if="applyResult" v-model:show="showApplyDialog" width="440px">
      <template #header>
        <span class="text-base font-semibold">
          {{ t("profiles.apply.applied", { name: applyResult.profile.name }) }}
        </span>
      </template>
      <div class="flex flex-col gap-2">
        <div v-if="applyResult.enabledModIds.length > 0" class="flex items-center gap-2 text-sm">
          <Badge variant="success" size="sm">+{{ applyResult.enabledModIds.length }}</Badge>
          <span class="text-c-secondary">{{ t("profiles.apply.enabled") }}</span>
        </div>
        <div v-if="applyResult.disabledModIds.length > 0" class="flex items-center gap-2 text-sm">
          <Badge variant="neutral" size="sm">-{{ applyResult.disabledModIds.length }}</Badge>
          <span class="text-c-secondary">{{ t("profiles.apply.disabled") }}</span>
        </div>
        <div v-if="applyResult.missingModIds.length > 0" class="flex items-center gap-2 text-sm">
          <Badge variant="warning" size="sm">!{{ applyResult.missingModIds.length }}</Badge>
          <span class="text-c-secondary">{{ t("profiles.apply.missing") }}</span>
        </div>
      </div>
      <template #footer>
        <div class="flex justify-end">
          <Button variant="solid" size="sm" @click="showApplyDialog = false">
            {{ t("profiles.apply.ok") }}
          </Button>
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
        {{
          t("profiles.importDialog.summary", {
            n1: bundlePreview.manifest.mods.length,
            n2: bundlePreview.manifest.profile.modIds.length,
          })
        }}
      </div>

      <Alert
        v-if="bundlePreview.conflicts.length > 0"
        variant="warning"
        material="glass"
        :title="t('profiles.importDialog.conflictCount', { n: bundlePreview.conflicts.length })"
        class="mb-3"
      >
        <div class="space-y-1 text-xs">
          <div v-for="c in bundlePreview.conflicts" :key="c.modId">
            · {{ c.name }}: {{ c.reason }}
          </div>
          <div class="pt-1 text-c-secondary">{{ t("profiles.importDialog.conflictHint") }}</div>
        </div>
      </Alert>

      <div v-if="bundlePreview.missingIds.length > 0" class="mb-3 text-xs text-c-secondary">
        {{ t("profiles.importDialog.missingHint", { n: bundlePreview.missingIds.length }) }}
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button variant="ghost" size="sm" @click="showImportDialog = false">{{
            t("common.cancel")
          }}</Button>
          <Button variant="solid" size="sm" :loading="loading" @click="confirmImport">
            {{ t("profiles.importDialog.importBtn") }}
          </Button>
        </div>
      </template>
    </AppDialog>

    <LoadingOverlay :loading="loading && !showImportDialog" :text="t('common.loading')" />
  </AppDialog>
</template>
