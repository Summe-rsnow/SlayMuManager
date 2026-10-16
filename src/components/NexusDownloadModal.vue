<script setup lang="ts">
import { ref, watch } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NButton, NIcon, NTag, useMessage } from "naive-ui"
import { Download, ExternalLink, HardDrive, Sparkles, AlertCircle } from "@lucide/vue"
import type { NexusModFile, AppBootstrap } from "@/types"
import { useDownloadStore } from "@/stores/useDownloadStore"
import AppDialog from "@/components/AppDialog.vue"
import EmptyState from "@/components/EmptyState.vue"

const props = defineProps<{
  show: boolean
  modId: number | null
  modName: string
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
}>()

const { t } = useI18n()
const message = useMessage()
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
    message.error(`${t("download.error.fetchFilesFailed")}: ${String(e)}`)
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
    message.success(t("download.success.addedToQueue"))
    emit("update:show", false)
    downloadStore.showDrawer = true
  } catch (e: unknown) {
    message.error(`${t("download.error.startFailed")}: ${String(e)}`)
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
        <NIcon :size="20" class="text-primary-theme"><Download /></NIcon>
        <span class="font-bold text-base text-c-primary truncate max-w-[420px]">
          {{ t("download.selectFileTitle", { name: modName }) }}
        </span>
      </div>
    </template>

    <div
      v-if="!isPremium"
      class="mb-3 p-3 rounded-xl bg-amber-500/10 border border-amber-500/25 flex items-start gap-2.5"
    >
      <NIcon :size="16" class="text-amber-500 flex-shrink-0 mt-0.5"><AlertCircle /></NIcon>
      <div class="text-xs text-amber-500 leading-relaxed">
        {{ t("download.nonPremiumNotice") }}
      </div>
    </div>

    <div
      v-else
      class="mb-3 px-3 py-2 rounded-xl bg-primary-theme/10 border border-primary-theme/20 flex items-center gap-2"
    >
      <NIcon :size="15" class="text-primary-theme"><Sparkles /></NIcon>
      <span class="text-xs text-primary-theme font-medium">
        {{ t("download.premiumNotice") }}
      </span>
    </div>

    <div class="max-h-[60vh] overflow-y-auto pr-1">
      <div
        v-if="loading"
        class="py-12 flex flex-col items-center justify-center gap-2 text-c-muted"
      >
        <NIcon :size="24" class="animate-spin text-primary-theme"><Download /></NIcon>
        <span class="text-xs">{{ t("common.loading") }}</span>
      </div>

      <div v-else-if="files.length > 0" class="flex flex-col gap-2.5">
        <div
          v-for="file in files"
          :key="file.file_id"
          class="p-3 rounded-xl border border-c-default bg-c-surface transition-all flex items-center justify-between gap-3"
        >
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-1.5 flex-wrap">
              <span class="font-semibold text-sm text-c-primary truncate">{{ file.name }}</span>
              <NTag v-if="file.version" size="tiny" :bordered="false" class="font-mono"
                >v{{ file.version }}</NTag
              >
              <NTag
                v-if="file.category_id === 1 || file.category_name === 'MAIN' || file.is_primary"
                type="primary"
                size="tiny"
                :bordered="false"
              >
                {{ t("download.mainFile") }}
              </NTag>
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
            <NButton
              v-if="isPremium"
              size="small"
              type="primary"
              :loading="startingDownloadFileId === file.file_id"
              @click="handleDownload(file)"
            >
              <template #icon
                ><NIcon :size="14"><Download /></NIcon
              ></template>
              {{ t("download.directDownload") }}
            </NButton>
            <NButton v-else size="small" secondary @click="handleOpenWebDownload(file)">
              <template #icon
                ><NIcon :size="14"><ExternalLink /></NIcon
              ></template>
              {{ t("download.webDownload") }}
            </NButton>
          </div>
        </div>
      </div>

      <EmptyState v-else :icon="HardDrive" :title="t('download.empty.noFiles')" bordered />
    </div>

    <template #footer>
      <div class="flex justify-end">
        <NButton @click="emit('update:show', false)">{{ t("common.close") }}</NButton>
      </div>
    </template>
  </AppDialog>
</template>
