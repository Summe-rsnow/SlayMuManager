<script setup lang="ts">
import { ref, watch } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { Alert, Badge, Button, Card, EmptyState, Spinner, useToast } from "@miuui/vue"
import { Download, ExternalLink, HardDrive, Sparkles, AlertCircle } from "@lucide/vue"
import type { NexusModFile, AppBootstrap } from "@/types"
import { useDownloadStore } from "@/stores/useDownloadStore"
import AppDialog from "@/components/AppDialog.vue"

const props = defineProps<{
  show: boolean
  modId: number | null
  modName: string
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
}>()

const { t } = useI18n()
const { toast } = useToast()
const downloadStore = useDownloadStore()

const loading = ref(false)
const files = ref<NexusModFile[]>([])
const isPremium = ref(false)
const startingDownloadFileId = ref<number | null>(null)

function formatBytes(bytes?: number | null): string {
  if (!bytes || bytes <= 0) return "--"
  const k = 1024
  const sizes = ["B", "KB", "MB", "GB"]
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
}

async function loadFiles(modId: number) {
  loading.value = true
  files.value = []
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    isPremium.value = bootstrap.nexusIsPremium

    const list = await invoke<NexusModFile[]>("get_nexus_mod_files", { modId })
    // 将主要文件排在最前
    list.sort((a: NexusModFile, b: NexusModFile) => {
      const aMain = a.category_id === 1 || a.category_name === "MAIN" || a.is_primary ? 1 : 0
      const bMain = b.category_id === 1 || b.category_name === "MAIN" || b.is_primary ? 1 : 0
      return bMain - aMain
    })
    files.value = list
  } catch (e: unknown) {
    toast.error(`${t("download.error.fetchFilesFailed")}: ${String(e)}`)
  } finally {
    loading.value = false
  }
}

async function handleDownload(file: NexusModFile) {
  if (!props.modId) return
  if (!isPremium.value) {
    handleOpenWebDownload(file)
    return
  }

  startingDownloadFileId.value = file.file_id
  try {
    const fileName = file.file_name || `${file.name}.zip`
    await downloadStore.startNexusDownload(props.modId, file.file_id, props.modName, fileName)
    toast.success(t("download.success.addedToQueue"))
    emit("update:show", false)
    downloadStore.showDrawer = true
  } catch (e: unknown) {
    toast.error(`${t("download.error.startFailed")}: ${String(e)}`)
  } finally {
    startingDownloadFileId.value = null
  }
}
function handleOpenWebDownload(file: NexusModFile) {
  if (!props.modId) return
  const url = `https://www.nexusmods.com/slaythespire2/mods/${props.modId}?tab=files&file_id=${file.file_id}`
  invoke("open_url_in_browser", { url })
}

watch(
  () => props.show,
  (val) => {
    if (val && props.modId) {
      loadFiles(props.modId)
    }
  },
)
</script>

<template>
  <AppDialog :show="show" width="600px" @update:show="(v: boolean) => emit('update:show', v)">
    <template #header>
      <div class="flex items-center gap-2">
        <Download :size="20" class="text-primary-theme" />
        <span class="font-bold text-base text-c-primary truncate max-w-[420px]">
          {{ t("download.selectFileTitle", { name: modName }) }}
        </span>
      </div>
    </template>

    <Alert v-if="!isPremium" variant="warning" material="glass" class="mb-3">
      <template #icon><AlertCircle :size="16" /></template>
      {{ t("download.nonPremiumNotice") }}
    </Alert>

    <Alert v-else variant="info" material="glass" class="mb-3">
      <template #icon><Sparkles :size="15" /></template>
      {{ t("download.premiumNotice") }}
    </Alert>

    <div class="max-h-[60vh] overflow-y-auto pr-1">
      <div
        v-if="loading"
        class="py-12 flex flex-col items-center justify-center gap-2 text-c-muted"
      >
        <Spinner size="md" class="text-primary-theme" />
        <span class="text-xs">{{ t("common.loading") }}</span>
      </div>

      <div v-else-if="files.length > 0" class="flex flex-col gap-2.5">
        <Card v-for="file in files" :key="file.file_id" material="glass" padding="sm">
          <div class="flex items-center justify-between gap-3">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="font-semibold text-sm text-c-primary truncate">{{ file.name }}</span>
                <Badge v-if="file.version" size="xs" variant="neutral" class="font-mono"
                  >v{{ file.version }}</Badge
                >
                <Badge
                  v-if="file.category_id === 1 || file.category_name === 'MAIN' || file.is_primary"
                  variant="primary"
                  size="xs"
                >
                  {{ t("download.mainFile") }}
                </Badge>
              </div>
              <div v-if="file.description" class="text-xs text-c-muted truncate mt-1">
                {{ file.description }}
              </div>
              <div class="flex items-center gap-2 text-xs text-c-muted mt-1.5 font-mono">
                <span>{{ formatBytes(file.size_in_bytes) }}</span>
                <span v-if="file.uploaded_time"
                  >· {{ new Date(file.uploaded_time).toLocaleDateString() }}</span
                >
              </div>
            </div>

            <div class="flex-shrink-0">
              <Button
                v-if="isPremium"
                size="sm"
                variant="solid"
                :loading="startingDownloadFileId === file.file_id"
                @click="handleDownload(file)"
              >
                <template #icon><Download :size="14" /></template>
                {{ t("download.directDownload") }}
              </Button>
              <Button v-else size="sm" variant="secondary" @click="handleOpenWebDownload(file)">
                <template #icon><ExternalLink :size="14" /></template>
                {{ t("download.webDownload") }}
              </Button>
            </div>
          </div>
        </Card>
      </div>

      <Card v-else material="glass" padding="sm">
        <EmptyState :title="t('download.empty.noFiles')">
          <template #icon><HardDrive :size="36" /></template>
        </EmptyState>
      </Card>
    </div>

    <template #footer>
      <div class="flex justify-end">
        <Button variant="ghost" size="sm" @click="emit('update:show', false)">{{
          t("common.close")
        }}</Button>
      </div>
    </template>
  </AppDialog>
</template>
